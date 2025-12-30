use crate::iconize::IconManager;
use serde::{Deserialize, Serialize};

use crate::main_area::content_enum::Content;
use crate::MShape;
use eframe::egui::{CentralPanel, FontId, RichText};

#[cfg(not(target_os = "android"))]
use rfd::FileDialog;
use std::fs;
use std::path::Path;

use crate::configuraciones::AndroidStorage;

#[cfg(target_os = "android")]
use egui_winit::winit;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Screen {
    Main,
    Configuracion,
    Default,
    Server,
    CreateVault,
    Appearance,
    Vaults,
}

pub fn create_vault_screen(
    ctx: &egui::Context,
    current_window: &mut Screen,
    prev_window: &mut Screen,
    nw_vault_str: &mut String,
    folder: &mut String,
    error: &mut String,
    vaults: &mut Vec<String>,
    vault: &mut String,
    vault_changed: &mut bool,
) {
    CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Create New Vault");
            ui.add_space(20.0);
            
            ui.label(RichText::new(format!("Location: {}", folder)).weak());
            ui.add_space(10.0);

            ui.label("Vault Name:");
            let edit = egui::TextEdit::singleline(nw_vault_str)
                .hint_text("Enter vault name...")
                .desired_width(f32::INFINITY);
            
            let response = ui.add(edit);
            if response.changed() {
                let full_path = format!("{}/{}", folder, nw_vault_str);
                let new_vault = Path::new(&full_path);
                if new_vault.exists() {
                    *error = String::from("Folder already Exists");
                } else {
                    *error = String::new();
                }
            }

            if !error.is_empty() {
                ui.label(RichText::new(error.as_str()).color(ui.visuals().error_fg_color));
            }

            ui.add_space(30.0);

            let btn_width = ui.available_width() * 0.8;
            
            if ui.add_sized([btn_width, 50.0], egui::Button::new("✨ Confirm Creation")).clicked() {
                if nw_vault_str.is_empty() {
                    *error = "Name cannot be empty".to_string();
                } else {
                    let full_path = format!("{}/{}", folder, nw_vault_str);
                    if fs::create_dir(&full_path).is_ok() {
                        let _ = fs::create_dir(format!("{}/.obsidian/", full_path));
                        if !vaults.contains(&full_path) {
                            vaults.push(full_path.clone());
                        }
                        *vault = full_path;
                        *vault_changed = true;
                        #[cfg(target_os = "android")]
                        {
                            *current_window = Screen::Configuracion;
                        }
                        #[cfg(not(target_os = "android"))]
                        {
                            *current_window = Screen::Main;
                        }
                        *nw_vault_str = String::new();
                        *error = String::new();
                    } else {
                        *error = "Failed to create vault folder".to_string();
                    }
                }
            }

            ui.add_space(10.0);

            if ui.add_sized([btn_width, 50.0], egui::Button::new("Cancel")).clicked() {
                *current_window = *prev_window;
                *nw_vault_str = String::new();
                *error = String::new();
            }
        });
    });
}

pub fn default(
    ctx: &egui::Context,
    current_window: &mut Screen,
    prev_window: &mut Screen,
    nuevo_vault_name: &mut String,
    vaults_vec: &mut Vec<String>,
    vault: &mut String,
    content: &mut Content,
    window_size: &MShape,
    show_creation_ui: &mut bool,
    parent_folder: &mut String,
    creation_error: &mut String,
    can_create: &mut bool,
    _android_storage: &mut AndroidStorage,
    #[cfg(target_os = "android")]
    android_app: &Option<egui_winit::winit::platform::android::activity::AndroidApp>,
) {
    CentralPanel::default().show(ctx, |ui| {
        let text = RichText::new("Marmol").strong().size(60.0);
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let button_width = window_size.width * 0.5;
            let button_height = 50.0;
            ui.add_space(50.0);

            let image = egui::Image::new(egui::include_image!("../logo/cubov2.svg"))
                .fit_to_exact_size(egui::vec2(150.0, 150.0));
            ui.add(image);
            ui.add_space(20.0);

            ui.label(text);
            ui.add_space(40.0);

            if !vaults_vec.is_empty() {
                ui.label(RichText::new("Recent Vaults").strong().size(20.0));
                ui.add_space(10.0);
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let mut to_remove = None;
                        for (idx, v) in vaults_vec.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui
                                    .add_sized([button_width * 0.8, 30.0], egui::Button::new(v))
                                    .clicked()
                                {
                                    *vault = v.clone();
                                    *current_window = Screen::Main;
                                    *content = Content::Blank;
                                }
                                if ui.button("🗑").on_hover_text("Remove from list").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                            ui.add_space(5.0);
                        }
                        if let Some(idx) = to_remove {
                            vaults_vec.remove(idx);
                        }
                    });
                ui.add_space(20.0);
            }

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - button_width) / 2.0);
                ui.vertical(|ui| {
                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new("📂 Open existing Vault"),
                        )
                        .clicked()
                    {
                        #[cfg(not(target_os = "android"))]
                        if let Some(x) = FileDialog::new().set_title("Select a Folder").pick_folder() {
                            let selected_vault = x.to_str().unwrap().to_owned();
                            if !vaults_vec.contains(&selected_vault) {
                                vaults_vec.push(selected_vault.clone());
                            }
                            *vault = selected_vault;
                            *current_window = Screen::Main;
                            *content = Content::Blank;
                        }
                        #[cfg(target_os = "android")]
                        {
                           // On Android, we might just want to ask for a path manually for now
                           // or use a different file picker.
                        }
                    }

                    ui.add_space(10.0);

                    if ui
                        .add_sized(
                            [button_width, button_height],
                            egui::Button::new("✨ Create new Vault"),
                        )
                        .clicked()
                    {
                        #[cfg(not(target_os = "android"))]
                        if let Some(x) = FileDialog::new().set_title("Select Parent Folder").pick_folder() {
                            *show_creation_ui = true;
                            *parent_folder = x.to_str().unwrap().to_owned();
                        }
                        #[cfg(target_os = "android")]
                        {
                            *show_creation_ui = true;
                            *parent_folder = if *_android_storage == AndroidStorage::Internal {
                                crate::configuraciones::get_config_dir()
                            } else {
                                crate::configuraciones::get_external_dir()
                            };
                        }
                    }

                    if *show_creation_ui {
                        #[cfg(target_os = "android")]
                        {
                            ui.horizontal(|ui| {
                                ui.label("Storage:");
                                if ui.radio_value(_android_storage, AndroidStorage::Unselected, "None").clicked() {
                                    *parent_folder = String::new();
                                }
                                if ui.radio_value(_android_storage, AndroidStorage::Internal, "App Internal").clicked() {
                                    *parent_folder = crate::configuraciones::get_config_dir();
                                }
                    if ui.radio_value(_android_storage, AndroidStorage::System, "System (Documents)").clicked() {
                        *parent_folder = crate::configuraciones::get_external_dir();
                        #[cfg(target_os = "android")]
                        if let Some(app) = android_app {
                            request_android_permission(app);
                        }
                    }
                            });
                            if *_android_storage == AndroidStorage::System {
                                ui.horizontal(|ui| {
                                    ui.label("Path:");
                                    ui.text_edit_singleline(parent_folder);
                                });
                                ui.label(RichText::new("⚠️ You must grant 'All Files Access' in Android Settings for this to work.").small().color(ui.ctx().style().visuals.warn_fg_color));
                            }
                        }
                        ui.add_space(10.0);
                        if !parent_folder.is_empty() {
                            ui.label(format!("Creating in: {}", parent_folder));
                        } else {
                            ui.label(RichText::new("Please select a storage type").small().color(ui.ctx().style().visuals.warn_fg_color));
                        }
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            let response = ui.add(egui::TextEdit::singleline(nuevo_vault_name));
                            if response.changed() {
                                let full_path = format!("{}/{}", parent_folder, nuevo_vault_name);
                                if Path::new(&full_path).exists() {
                                    *creation_error = "Folder already exists".to_string();
                                    *can_create = false;
                                } else {
                                    *creation_error = String::new();
                                    *can_create = !nuevo_vault_name.is_empty();
                                }
                            }
                        });

                        if !creation_error.is_empty() {
                            ui.label(RichText::new(creation_error.as_str()).color(ui.ctx().style().visuals.error_fg_color));
                        }

                        ui.horizontal(|ui| {
                            if ui.add_enabled(*can_create, egui::Button::new("Confirm")).clicked() {
                                let full_path = format!("{}/{}", parent_folder, nuevo_vault_name);
                                if fs::create_dir_all(&full_path).is_ok() {
                                    let _ = fs::create_dir_all(format!("{}/.obsidian", full_path));
                                    vaults_vec.push(full_path.clone());
                                    *vault = full_path;
                                    *prev_window = *current_window;
                                    #[cfg(target_os = "android")]
                                    {
                                        *current_window = Screen::Configuracion;
                                    }
                                    #[cfg(not(target_os = "android"))]
                                    {
                                        *current_window = Screen::Main;
                                    }
                                    *content = Content::Blank;
                                    *show_creation_ui = false;
                                    *nuevo_vault_name = String::new();
                                } else {
                                    *creation_error = "Failed to create directory".to_string();
                                }
                            }
                            if ui.button("Cancel").clicked() {
                                *show_creation_ui = false;
                                *nuevo_vault_name = String::new();
                            }
                        });
                    }

                    ui.add_space(20.0);
                    if ui
                        .add_sized(
                            [button_width, 40.0],
                            egui::Button::new("⚙ Configuration"),
                        )
                        .clicked()
                    {
                        *prev_window = *current_window;
                        *current_window = Screen::Configuracion;
                    };
                });
            });
        });
    });
}

pub fn configuracion(
    ctx: &egui::Context,
    prev_window: &mut Screen,
    current_window: &mut Screen,
    vaults: &mut Vec<String>,
    vault: &mut String,
    nw_vault_str: &mut String,
    show: &mut bool,
    folder: &mut String,
    error: &mut String,
    button: &mut bool,
    vault_changed: &mut bool,
    font_size: &mut f32,
    center_size: &mut f32,
    center_size_remain: &mut f32,
    sort_files: &mut bool,
    enable_icon_folder: &mut bool,  // Este parámetro se usa ahora
    icon_manager: &mut IconManager, // AGREGAR ESTO en los argumentos de la función
    _window_size: &MShape,
) {
    CentralPanel::default().show(ctx, |ui| {
        let button_width = ui.available_width() * 0.5;
        let button_height = 40.0;
        let button_size = [button_width, button_height];

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.heading("Configuration");

            #[cfg(not(target_os = "android"))]
            {
                vault_management(
                    ui,
                    current_window,
                    prev_window,
                    vaults,
                    vault,
                    nw_vault_str,
                    show,
                    folder,
                    error,
                    button,
                    vault_changed,
                    button_size,
                );

                appearance_settings(
                    ui,
                    ctx,
                    vault,
                    font_size,
                    center_size,
                    center_size_remain,
                    sort_files,
                    enable_icon_folder,
                    icon_manager,
                    button_size,
                );
            }

            #[cfg(target_os = "android")]
            {
                ui.add_space(10.0);
                if ui.add_sized(button_size, egui::Button::new("📂 Vaults")).clicked() {
                    *current_window = Screen::Vaults;
                }
                ui.add_space(10.0);
                if ui.add_sized(button_size, egui::Button::new("🎨 Appearance")).clicked() {
                    *current_window = Screen::Appearance;
                }
            }

            server_settings(ui, current_window, button_size);

            ui.add_space(30.0);
            if ui
                .add_sized(button_size, egui::Button::new("Return"))
                .clicked()
            {
                *current_window = *prev_window;
            };
        });
    });
}

pub fn vault_management(
    ui: &mut egui::Ui,
    current_window: &mut Screen,
    prev_window: &mut Screen,
    vaults: &mut Vec<String>,
    vault: &mut String,
    nw_vault_str: &mut String,
    show: &mut bool,
    folder: &mut String,
    error: &mut String,
    button: &mut bool,
    vault_changed: &mut bool,
    button_size: [f32; 2],
) {
    #[cfg(not(target_os = "android"))]
    egui::CollapsingHeader::new(RichText::new("Vaults").strong()).show(ui, |ui| {
        vault_management_content(
            ui,
            current_window,
            prev_window,
            vaults,
            vault,
            nw_vault_str,
            show,
            folder,
            error,
            button,
            vault_changed,
            button_size,
        );
    });
    #[cfg(target_os = "android")]
    vault_management_content(
        ui,
        current_window,
        prev_window,
        vaults,
        vault,
        nw_vault_str,
        show,
        folder,
        error,
        button,
        vault_changed,
        button_size,
    );
}

fn vault_management_content(
    ui: &mut egui::Ui,
    current_window: &mut Screen,
    prev_window: &mut Screen,
    vaults: &mut Vec<String>,
    vault: &mut String,
    nw_vault_str: &mut String,
    show: &mut bool,
    folder: &mut String,
    error: &mut String,
    button: &mut bool,
    vault_changed: &mut bool,
    button_size: [f32; 2],
) {
    create_new_vault(
        ui,
        nw_vault_str,
        show,
        folder,
        error,
        button,
        vaults,
        vault,
        current_window,
        prev_window,
        vault_changed,
        button_size,
    );
    manage_existing_vaults(ui, vaults, vault, vault_changed, button_size);
    add_existing_vault(ui, vaults, vault, current_window, vault_changed, button_size);
}

fn create_new_vault(
    ui: &mut egui::Ui,
    nw_vault_str: &mut String,
    show: &mut bool,
    folder: &mut String,
    error: &mut String,
    button: &mut bool,
    vaults: &mut Vec<String>,
    vault: &mut String,
    current_window: &mut Screen,
    prev_window: &mut Screen,
    vault_changed: &mut bool,
    button_size: [f32; 2],
) {
    if ui
        .add_sized(button_size, egui::Button::new("✨ Create a New Vault"))
        .clicked()
    {
        #[cfg(not(target_os = "android"))]
        {
            let files = FileDialog::new().set_title("Select Parent Folder").pick_folder();
            if let Some(x) = files {
                *show = true;
                *folder = String::from(x.to_str().unwrap());
            } else {
                *show = false;
            }
        }
        #[cfg(target_os = "android")]
        {
            *prev_window = *current_window;
            *current_window = Screen::CreateVault;
            *folder = crate::configuraciones::get_config_dir();
        }
    }
    #[cfg(not(target_os = "android"))]
    if *show {
        ui.label(format!("Creating in: {}", folder));
        let edit = egui::TextEdit::singleline(nw_vault_str);
        let response = ui.add(edit);
        if response.changed() {
            let full_path = format!("{}/{}", folder, nw_vault_str);
            let new_vault = Path::new(&full_path);
            if new_vault.exists() {
                *error = String::from("Folder already Exists");
                *button = false;
            } else {
                *error = String::new();
                *button = !nw_vault_str.is_empty();
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    if *button {
        if ui
            .add_sized(button_size, egui::Button::new("Confirm Creation"))
            .clicked()
        {
            let full_path = format!("{}/{}", folder, nw_vault_str);
            if fs::create_dir(&full_path).is_ok() {
                let _ = fs::create_dir(format!("{}/.obsidian/", full_path));
                if !vaults.contains(&full_path) {
                    vaults.push(full_path.clone());
                }
                *vault = full_path;
                *vault_changed = true;
                *current_window = Screen::Main;
            } else {
                *error = "Failed to create vault folder".to_string();
            }
            *nw_vault_str = String::new();
            *button = false;
            *show = false;
        }
    }
    ui.label(RichText::new(error.as_str()).color(ui.ctx().style().visuals.error_fg_color));
}

fn manage_existing_vaults(
    ui: &mut egui::Ui,
    vaults: &mut Vec<String>,
    vault: &mut String,
    vault_changed: &mut bool,
    button_size: [f32; 2],
) {
    egui::CollapsingHeader::new(RichText::new("Manage Vaults").strong()).show(ui, |ui| {
        let mut to_remove = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, v) in vaults.iter().enumerate() {
                ui.horizontal(|ui| {
                    let is_current = v == vault;
                    let btn_text = if is_current {
                        format!("{} (Current)", v)
                    } else {
                        v.clone()
                    };

                    if ui
                        .add_sized([button_size[0] * 0.85, 30.0], egui::Button::new(btn_text))
                        .clicked()
                    {
                        if !is_current {
                            *vault = v.clone();
                            *vault_changed = true;
                        }
                    }
                    if ui.button("🗑").on_hover_text("Remove from list").clicked() {
                        to_remove = Some(idx);
                    }
                });
                ui.add_space(5.0);
            }
        });
        if let Some(idx) = to_remove {
            vaults.remove(idx);
        }
    });
}

fn add_existing_vault(
    ui: &mut egui::Ui,
    vaults: &mut Vec<String>,
    vault: &mut String,
    current_window: &mut Screen,
    vault_changed: &mut bool,
    button_size: [f32; 2],
) {
    if ui
        .add_sized(button_size, egui::Button::new("📂 Add an Existing Vault"))
        .clicked()
    {
        #[cfg(not(target_os = "android"))]
        if let Some(x) = FileDialog::new().set_title("Select a Folder").pick_folder() {
            let selected_vault = x.to_str().unwrap().to_owned();
            if !vaults.contains(&selected_vault) {
                vaults.push(selected_vault.clone());
            }
            *vault = selected_vault;
            *vault_changed = true;
            *current_window = Screen::Main;
        }
        #[cfg(target_os = "android")]
        {
            // Manual input maybe?
        }
    }
}

pub fn appearance_settings(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    vault: &str,
    font_size: &mut f32,
    center_size: &mut f32,
    center_size_remain: &mut f32,
    sort_files: &mut bool,
    enable_icon_folder: &mut bool,
    icon_manager: &mut IconManager,
    _button_size: [f32; 2],
) {
    ui.add_space(10.0);
    #[cfg(not(target_os = "android"))]
    egui::CollapsingHeader::new(RichText::new("Appearance").strong()).show(ui, |ui| {
        appearance_settings_content(
            ui,
            ctx,
            vault,
            font_size,
            center_size,
            center_size_remain,
            sort_files,
            enable_icon_folder,
            icon_manager,
        );
    });
    #[cfg(target_os = "android")]
    appearance_settings_content(
        ui,
        ctx,
        vault,
        font_size,
        center_size,
        center_size_remain,
        sort_files,
        enable_icon_folder,
        icon_manager,
    );
}

fn appearance_settings_content(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    vault: &str,
    font_size: &mut f32,
    center_size: &mut f32,
    center_size_remain: &mut f32,
    sort_files: &mut bool,
    enable_icon_folder: &mut bool,
    icon_manager: &mut IconManager,
) {
    ui.checkbox(sort_files, "Show files sorted");
    ui.add_space(5.0);

    ui.separator();
    ui.checkbox(enable_icon_folder, "Enable Obsidian Icon Folder");

    if *enable_icon_folder {
        let old_settings = icon_manager.settings.clone();

        ui.indent("icons_settings", |ui| {
            let s = &mut icon_manager.settings;

            ui.label(RichText::new("Icon Folder Settings").strong());

            ui.horizontal(|ui| {
                ui.label("Icon Packs Path:");
                ui.text_edit_singleline(&mut s.icon_packs_path);
            });
            ui.add(egui::Slider::new(&mut s.font_size, 8.0..=32.0).text("Icon Font Size"));

            ui.collapsing("Visibility & Position", |ui| {
                ui.checkbox(&mut s.icon_in_tabs_enabled, "Icon in Tabs");
                ui.checkbox(&mut s.icon_in_title_enabled, "Icon in Title");
                if s.icon_in_title_enabled {
                    egui::ComboBox::from_label("Position")
                        .selected_text(&s.icon_in_title_position)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut s.icon_in_title_position,
                                "above".to_string(),
                                "Above",
                            );
                            ui.selectable_value(
                                &mut s.icon_in_title_position,
                                "inline".to_string(),
                                "Inline",
                            );
                        });
                }
                ui.checkbox(&mut s.icons_in_notes_enabled, "Icons in Notes");
                ui.checkbox(&mut s.icons_in_links_enabled, "Icons in Links");
            });

            ui.collapsing("Margins", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Top:");
                    ui.add(egui::DragValue::new(&mut s.extra_margin.top));
                    ui.label("Right:");
                    ui.add(egui::DragValue::new(&mut s.extra_margin.right));
                });
                ui.horizontal(|ui| {
                    ui.label("Bottom:");
                    ui.add(egui::DragValue::new(&mut s.extra_margin.bottom));
                    ui.label("Left:");
                    ui.add(egui::DragValue::new(&mut s.extra_margin.left));
                });
            });

            ui.collapsing("Frontmatter", |ui| {
                ui.checkbox(&mut s.icon_in_frontmatter_enabled, "Use Frontmatter");
                if s.icon_in_frontmatter_enabled {
                    ui.text_edit_singleline(&mut s.icon_in_frontmatter_field_name)
                        .on_hover_text("Field Name");
                }
            });

            ui.checkbox(&mut s.debug_mode, "Debug Mode");
        });

        if old_settings != icon_manager.settings {
            icon_manager.save_settings(vault);
        }
        ui.separator();
    }

    if ui
        .add(egui::Slider::new(center_size, 0.35..=0.9).text("Line length"))
        .changed()
    {
        *center_size_remain = (1.0 - *center_size) / 2.0;
    };
    ui.add_space(10.0);

    if ui
        .add(egui::Slider::new(font_size, 10.0..=80.0).text("Font size"))
        .changed()
    {
        let mut style = (*ctx.style()).clone();
        let font_id = FontId::proportional(*font_size);
        style.override_font_id = Some(font_id);
        ctx.set_style(style);
    }
}

fn server_settings(ui: &mut egui::Ui, current_window: &mut Screen, button_size: [f32; 2]) {
    ui.add_space(10.0);
    #[cfg(not(target_os = "android"))]
    egui::CollapsingHeader::new(RichText::new("Server").strong()).show(ui, |ui| {
        if ui
            .add_sized(button_size, egui::Button::new("Configure Backup Server"))
            .clicked()
        {
            *current_window = Screen::Server;
        };
    });
    #[cfg(target_os = "android")]
    if ui.add_sized(button_size, egui::Button::new("☁ Server")).clicked() {
        *current_window = Screen::Server;
    }
}

pub fn set_server(_ctx: &egui::Context) {}



#[cfg(target_os = "android")]



fn request_android_permission(app: &egui_winit::winit::platform::android::activity::AndroidApp) {







    use jni::{objects::JValue, JavaVM};



    let vm = match unsafe { JavaVM::from_raw(app.vm_as_ptr() as _) } {



        Ok(vm) => vm,



        Err(e) => { log::error!("Failed to get JavaVM: {:?}", e); return; }



    };



    let mut env = match vm.attach_current_thread() {



        Ok(env) => env,



        Err(e) => { log::error!("Failed to attach thread: {:?}", e); return; }



    };



    let activity = unsafe { jni::objects::JObject::from_raw(app.activity_as_ptr() as _) };



    let launch_intent = |env: &mut jni::JNIEnv| -> Result<(), Box<dyn std::error::Error>> {



        let pkg_name_obj = env.call_method(&activity, "getPackageName", "()Ljava/lang/String;", &[])?.l()?;



        let pkg_name: String = env.get_string(&pkg_name_obj.into())?.into();



        let action = env.new_string("android.settings.MANAGE_APP_ALL_FILES_ACCESS_PERMISSION")?;



        let uri_str = env.new_string(format!("package:{}", pkg_name))?;



        let uri = env.call_static_method("android/net/Uri", "parse", "(Ljava/lang/String;)Landroid/net/Uri;", &[JValue::from(&uri_str)])?.l()?;



        let intent = env.new_object("android/content/Intent", "(Ljava/lang/String;Landroid/net/Uri;)V", &[JValue::from(&action), JValue::from(&uri)])?;



        env.call_method(&activity, "startActivity", "(Landroid/content/Intent;)V", &[JValue::from(&intent)])?;



        Ok(())



    };



    if let Err(e) = launch_intent(&mut env) {



        log::error!("Error launching permission intent: {:?}", e);



    }



}




