use egui_extras::RetainedImage;
use std::path::Path;
use egui::*;
use egui_extras::{Size,StripBuilder};
use egui_commonmark::*;
use directories::BaseDirs;
use std::fs;
use yaml_rust::{YamlLoader,Yaml};
use std::io::Write;

mod search;
mod main_area;
mod files;
mod default_screen;
//mod tabs;

//fn main() -> std::io::Result<()>{
fn main() -> Result<(), eframe::Error>{
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
    buffer: String,
    prev_current_file: String,
    //tabs: Vec<tabs::Tab>,
    current_window: i8,
    buffer_image:RetainedImage,
    commoncache:CommonMarkCache,
    renderfile:bool,
    config_path:String,

    left_collpased:bool,
    vault: String,
    vault_vec: Vec<Yaml>,
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
        let mut vault_var:String=String::from("/home/plof/Pictures/");
        let mut vault_vec_var:Vec<Yaml> = vec![];
        let binding = BaseDirs::new().unwrap();
        let home_dir = binding.home_dir().to_str().unwrap();
        let mut config_path_var = String::from(home_dir);
        config_path_var=config_path_var+"/.config/marmol";
        let dir = Path::new(&config_path_var);
        if dir.exists(){
            let file_saved = String::from(&config_path_var)+"/ProgramState";
            let dir2 = Path::new(&file_saved);
                if dir2.exists(){
                        let data = fs::read_to_string(file_saved)
                            .expect("Unable to read file");
                        let docs = YamlLoader::load_from_str(&data).unwrap();
                        let docs = &docs[0];
                        vault_var = docs["vault"].as_str().unwrap().to_string();
                        vault_vec_var = docs["vault_vec"].as_vec().unwrap().to_vec();
                    println!("Estado anterior cargado");
                    //return load_file(file_saved, 0).unwrap()
                }
        }else{
            fs::create_dir(&dir);
            println!("Dir created");
        }

        Self {
            config_path:config_path_var.to_owned(),
            renderfile:true,
            current_window:0,
            prev_current_file: current.to_owned(),
            buffer: files::read_file(current),
            buffer_image: RetainedImage::from_image_bytes("colapse",include_bytes!("../colapse.png"),).unwrap(),
            commoncache:CommonMarkCache::default(),
            //tabs:vec![tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),],
            vault:vault_var,
            vault_vec:vault_vec_var,
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
            default_screen::default(ctx)
            
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
                           &mut self.search_string_menu,&mut self.prev_search_string_menu, 
                           &mut self.search_results,
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
            let new_content= format!("{}\n{}",&vault_vec_str,vault_str);
            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(new_content.as_bytes());
            true
    }
}
