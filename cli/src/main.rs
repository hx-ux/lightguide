mod modes;
use crate::modes::automatic::arp::Arpeggiator;
use crate::modes::automatic::AutomaticMode;
use crate::modes::listen::AutomaticListenMode;
use backend::midi::{list_midi_out_ports, list_midi_ports};
use backend::targetdevice::TargetDevice;
use backend::{display::TextAlignment, keyboard_error::DeviceError};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "lightguide",
    about = "CLI controller and MIDI merger for the NI Komplete Kontrol S49 MK1"
)]
struct Cli {
    /// Optional path to a JSON configuration file
    #[arg(short, long)]
    #[arg(default_value = "device.json")]
    device_config: std::path::PathBuf,
    #[arg(default_value = "arp.json")]
    arp_config: std::path::PathBuf,
    /// Override the MIDI output devices (repeat the flag for multiple outputs)
    #[arg(long)]
    midi_out: Vec<String>,
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
    InitConfig {},
}

fn load_config<'a, T: Default>(
    name: &str,
    path: Option<&'a std::path::Path>,
    read: impl FnOnce(&'a std::path::Path) -> Result<T, DeviceError>,
) -> Result<T, DeviceError> {
    if let Some(p) = path.filter(|p| p.exists()) {
        println!("Load {name} config from {}", p.display());
        read(p)
    } else {
        println!("No config loaded for {name}\nFallback to default");
        Ok(T::default())
    }
}

pub fn main() -> Result<(), DeviceError> {
    let cli = Cli::parse();

    let should_load = matches!(
        cli.command,
        Commands::Off | Commands::Listen | Commands::Display { .. } | Commands::Automatic
    );

    let mut tg = load_config(
        "device",
        should_load.then_some(cli.device_config.as_path()),
        TargetDevice::read,
    )?;
    let arp = load_config(
        "arp",
        should_load.then_some(cli.arp_config.as_path()),
        Arpeggiator::read,
    )?;

    if !cli.midi_out.is_empty() {
        tg.midi_out_devices = cli.midi_out;
    }

    match cli.command {
        Commands::InitConfig {} => {
            tg.save("device.json")?;
            arp.save("arp.json")?;
            println!("Device configuration saved to device.json");
            println!("Arp configuration saved to arp.json");
            return Ok(());
        }
        _ => {}
    }

    if should_load {
        // sudo chmod a+rw /dev/hidraw10
        // sudo chmod 600 /dev/hidraw10
        tg.open_hid_device()?;
    }

    match cli.command {
        Commands::InitConfig { .. } => {}
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
            let mut automatic = AutomaticMode::new(arp);

            println!("automatic mode - press SCALE to toggle Performance / Scale, Ctrl+C to quit");
            for (filter, res) in tg.open_midi_outs() {
                match res {
                    Ok(_) => println!("MIDI out connected to {filter}"),
                    Err(e) => eprintln!("{e}"),
                }
            }
            automatic.run(&mut tg)?;
        }
        Commands::Bar { display, value } => tg.display_bar(display, value)?,
    }
    Ok(())
}
