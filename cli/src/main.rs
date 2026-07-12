use backend::{keycolor::KeyColor, targetdevice::TargetDevice};
use clap::{Parser, Subcommand};
use std::{error::Error, thread, time::Duration};

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
    /// Poll the device and print raw input reports (keys, sliders, pitch, mod)
    Listen {
        /// Stop after N reports (0 = run forever)
        #[arg(default_value_t = 0)]
        count: usize,
    },
    /// Control the scale via the hardware itself
    Automatic {},
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut tg = TargetDevice::new(
        "kontrol49mk1".to_string(),
        49,
        250,
        0x82,
        0x17cc,
        0x1350,
        KeyColor::new(255, 0, 0),
        KeyColor::new(0, 0, 0),
        KeyColor::new(0, 100, 0),
    );
    // sudo chmod a+rw /dev/hidraw10
    // sudo chmod 600 /dev/hidraw10
    tg.open()?;

    match cli.command {
        Commands::Off => {
            tg.notes_off()?;
            println!("ok: lights off");
        }
        Commands::Scale { root, scale } => {
            tg.notes_off()?;
            let concat = format!("{} {}", root.trim().to_uppercase(), scale.trim());
            tg.set_scale_string(concat)?;
        }
        Commands::Listen { count } => {
            let mut n = 0;
            loop {
                tg.read_input()?;
                n += 1;
                if count != 0 && n >= count {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        Commands::Automatic {} => {}
    }
    Ok(())
}
