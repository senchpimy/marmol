use eframe::egui;
use egui::text::LayoutJob;
use egui::{ TextFormat, FontId, Color32, Stroke, TextStyle, RichText };

mod files;
mod syntax_highlighting;



fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(Marmol::default())),
    )
}

struct Marmol {
    buffer: String,
}

impl Default for Marmol {
    fn default() -> Self {
        Self {
            buffer: "Arthur".to_owned(),
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut job2 = LayoutJob::default();
         let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = LayoutJob::default();//crate::syntax_highlighting::highlight(ui.ctx(), &theme, string, language);
            println!("{}",string);
            layout_job.append(
                string,
    10.0,
    TextFormat {
        color: Color32::WHITE,
        ..Default::default()
    },
);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.buffer)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
                );
           // syntax_highlighting::word_to_canvas(files::read_file("test.md"),&mut job2,ui);
        });
    }
}

