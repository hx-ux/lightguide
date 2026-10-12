use backend::{keyboard_error::KeyboardError, targetdevice::TargetDevice};

pub struct AutomaticListenMode {}

impl AutomaticListenMode {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_listen(&mut self, tg: &mut TargetDevice) -> Result<(), KeyboardError> {
        tg.on_button(|_dev, button, pressed| {
            println!(
                "BUTTON {:?} {}",
                button,
                if pressed { "down" } else { "up" }
            );
        });
        tg.on_key(|_dev, key| {
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

        loop {
            tg.poll(5)?.len();
        }
    }
}
