use eframe::egui;
use eframe::{egui::{CentralPanel,ScrollArea,Separator,TopBottomPanel,SidePanel,Context}};
use egui::text::LayoutJob;
use egui::{ TextFormat, FontId, Color32, Stroke, TextStyle, RichText };
use egui_demo_lib::easy_mark;

mod files;




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
/////////////////////////////////////////////////////////////////////////////////
        left_side_settings(ctx);
        let mut edit=easy_mark::EasyMarkEditor::default();
        CentralPanel::default().show(ctx, |ui| edit.ui(ui));
        right_side_settings(ctx);
/////////////////////////////////////////////////////////////////////////////////
    }
}

fn left_side_settings(ctx:&Context,){
    SidePanel::left("my_side_panel").show(ctx,|ui| {
         ui.label("lalal aqui van los contactos y grupos");
         for i in 0..10 {
            ui.horizontal(|ui| {
                ui.label(format!("{} aaaa papu",i));
            });
            ui.add(Separator::default().spacing(3.).horizontal());
        }
    });
}

fn right_side_settings(ctx:&Context,){
    SidePanel::right("my_right_panel").show(ctx,|ui| {
         ui.label("lalal aqui van los contactos y grupos");
         for i in 0..10 {
            ui.horizontal(|ui| {
                ui.label(format!("{} aaaa papu",i));
            });
            ui.add(Separator::default().spacing(3.).horizontal());
        }
    });
}
