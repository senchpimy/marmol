use egui_extras::RetainedImage;
//use egui::text::LayoutJob;
//use egui_demo_lib::easy_mark;


mod search;
mod main_area;
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
    current_window: i8,

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
        Self {
            current_window:1,
            //buffer: "Arthur".to_owned(),
            //tabs:vec![tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),tabs::Tab::new(),],
            vault:String::from("/home/plof/Documents/1er-semestre-Fes/1er semestre/"),
            current_file:String::new(),

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
        if self.current_window==0 { //welcome screen
            
            unimplemented!()
        }   else if self.current_window==1{
            let side_settings_images = [&self.colapse_image,
                                        &self.switcher_image,
                                        &self.graph_image,
                                        &self.canvas_image,
                                        &self.daynote_image,
                                        &self.command_image,
                                        &self.vault_image,
                                        &self.help_image,
                                        &self.config_image];
            main_area::left_side_settings(ctx,&mut self.left_collpased, &side_settings_images);
    
            let menu_images = vec![&self.files_image, 
                                   &self.search_image, 
                                   &self.starred_image,];
            main_area::left_side_menu(ctx,&self.left_collpased,menu_images, 
                           &self.vault, &self.current_file, &mut self.current_left_tab,
                           &mut self.search_string_menu,&mut self.prev_search_string_menu, &mut self.search_results,
                           &mut self.regex_search);
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
