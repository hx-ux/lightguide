use backend::{
    keycolor::KeyColor,
    targetdevice::{KeyboardError, TargetDevice},
};
use clap::{Parser, Subcommand};
use std::{thread, time::Duration};

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

pub fn main() -> Result<(), KeyboardError> {
    let cli = Cli::parse();

    let mut tg = TargetDevice::default();
    // sudo chmod a+rw /dev/hidraw10
    // sudo chmod 600 /dev/hidraw10
    tg.open()?;

    match cli.command {
        Commands::Off => {
            tg.notes_off()?;
            println!("ok: lights off");
        }
        Commands::Scale { root, scale } => {
            let concat = format!("{} {}", root.trim().to_uppercase(), scale.trim());
            tg.set_scale_string(concat)?;
            tg.notes_off()?;
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
