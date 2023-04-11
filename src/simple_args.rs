use egui::*;
use egui_extras::RetainedImage;
use egui_commonmark::*;

pub struct MarmolOne {
    path: String,
    image:RetainedImage,
    content:String,
    commoncache:CommonMarkCache,
}

impl MarmolOne{
    pub fn new(paths:&Vec<String>)->Self{
        Self{
            path:"test".to_owned(),
            content:"content".to_owned(),
            image: RetainedImage::from_image_bytes("dummy",include_bytes!("../new_file.png"),).unwrap(),
            commoncache:CommonMarkCache::default(),
        }
    }
}

impl eframe::App for MarmolOne {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {});
    }
}
