use crate::graph_state::Graph;
use egui::*;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

extern crate json;

mod configuraciones;
mod excalidraw;
mod files;
mod format;
mod graph_state;
mod graph_ui;
mod income;
mod main_area;
mod screens;
mod search;
mod server;
mod switcher;
mod tabs;
mod tasks;
mod theme;

#[derive(PartialEq, Debug)]
enum NewFileType {
    Markdown,
    Income,
    Tasks,
}
pub struct MShape {
    height: f32,
    width: f32,
    btn_size: f32,
}

impl fmt::Display for NewFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Marmol",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(Marmol::new(cc)))
        }),
    )
}

struct Marmol {
    tabs_counter: usize,
    switcher: switcher::QuickSwitcher,
    prev_current_file: String,
    new_vault_str: String,
    content: main_area::content_enum::Content,

    current_window: screens::Screen,
    prev_window: screens::Screen,
    config_path: String,
    left_controls: main_area::left_controls::LeftControls,
    new_file_str: String,

    left_collpased: bool,
    vault: String,
    vault_vec: Vec<String>,
    current_file: String,
    window_size: MShape,

    create_new_vault: bool,
    create_file_error: String,
    show_create_button: bool,
    new_vault_folder: String,
    new_vault_folder_err: String,
    vault_changed: bool,
    font_size: f32,
    center_size: f32,
    center_size_remain: f32,
    sort_files: bool,
    tabs: tabs::Tabs,

    new_file_type: NewFileType,
    marker: Graph,
}

impl Marmol {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let font_size = configuraciones::load_context();

        crate::theme::load_and_apply_theme(&cc.egui_ctx);

        // Load the full program state
        let (
            vault_var,
            vault_vec_var,
            current_file_opt,
            config_dir_path,
            window,
            left_coll,
            center_size,
            sort_files,
            dock_state,
        ) = configuraciones::load_program_state();

        let initial_state = configuraciones::MarmolProgramState {
            vault: vault_var,
            vault_vec: vault_vec_var,
            current_file: current_file_opt,
            initial_screen: window,
            collapsed_left: left_coll,
            center_size,
            sort_files,
            dock_state,
        };

        let mut app = Self::from_program_state(initial_state);
        app.font_size = font_size; // Set font_size after loading state
        app.config_path = config_dir_path; // Set config_path after loading state

        app
    }

    fn from_program_state(state: configuraciones::MarmolProgramState) -> Self {
        let current_path_str = state.current_file.clone().unwrap_or_default();
        Self {
            tabs_counter: 0,
            window_size: MShape {
                height: 0.,
                width: 0.,
                btn_size: 20.,
            },
            switcher: switcher::QuickSwitcher::default(),
            tabs: tabs::Tabs::new_from_dock_state(state.dock_state),
            center_size: state.center_size,
            center_size_remain: (1.0 - state.center_size) / 2.0,
            font_size: 12.0,
            marker: Graph::new(&state.vault),
            new_file_str: String::new(),
            content: main_area::content_enum::Content::View,
            left_controls: main_area::left_controls::LeftControls::default(),
            new_vault_folder: String::from(""),
            new_vault_folder_err: String::from(""),
            new_vault_str: String::from(""),
            config_path: String::new(), // Will be set in Marmol::new
            create_new_vault: false,
            show_create_button: false,
            current_window: state.initial_screen,
            prev_window: state.initial_screen,
            prev_current_file: current_path_str.clone(),
            create_file_error: String::new(),
            vault: state.vault,
            vault_vec: state.vault_vec,
            current_file: current_path_str,
            new_file_type: NewFileType::Markdown,

            left_collpased: state.collapsed_left,
            vault_changed: false,
            sort_files: state.sort_files,
        }
    }
}

impl Default for Marmol {
    fn default() -> Self {
        Self {
            tabs_counter: 0,
            window_size: MShape {
                height: 0.,
                width: 0.,
                btn_size: 20.,
            },
            switcher: switcher::QuickSwitcher::default(),
            tabs: tabs::Tabs::new_empty(),
            center_size: 0.8,
            center_size_remain: 0.1,
            font_size: 12.0,
            marker: Graph::new(""), // Default empty vault
            new_file_str: String::new(),
            content: main_area::content_enum::Content::Blank, // Default to blank content
            left_controls: main_area::left_controls::LeftControls::default(),
            new_vault_folder: String::new(),
            new_vault_folder_err: String::new(),
            new_vault_str: String::new(),
            config_path: "/home/plof/.config/marmol".to_string(), // Default config path
            create_new_vault: false,
            show_create_button: false,
            current_window: screens::Screen::Default,
            prev_window: screens::Screen::Default,
            prev_current_file: String::new(),
            create_file_error: String::new(),
            vault: String::new(),
            vault_vec: Vec::new(),
            current_file: String::new(),
            new_file_type: NewFileType::Markdown,
            left_collpased: true,
            vault_changed: false,
            sort_files: false,
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.current_window == screens::Screen::Default {
            //welcome screen
            screens::default(
                ctx,
                &mut self.current_window,
                &mut self.new_vault_str,
                &mut self.vault_vec,
                &mut self.vault,
                &mut self.content,
                &self.window_size,
            );
        } else if self.current_window == screens::Screen::Main {
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
                self.switcher.open(&self.vault);
            }
            //Main screen
            self.left_controls.left_side_settings(
                ctx,
                &mut self.left_collpased,
                &mut self.vault,
                &mut self.current_file,
                &mut self.current_window,
                &mut self.content,
                &mut self.tabs,
                &mut self.tabs_counter,
                &self.window_size,
            );
            self.left_controls.left_side_menu(
                ctx,
                &self.left_collpased,
                &self.vault,
                &mut self.current_file,
                &self.sort_files,
                &self.window_size,
            );
            CentralPanel::default().show(ctx, |ui| {
                if self.prev_current_file != self.current_file {
                    self.content = main_area::content_enum::Content::View;
                    self.prev_current_file = self.current_file.clone();
                    self.tabs.file_changed(self.current_file.clone());
                }

                if self.content == main_area::content_enum::Content::NewFile
                    || self.content == main_area::content_enum::Content::NewTask
                {
                    self.new_file(ui, ctx.input(|i| i.key_pressed(Key::Enter)));
                    return;
                }

                if let Some(file_to_open) = self.switcher.ui(ctx, &self.vault) {
                    self.current_file = file_to_open;
                    self.content = crate::main_area::content_enum::Content::View;
                }

                self.tabs.ui(
                    ui,
                    &mut self.marker,
                    &mut self.current_file,
                    &mut self.content,
                    &self.vault,
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "Marmol - {}",
                    self.current_file.split("/").last().unwrap()
                )));
            });
        } else if self.current_window == screens::Screen::Configuracion {
            screens::configuracion(
                ctx,
                &mut self.prev_window,
                &mut self.current_window,
                &mut self.vault_vec,
                &mut self.vault,
                &mut self.new_vault_str,
                &mut self.create_new_vault,
                &mut self.new_vault_folder,
                &mut self.new_vault_folder_err,
                &mut self.show_create_button,
                &mut self.vault_changed,
                &mut self.font_size,
                &mut self.center_size,
                &mut self.center_size_remain,
                &mut self.sort_files,
                &self.window_size,
            );
            if self.vault_changed {
                self.marker.update_vault(Path::new(&self.vault));
            }
        } else if self.current_window == screens::Screen::Server {
            screens::set_server(ctx);
        };
        /////////////////////////////////////////////////////////////////////////////////
        let rect = ctx.content_rect();
        let btn_size = match rect.width() / 45. {
            20.0..=30.0 => rect.width() / 45.,
            ..=20. => 20.,
            _ => 30.,
        };
        self.window_size = MShape {
            width: rect.width(),
            height: rect.height(),
            btn_size,
        };
        egui::Area::new("window_size_display".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
            .show(ctx, |ui| {
                ui.label(format!(
                    "w: {:.0}, h: {:.0}",
                    self.window_size.width, self.window_size.height
                ));
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let state = configuraciones::MarmolProgramState {
            vault: self.vault.clone(),
            vault_vec: self.vault_vec.clone(),
            current_file: Some(self.current_file.clone()),
            initial_screen: self.current_window.clone(),
            collapsed_left: self.left_collpased,
            center_size: self.center_size,
            sort_files: self.sort_files,
            dock_state: self.tabs.dock_state().clone(),
        };
        configuraciones::save_program_state(&state);

        let context_path = String::from(&self.config_path) + "/ContextState";
        let mut file2 = fs::File::create(context_path).unwrap();
        let font_size = format!("font_size: {}", &self.font_size);
        file2.write_all(font_size.as_str().as_bytes()).unwrap();
    }
}

impl Marmol {
    fn new_file(&mut self, ui: &mut Ui, enter_clicked: bool) {
        if self.content == main_area::content_enum::Content::NewTask {
            self.new_file_type = NewFileType::Tasks
        }
        ui.label("Create New File");
        ui.add(egui::TextEdit::singleline(&mut self.new_file_str));
        let new_path = format!("{}/{}", &self.vault, &self.new_file_str);
        egui::ComboBox::from_label("Editar categoria")
            .selected_text(&self.new_file_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.new_file_type, NewFileType::Markdown, "Markdown");
                ui.selectable_value(&mut self.new_file_type, NewFileType::Tasks, "Tasks");
                ui.selectable_value(&mut self.new_file_type, NewFileType::Income, "Income");
            });
        let path = if self.new_file_type == NewFileType::Tasks {
            format!("{}.graph", new_path)
        } else if self.new_file_type == NewFileType::Income {
            format!("{}.inc", new_path)
        } else {
            String::new()
        };
        let new_file = if self.new_file_type == NewFileType::Markdown {
            Path::new(&new_path)
        } else {
            Path::new(&path)
        };
        ui.label(RichText::new(&self.create_file_error).color(Color32::RED));
        if new_file.exists() {
            self.create_file_error = String::from("File already exist");
        } else {
            if ui.button("Create").clicked() || enter_clicked {
                self.content = main_area::content_enum::Content::View;
                let res = File::create(new_file);
                match res {
                    Ok(mut re) => {
                        self.create_file_error = String::new();
                        if self.new_file_type == NewFileType::Tasks {
                            let contents = String::from("{\"tasks\":[],\"days\":[],\"top_id\":0}");
                            re.write_all(contents.as_bytes()).unwrap();
                        } else if self.new_file_type == NewFileType::Income {
                            let contents=String::from("{\"transacciones\":[],\"categorias\":[ \"Categoria\"],\"colores\":[[0.0,0.0,0.0]]}");
                            re.write_all(contents.as_bytes()).unwrap();
                        }
                        self.current_file = String::from(new_file.to_str().unwrap());
                    }
                    Err(x) => {
                        self.create_file_error = x.to_string();
                    }
                }
                self.new_file_str = String::new();
            }
            self.create_file_error = String::new();
        }
        if ui.button("Cancel").clicked() {
            self.content = main_area::content_enum::Content::View;
            self.new_file_str = String::new();
        }
    }
}
