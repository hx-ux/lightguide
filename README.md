# Lightguide

"Replacement" for the now discontinued Komplete Kontrol Software for older hardware
Currently, only the Komplete Kontrol S49 MK1 is supported

## Features

- Display Scales
- Change Key-Colors
- Callbacks for buttons, keys, sliders (touch strips / pedals) and knobs
- Text on the 9 displays

## Run

```
cargo run --bin Lightguide_CLI
```

Test commands:

```
cargo run --bin Lightguide_CLI -- listen           # print all input events
cargo run --bin Lightguide_CLI -- display "HELLO" "WORLD"
cargo run --bin Lightguide_CLI -- midi-ports       # list MIDI inputs
```

On the Raspberry Pi the MIDI support needs the ALSA headers:

```
sudo apt install libasound2-dev
```

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
