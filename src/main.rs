use egui_extras::RetainedImage;
use std::path::Path;
use egui::*;
//use egui::text::LayoutJob;
use egui_demo_lib::easy_mark;
use egui_extras::{Size,StripBuilder};
//use egui_commonmark::*;


mod search;
mod main_area;
mod files;
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
    );
}


struct Marmol{
    buffer: String,
    prev_current_file: String,
    //tabs: Vec<tabs::Tab>,
    current_window: i8,
    buffer_image:RetainedImage,

    left_collpased:bool,
    vault: String,
    current_file: String,
    current_left_tab: i8,
    search_string_menu:String,
    prev_search_string_menu:String,
    search_results:Vec<search::MenuItem>,
    regex_search:bool,

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
        let current="/home/plof/Documents/1er-semestre-Fes/1er semestre/Tareas.md";
        Self {
            current_window:1,
            prev_current_file: current.to_owned(),
            buffer: files::read_file(current),
            buffer_image: RetainedImage::from_image_bytes("colapse",include_bytes!("../colapse.png"),).unwrap(),
            //tabs:vec![tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),],
            vault:String::from("/home/plof/Documents/1er-semestre-Fes/1er semestre/"),
//            vault:String::from("/home/plof/Pictures/"),
            current_file:current.to_owned(),

            search_string_menu:"".to_owned(),
            prev_search_string_menu:"".to_owned(),
            search_results:vec![],
            regex_search:false,

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
        //dbg!(Size::exact(50.0));
        if self.current_window==0 { //welcome screen
            CentralPanel::default().show(ctx,|ui|{
                ui.heading("Marmol");
                ui.label("select a vault");
                ui.label("configuration");
            });
            
        }   else if self.current_window==1{ //Main screen
            let side_settings_images = [&self.colapse_image,
                                        &self.switcher_image,
                                        &self.graph_image,
                                        &self.canvas_image,
                                        &self.daynote_image,
                                        &self.command_image,
                                        &self.vault_image,
                                        &self.help_image,
                                        &self.config_image];
            main_area::left_side_settings(ctx,&mut self.left_collpased, &side_settings_images,&mut self.vault ,&mut self.current_file);
    
            let menu_images = vec![&self.files_image, 
                                   &self.search_image, 
                                   &self.starred_image,];
            main_area::left_side_menu(ctx,&self.left_collpased,menu_images, 
                           &self.vault, &mut self.current_file, &mut self.current_left_tab,
                           &mut self.search_string_menu,&mut self.prev_search_string_menu, &mut self.search_results,
                           &mut self.regex_search);

            CentralPanel::default().show(ctx, |ui|{
               if self.prev_current_file!=self.current_file{
                    self.prev_current_file = String::from(&self.current_file);
                    if self.current_file.ends_with(".png") || 
                       self.current_file.ends_with("jpeg") ||
                       self.current_file.ends_with("jpg"){
                        self.buffer_image=RetainedImage::from_image_bytes("buffer_image",&files::read_image(&self.current_file)).unwrap()
                    }
               }

               if self.current_file.ends_with(".png")||
                   self.current_file.ends_with("jpeg") ||
                   self.current_file.ends_with("jpg"){
                       let image_size = self.buffer_image.size_vec2();
                       dbg!(image_size[0]);
                       //(Horizontal, Vertical)
                       let size = egui::vec2(1000.0, 1000.0);
                       let scrolling_buffer = ScrollArea::vertical();
                           scrolling_buffer.show(ui,|ui| {
                               ui.add(
                                   Image::new(self.buffer_image.texture_id(ctx), size)
                                );
                           });
               }else{
                self.buffer = files::read_file(&self.current_file);
                //Comienza loop
                CentralPanel::default().show(ctx, |ui| {
              //  ui.columns(1, |columns|{
              //  for i in 0..1{
                        //ScrollArea::vertical().id_source(format!("{}",i)).show(&mut columns[i],|ui| {
                        egui::ScrollArea::vertical().show(ui,|ui| {
                            let header = Path::new(&self.current_file).file_name().unwrap();
                            ui.heading(header.to_str().unwrap());
                            let (content, metadata)=files::contents(&self.buffer);
                            if metadata.len()!=0{
                                main_area::create_metadata(metadata,ui);
                            }
                                easy_mark::easy_mark(ui,&content);
                          //  let mut cache = CommonMarkCache::default();
                          //  CommonMarkViewer::new("viewer").show(ui, &mut cache, &content);
                        });
                //}//termina for
                //});//termina coluns
                    }); //termina CentralPanel
                //termina loop
               }
            });
        }else if self.current_window==2{ //configuration
            
        };
//       let mut edit=easy_mark::EasyMarkEditor::default();
//        CentralPanel::default().show(ctx, |ui| edit.ui(ui));
//        egui::Area::new("my_area")
//    .show(ctx, |ui| {
//        ui.label("Floating text!");
//    });
/////////////////////////////////////////////////////////////////////////////////
    }
}
