use backend::midi::list_midi_ports;
use backend::targetdevice::TargetDevice;
use backend::{display::TextAlignment, keyboard_error::KeyboardError};
use clap::{Parser, Subcommand};
use harmonia::{PitchClass, ScaleKind};

#[derive(Parser)]
#[command(
    name = "lightguide",
    about = "CLI controller for the NI Komplete Kontrol S49 MK1"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Turn off all key lights
    Off,
    /// Highlight a musical scale on the keyboard
    Scale {
        #[arg(default_value = "C")]
        root: String,
        #[arg(default_value = "Ionian")]
        scale: String,
    },
    /// Print all input events (buttons, keys, sliders, knobs)
    Listen {
        /// Stop after N events (0 = run forever)
        #[arg(default_value_t = 0)]
        count: usize,
    },
    /// Show text on the displays (spread over all 9 displays, 2 rows with 8 chars per display )
    Display {
        #[arg(default_value = "Hello")]
        upper_text: String,
        #[arg(default_value = "World")]
        lower_text: String,
    },
    Bar {
        #[arg(long, default_value_t = 0)]
        display: usize,
        #[arg(long, default_value_t = 1.0)]
        value: f32,
    },
    /// List all MIDI input ports (to find the keyboard's port name)
    MidiPorts,
    /// Control the scale via the hardware itself
    Automatic {},
}

pub fn main() -> Result<(), KeyboardError> {
    let cli = Cli::parse();

    let mut tg = TargetDevice::default();
    // sudo chmod a+rw /dev/hidraw10
    // sudo chmod 600 /dev/hidraw10
    tg.open()?;

    match cli.command {
        Commands::Off => match tg.notes_off() {
            Ok(_) => println!("lights off"),
            Err(err) => println!("Err: {}", err),
        },
        Commands::Scale { root, scale } => {
            match tg.set_scale_string(format!("{} {}", root.trim().to_uppercase(), scale.trim())) {
                Ok(_) => println!("set scale {}-{}", root, scale),
                Err(err) => println!("Err: {}", err),
            };
        }
        Commands::Listen { count } => {
            match tg.midi_port_name() {
                Some(name) => println!("MIDI port: {name}"),
                None => println!("MIDI port not connected - no key/slider events"),
            }

            tg.on_button(|dev, button, pressed| {
                println!(
                    "BUTTON {:?} {}",
                    button,
                    if pressed { "down" } else { "up" }
                );
            });
            tg.on_key(|dev, key| {
                println!(
                    "KEY note={} index={:?} vel={} {}",
                    key.note,
                    key.key_index,
                    key.velocity,
                    if key.pressed { "down" } else { "up" }
                );
            });

            tg.on_slider(|_dev, slider, value| {
                println!("SLIDER {:?} = {}", slider, value);
            });

            tg.on_encoder(|_dev, enc| {
                println!("ENCODER {} delta={} raw={}", enc.index, enc.delta, enc.raw);
            });

            let mut n = 0;
            loop {
                n += tg.poll(5)?.len();
                if count != 0 && n >= count {
                    break;
                }
            }
        }
        Commands::Display {
            upper_text: top,
            lower_text: bottom,
        } => {
            match tg.display_message_stretch(&top, &bottom, TextAlignment::Center) {
                Ok(_) => println!("set text {} {}", top, bottom),
                Err(err) => println!("Err: {}", err),
            };
        }
        Commands::MidiPorts => {
            for p in list_midi_ports()? {
                println!("{p}");
            }
        }
        Commands::Automatic {} => {
            tg.on_button(|dev, button, pressed| {
                println!(
                    "BUTTON {:?} {}",
                    button,
                    if pressed { "down" } else { "up" }
                );
            });
            tg.on_key(|dev, key| {
                println!(
                    "KEY note={} index={:?} vel={} {}",
                    key.note,
                    key.key_index,
                    key.velocity,
                    if key.pressed { "down" } else { "up" }
                );
            });

            tg.on_slider(|_dev, slider, value| {
                println!("SLIDER {:?} = {}", slider, value);
            });

            // GLOBAL
            // set to c by default;
            let pitch = 0;
            let _ = tg.set_scale(PitchClass::new(0), ScaleKind::Ionian);
            // 0 == performange view

            let mut n = 0;
            loop {
                n += tg.poll(5)?.len();
            }
        }
        Commands::Bar { display, value } => tg.display_bar(display, value)?,
    }
    Ok(())
}
