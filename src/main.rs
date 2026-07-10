use clap::{Parser, Subcommand};
use std::error::Error;
use std::thread;
use std::time::Duration;

use crate::keycolor::KeyColor;
use crate::targetdevice::TargetDevice;
mod keycolor;
mod targetdevice;
const VENDOR_ID: u16 = 0x17cc;
const PRODUCT_ID: u16 = 0x1350;
const LED_ADRESS: u8 = 0x82; // MK1 LED report (0x81 for MK2)
const REPORT_SIZE: usize = 250;
const NUM_KEYS: usize = 49;

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
        /// Root note name, e.g. c, d, f#
        #[arg(default_value = "c")]
        root: String,
        /// Scale name, e.g. major, minor
        #[arg(default_value = "major")]
        name: String,
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

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut tg = TargetDevice::new(
        "kontrol49mk1".to_string(),
        NUM_KEYS,
        REPORT_SIZE,
        LED_ADRESS,
        VENDOR_ID,
        PRODUCT_ID,
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
        Commands::Scale { root, name } => {
            tg.notes_off()?;
            let on_color = KeyColor::new(255, 0, 0);
            let off_color = KeyColor::new(0, 0, 0);

            tg.highlight_scale(&root, &name, on_color, off_color)?;
            println!("ok: scale {root} {name}");
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
        Commands::Automatic {} => todo!(),
    }
    Ok(())
}
