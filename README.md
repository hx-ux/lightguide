# Lightguide

"Replacement" for the now discontinued Komplete Kontrol Software for older hardware
Currently, only the Komplete Kontrol S49 MK1 is supported

## Features

- Change key-colors
- Callbacks for buttons, keys, sliders (touch strips / pedals) and knobs
- Show text and value bars on the displays
- Standalone **Automatic Mode**, which tries to recreate the behaviour of the **Komplete Kontrol** software
- **Midi Output**, supports midi output via
    - Built-In Midi Port
    - External Midi Interface

## Test the keyboard

```bash
cargo run --bin Lightguide_CLI -- listen           # print all input events
cargo run --bin Lightguide_CLI -- display "HELLO" "WORLD"
cargo run --bin Lightguide_CLI -- midi-ports       # list all MIDI inputs & outputs
```

## Automatic Mode

You can run the automatic mode via:
`cargo run -p cli -- automatic`

You can also create your own config:
`cargo run -p cli -- save-config config.json`

And run automatic mode with it
`cargo run -p cli -- --config config.json automatic`

## Raspberry Pi

On the Raspberry Pi the MIDI support needs the ALSA headers:

```bash
sudo apt install libasound2-dev
sudo apt-get install libudev-dev
```

Then, set read/write/ permissions for your device
`sudo chmod a+rw /dev/hidraw0`

## Library usage

```rust
let mut tg = TargetDevice::default();
tg.open()?;
tg.on_button(|dev, button, pressed| println!("{button:?} {pressed}"));
tg.on_key(|dev, key| println!("{key:?}"));
tg.on_slider(|dev, slider, value| println!("{slider:?} {value}"));
tg.on_encoder(|dev, enc| println!("{enc:?}"));
tg.display_message("LIGHTGUIDE", "READY")?;
tg.run()?; // or call tg.poll(5) in your own loop
```
