mod keycolor;
mod targetdevice;
use hidapi::HidDevice;
use keycolor::ColorConverter;
use keycolor::KeyColor;
use std::error::Error;
use targetdevice::TargetDevice;

// sudo chmod a+rw /dev/hidraw10
// sudo chmod 600 /dev/hidraw10

fn highlight_c_major(device: &HidDevice, config: &TargetDevice) -> Result<(), Box<dyn Error>> {
    let mut report = [0u8; 250];
    report[0] = config.header.clone();

    let blue = KeyColor::new(0, 100, 255);
    let dim_white = ColorConverter::rgb_to_uint8(0, 0, 0);

    // C Major scale semitones from C (mod 12)
    let scale_semitones: [usize; 7] = [0, 2, 4, 5, 7, 9, 11];

    for key in 0..config.keys {
        let semitone = key % 12;
        let is_in_scale = scale_semitones.contains(&semitone);

        let rgb = if is_in_scale {
            blue.to_bytes()
        } else {
            dim_white
        };

        let offset = 1 + key * 3;
        if offset + 3 <= 250 {
            report[offset..offset + 3].copy_from_slice(&rgb);
        }
    }

    let bytes_written = device.write(&report)?;
    println!(
        "Highlighted C Major scale: {} byte(s) written ",
        bytes_written
    );
    Ok(())
}

fn main() {
    let config = TargetDevice::new("name".to_string(), 49, 250, 0x82, 0x17cc, 0x1350);

    let api = hidapi::HidApi::new().unwrap();

    let device = api
        .open(config.vendorId, config.deviceId)
        .expect("Failed to open device (check permissions: sudo chmod a+rw /dev/hidraw*)");

    const STARTUP_REPORT: [u8; 3] = [0xa0, 0x00, 0x00];

    let g = device.write(&STARTUP_REPORT);
    notes_off(&device).unwrap();

    // color(&device);
    highlight_c_major(&device, &config);

    println!("Wrote: {:?} byte(s)", g);

    // println!("{:#?}", device);
}

/// Turn off lights for all notes (equivalent to the Python `notes_off()`)
fn notes_off(device: &HidDevice) -> Result<(), Box<dyn Error>> {
    let header = 0x82;

    // 250-byte report: header byte + 249 zero bytes
    let mut report = [0u8; 250];
    report[0] = header;

    let bytes_written = device.write(&report)?;
    println!("Wrote notes_off report: {} byte(s)", bytes_written);
    Ok(())
}

fn color(device: &HidDevice) -> Result<(), Box<dyn Error>> {
    let header = 0x82;

    // 250-byte report: header byte + 249 zero bytes

    let color1 = ColorConverter::rgb_to_uint8(255, 128, 0);

    let color2 = KeyColor::new(0, 255, 64);
    let h = color2.to_bytes();
    let dd = [0x82, 0, 0, 255, 255, 0, 0];

    let bytes_written = device.write(&dd)?;
    println!("Wrote notes_off report: {} byte(s)", bytes_written);
    Ok(())
}

fn read_input(device: &HidDevice) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 64];
    match device.read_timeout(&mut buf, 100) {
        Ok(0) => return Ok(()), // Timeout
        Ok(_) => {
            println!(
                "Raw HID report: {:02X?} (first 16 bytes; capture more for sliders/pitch)",
                &buf[0..16]
            );

            match buf[0] {
                0x01 => {
                    // Key press (from projects)
                    let note = buf[1];
                    let vel = buf[2];
                    if vel > 0 {
                        println!(
                            "KEY PRESSED: MIDI note {} (velocity {}) - map to scale or uinput",
                            note, vel
                        );
                    } else {
                        println!("KEY RELEASED: MIDI note {}", note);
                    }
                }
                0x02 => {
                    // CC / Sliders / Mod wheel (CC#1 = mod, CC#7 = volume)
                    let cc = buf[1];
                    let value = buf[2];
                    match cc {
                        1 => println!("MOD WHEEL: {}", value),
                        7 => println!("VOLUME SLIDER: {}", value),
                        _ => println!("CC {} value: {} (slider or control)", cc, value),
                    }
                }
                0x03 => {
                    // Pitch bend (14-bit value, center 8192)
                    let pitch = ((buf[2] as i16) << 7) | (buf[1] as i16);
                    println!("PITCH BEND: {} (centered at 8192)", pitch - 8192);
                }
                _ => println!(
                    "Unknown report type; log full bytes for reverse-engineering (per komplementary-kontrol)"
                ),
            }
        }
        Err(e) => eprintln!("HID read error: {}", e),
    }
    Ok(())
}
