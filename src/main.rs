//use eframe::egui;
use eframe::{egui::{CentralPanel,ScrollArea,Separator,TopBottomPanel,SidePanel,Context,Layout,Align,ImageButton,TextureId}};
use egui_extras::RetainedImage;
//use egui::text::LayoutJob;
//use egui::{ TextFormat, FontId, Color32, Stroke, TextStyle, RichText };
//use egui_demo_lib::easy_mark;

use std::path::{PathBuf,Path};
use std::fs;

//mod files;
//mod tabs;

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Marmol",
        options,
        Box::new(|_cc| Box::new(Marmol::default())),
    )
}


struct Marmol{
    //buffer: String,
    //tabs: Vec<tabs::Tab>,
    left_collpased:bool,
    vault: String,
    current_file: String,
    current_left_tab: i8,
    //right_collpased:bool,
    colapse_image:RetainedImage,
    files_image:RetainedImage,
    search_image:RetainedImage,
    new_file:RetainedImage,
    starred_image:RetainedImage,
    config_image:RetainedImage,
    vault_image:RetainedImage,
    help_image:RetainedImage,
    switcher_image:RetainedImage,
    graph_image:RetainedImage,
    canvas_image:RetainedImage,
    daynote_image:RetainedImage,
    command_image:RetainedImage,
}

impl Default for Marmol {
    fn default() -> Self {
        Self {
            //buffer: "Arthur".to_owned(),
            //tabs:vec![tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),],
            vault:String::from("/home/plof/Documents/1er-semestre-Fes/1er semestre/"),
            current_file:String::new(),
            current_left_tab:0,
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
            switcher_image: RetainedImage::from_image_bytes("switcher",include_bytes!("../switcher.png"),).unwrap(),
            graph_image: RetainedImage::from_image_bytes("graph",include_bytes!("../graph.png"),).unwrap(),
            canvas_image: RetainedImage::from_image_bytes("canvas",include_bytes!("../canvas.png"),).unwrap(),
            daynote_image: RetainedImage::from_image_bytes("daynote",include_bytes!("../daynote.png"),).unwrap(),
            command_image: RetainedImage::from_image_bytes("command",include_bytes!("../command.png"),).unwrap(),
            
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let side_settings_images = [&self.colapse_image,
                                    &self.switcher_image,
                                    &self.graph_image,
                                    &self.canvas_image,
                                    &self.daynote_image,
                                    &self.command_image,
                                    &self.vault_image,
                                    &self.help_image,
                                    &self.config_image];
        left_side_settings(ctx,&mut self.left_collpased, &side_settings_images);

        let menu_images = vec![&self.files_image, 
                               &self.search_image, 
                               &self.starred_image,];
        left_side_menu(ctx,&self.left_collpased,menu_images, &self.vault, &self.current_file, &mut self.current_left_tab);

 //       let mut edit=easy_mark::EasyMarkEditor::default();
//        CentralPanel::default().show(ctx, |ui| edit.ui(ui));
//        egui::Area::new("my_area")
//    .show(ctx, |ui| {
//        ui.label("Floating text!");
//    });
/////////////////////////////////////////////////////////////////////////////////
    }
}
fn left_side_menu(ctx:&Context, colapse:&bool, images:Vec<&RetainedImage> , path:&str, current_file:&str,left_tab:&mut i8){
    let left_panel = SidePanel::left("buttons left menu").default_width(100.).min_width(100.).max_width(300.);
    let textures = vec![images[0].texture_id(ctx), images[1].texture_id(ctx), images[2].texture_id(ctx)];
    left_panel.show_animated(ctx, *colapse,|ui| {
        top_panel_menu_left(ui,textures, path, current_file,left_tab);
    });
}

fn top_panel_menu_left (ui:&mut egui::Ui, textures:Vec<TextureId>, path:&str, current_file:&str,left_tab:&mut i8){
    TopBottomPanel::top("Left Menu").show_inside(ui, |ui|{
        ui.with_layout(Layout::left_to_right(Align::Min),|ui| {
     if ui.add(ImageButton::new(textures[0], egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("files");*left_tab=0;}
     if ui.add(ImageButton::new(textures[1], egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("search"); *left_tab=1;}
     if ui.add(ImageButton::new(textures[2], egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("starred"); *left_tab=2;}
        });
    });
    if *left_tab==0{
        let scrolling_files = ScrollArea::vertical();
        scrolling_files.show(ui,|ui| {
        render_files(ui,path, &current_file);
        });
    }else if *left_tab==1{
        println!("searching")
    }else if *left_tab==2{
        println!("stars")
    }
}
fn render_files(ui:&mut egui::Ui, path:&str, current_file:&str){
        for entry in fs::read_dir(path).unwrap(){
            let file_location = entry.unwrap().path().to_str().unwrap().to_string();
            let file_name=Path::new(&file_location).file_name().expect("No fails").to_str().unwrap();
            if Path::new(&file_location).is_dir(){
                let col = egui::containers::collapsing_header::CollapsingHeader::new(file_name);
                col.show(ui, |ui| {
                render_files(ui,&file_location, current_file);
                });
            }else{
                if ui.add(egui::SelectableLabel::new(true, file_name)).clicked() {
                    let current_file = &file_location;
                    println!("{}",current_file);
                }
            }
        }

 }

fn left_side_settings(ctx:&Context, colapse:&mut bool, images:&[&RetainedImage]){
    let left_panel = SidePanel::left("buttons left").resizable(false).default_width(1.);
    let space = 10.;
    left_panel.show(ctx,|ui| {
        ui.add_space(5.);
        ui.vertical(|ui| {
        if ui.add(ImageButton::new(images[0].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){
            if *colapse{
                *colapse=false;
            }else{
                *colapse=true;
            }
        }
        ui.add(Separator::default());
        if ui.add(ImageButton::new(images[1].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("switcher")} //quick switcher
        ui.add_space(space);
        if ui.add(ImageButton::new(images[2].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("graph")}//graph
        ui.add_space(space);
        if ui.add(ImageButton::new(images[3].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("canvas")}//canvas
        ui.add_space(space);
        if ui.add(ImageButton::new(images[4].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("daynote")}//note
        ui.add_space(space);
        if ui.add(ImageButton::new(images[5].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("commandpale")}//palette
        ui.with_layout(Layout::bottom_up(Align::Max),|ui|{
        ui.add_space(5.);
             if ui.add(ImageButton::new(images[6].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("conf")}
        ui.add_space(5.);
             if ui.add(ImageButton::new(images[7].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("help")}
        ui.add_space(5.);
             if ui.add(ImageButton::new(images[8].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("vault")}

        });
        });
    });
}
