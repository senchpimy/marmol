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
    //buffer: String,
    //tabs: Vec<tabs::Tab>,
    left_collpased:bool,
    //right_collpased:bool,
    colapse_image:RetainedImage,
    files_image:RetainedImage,
    search_image:RetainedImage,
    new_file:RetainedImage,
    starred_image:RetainedImage,
    config_image:RetainedImage,
    vault_image:RetainedImage,
    help_image:RetainedImage,
}

impl Default for Marmol {
    fn default() -> Self {
        Self {
            //buffer: "Arthur".to_owned(),
            //tabs:vec![tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),],
            left_collpased:true,
            //right_collpased:true,
            colapse_image: RetainedImage::from_image_bytes("colapse",include_bytes!("../colapse.png"),).unwrap(),
            search_image: RetainedImage::from_image_bytes("search",include_bytes!("../search.png"),).unwrap(),
            new_file: RetainedImage::from_image_bytes("search",include_bytes!("../new_file.png"),).unwrap(),
            starred_image: RetainedImage::from_image_bytes("starred",include_bytes!("../starred.png"),).unwrap(),
            files_image: RetainedImage::from_image_bytes("files",include_bytes!("../files.png"),).unwrap(),
            config_image: RetainedImage::from_image_bytes("cpnfiguration",include_bytes!("../configuration.png"),).unwrap(),
            help_image: RetainedImage::from_image_bytes("help",include_bytes!("../help.png"),).unwrap(),
            vault_image: RetainedImage::from_image_bytes("vault",include_bytes!("../vault.png"),).unwrap(),
            
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
/////////////////////////////////////////////////////////////////////////////////
        left_side_settings(ctx,&mut self.left_collpased,&self.colapse_image,&self.vault_image, &self.help_image, &self.config_image);
        left_side_menu(ctx,&self.left_collpased, &self.files_image, &self.search_image, &self.starred_image);
 //       let mut edit=easy_mark::EasyMarkEditor::default();
//        CentralPanel::default().show(ctx, |ui| edit.ui(ui));
//        egui::Area::new("my_area")
//    .show(ctx, |ui| {
//        ui.label("Floating text!");
//    });
/////////////////////////////////////////////////////////////////////////////////
    }
}
fn left_side_menu(ctx:&Context, colapse:&bool, new_file:&RetainedImage, search:&RetainedImage, starred_image:&RetainedImage){
    let left_panel = SidePanel::left("buttons left menu").resizable(false).default_width(150.);
    left_panel.show_animated(ctx, *colapse,|ui| {
        top_panel_menu_left(ui,new_file,search,starred_image,ctx);
    });
}

fn top_panel_menu_left (ui:&mut egui::Ui, new_file:&RetainedImage, search:&RetainedImage, starred_image:&RetainedImage, ctx:&Context)  {
    TopBottomPanel::top("Left Menu").show_inside(ui, |ui|{
        ui.add_space(5.);
        ui.with_layout(Layout::left_to_right(Align::Max),|ui| {
     if ui.add(egui::ImageButton::new(new_file.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("new file")}
     if ui.add(egui::ImageButton::new(search.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("search")}
     if ui.add(egui::ImageButton::new(starred_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("starred")}
        });
    });
}

fn left_side_settings(ctx:&Context, colapse:&mut bool,colapse_img:&RetainedImage, vault:&RetainedImage, help:&RetainedImage, configuration:&RetainedImage){
    let left_panel = SidePanel::left("buttons left").resizable(false).default_width(1.);
    let space = 10.;
    left_panel.show(ctx,|ui| {
        ui.add_space(5.);
        ui.vertical(|ui| {
        if ui.add(egui::ImageButton::new(colapse_img.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){
            if *colapse{
                *colapse=false;
            }else{
                *colapse=true;
            }
        }
        ui.separator();
        ui.add(egui::Button::new("q").frame(true)); //quick switcher
        ui.add_space(space);
        ui.add(egui::Button::new("g").frame(true)); //graph
        ui.add_space(space);
        ui.add(egui::Button::new("C").frame(true)); //canvas
        ui.add_space(space);
        ui.add(egui::Button::new("D").frame(true)); //dayli note
        ui.add_space(space);
        ui.add(egui::Button::new("P").frame(true)); //Command palette
        ui.with_layout(Layout::bottom_up(Align::Max),|ui|{
        ui.add_space(5.);
             if ui.add(egui::ImageButton::new(configuration.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("conf")}
        ui.add_space(5.);
             if ui.add(egui::ImageButton::new(help.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("help")}
        ui.add_space(5.);
             if ui.add(egui::ImageButton::new(vault.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("vault")}

        });
        });
    });
}
