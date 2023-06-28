use egui_extras::RetainedImage;
use std::fs::File;
use std::path::Path;
use egui::*;
use egui_extras::{Size,StripBuilder};
use egui_commonmark::*;
use std::fs;
use std::io::Write;
use yaml_rust::Yaml;
use std::fmt;

#[macro_use]
extern crate json;

mod search;
mod format;
mod main_area;
mod files;
mod screens;
mod configuraciones;
mod toggle_switch;
mod graph;
mod tasks;
mod income;

#[derive(PartialEq,Debug)]
enum NewFileType{Markdown, Income,Tasks}

impl fmt::Display for NewFileType{
    fn fmt(&self,f:&mut fmt::Formatter)->fmt::Result{
        write!(f,"{:?}",self)
    }
}

fn main() -> Result<(), eframe::Error>{
    let options = eframe::NativeOptions {
        ..Default::default()
    };
        eframe::run_native(
            "Marmol",
            options,
            Box::new(|cc| Box::new(Marmol::new(cc))),
        )
}


struct Marmol{
    buffer: String,
    prev_current_file: String,
    new_vault_str:String,
    //tabs: Vec<tabs::Tab>,
    content:main_area::Content,
    text_edit:String,

    current_window: screens::Screen,
    prev_window: screens::Screen,
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
    create_file_error:String,
    show_create_button:bool,
    new_vault_folder: String,
    new_vault_folder_err: String,
    vault_changed:bool,
    font_size:f32,
    center_size:f32,
    center_size_remain:f32,
    sort_files:bool,

    new_file_type:NewFileType,
    marker:graph::Graph,
    tasks:tasks::TasksGui,
    income:income::IncomeGui,
}

impl Marmol{
        fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let font_size=configuraciones::load_context();
        let ctx = &cc.egui_ctx;
        let mut style = (*ctx.style()).clone();
        let font_id = FontId::proportional(font_size);
        style.override_font_id = Some(font_id);
        ctx.set_style(style);
        //ctx.set_visuals(configuraciones::load_colors());
        Self{
            font_size,
            ..Default::default()
        }
    }
}

impl Default for Marmol {
    fn default() -> Self {
        let (vault_var, vault_vec_var, current, config_path_var,
             window,left_coll,center_size,sort_files)=configuraciones::load_vault();
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
            tasks:tasks::TasksGui::default(),
            income:income::IncomeGui::default(),
            center_size,
            center_size_remain:(1.0-center_size)/2.0,
            font_size:12.0,
            marker:graph::Graph::new(&vault_var),
            new_file_str:String::new(),
            content: main_area::Content::View,
            left_controls:main_area::LeftControls::default(),
            new_vault_folder:String::from(""),
            new_vault_folder_err:String::from(""),
            new_vault_str:String::from(""),
            config_path:config_path_var.to_owned(),
            renderfile:true,
            create_new_vault:false,
            show_create_button:false,
            current_window: window,
            prev_window: window,
            prev_current_file: current.to_owned(),
            buffer: buf.clone(),
            text_edit: buf,
            buffer_image: buffer_image_pre,
            commoncache:CommonMarkCache::default(),
            is_image:is_image_pre,
            create_file_error:String::new(),
            vault:vault_var,
            vault_vec:vault_vec_var,
            current_file:current.to_owned(),
            new_file_type:NewFileType::Markdown,

            left_collpased:left_coll,
            vault_changed:false,
            sort_files
            //right_collpased:true,
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.current_window == screens::Screen::Default { //welcome screen
            screens::default(ctx,&mut self.current_window,
                             &mut self.new_vault_str,&mut self.vault_vec,&mut self.vault,&mut self.content);
        }else if self.current_window == screens::Screen::Main{ //Main screen
            self.left_controls.left_side_settings(ctx,&mut self.left_collpased,&mut self.vault ,
                                                  &mut self.current_file,&mut self.current_window,
                                                  &mut self.content);
            self.left_controls.left_side_menu(ctx,&self.left_collpased, 
                           &self.vault, &mut self.current_file,&self.sort_files);
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
                       let size:egui::Vec2=
                       if image_size[0]>800.0{
                            let vertical = (800.0*image_size[1])/image_size[0];
                            egui::vec2(800.0, vertical)
                       }else{
                            image_size
                       };
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
                        egui::TopBottomPanel::top("tabs").show_inside(ui,|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                if self.content !=main_area::Content::NewFile{
                                    ui.label("✏");
                                    ui.add(toggle_switch::toggle(&mut self.content));
                                    ui.label(RichText::new("👁").font(FontId::proportional(self.font_size)));
                                }
                            });
                        });
                        }
                        if self.content == main_area::Content::Edit{
                            let cont = StripBuilder::new(ui)
                                .size(Size::relative(self.center_size_remain))
                                .size(Size::relative(self.center_size));
                            cont.horizontal(|mut strip|{
                                strip.cell(|_|{});
                                strip.cell(|ui|{
                                egui::ScrollArea::vertical().show(ui,|ui| {
                                    let zone = egui::TextEdit::multiline(&mut self.text_edit)
                                        .font(FontId::proportional(15.0));
                                    let response = ui.add_sized(ui.available_size(), zone);
                                    if response.changed(){
                                        let mut f = std::fs::OpenOptions::new().write(true).truncate(true)
                                        .open(&self.current_file).unwrap();
                                        f.write_all(self.text_edit.as_bytes()).unwrap();
                                        f.flush().unwrap();
                                    }
                                    if ctx.input(|i| i.key_pressed(Key::Enter)) && response.has_focus(){
                                            let mut f = std::fs::OpenOptions::new().write(true).truncate(true)
                                            .open(&self.current_file).unwrap();
                                            f.write_all(format::indent(&self.text_edit).as_bytes()).unwrap();
                                            f.flush().unwrap();
                                    }
                                });
                                });
                            });
                        //}else if self.content == main_area::Content::NewTask{
                        //    self.new_file(ui,ctx.input(|i| i.key_pressed(Key::Enter)));
                        }else if self.content == main_area::Content::NewFile{
                            self.new_file(ui,ctx.input(|i| i.key_pressed(Key::Enter)));
                        }else if self.current_file.ends_with(".graph"){
                            self.tasks.set_path(&self.current_file);
                            self.tasks.show(ui);
                        }else if self.current_file.ends_with(".inc"){
                            self.income.set_path(&self.current_file);
                            self.income.show(ui);
                        }else if self.content == main_area::Content::View{
                            if ctx.input(|i| i.key_pressed(Key::F))
                            {
                                println!("Search");
                            }
                            let cont = StripBuilder::new(ui)
                                .size(Size::relative(self.center_size_remain))
                                .size(Size::relative(self.center_size));
                            cont.horizontal(|mut strip|{
                                strip.cell(|_|{});
                                strip.cell(|ui|{
                                egui::ScrollArea::vertical().show(ui,|ui| {
                                    let header = Path::new(&self.current_file).file_name().unwrap();
                                    let (content, metadata)=files::contents(&self.buffer);
                                                ui.heading(header.to_str().unwrap());
                                                if !metadata.is_empty(){
                                                    main_area::create_metadata(metadata,ui);
                                                    }
                                                CommonMarkViewer::new("v").show(ui, &mut self.commoncache, &content);
                                });
                            });
                            });
                        }else if self.content == main_area::Content::Graph{
                            self.marker.ui(ui,&mut self.current_file, &mut self.content,&self.vault);
                            self.marker.controls(ctx);
                        }
                    }); //termina CentralPanel
                //Termina Principal
               }
            });
        }else if self.current_window==screens::Screen::Configuracion { //configuration
            screens::configuracion(ctx,&mut self.prev_window,&mut self.current_window, &mut self.vault_vec, &mut self.vault,
                                   &mut self.new_vault_str,&mut self.create_new_vault,&mut self.new_vault_folder,
                                   &mut self.new_vault_folder_err,&mut self.show_create_button,
                                   &mut self.vault_changed, &mut self.font_size,
                                   &mut self.center_size, &mut self.center_size_remain,&mut self.sort_files);
            if self.vault_changed{
                self.marker.update_vault(Path::new(&self.vault));
            }
        }else if self.current_window == screens::Screen::Server{
            screens::set_server(ctx);
        };
/////////////////////////////////////////////////////////////////////////////////
    }


    fn on_close_event(&mut self) -> bool{
        let vault_str = format!("vault: '{}'",&self.vault);
        let mut vec_str=String::new();
        for i in &self.vault_vec{
            let u =i.as_str().unwrap();
            vec_str = vec_str.to_owned() + format!(" '{}' ,",&u).as_str();
        }

        let dir = Path::new(&self.config_path);
        println!("{}",&self.config_path);
        if !dir.exists(){
            fs::create_dir(&self.config_path);
        }
        let vault_vec_str = format!("vault_vec: [ {} ]",vec_str);
            let file_path = String::from(&self.config_path) + "/ProgramState";
            let current_file = format!("current: {}", &self.current_file);
            let center_size = format!("center_size: {}", &self.center_size);
            let left_menu = format!("left_menu: {}", &self.left_collpased);
            let sort_files = format!("sort_files: {}", &self.sort_files);
            let new_content= format!("{}\n{}\n{}\n{}\n{}\n{}",
                                     &vault_vec_str,vault_str,current_file,left_menu,center_size,sort_files);
            let mut file = fs::File::create(file_path).unwrap();
            file.write_all(new_content.as_bytes()).unwrap();

            let context_path = String::from(&self.config_path) + "/ContextState";
            let mut file2 = fs::File::create(context_path).unwrap();
            let font_size = format!("font_size: {}", &self.font_size);
            //let context_contents=font_size;
            file2.write_all(font_size.as_str().as_bytes()).unwrap();
            true
    }
}

impl Marmol{
    //fn new_file(&mut self,ui:&mut Ui,selected:NewFileType,enter_clicked:bool){
    fn new_file(&mut self,ui:&mut Ui,enter_clicked:bool){
        ui.label("Create New File");
        ui.add(egui::TextEdit::singleline(&mut self.new_file_str));
        let new_path = format!("{}/{}",&self.vault, &self.new_file_str);
        egui::ComboBox::from_label("Editar categoria")
        .selected_text(&self.new_file_type.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut self.new_file_type,NewFileType::Markdown ,"Markdown");
            ui.selectable_value(&mut self.new_file_type,NewFileType::Tasks ,"Tasks");
            ui.selectable_value(&mut self.new_file_type,NewFileType::Income ,"Income");
        });
        let path =if self.new_file_type==NewFileType::Tasks{
            format!("{}.graph",new_path)
        }else if self.new_file_type==NewFileType::Income{
            format!("{}.inc",new_path)
        }else{
            String::new()
        };
        let new_file= if self.new_file_type==NewFileType::Markdown{
            Path::new(&new_path)
        }else{
            Path::new(&path)
        };
        ui.label(RichText::new(&self.create_file_error).color(Color32::RED));
        if new_file.exists(){
            self.create_file_error=String::from("File already exist");
        }else{
            if ui.button("Create").clicked() || enter_clicked{
                self.content = main_area::Content::View;
                let res = File::create(new_file);
                match res{
                    Ok(mut re)=>{
                        self.create_file_error=String::new();
                        if self.new_file_type==NewFileType::Tasks{
                            let contents=String::from("{\"tasks\":[],\"days\":[],\"top_id\":0}");
                            re.write_all(contents.as_bytes()).unwrap();
                        }else if self.new_file_type==NewFileType::Income{
                            let contents=String::from("{\"transacciones\":[],\"categorias\":[ \"Categoria\"],\"colores\":[[0.0,0.0,0.0]]}");
                            re.write_all(contents.as_bytes()).unwrap();
                        }
                        self.current_file=String::from(new_file.to_str().unwrap());
                    }
                    Err(x)=>{self.create_file_error=x.to_string();}
                }
                    self.new_file_str = String::new();
            }
            self.create_file_error=String::new();
        }
        if ui.button("Cancel").clicked(){
            self.content = main_area::Content::View;
            self.new_file_str = String::new();
        }
    }
}
