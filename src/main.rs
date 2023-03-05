use egui_extras::RetainedImage;
use std::path::Path;
use egui::*;
use egui_extras::{Size,StripBuilder};
use egui_commonmark::*;
use std::fs;
use std::io::Write;
use yaml_rust::Yaml;
//use directories::BaseDirs;

mod search;
mod main_area;
mod files;
mod screens;
mod configuraciones;

fn main() -> Result<(), eframe::Error>{
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Marmol",
        options,
        Box::new(|_cc| Box::new(Marmol::default())),
    )
}


struct Marmol{
    buffer: String,
    prev_current_file: String,
    open_vault_str:String,
    new_vault_str:String,
    //tabs: Vec<tabs::Tab>,
    current_window: screens::Screen,
    buffer_image:RetainedImage,
    commoncache:CommonMarkCache,
    renderfile:bool,
    is_image:bool,
    config_path:String,
    left_controls:main_area::LeftControls,

    left_collpased:bool,
    vault: String,
    vault_vec: Vec<Yaml>,
    current_file: String,
}

impl Default for Marmol {
    fn default() -> Self {

        let (vault_var, vault_vec_var, current, config_path_var,window)=configuraciones::load_vault();
        Self {
            left_controls:main_area::LeftControls::default(),
            open_vault_str:String::from(""),
            new_vault_str:String::from(""),
            config_path:config_path_var.to_owned(),
            renderfile:true,
            current_window: window,
            prev_current_file: current.to_owned(),
            buffer: files::read_file(&current),
            buffer_image: RetainedImage::from_image_bytes("colapse",include_bytes!("../colapse.png"),).unwrap(),
            commoncache:CommonMarkCache::default(),
            is_image:false,
            //tabs:vec![tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),],
            vault:vault_var,
            vault_vec:vault_vec_var,
            current_file:current.to_owned(),

            left_collpased:true,
            //right_collpased:true,
            
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //dbg!(Size::exact(50.0));
        if self.current_window == screens::Screen::Default { //welcome screen
            screens::default(ctx,&mut self.current_window,&mut self.open_vault_str,&mut self.new_vault_str);
        }else if self.current_window == screens::Screen::Main{ //Main screen
            self.left_controls.left_side_settings(ctx,&mut self.left_collpased,&mut self.vault ,&mut self.current_file,&mut self.current_window);
    
            self.left_controls.left_side_menu(ctx,&self.left_collpased, 
                           &self.vault, &mut self.current_file);

            CentralPanel::default().show(ctx, |ui|{
               if self.prev_current_file!=self.current_file{
                    self.prev_current_file = String::from(&self.current_file);
                    if self.current_file.ends_with(".png") || 
                       self.current_file.ends_with("jpeg") ||
                       self.current_file.ends_with("jpg"){
                        self.buffer_image=RetainedImage::from_image_bytes("buffer_image",&files::read_image(&self.current_file)).unwrap();
                        self.is_image=true;
                    }else{
                        self.is_image=false;
                    }
               }

               if self.is_image {
                       let image_size = self.buffer_image.size_vec2();
                       //(Horizontal, Vertical)
                       let size:egui::Vec2;
                       if image_size[0]>800.0{
                            let vertical = (800.0*image_size[1])/image_size[0];
                            size = egui::vec2(800.0, vertical);
                       }else{
                            size = image_size;
                       }
                       let scrolling_buffer = ScrollArea::vertical();
                           scrolling_buffer.show(ui,|ui| {
                               ui.add(
                                   Image::new(self.buffer_image.texture_id(ctx), size)
                                );
                           });
               }else{
                self.buffer = files::read_file(&self.current_file);
                //Principal
                CentralPanel::default().show(ctx, |ui| {
                    if self.renderfile{
              //  ui.columns(1, |columns|{
              //  for i in 0..1{
                        //ScrollArea::vertical().id_source(format!("{}",i)).show(&mut columns[i],|ui| {
                        egui::TopBottomPanel::top("tabs").show_inside(ui,|ui| {
                            let tabs= StripBuilder::new(ui);
                            tabs.size(Size::exact(1.0))
                            .vertical(|mut strip|{
                                strip.cell(|ui|{
                                    ui.label("here be a tab");
                                });
                            });
                            //ui.label("here be tabs")
                        });
                        egui::ScrollArea::vertical().show(ui,|ui| {
                            let header = Path::new(&self.current_file).file_name().unwrap();
                            ui.heading(header.to_str().unwrap());
                            let (content, metadata)=files::contents(&self.buffer);
                            if metadata.len()!=0{
                                main_area::create_metadata(metadata,ui);
                            }
                           CommonMarkViewer::new("viewer").show(ui, &mut self.commoncache, &content);
                        });
                //}//termina for
                //});//termina coluns
                }else{
                }
                    }); //termina CentralPanel
                //Termina Principal
               }
            });
        }else if self.current_window==screens::Screen::Configuracion { //configuration
                            screens::configuracion(ctx,&mut self.current_window);
        }else if self.current_window == screens::Screen::Server{
            screens::set_server(ctx);
        };
/////////////////////////////////////////////////////////////////////////////////
    }


    fn on_close_event(&mut self) -> bool{
        let vault_str = format!("vault: '{}'",&self.vault);
        let mut vec_str=String::new();
        //dbg!(vault_str);
        for i in &self.vault_vec{
            let u =i.as_str().unwrap();
            vec_str = vec_str.to_owned() + format!(" '{}' ,",&u).as_str();
        }

        let vault_vec_str = format!("vault_vec: [ {} ]",vec_str);
            let file_path = String::from(&self.config_path) + "/ProgramState";
            let current_file = format!("current: {}", &self.current_file);
            let new_content= format!("{}\n{}\n{}",&vault_vec_str,vault_str,current_file);
            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(new_content.as_bytes());
            true
    }
}
