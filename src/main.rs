use egui_extras::RetainedImage;
use std::fs::File;
use std::path::Path;
use egui::*;
use egui_extras::{Size,StripBuilder};
use egui_commonmark::*;
use std::fs;
use std::io::Write;
use yaml_rust::Yaml;
//use directories::BaseDirs;

#[macro_use]
extern crate json;

mod search;
mod main_area;
mod files;
mod screens;
mod configuraciones;
mod toggle_switch;
mod graph;

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
    content:main_area::Content,
    text_edit:String,


    current_window: screens::Screen,
    buffer_image:RetainedImage,
    commoncache:CommonMarkCache,
    renderfile:bool,
    is_image:bool,
    config_path:String,
    left_controls:main_area::LeftControls,
    new_file_str:String,

    left_collpased:bool,
    vault: String,
    vault_vec: Vec<Yaml>,
    current_file: String,

    create_new_vault:bool,
    show_create_button:bool,
    new_vault_folder: String,
    new_vault_folder_err: String,
    vault_changed:bool,

    marker:graph::Graph
}

impl Default for Marmol {
    fn default() -> Self {
        let (vault_var, vault_vec_var, current, config_path_var,window)=configuraciones::load_vault();
        let buf:String;
        let mut is_image_pre=false;
        let mut buffer_image_pre:RetainedImage =  RetainedImage::from_image_bytes("colapse",include_bytes!("../colapse.png"),).unwrap();
        println!("{}",current);
        if current.ends_with(".png") || 
            current.ends_with("jpeg") ||
            current.ends_with("jpg"){
            is_image_pre=true;
            buf = String::from("file");
            buffer_image_pre = RetainedImage::from_image_bytes("buffer_image",&files::read_image(&current)).unwrap();
        }else{
            buf = files::read_file(&current);
        }
        Self {
            marker:graph::Graph::new(&vault_var),
            new_file_str:String::new(),
            content: main_area::Content::View,
            left_controls:main_area::LeftControls::default(),
            open_vault_str:String::from(""),
            new_vault_folder:String::from(""),
            new_vault_folder_err:String::from(""),
            new_vault_str:String::from(""),
            config_path:config_path_var.to_owned(),
            renderfile:true,
            create_new_vault:false,
            show_create_button:false,
            current_window: window,
            prev_current_file: current.to_owned(),
            buffer: buf.clone(),
            text_edit: buf,
            buffer_image: buffer_image_pre,
            commoncache:CommonMarkCache::default(),
            is_image:is_image_pre,
            //tabs:vec![tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),],
            vault:vault_var,
            vault_vec:vault_vec_var,
            current_file:current.to_owned(),

            left_collpased:true,
            vault_changed:false,
            //right_collpased:true,
            
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.current_window == screens::Screen::Default { //welcome screen
            screens::default(ctx,&mut self.current_window,&mut self.open_vault_str,
                             &mut self.new_vault_str);
        }else if self.current_window == screens::Screen::Main{ //Main screen
            self.left_controls.left_side_settings(ctx,&mut self.left_collpased,&mut self.vault ,
                                                  &mut self.current_file,&mut self.current_window,
                                                  &mut self.content);
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
                self.text_edit = self.buffer.clone();
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
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                if self.content !=main_area::Content::NewFile{
                                    ui.label("✏");
                                    ui.add(toggle_switch::toggle(&mut self.content));
                                    ui.label(RichText::new("👁").font(FontId::proportional(20.0)));
                                }
                            });
                        });
                        }
                        if self.content == main_area::Content::Edit{
                            egui::ScrollArea::vertical().show(ui,|ui| {
                                let zone = egui::TextEdit::multiline(&mut self.text_edit)
                                    .font(FontId::proportional(15.0));
                                let response = ui.add_sized(ui.available_size(), zone);
                                if response.changed(){
                                    let mut f = std::fs::OpenOptions::new().write(true).truncate(true)
                                    .open(&self.current_file).unwrap();
                                    f.write_all(&self.text_edit.as_bytes()).unwrap();
                                    f.flush().unwrap();
                                }
                            });
                        }else if self.content == main_area::Content::View{
                            egui::ScrollArea::vertical().show(ui,|ui| {
                                let header = Path::new(&self.current_file).file_name().unwrap();
                                ui.heading(header.to_str().unwrap());
                                let (content, metadata)=files::contents(&self.buffer);
                                if metadata.len()!=0{
                                    main_area::create_metadata(metadata,ui);
                                }
                               CommonMarkViewer::new("viewer").show(ui, &mut self.commoncache, &content);
                            });
                        }
                        else if self.content == main_area::Content::NewFile{
                                ui.label("Create New File");
                                ui.add(egui::TextEdit::singleline(&mut self.new_file_str));
                                if ui.button("Create").clicked(){
                                    self.content = main_area::Content::View;
                                    let new_path = format!("{}/{}",&self.vault, &self.new_file_str);
                                    let new_file =Path::new(&new_path);
                                    let res = File::create(new_file);
                                    match res{
                                        Ok(_)=>{},
                                        Err(x)=>{println!("{}",x);}//todo
                                    }
                                    self.new_file_str = String::new();
                                }
                                if ui.button("Cancel").clicked(){
                                    self.content = main_area::Content::View;
                                }
                        }else if self.content == main_area::Content::Graph{
                            self.marker.ui(ui,&mut self.current_file, &mut self.content);
                            self.marker.controls(ctx);
                        }
                    }); //termina CentralPanel
                //Termina Principal
               }
            });
        }else if self.current_window==screens::Screen::Configuracion { //configuration
            screens::configuracion(ctx,&mut self.current_window, &mut self.vault_vec, &mut self.vault,
                                   &mut self.new_vault_str,&mut self.create_new_vault,&mut self.new_vault_folder,
                                   &mut self.new_vault_folder_err,&mut self.show_create_button,
                                   &mut self.vault_changed);
            if self.vault_changed{
                self.marker.update_vault(&Path::new(&self.vault));
            }
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
            file.write_all(new_content.as_bytes()).unwrap();
            true
    }
}
