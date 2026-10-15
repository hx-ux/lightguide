mod modes;
use backend::midi::{list_midi_out_ports, list_midi_ports};
use backend::targetdevice::TargetDevice;
use backend::{display::TextAlignment, keyboard_error::KeyboardError};
use clap::{Parser, Subcommand};

use crate::modes::automatic::AutomaticMode;
use crate::modes::listen::AutomaticListenMode;

#[derive(Parser)]
#[command(
    name = "lightguide",
    about = "CLI controller and MIDI merger for the NI Komplete Kontrol S49 MK1"
)]
struct Cli {
    /// Optional path to a JSON configuration file
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    /// Override the MIDI input device to merge with keyboard
    #[arg(long)]
    midi_in: Option<String>,

    /// Override the MIDI output device
    #[arg(long)]
    midi_out: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, PartialEq)]
enum Commands {
    /// Turn off all key lights
    Off,
    /// Print all input events (buttons, keys, sliders, knobs)
    Listen,
    /// Show text on the displays (spread over all 9 displays, 2 rows with 8 chars per display)
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
    /// List all available MIDI input and output ports
    MidiPorts,
    /// Standalone Automatic Mode
    Automatic,
    SaveConfig {
        #[arg(default_value = "config.json")]
        path: std::path::PathBuf,
    },
    PrintConfig,
}

pub fn main() -> Result<(), KeyboardError> {
    let cli = Cli::parse();

    let mut tg = match &cli.config {
        Some(path) => {
            println!("Loading config from {}", path.display());
            TargetDevice::read(path)?
        }
        None => TargetDevice::default(),
    };

    if let Some(midi_out) = cli.midi_out {
        tg.midi_out_device = midi_out;
    }

    match cli.command {
        Commands::SaveConfig { path } => {
            tg.save(&path)?;
            println!("Configuration saved to {}", path.display());
            return Ok(());
        }
        Commands::PrintConfig => {
            println!("{}", tg.to_json()?);
            return Ok(());
        }
        _ => {}
    }

    if matches!(
        cli.command,
        Commands::Off | Commands::Listen | Commands::Display { .. } | Commands::Automatic
    ) {
        // sudo chmod a+rw /dev/hidraw10
        // sudo chmod 600 /dev/hidraw10
        tg.open_hid_device()?;
    }

    match cli.command {
        Commands::SaveConfig { .. } | Commands::PrintConfig => unreachable!(),
        Commands::Off => match tg.notes_off() {
            Ok(_) => println!("lights off"),
            Err(err) => println!("Err: {}", err),
        },
        Commands::Listen => {
            let mut listen = AutomaticListenMode::new();
            println!("listen mode - press Ctrl+C to quit");
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
            println!("Available MIDI Input ports:");
            match list_midi_ports() {
                Ok(ports) if ports.is_empty() => println!("  (none found)"),
                Ok(ports) => {
                    for (i, p) in ports.iter().enumerate() {
                        println!("  [{i}] {p}");
                    }
                }
                Err(e) => eprintln!("Error listing MIDI input ports: {e}"),
            }

            println!("\nAvailable MIDI Output ports:");
            match list_midi_out_ports() {
                Ok(ports) if ports.is_empty() => println!("  (none found)"),
                Ok(ports) => {
                    for (i, p) in ports.iter().enumerate() {
                        println!("  [{i}] {p}");
                    }
                }
                Err(e) => eprintln!("Error listing MIDI output ports: {e}"),
            }
        }
        Commands::Automatic => {
            let mut automatic = AutomaticMode::new();

            println!("automatic mode - press SCALE to toggle Performance / Scale, Ctrl+C to quit");
            match tg.open_midi_out(&tg.midi_out_device.to_string()) {
                Ok(_) => println!("MIDI out connected to {}", &tg.midi_out_device),
                Err(e) => eprintln!("{e}"),
            }
            automatic.run(&mut tg)?;
        }
        Commands::Bar { display, value } => tg.display_bar(display, value)?,
    }
    Ok(())
}
