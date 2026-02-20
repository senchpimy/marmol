use crate::command_palette::{CommandAction, CommandPalette};
use lz_str;
use crate::graph::Graph;
use crate::iconize::{IconPackInstaller, IconSelector};
use egui::*;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

extern crate json;
extern crate log;

pub mod command_palette;
pub mod configuraciones;
pub mod canvas;
pub mod easy_mark;
pub mod egui_commonmark;
pub mod egui_commonmark_backend;
pub mod egui_dock;
pub mod emojis;
pub mod excalidraw;
pub mod files;
pub mod format;
pub mod graph;
pub mod iconize;
pub mod income;
pub mod kanban;
pub mod main_area;
pub mod screens;
pub mod search;
pub mod server;
pub mod switcher;
pub mod tabs;
pub mod tasks;
pub mod theme;

#[derive(PartialEq, Debug)]
pub enum NewFileType {
    Markdown,
    Income,
    Tasks,
    Excalidraw,
    Canvas,
}
pub struct MShape {
    pub height: f32,
    pub width: f32,
    pub btn_size: f32,
}

impl fmt::Display for NewFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Marmol {
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
    enable_icon_folder: bool,
    icon_selector: IconSelector,
    icon_pack_installer: IconPackInstaller,
    command_palette: CommandPalette,
    #[cfg(target_os = "android")]
    keyboard: egui_keyboard::Keyboard,
    android_storage: configuraciones::AndroidStorage,
    #[cfg(target_os = "android")]
    pub android_app: Option<winit::platform::android::activity::AndroidApp>,
    
    // Style
    dock_style: crate::egui_dock::Style,
}

impl Marmol {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let font_size = configuraciones::load_context();

        let dock_style = crate::theme::load_and_apply_theme(&cc.egui_ctx);

        // Load the full program state
        let (initial_state, config_dir_path) = configuraciones::load_program_state();

        let mut app = Self::from_program_state(initial_state, &cc.egui_ctx);
        app.font_size = font_size; // Set font_size after loading state
        app.config_path = config_dir_path; // Set config_path after loading state
        app.dock_style = dock_style;

        app
    }

    fn from_program_state(state: configuraciones::MarmolProgramState, ctx: &egui::Context) -> Self {
        let current_path_str = state.current_file.unwrap_or_default();
        Self {
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
            marker: Graph::new(&state.vault, ctx),
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
            enable_icon_folder: state.enable_icon_folder,
            icon_selector: IconSelector::default(),
            icon_pack_installer: IconPackInstaller::default(),
            command_palette: CommandPalette::default(),
            #[cfg(target_os = "android")]
            keyboard: egui_keyboard::Keyboard::default(),
            android_storage: state
                .android_storage
                .unwrap_or(configuraciones::AndroidStorage::Unselected),
            #[cfg(target_os = "android")]
            android_app: None,
            dock_style: crate::egui_dock::Style::from_egui(ctx.style().as_ref()),
        }
    }
}

impl Default for Marmol {
    fn default() -> Self {
        Self {
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
            marker: Graph::new("", &egui::Context::default()), // Default empty vault
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
            enable_icon_folder: false,
            icon_selector: IconSelector::default(),
            icon_pack_installer: IconPackInstaller::default(),
            command_palette: CommandPalette::default(),
            #[cfg(target_os = "android")]
            keyboard: egui_keyboard::Keyboard::default(),
            android_storage: configuraciones::AndroidStorage::Unselected,
            #[cfg(target_os = "android")]
            android_app: None,
            dock_style: crate::egui_dock::Style::default(),
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "android")]
        {
            self.keyboard.pump_events(ctx);
            egui::TopBottomPanel::top("android_top_spacing")
                .frame(egui::Frame::none().fill(ctx.style().visuals.window_fill))
                .show_separator_line(false)
                .show(ctx, |ui| {
                    ui.add_space(30.0);
                });
        }

        if self.current_window == screens::Screen::Default {
            let prev_win = self.current_window;
            let prev_vaults_count = self.vault_vec.len();
            let prev_storage = self.android_storage;
            //welcome screen
            screens::default(
                ctx,
                &mut self.current_window,
                &mut self.prev_window,
                &mut self.new_vault_str,
                &mut self.vault_vec,
                &mut self.vault,
                &mut self.content,
                &self.window_size,
                &mut self.create_new_vault,
                &mut self.new_vault_folder,
                &mut self.new_vault_folder_err,
                &mut self.show_create_button,
                &mut self.android_storage,
                #[cfg(target_os = "android")]
                &self.android_app,
            );
            if prev_win != self.current_window
                || prev_vaults_count != self.vault_vec.len()
                || prev_storage != self.android_storage
            {
                self.save_to_disk();
            }
        } else if self.current_window == screens::Screen::CreateVault {
            screens::create_vault_screen(
                ctx,
                &mut self.current_window,
                &mut self.prev_window,
                &mut self.new_vault_str,
                &mut self.new_vault_folder,
                &mut self.new_vault_folder_err,
                &mut self.vault_vec,
                &mut self.vault,
                &mut self.vault_changed,
            );
        } else if self.current_window == screens::Screen::Main {
            // Check for file deletion signal (from file_options)
            let deleted_file: Option<String> = ctx.data_mut(|d| d.get_temp(egui::Id::new("file_deleted_signal")).flatten());
            if let Some(path) = deleted_file {
                self.tabs.close_tab_by_path(&path);
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("file_deleted_signal"), None::<String>));
            }

            // Check for delete file request (from tab menu)
            let delete_req: Option<String> = ctx.data_mut(|d| d.get_temp(egui::Id::new("delete_file_request")).flatten());
            if let Some(path) = delete_req {
                let _ = std::fs::remove_file(&path);
                self.tabs.close_tab_by_path(&path);
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("delete_file_request"), None::<String>));
            }

            // Check for reveal in navigation signal
            let reveal_req: Option<String> = ctx.data_mut(|d| d.get_temp(egui::Id::new("reveal_in_nav_signal")).flatten());
            if let Some(path) = reveal_req {
                self.left_controls.file_tree.reveal_path = Some(path);
                self.left_collpased = false; // Ensure menu is open
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("reveal_in_nav_signal"), None::<String>));
            }

            // Check for open icon selector signal
            let icon_req: Option<String> = ctx.data_mut(|d| d.get_temp(egui::Id::new("open_icon_selector_signal")).flatten());
            if let Some(rel_path) = icon_req {
                self.icon_selector.open(rel_path, &mut self.left_controls.icon_manager);
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("open_icon_selector_signal"), None::<String>));
            }

            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::O)) {
                self.switcher.open(&self.vault);
            }

            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::P)) {
                self.command_palette.open();
            }

            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F)) {
                todo!("Find functionality");
            }

            self.icon_selector
                .ui(ctx, &self.vault, &mut self.left_controls.icon_manager);

            // Render Command Palette and handle actions
            match self.command_palette.ui(ctx) {
                CommandAction::OpenIconInstaller => {
                    self.icon_pack_installer.is_open = true;
                }
                CommandAction::CreateKanban => {
                    let mut name = "untitledKanban.md".to_string();
                    let mut path = format!("{}/{}", self.vault, name);
                    let mut count = 1;
                    while Path::new(&path).exists() {
                        name = format!("untitledKanban {}.md", count);
                        path = format!("{}/{}", self.vault, name);
                        count += 1;
                    }

                    if let Ok(mut file) = File::create(&path) {
                        let content = "---\nkanban-plugin: board\n---\n";
                        let _ = file.write_all(content.as_bytes());
                        self.current_file = path;
                        self.content = main_area::content_enum::Content::View;
                        self.tabs.file_changed(&self.current_file);
                    }
                }
                CommandAction::CreateExcalidraw => {
                    let mut name = "untitledExcalidraw.excalidraw.md".to_string();
                    let mut path = format!("{}/{}", self.vault, name);
                    let mut count = 1;
                    while Path::new(&path).exists() {
                        name = format!("untitledExcalidraw {}.excalidraw.md", count);
                        path = format!("{}/{}", self.vault, name);
                        count += 1;
                    }

                    if let Ok(mut file) = File::create(&path) {
                        let json_contents = String::from("{\"type\":\"excalidraw\",\"version\":2,\"source\":\"https://excalidraw.com\",\"elements\":[],\"appState\":{\"viewBackgroundColor\":\"#ffffff\"},\"files\":{}}");
                        let compressed = lz_str::compress_to_base64(&json_contents);
                        let full_content = format!(
"---

excalidraw-plugin: parsed
tags: [excalidraw]

---
==⚠  Switch to EXCALIDRAW VIEW in the MORE OPTIONS menu of this document. ⚠== You can decompress Drawing data with the command palette: 'Decompress current Excalidraw file'. For more info check in plugin settings under 'Saving'


# Excalidraw Data

## Text Elements

%%
## Drawing
```compressed-json
{}
```", compressed);
                        let _ = file.write_all(full_content.as_bytes());
                        self.current_file = path;
                        self.content = main_area::content_enum::Content::View;
                        self.tabs.file_changed(&self.current_file);
                    }
                }
                CommandAction::CreateCanvas => {
                    let mut name = "untitledCanvas.canvas".to_string();
                    let mut path = format!("{}/{}", self.vault, name);
                    let mut count = 1;
                    while Path::new(&path).exists() {
                        name = format!("untitledCanvas {}.canvas", count);
                        path = format!("{}/{}", self.vault, name);
                        count += 1;
                    }

                    if let Ok(mut file) = File::create(&path) {
                        let content = "{\"nodes\":[],\"edges\":[]}";
                        let _ = file.write_all(content.as_bytes());
                        self.current_file = path;
                        self.content = main_area::content_enum::Content::View;
                        self.tabs.file_changed(&self.current_file);
                    }
                }
                CommandAction::CloseTab => {
                    self.tabs.close_current_tab();
                }
                CommandAction::ToggleLeftMenu => {
                    self.left_collpased = !self.left_collpased;
                }
                CommandAction::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                CommandAction::CreateFile(mut name) => {
                    if !name.ends_with(".md") {
                        name.push_str(".md");
                    }
                    let path = format!("{}/{}", self.vault, name);
                    if !Path::new(&path).exists() {
                        let _ = File::create(&path);
                    }
                    self.current_file = path;
                    self.tabs.file_changed(&self.current_file);
                }
                CommandAction::None => {}
            }

            // Render Icon Pack Installer
            self.icon_pack_installer
                .ui(ctx, &self.vault, &mut self.left_controls.icon_manager);

            //Main screen
            self.left_controls.left_side_settings(
                ctx,
                &mut self.left_collpased,
                &mut self.vault,
                &mut self.current_file,
                &mut self.current_window,
                &mut self.prev_window,
                &mut self.content,
                &mut self.tabs,
                &self.window_size,
                &mut self.command_palette,
            );
            self.left_controls.left_side_menu(
                ctx,
                &self.left_collpased,
                &self.vault,
                &mut self.current_file,
                &self.sort_files,
                &self.window_size,
                self.enable_icon_folder,
                &mut self.icon_selector,
            );
            CentralPanel::default().show(ctx, |ui| {
                if self.prev_current_file != self.current_file {
                    self.content = main_area::content_enum::Content::View;
                    self.prev_current_file = self.current_file.clone();
                    self.tabs.file_changed(&self.current_file);
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
                    &mut self.left_controls.icon_manager,
                    &self.dock_style,
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                    "Marmol - {}",
                    self.current_file.split("/").last().unwrap()
                )));
            });
        } else if self.current_window == screens::Screen::Configuracion {
            let prev_win = self.current_window;
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
                &mut self.enable_icon_folder,
                &mut self.left_controls.icon_manager,
                &self.window_size,
            );
            if prev_win != self.current_window {
                self.save_to_disk();
            }
        } else if self.current_window == screens::Screen::Appearance {
            let prev_win = self.current_window;
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Appearance");
                    ui.add_space(20.0);
                    screens::appearance_settings(
                        ui,
                        ctx,
                        &self.vault,
                        &mut self.font_size,
                        &mut self.center_size,
                        &mut self.center_size_remain,
                        &mut self.sort_files,
                        &mut self.enable_icon_folder,
                        &mut self.left_controls.icon_manager,
                        [ui.available_width() * 0.8, 40.0],
                    );
                    ui.add_space(20.0);
                    if ui.button("Return").clicked() {
                        self.current_window = screens::Screen::Configuracion;
                    }
                });
            });
            if prev_win != self.current_window {
                self.save_to_disk();
            }
        } else if self.current_window == screens::Screen::Vaults {
            let prev_win = self.current_window;
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Vaults");
                    ui.add_space(20.0);
                    screens::vault_management(
                        ui,
                        &mut self.current_window,
                        &mut self.prev_window,
                        &mut self.vault_vec,
                        &mut self.vault,
                        &mut self.new_vault_str,
                        &mut self.create_new_vault,
                        &mut self.new_vault_folder,
                        &mut self.new_vault_folder_err,
                        &mut self.show_create_button,
                        &mut self.vault_changed,
                        [ui.available_width() * 0.8, 40.0],
                    );
                    ui.add_space(20.0);
                    if ui.button("Return").clicked() {
                        self.current_window = screens::Screen::Configuracion;
                    }
                });
            });
            if prev_win != self.current_window {
                self.save_to_disk();
            }
        } else if self.current_window == screens::Screen::Server {
            screens::set_server(ctx);
        };

        #[cfg(target_os = "android")]
        {
            self.keyboard.show(ctx);
        }

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
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_to_disk();
    }
}

impl Marmol {
    fn save_to_disk(&mut self) {
        let state = configuraciones::MarmolProgramState {
            vault: self.vault.clone(),
            vault_vec: self.vault_vec.clone(),
            current_file: Some(self.current_file.clone()),
            initial_screen: self.current_window,
            collapsed_left: self.left_collpased,
            center_size: self.center_size,
            sort_files: self.sort_files,
            dock_state: self.tabs.dock_state().clone(),
            enable_icon_folder: self.enable_icon_folder,
            android_storage: Some(self.android_storage),
        };
        configuraciones::save_program_state(&state);

        let context_path = String::from(&self.config_path) + "/ContextState";
        if let Ok(mut file2) = fs::File::create(context_path) {
            let font_size = format!("font_size: {}", &self.font_size);
            let _ = file2.write_all(font_size.as_str().as_bytes());
        }
    }

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
                ui.selectable_value(&mut self.new_file_type, NewFileType::Excalidraw, "Excalidraw");
                ui.selectable_value(&mut self.new_file_type, NewFileType::Canvas, "Canvas");
            });
        let path = if self.new_file_type == NewFileType::Tasks {
            format!("{}.graph", new_path)
        } else if self.new_file_type == NewFileType::Income {
            format!("{}.inc", new_path)
        } else if self.new_file_type == NewFileType::Excalidraw {
            format!("{}.excalidraw.md", new_path)
        } else if self.new_file_type == NewFileType::Canvas {
            format!("{}.canvas", new_path)
        } else {
            String::new()
        };
        let new_file = if self.new_file_type == NewFileType::Markdown {
            Path::new(&new_path)
        } else {
            Path::new(&path)
        };
        ui.label(
            RichText::new(&self.create_file_error).color(ui.ctx().style().visuals.error_fg_color),
        );
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
                        } else if self.new_file_type == NewFileType::Excalidraw {
                            let json_contents = String::from("{\"type\":\"excalidraw\",\"version\":2,\"source\":\"https://excalidraw.com\",\"elements\":[],\"appState\":{\"viewBackgroundColor\":\"#ffffff\"},\"files\":{}}");
                            let compressed = lz_str::compress_to_base64(&json_contents);
                            let full_content = format!(
"---

excalidraw-plugin: parsed
tags: [excalidraw]

---
==⚠  Switch to EXCALIDRAW VIEW in the MORE OPTIONS menu of this document. ⚠== You can decompress Drawing data with the command palette: 'Decompress current Excalidraw file'. For more info check in plugin settings under 'Saving'


# Excalidraw Data

## Text Elements

%%
## Drawing
```compressed-json
{}
```", compressed);
                            re.write_all(full_content.as_bytes()).unwrap();
                            re.write_all(full_content.as_bytes()).unwrap();
                        } else if self.new_file_type == NewFileType::Canvas {
                            let contents = String::from("{\"nodes\":[],\"edges\":[]}");
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

#[cfg(target_os = "android")]
use egui_winit::winit;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use eframe::Renderer;

    std::env::set_var("RUST_BACKTRACE", "full");

    // Capture internal data path for Android
    if let Some(path) = app.internal_data_path() {
        std::env::set_var("MARMOL_DATA_DIR", path.to_string_lossy().to_string());
    }

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app.clone()),
        renderer: Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "Marmol",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let mut m = Marmol::new(cc);
            #[cfg(target_os = "android")]
            {
                m.android_app = Some(app);
            }
            Ok(Box::new(m))
        }),
    )
    .unwrap();
}
