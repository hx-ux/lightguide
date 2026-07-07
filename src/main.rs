use clap::{Parser, Subcommand};
use hidapi::{HidApi, HidDevice};
use std::error::Error;
use std::thread;
use std::time::Duration;

use crate::keycolor::KeyColor;
mod keycolor;
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

// ---------------------------------------------------------------------------
// Device helpers
// ---------------------------------------------------------------------------
fn open_device() -> Result<HidDevice, Box<dyn Error>> {
    let api = HidApi::new()?;
    let device = api.open(VENDOR_ID, PRODUCT_ID)?;
    const STARTUP: [u8; 3] = [0xa0, 0x00, 0x00];
    let _ = device.write(&STARTUP);
    Ok(device)
}

fn notes_off(device: &HidDevice) -> Result<(), Box<dyn Error>> {
    let mut report = [0u8; REPORT_SIZE];
    report[0] = LED_ADRESS;
    device.write(&report)?;
    Ok(())
}

fn set_key_color(report: &mut [u8; REPORT_SIZE], key: usize, color: KeyColor) {
    let offset = 1 + key * 3;
    if offset + 3 <= REPORT_SIZE {
        report[offset..offset + 3].copy_from_slice(&color.to_bytes());
    }
}

fn scale_intervals(name: &str) -> Option<[usize; 7]> {
    match name.to_lowercase().as_str() {
        "major" => Some([0, 2, 4, 5, 7, 9, 11]),
        "minor" => Some([0, 2, 3, 5, 7, 8, 10]),
        "dorian" => Some([0, 2, 3, 5, 7, 9, 10]),
        "penta" | "pentatonic" => Some([0, 2, 4, 7, 9, 0, 0]),
        _ => None,
    }
}

fn root_semitone(root: &str) -> Option<usize> {
    match root.to_lowercase().as_str() {
        "c" => Some(0),
        "c#" | "db" => Some(1),
        "d" => Some(2),
        "d#" | "eb" => Some(3),
        "e" => Some(4),
        "f" => Some(5),
        "f#" | "gb" => Some(6),
        "g" => Some(7),
        "g#" | "ab" => Some(8),
        "a" => Some(9),
        "a#" | "bb" => Some(10),
        "b" => Some(11),
        _ => None,
    }
}

fn highlight_scale(
    device: &HidDevice,
    root: &str,
    name: &str,
    on_color: KeyColor,
    off_color: KeyColor,
) -> Result<(), Box<dyn Error>> {
    
    let root = root_semitone(root).ok_or("unknown root note")?;
    let intervals = scale_intervals(name).ok_or("unknown scale name")?;

    let mut report = [0u8; REPORT_SIZE];
    report[0] = LED_ADRESS;

    for key in 0..NUM_KEYS {
        let semitone = (key + root) % 12;
        let in_scale = intervals.iter().any(|&iv| (iv + root) % 12 == semitone);
        set_key_color(
            &mut report,
            key,
            if in_scale { on_color } else { off_color },
        );
    }

    device.write(&report)?;
    Ok(())
}

fn read_input(device: &HidDevice) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 64];
    match device.read_timeout(&mut buf, 100) {
        Ok(0) => {}
        Ok(_) => {
            println!("RAW {:02X?}", &buf[0..16]);
            match buf[0] {
                0x01 => {
                    let note = buf[1];
                    let vel = buf[2];
                    if vel > 0 {
                        println!("KEY {} VEL {}", note, vel);
                    } else {
                        println!("KEY_OFF {}", note);
                    }
                }
                0x02 => {
                    let cc = buf[1];
                    let value = buf[2];
                    match cc {
                        1 => println!("MOD {}", value),
                        7 => println!("VOL {}", value),
                        _ => println!("CC {} = {}", cc, value),
                    }
                }
                0x03 => {
                    let pitch = ((buf[2] as i16) << 7) | (buf[1] as i16);
                    println!("PITCH {}", pitch - 8192);
                }
                _ => {}
            }
        }
        Err(e) => eprintln!("read error: {e}"),
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let device = open_device()?;

    match cli.command {
        Commands::Off => {
            notes_off(&device)?;
            println!("ok: lights off");
        }
        Commands::Scale { root, name } => {
            notes_off(&device)?;
            let on_color = KeyColor::new(255, 0, 0);
            let off_color = KeyColor::new(0, 0, 0);

            highlight_scale(&device, &root, &name, on_color, off_color)?;
            println!("ok: scale {root} {name}");
        }
        Commands::Listen { count } => {
            let mut n = 0;
            loop {
                read_input(&device)?;
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
