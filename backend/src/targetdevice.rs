use crate::{
    callbacks::Callbacks,
    display::{self, DisplayBuffer, TextAlignment},
    events::{Button, EncoderEvent, InputEvent, KeyEvent, Slider},
    extensions::{pitch_extension::pitch_serde, scale_extension::scale_serde},
    keyboard_error::KeyboardError,
    midi::{self, MidiLink},
    models::{keyboard_key::KeyboardKey, keycolor::KeyColor},
};
use harmonia::{PitchClass, Scale, ScaleKind};
use hidapi::HidDevice;
use serde::{Deserialize, Serialize};

/// Number of bytes of the button bitfield in the HID input report
const BUTTON_BYTES: usize = 5;
/// Main encoder + 8 knobs
const NUM_ENCODERS: usize = 9;

/// Accepts either a single string or a list of strings, so old configs with
/// `"midi_out_device": "name"` keep working.
fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(s) => vec![s],
        OneOrMany::Many(v) => v,
    })
}

fn default_keyboard_filter() -> String {
    midi::DEFAULT_PORT_FILTER.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct TargetDevice {
    pub name: String,
    pub keys: usize,
    #[serde(skip)]
    pub report_size: usize,
    #[serde(skip)]
    pub key_leds_adress: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub keys_hightlight_color: KeyColor,
    pub keys_dim_color: KeyColor,
    pub response_color: KeyColor,
    #[serde(skip)]
    device: Option<HidDevice>,
    #[serde(with = "pitch_serde")]
    pub root_note: PitchClass,
    #[serde(with = "scale_serde")]
    pub scale: ScaleKind,
    #[serde(skip)]
    pub first_note: u8,
    #[serde(skip)]
    midi_keyboard_link: Option<MidiLink>,
    #[serde(skip)]
    midi_merge_link: Option<MidiLink>,
    #[serde(skip)]
    pub midi_keyboard_device: String,
    /// Filters of all MIDI output ports to open (events are sent to every one).
    #[serde(alias = "midi_out_device", deserialize_with = "string_or_vec")]
    pub midi_out_devices: Vec<String>,
    #[serde(skip)]
    midi_outs: Vec<midi::MidiOut>,
    #[serde(skip)]
    pub callbacks: Callbacks,
    #[serde(skip)]
    button_state: [u8; BUTTON_BYTES],
    #[serde(skip)]
    encoder_state: Option<[u16; NUM_ENCODERS]>,
    #[serde(skip)]
    display: DisplayBuffer,
}

impl Default for TargetDevice {
    fn default() -> Self {
        Self {
            name: "kontrol49mk1".to_string(),
            keys: 49,
            report_size: 250,
            key_leds_adress: 0x82,
            vendor_id: 0x17cc,
            product_id: 0x1350,
            keys_hightlight_color: KeyColor::new(255, 0, 0),
            keys_dim_color: KeyColor::new(0, 0, 0),
            response_color: KeyColor::new(255, 255, 255),
            device: None,
            root_note: PitchClass::C,
            scale: ScaleKind::Ionian,
            first_note: 36,
            midi_keyboard_link: None,
            midi_merge_link: None,
            midi_keyboard_device: default_keyboard_filter(),
            midi_out_devices: vec!["KOMPLETE KONTROL S49 Anschluss 2".to_string()],
            callbacks: Callbacks::default(),
            button_state: [0; BUTTON_BYTES],
            encoder_state: None,
            display: DisplayBuffer::default(),
            midi_outs: Vec::new(),
        }
    }
}

impl TargetDevice {
    pub fn new(
        name: String,
        keys: usize,
        report_size: usize,
        key_leds_adress: u8,
        vendor_id: u16,
        product_id: u16,
        keys_hightlight_color: KeyColor,
        keys_dim_color: KeyColor,
        response_color: KeyColor,
    ) -> Self {
        Self {
            name,
            keys,
            report_size,
            key_leds_adress,
            vendor_id,
            product_id,
            keys_hightlight_color,
            keys_dim_color,
            response_color,
            ..Default::default()
        }
    }

    /// Serialize the device configuration to a JSON string.
    pub fn to_json(&self) -> Result<String, KeyboardError> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Deserialize a `TargetDevice` configuration from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, KeyboardError> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Save the device configuration to a JSON file.
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), KeyboardError> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Read and deserialize a device configuration from a JSON file.
    pub fn read<P: AsRef<std::path::Path>>(path: P) -> Result<Self, KeyboardError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Alias for `save`.
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), KeyboardError> {
        self.save(path)
    }

    /// Alias for `read`.
    pub fn read_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, KeyboardError> {
        Self::read(path)
    }

    /// Is the HID device is currently open.
    pub fn is_hid_connected(&self) -> bool {
        self.device.is_some()
    }

    /// Whether the keyboard MIDI input port is open.
    pub fn is_midi_connected(&self) -> bool {
        self.midi_keyboard_link.is_some()
    }

    /// Name of the connected keyboard MIDI input port, if any.
    pub fn midi_port_name(&self) -> Option<&str> {
        self.midi_keyboard_link
            .as_ref()
            .map(|m| m.port_name.as_str())
    }

    /// Whether the merged secondary MIDI input port is open.
    pub fn is_midi_in_connected(&self) -> bool {
        self.midi_merge_link.is_some()
    }

    /// Name of the connected secondary MIDI input port, if any.
    pub fn midi_in_port_name(&self) -> Option<&str> {
        self.midi_merge_link.as_ref().map(|m| m.port_name.as_str())
    }

    /// Whether at least one MIDI output port is open.
    pub fn is_midi_out_connected(&self) -> bool {
        !self.midi_outs.is_empty()
    }

    /// Names of all connected MIDI output ports.
    pub fn midi_out_port_names(&self) -> Vec<&str> {
        self.midi_outs.iter().map(|m| m.port_name()).collect()
    }

    /// Open the HID device and store the handle inside this struct.
    ///
    /// Also tries to open the keyboard's MIDI port (see [`Self::open_midi`]).
    /// A missing MIDI port is *not* treated as an error, only a warning is
    /// printed; button / encoder events and the lights still work without it.
    ///
    pub fn open_hid_device(&mut self) -> Result<(), KeyboardError> {
        let api =
            hidapi::HidApi::new().map_err(|e| KeyboardError::CantConnectKeyboard(e.to_string()))?;

        let device = api
            .open(self.vendor_id, self.product_id)
            .map_err(|e| KeyboardError::CantConnectKeyboard(e.to_string()))?;

        // Short startup sequence, which blacks out everything on the device
        // REQUIRED !
        const STARTUP_SEQUENCE: [u8; 3] = [0xa0, 0x00, 0x00];

        device
            .write(&STARTUP_SEQUENCE)
            .map_err(|e| KeyboardError::CannotWriteData(e.to_string()))?;

        self.device = Some(device);
        self.button_state = [0; BUTTON_BYTES];
        self.encoder_state = None;

        if let Err(e) = self.open_midi() {
            eprintln!("warning: {e} (key and slider events will not be available)");
        }
        Ok(())
    }

    /// Open the keyboard's MIDI input port using `self.midi_keyboard_device`.
    pub fn open_midi(&mut self) -> Result<(), KeyboardError> {
        let filter = if self.midi_keyboard_device.is_empty() {
            midi::DEFAULT_PORT_FILTER.to_string()
        } else {
            self.midi_keyboard_device.clone()
        };
        self.open_keyboard_midi(&filter)
    }

    /// Connect to a specific keyboard MIDI input port matching `filter` (case-insensitive).
    pub fn open_keyboard_midi(&mut self, filter: &str) -> Result<(), KeyboardError> {
        self.midi_keyboard_link = None;
        self.midi_keyboard_link = Some(midi::MidiLink::connect(filter)?);
        Ok(())
    }

    /// Connect to a secondary MIDI input port to merge with keyboard notes (case-insensitive).
    pub fn open_midi_in(&mut self, filter: &str) -> Result<(), KeyboardError> {
        if filter.trim().is_empty() {
            return Ok(());
        }
        self.midi_merge_link = None;
        self.midi_merge_link = Some(midi::MidiLink::connect(filter)?);
        Ok(())
    }

    /// Add a MIDI output port matching `filter` (case-insensitive) to the
    /// list of outputs. Already connected ports are kept.
    pub fn open_midi_out(&mut self, filter: &str) -> Result<(), KeyboardError> {
        if filter.trim().is_empty() {
            return Ok(());
        }
        let out = midi::MidiOut::connect(filter)?;
        if self
            .midi_outs
            .iter()
            .any(|o| o.port_name() == out.port_name())
        {
            return Ok(());
        }
        self.midi_outs.push(out);
        Ok(())
    }

    /// Close all outputs and connect every port in `self.midi_out_devices`.
    /// Returns one result per configured filter.
    pub fn open_midi_outs(&mut self) -> Vec<(String, Result<(), KeyboardError>)> {
        self.midi_outs.clear();
        let filters = self.midi_out_devices.clone();
        filters
            .into_iter()
            .map(|f| {
                let r = self.open_midi_out(&f);
                (f, r)
            })
            .collect()
    }

    pub fn midi_outs_mut(&mut self) -> &mut [midi::MidiOut] {
        &mut self.midi_outs
    }

    /// Send raw MIDI to every connected output.
    pub fn send_midi_raw(&mut self, msg: &[u8]) {
        for out in &mut self.midi_outs {
            let _ = out.send_raw(msg);
        }
    }

    /// Send an [`InputEvent`] to every connected output.
    pub fn send_midi_event(&mut self, event: &InputEvent) {
        for out in &mut self.midi_outs {
            let _ = out.send_input_event(event);
        }
    }

    // set all keys dark
    pub fn notes_off(&self) -> Result<(), KeyboardError> {
        let device = self
            .device
            .as_ref()
            .ok_or(KeyboardError::KeyboardNotConnected)?;

        let mut report = vec![0u8; self.report_size];
        report[0] = self.key_leds_adress;
        device
            .write(&report)
            .map_err(|e| KeyboardError::CannotWriteData(e.to_string()))?;
        Ok(())
    }

    pub fn set_scale_string(&mut self, scale: String) -> Result<(), KeyboardError> {
        let s: Scale = scale.parse().map_err(|_| KeyboardError::InvalidScale)?;
        self.scale = s.kind;
        self.highlight_scale()?;

        Ok(())
    }

    pub fn set_scale(&mut self, root: PitchClass, scale: ScaleKind) -> Result<(), KeyboardError> {
        self.root_note = root;
        self.scale = scale;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn set_root(&mut self, root: PitchClass) -> Result<(), KeyboardError> {
        self.root_note = root;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn set_scale_kind(&mut self, scale: ScaleKind) -> Result<(), KeyboardError> {
        self.scale = scale;
        self.highlight_scale()?;
        Ok(())
    }

    /// Pitch class of the physical key at `pos` (0 = left-most key),
    /// taking the current octave shift / transposition into account.
    pub fn pitch_class_at(&self, pos: usize) -> PitchClass {
        PitchClass::new(((self.first_note as usize + pos) % 12) as u8)
    }

    pub fn keyboard_keys(&self) -> Vec<KeyboardKey> {
        let build_scale = Scale::new(self.root_note, self.scale);
        (0..self.keys)
            .map(|pos| {
                let pc = self.pitch_class_at(pos);
                KeyboardKey::from_note(pc, pos, build_scale.contains(pc))
            })
            .collect()
    }

    fn highlight_scale(&self) -> Result<(), KeyboardError> {
        let colors: Vec<KeyColor> = self
            .keyboard_keys()
            .iter()
            .map(|key| {
                if key.led_active {
                    self.keys_hightlight_color
                } else {
                    self.keys_dim_color
                }
            })
            .collect();
        self.set_key_leds(&colors)
    }

    /// Set the color of every key LED at once.
    ///
    /// `colors[i]` is the color of key `i` (0 = left-most key). Keys without
    /// an entry are switched off, surplus entries are ignored.
    pub fn set_key_leds(&self, colors: &[KeyColor]) -> Result<(), KeyboardError> {
        let device = self
            .device
            .as_ref()
            .ok_or(KeyboardError::KeyboardNotConnected)?;

        let mut report = vec![0u8; self.report_size];
        report[0] = self.key_leds_adress;
        for (pos, color) in colors.iter().take(self.keys).enumerate() {
            set_single_key_color(&mut report, pos, *color);
        }
        device
            .write(&report)
            .map_err(|e| KeyboardError::CannotWriteData(e.to_string()))?;
        Ok(())
    }

    pub fn set_highlight_color(&mut self, color: KeyColor) -> Result<(), KeyboardError> {
        self.keys_hightlight_color = color;
        self.highlight_scale()?;
        Ok(())
    }

    pub fn set_dim_color(&mut self, color: KeyColor) -> Result<(), KeyboardError> {
        self.keys_dim_color = color;
        self.highlight_scale()?;
        Ok(())
    }

    // ---------------------------------------------------------------------
    // Callbacks
    // ---------------------------------------------------------------------

    /// Register a callback for **all** button presses and releases.
    ///
    /// The callback receives the device itself, so it can e.g. change the
    /// scale or write to the display.
    pub fn on_button<F>(&mut self, f: F)
    where
        F: FnMut(&mut TargetDevice, Button, bool) + Send + 'static,
    {
        self.callbacks.button.push(Box::new(f));
    }

    /// Register a callback for piano key presses and releases (needs MIDI).
    pub fn on_key<F>(&mut self, f: F)
    where
        F: FnMut(&mut TargetDevice, KeyEvent) + Send + 'static,
    {
        self.callbacks.key.push(Box::new(f));
    }

    /// Register a callback for the pitch / modulation touch strips, pedals and
    /// other MIDI controllers (needs MIDI).
    pub fn on_slider<F>(&mut self, f: F)
    where
        F: FnMut(&mut TargetDevice, Slider, i32) + Send + 'static,
    {
        self.callbacks.slider.push(Box::new(f));
    }

    /// Register a callback for the 8 knobs and the main encoder.
    pub fn on_encoder<F>(&mut self, f: F)
    where
        F: FnMut(&mut TargetDevice, EncoderEvent) + Send + 'static,
    {
        self.callbacks.encoder.push(Box::new(f));
    }

    /// Register a callback for raw MIDI messages from the merged secondary MIDI device.
    pub fn on_midi_merged<F>(&mut self, f: F)
    where
        F: FnMut(&mut TargetDevice, &[u8]) + Send + 'static,
    {
        self.callbacks.midi_merged.push(Box::new(f));
    }

    /// Remove all registered callbacks.
    pub fn clear_callbacks(&mut self) {
        self.callbacks = Callbacks::default();
    }

    /// Read all pending input (HID + MIDI), invoke the registered callbacks
    /// and return the events.
    ///
    /// Blocks for at most `timeout_ms` waiting for a HID report. Keep the
    /// timeout small (e.g. 5 ms), MIDI key events are only delivered while
    /// this function is called.
    pub fn poll(&mut self, timeout_ms: i32) -> Result<Vec<InputEvent>, KeyboardError> {
        let mut events = Vec::new();
        let mut had_midi = false;

        // 1. Keyboard MIDI (keys, touch strips) - already buffered by the midir thread
        if let Some(link) = &self.midi_keyboard_link {
            while let Ok(msg) = link.rx.try_recv() {
                had_midi = true;
                if let Some(ev) = midi::parse_message(&msg, self.first_note, self.keys) {
                    events.push(ev);
                }
            }
        }

        // 2. Merged MIDI device (forward raw messages directly to MIDI output)
        let mut merged_msgs = Vec::new();
        if let Some(link) = &self.midi_merge_link {
            while let Ok(msg) = link.rx.try_recv() {
                had_midi = true;
                for out in &mut self.midi_outs {
                    let _ = out.send_raw(&msg);
                }
                merged_msgs.push(msg);
            }
        }

        // 3. HID (buttons, encoders). Don't block if MIDI already delivered something.
        if let Some(device) = &self.device {
            let mut buf = [0u8; 64];
            let timeout = if had_midi { 0 } else { timeout_ms };
            let n = device
                .read_timeout(&mut buf, timeout)
                .map_err(|e| KeyboardError::CannotReadData(e.to_string()))?;
            if n > 0 {
                self.parse_hid_report(&buf[..n], &mut events);
            }
        } else if !had_midi && timeout_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(timeout_ms as u64));
        }

        // Invoke callbacks for merged MIDI input
        if !merged_msgs.is_empty() && !self.callbacks.midi_merged.is_empty() {
            let mut cbs = std::mem::take(&mut self.callbacks);
            for msg in &merged_msgs {
                for cb in cbs.midi_merged.iter_mut() {
                    cb(self, msg);
                }
            }
            self.callbacks = cbs;
        }

        self.dispatch(&events);
        Ok(events)
    }

    /// Run [`Self::poll`] forever. Only returns on error.
    pub fn run(&mut self) -> Result<(), KeyboardError> {
        loop {
            self.poll(5)?;
        }
    }

    /// Kept for backwards compatibility: polls once (100 ms timeout) and
    /// dispatches the callbacks.
    pub fn read_input(&mut self) -> Result<(), KeyboardError> {
        self.poll(100).map(|_| ())
    }

    fn parse_hid_report(&mut self, buf: &[u8], events: &mut Vec<InputEvent>) {
        // Only report 0x01 contains the button / encoder state
        if buf.first() != Some(&0x01) || buf.len() < 1 + BUTTON_BYTES {
            return;
        }

        // Buttons: bitfield in bytes 1..=5
        for byte in 0..BUTTON_BYTES {
            let new = buf[1 + byte];
            let changed = new ^ self.button_state[byte];
            if changed == 0 {
                continue;
            }
            for bit in 0..8u8 {
                if changed & (1 << bit) != 0 {
                    events.push(InputEvent::Button {
                        button: Button::from_bit(byte as u8 * 8 + bit),
                        pressed: new & (1 << bit) != 0,
                    });
                }
            }
            self.button_state[byte] = new;
        }

        // Encoders: byte 6 = main encoder (4 bit), bytes 7..=22 = 8 knobs (10 bit, LE)
        if buf.len() >= 23 {
            let mut values = [0u16; NUM_ENCODERS];
            values[0] = (buf[6] & 0x0F) as u16;
            for i in 0..8 {
                let lo = buf[7 + i * 2] as u16;
                let hi = buf[8 + i * 2] as u16;
                values[i + 1] = (lo | (hi << 8)) & 0x3FF;
            }

            if let Some(old) = self.encoder_state {
                for (index, (&new, &prev)) in values.iter().zip(old.iter()).enumerate() {
                    if new == prev {
                        continue;
                    }
                    let modulo: i32 = if index == 0 { 16 } else { 1024 };
                    let half = modulo / 2;
                    let delta = (new as i32 - prev as i32 + half).rem_euclid(modulo) - half;
                    events.push(InputEvent::Encoder(EncoderEvent {
                        index,
                        delta,
                        raw: new,
                    }));
                }
            }
            self.encoder_state = Some(values);
        }

        // Byte 37: MIDI note of the left-most key (changes with octave up/down)
        if let Some(&note) = buf.get(37) {
            if note > 0 && note < 128 {
                self.first_note = note;
            }
        }
    }

    fn dispatch(&mut self, events: &[InputEvent]) {
        if events.is_empty() {
            return;
        }

        for out in &mut self.midi_outs {
            for ev in events {
                let _ = out.send_input_event(ev);
            }
        }

        // Temporarily take the callbacks out so they can borrow `self` mutably
        let mut cbs = std::mem::take(&mut self.callbacks);
        for ev in events {
            match *ev {
                InputEvent::Button { button, pressed } => {
                    for cb in cbs.button.iter_mut() {
                        cb(self, button, pressed);
                    }
                }
                InputEvent::Key(key) => {
                    for cb in cbs.key.iter_mut() {
                        cb(self, key);
                    }
                }
                InputEvent::Slider { slider, value } => {
                    for cb in cbs.slider.iter_mut() {
                        cb(self, slider, value);
                    }
                }
                InputEvent::Encoder(enc) => {
                    for cb in cbs.encoder.iter_mut() {
                        cb(self, enc);
                    }
                }
            }
        }
        // Keep callbacks that were registered from inside a callback
        // cbs.append(&mut self.callbacks);
        self.callbacks = cbs;
    }

    // ---------------------------------------------------------------------
    // Display
    // ---------------------------------------------------------------------

    pub fn display_text(
        &mut self,
        display: usize,
        line: usize,
        text: &str,
        align: TextAlignment,
    ) -> Result<(), KeyboardError> {
        self.display.set_text(display, line, text, align);
        self.flush_display()
    }

    /// Show text spread over all 9 displays (8 chars each, 72 per line).
    pub fn display_message_stretch(
        &mut self,
        top: &str,
        bottom: &str,
        align: TextAlignment,
    ) -> Result<(), KeyboardError> {
        for (line, text) in [top, bottom].into_iter().enumerate() {
            let chunks = display::split_across_displays(text);
            for d in 0..display::NUM_DISPLAYS {
                let chunk = chunks.get(d).map(String::as_str).unwrap_or("");
                self.display.set_text(d, line, chunk, align);
            }
        }
        self.flush_display()
    }

    /// Show a bar graph (0.0 ..= 1.0) above the text of one display.
    pub fn display_bar(&mut self, display: usize, value: f32) -> Result<(), KeyboardError> {
        self.display.set_bar(display, value);
        self.flush_display()
    }

    /// Blank all displays.
    pub fn clear_display(&mut self) -> Result<(), KeyboardError> {
        self.display.clear();
        self.flush_display()
    }

    /// Direct access to the display buffer for batching several changes.
    /// Call [`Self::flush_display`] afterwards.
    pub fn display_buffer_mut(&mut self) -> &mut DisplayBuffer {
        &mut self.display
    }

    /// Send the display buffer to the hardware.
    pub fn flush_display(&self) -> Result<(), KeyboardError> {
        let device = self
            .device
            .as_ref()
            .ok_or(KeyboardError::KeyboardNotConnected)?;
        for report in self.display.reports() {
            device
                .write(&report)
                .map_err(|e| KeyboardError::CannotWriteData(e.to_string()))?;
        }
        Ok(())
    }
}

fn set_single_key_color(report: &mut [u8], key: usize, color: KeyColor) {
    let offset = 1 + key * 3;
    if offset + 3 <= report.len() {
        report[offset..offset + 3].copy_from_slice(&color.to_bytes());
    }
}
