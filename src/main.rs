use eframe::egui;
use eframe::{egui::{CentralPanel,ScrollArea,Separator,TopBottomPanel,SidePanel,Context,Layout,Align,ImageButton}};
use egui_extras::RetainedImage;
//use egui::text::LayoutJob;
//use egui::{ TextFormat, FontId, Color32, Stroke, TextStyle, RichText };
use egui_demo_lib::easy_mark;

mod files;
mod tabs;

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
    tabs: Vec<tabs::Tab>,
    left_collpased:bool,
    right_collpased:bool,
    colapseImage:RetainedImage,
}

impl Default for Marmol {
    fn default() -> Self {
        Self {
            buffer: "Arthur".to_owned(),
            tabs:vec![tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),],
            left_collpased:true,
            right_collpased:true,
            colapseImage: RetainedImage::from_image_bytes("colapse",include_bytes!("../colapse.png"),)
            .unwrap(),
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
/////////////////////////////////////////////////////////////////////////////////
        left_side_settings(ctx,&mut self.left_collpased,&self.colapseImage);
        left_side_menu(ctx,&self.left_collpased);
 //       let mut edit=easy_mark::EasyMarkEditor::default();
//        CentralPanel::default().show(ctx, |ui| edit.ui(ui));
//        egui::Area::new("my_area")
//    .show(ctx, |ui| {
//        ui.label("Floating text!");
//    });
/////////////////////////////////////////////////////////////////////////////////
    }
}

fn left_side_settings(ctx:&Context, colapse:&mut bool,image:&RetainedImage,){
    let left_panel = SidePanel::left("buttons left").resizable(false).default_width(10.);
    left_panel.show(ctx,|ui| {
        top_panel_left(ui,colapse,image,ctx);
        ui.button("q"); //quick switcher
        ui.button("g"); //graph
        ui.button("C"); //Canvas
        ui.button("D"); //Dayle note
        ui.button("P"); //Command Palette
    });
}
fn top_panel_menu_left (ui:&mut egui::Ui,)  {
        TopBottomPanel::top("Left Menu").show_inside(ui, |ui|{
        if ui.button("files").clicked(){println!("Files");}; 
        if ui.button("search").clicked(){println!("search")};
        if ui.button("starred").clicked(){println!("starred")};
        });
}
fn top_panel_left (ui:&mut egui::Ui, colapse:&mut bool,image:&RetainedImage, ctx:&Context)  {
        TopBottomPanel::top("configuraciones").show_inside(ui, |ui|{
        if ui.add(egui::ImageButton::new(image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){
            if *colapse{
                *colapse=false;
            }else{
                *colapse=true;
            }
        }
        });
}
fn left_side_menu(ctx:&Context, colapse:&bool){
    let left_panel = SidePanel::left("buttons left menu").resizable(false).default_width(100.);
    left_panel.show_animated(ctx, *colapse,|ui| {
        top_panel_menu_left(ui);
    });
}

