use backend::{keycolor::KeyColor, targetdevice::TargetDevice, utils::HarmoniaUtils};
use harmonia::{Note, PitchClass, Scale, ScaleKind};
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "LightGuide",
        options,
        Box::new(|cc| Ok(Box::<MyApp>::default())),
    )
}

struct MyApp {
    device: TargetDevice,
    is_connected: bool,
    sel_pitch: PitchClass,
    sel_scale: ScaleKind,
}

impl Default for MyApp {
    fn default() -> Self {
        let d = TargetDevice::new(
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

        Self {
            device: d,
            is_connected: false,
            sel_pitch: PitchClass::C,
            sel_scale: ScaleKind::Aeolian,
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Lightguide");
            ui.heading(self.is_connected.to_string());

            egui::ComboBox::from_label("Root")
                .selected_text(self.sel_pitch.to_string())
                .show_ui(ui, |ui| {
                    for radio_item in HarmoniaUtils::pitch_class_selection().iter() {
                        ui.selectable_value(
                            &mut self.sel_pitch,
                            radio_item.clone(),
                            &radio_item.clone().to_string(),
                        );
                    }
                });

            egui::ComboBox::from_label("Scale")
                .selected_text(self.sel_scale.clone().to_string())
                .show_ui(ui, |ui| {
                    for ss in ScaleKind::ALL {
                        ui.selectable_value(
                            &mut self.sel_scale,
                            ss.clone(),
                            &ss.clone().to_string(),
                        );
                    }
                });
            // if self.sel_scale != self.device.scale {
            //     self.device.set_scale_kind(self.sel_scale.clone());
            // }

            if ui.button("Connect").clicked() {
                let r = self.device.open();
                match r {
                    Ok(_) => {
                        self.is_connected = true;
                        // self.device
                        //     .set_scale(self.sel_pitch.clone(), self.sel_scale.clone());
                    }
                    Err(_) => {}
                }
            }
        });
    }
}
