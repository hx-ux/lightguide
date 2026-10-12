mod automatic;
mod listen;
use automatic::AutomaticMode;
use backend::midi::list_midi_ports;
use backend::targetdevice::TargetDevice;
use backend::{display::TextAlignment, keyboard_error::KeyboardError};
use clap::{Parser, Subcommand};

use crate::listen::AutomaticListenMode;

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
    /// Print all input events (buttons, keys, sliders, knobs)
    Listen,
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
    /// Control the keyboard via the hardware itself (SCALE button toggles the mode)
    Automatic,
}

pub fn main() -> Result<(), KeyboardError> {
    let cli = Cli::parse();

    let mut tg = TargetDevice::default();
    // sudo chmod a+rw /dev/hidraw10
    // sudo chmod 600 /dev/hidraw10
    tg.open_hid_device_midi()?;

    match cli.command {
        Commands::Off => match tg.notes_off() {
            Ok(_) => println!("lights off"),
            Err(err) => println!("Err: {}", err),
        },
        Commands::Listen => {
            let mut listen = AutomaticListenMode::new();
            println!("listen mode - press Ctrl+C to quit",);
            let _ = listen.run_listen(&mut tg);
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
            match tg.midi_port_name() {
                Some(name) => println!("MIDI port: {name}"),
                None => println!("MIDI port not connected - no key/slider events"),
            }
            println!("Aviable ports");
            for p in list_midi_ports()? {
                println!("{p}");
            }
        }
        Commands::Automatic => {
            let mut automatic = AutomaticMode::new();

            println!("automatic mode - press SCALE to toggle Performance / Scale, Ctrl+C to quit",);
            automatic.run(&mut tg)?;
        }
        Commands::Bar { display, value } => tg.display_bar(display, value)?,
    }
    Ok(())
}
