use backend::{
    keycolor::KeyColor,
    targetdevice::TargetDevice,
    utils::{HarmoniaUtils, KeyboardKey, KeyboardKeyType},
};
use egui::ScrollArea;
use harmonia::{PitchClass, ScaleKind};

#[derive(Clone)]
enum MessageType {
    Success,
    Error,
}

struct ConsoleMessage {
    text: String,
    msg_type: MessageType,
}

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "LightGuide",
        options,
        Box::new(|_cc| Ok(Box::<LightGuideApp>::default())),
    )
}

struct LightGuideApp {
    device: TargetDevice,
    is_connected: bool,
    sel_pitch: PitchClass,
    sel_scale: ScaleKind,
    messages: Vec<ConsoleMessage>,
}

impl Default for LightGuideApp {
    fn default() -> Self {
        Self {
            device: TargetDevice::default(),
            is_connected: false,
            sel_pitch: PitchClass::C,
            sel_scale: ScaleKind::Ionian,
            messages: Vec::new(),
        }
    }
}

impl LightGuideApp {
    pub fn log_event(&mut self, text: String, msg_type: MessageType) {
        self.messages.push(ConsoleMessage {
            text: text.to_string(),
            msg_type,
        });
    }
}

impl eframe::App for LightGuideApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading(format!("Lightguide {}", self.device.name.clone()));
            ui.heading(format!("connect {}", self.is_connected.to_string()));

            egui::ComboBox::from_label("Root")
                .selected_text(HarmoniaUtils::normalize_pitch_names(&self.sel_pitch))
                .show_ui(ui, |ui| {
                    for note in HarmoniaUtils::pitch_class_selection().iter() {
                        ui.selectable_value(
                            &mut self.sel_pitch,
                            note.clone(),
                            HarmoniaUtils::normalize_pitch_names(&note),
                        );
                    }
                });

            if self.sel_pitch != self.device.root_note {
                let _ = self.device.set_root(self.sel_pitch.clone());
            }

            if ui.button("Connect").clicked() {
                let r = self.device.open();
                match r {
                    Ok(_) => {
                        self.is_connected = true;
                        let _ = self.device.notes_off();
                        let _ = self
                            .device
                            .set_scale(self.sel_pitch.clone(), self.sel_scale.clone());
                        self.log_event("Connected".to_string(), MessageType::Success);
                    }
                    Err(err) => {
                        self.log_event(err.to_string(), MessageType::Error);
                    }
                }
            }

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

            if self.sel_scale != self.device.scale {
                let _ = self.device.set_scale_kind(self.sel_scale.clone());
            }

            draw_keyboard(
                ui,
                self.device.keyboard_keys(),
                self.device.keys_hightlight_color,
            );

            ScrollArea::vertical()
                .max_height(200.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for msg in &self.messages {
                        let color = match msg.msg_type {
                            MessageType::Success => egui::Color32::LIGHT_GREEN,
                            MessageType::Error => egui::Color32::LIGHT_RED,
                        };
                        ui.colored_label(color, &msg.text);
                    }
                });
        });
    }
}

fn draw_keyboard(ui: &mut egui::Ui, keys: Vec<KeyboardKey>, highlight: KeyColor) {
    let available = ui.available_width();
    let white_w = available
        / keys
            .clone()
            .into_iter()
            .filter(|x| x.ktype == KeyboardKeyType::White)
            .count() as f32;
    let black_w = white_w * 0.6;
    let height = 100.0;
    let black_h = height * 0.62;

    let (_, painter) = ui.allocate_painter(egui::vec2(available, height), egui::Sense::hover());

    let to_color = |c: KeyColor| egui::Color32::from_rgb(c.r, c.g, c.b);
    let stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);

    let mut white_index: u8 = 0;

    for i in keys.clone() {
        if i.ktype == KeyboardKeyType::Black {
            continue;
        }
        let x = white_index as f32 * white_w;
        white_index += 1;

        let outer_rect =
            egui::Rect::from_min_size(egui::pos2(x, 100.0), egui::vec2(white_w, height));

        let highlight_rect =
            egui::Rect::from_min_size(egui::pos2(x, 100.0), egui::vec2(white_w, height / 4.00));

        painter.rect_filled(outer_rect, egui::CornerRadius::ZERO, egui::Color32::WHITE);

        if i.active {
            painter.rect_filled(
                highlight_rect,
                egui::CornerRadius::ZERO,
                to_color(highlight),
            );
        }

        painter.rect_stroke(
            outer_rect,
            egui::CornerRadius::ZERO,
            stroke,
            egui::StrokeKind::Outside,
        );

        painter.text(
            outer_rect.center(),
            egui::Align2::CENTER_CENTER,
            i.display_name.clone(),
            egui::FontId::proportional(10.00),
            egui::Color32::BLACK,
        );
    }

    // black keys
    let mut white_index = 0;
    for i in keys.clone() {
        if i.ktype == KeyboardKeyType::Black {
            let x = white_index as f32 * white_w - black_w / 2.0;
            let outer_rect =
                egui::Rect::from_min_size(egui::pos2(x, 100.00), egui::vec2(black_w, black_h));

            let highlight_rect = egui::Rect::from_min_size(
                egui::pos2(x, 100.00),
                egui::vec2(black_w, black_h / 2.00),
            );

            painter.rect_filled(outer_rect, egui::CornerRadius::ZERO, egui::Color32::BLACK);

            if i.active {
                painter.rect_filled(
                    highlight_rect,
                    egui::CornerRadius::ZERO,
                    to_color(highlight),
                );
            }

            painter.rect_stroke(
                outer_rect,
                egui::CornerRadius::ZERO,
                stroke,
                egui::StrokeKind::Outside,
            );

            painter.text(
                outer_rect.center(),
                egui::Align2::CENTER_BOTTOM,
                i.display_name.clone(),
                egui::FontId::proportional(10.0),
                egui::Color32::WHITE,
            );
        } else {
            white_index += 1;
        }
    }
}
