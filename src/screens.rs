use serde::{Deserialize, Serialize};

use crate::main_area::content_enum::Content;
//use crate::toggle_switch;
use crate::MShape;
use eframe::egui::{Button, CentralPanel, Color32, FontId, RichText};

use rfd::FileDialog;
use std::fs;
use std::path::Path;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Screen {
    Main,
    Configuracion,
    Default,
    Server,
}

pub fn default(
    ctx: &egui::Context,
    current_window: &mut Screen,
    nuevo: &mut String,
    vaults_vec: &mut Vec<String>,
    vault: &mut String,
    content: &mut Content,
    window_size: &MShape,
) {
    let mut nuevo_bool = false;
    CentralPanel::default().show(ctx, |ui| {
        let text = RichText::new("Marmol").strong().size(60.0);
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            let button_width = window_size.width * 0.5;
            let button_height = 50.0;
            ui.add_space(50.0); // Reduced space to make room for the image

            // Add the SVG image
            let image = egui::Image::new(egui::include_image!("../logo/cubov2.svg"))
                .fit_to_exact_size(egui::vec2(150.0, 150.0)); // Adjust size as needed
            ui.add(image);
            ui.add_space(20.0); // Space between image and title

            ui.label(text);
            ui.add_space(100.0);
            if ui
                .add_sized(
                    [button_width, button_height],
                    egui::Button::new("Select a Vault"),
                )
                .clicked()
            {
                let files = FileDialog::new().set_title("Select a Folder").pick_folder();
                match files {
                    Some(x) => {
                        let selected_vault = x.to_str().unwrap();
                        vaults_vec.push(selected_vault.to_owned());
                        *vault = String::from(selected_vault);
                        *current_window = Screen::Main;
                        *content = Content::Blank;
                    }
                    None => {}
                }
            }
            ui.add_space(30.0);
            ui.add(egui::TextEdit::singleline(nuevo));
            if nuevo.len() > 2 {
                let path = Path::new(nuevo);
                let mut open_text = RichText::new("");
                if !path.exists() {
                    if path.is_dir() {
                        open_text = RichText::new("Good!").color(Color32::GREEN);
                        nuevo_bool = true;
                    }
                } else {
                    open_text = RichText::new("Path already exists").color(Color32::RED);
                }
                ui.label(open_text);
            }
            if ui
                .add_sized(
                    [button_width, button_height],
                    egui::Button::new("Create new Vault"),
                )
                .clicked()
                && nuevo_bool
            {
                unimplemented!();
            };
            ui.add_space(30.0);
            if ui
                .add_sized(
                    [button_width, button_height],
                    egui::Button::new("configuration"),
                )
                .clicked()
            {
                *current_window = Screen::Configuracion;
            };
        });
    });
}

/// Configuration screen
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
    _window_size: &MShape,
) {
    CentralPanel::default().show(ctx, |ui| {
        let button_width = ui.available_width() * 0.5;
        let button_height = 40.0;
        let button_size = [button_width, button_height];

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            // Title
            ui.heading("Configuration");

            // Vault Management
            vault_management(
                ui,
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

            // Appearance Settings
            appearance_settings(
                ui,
                ctx,
                font_size,
                center_size,
                center_size_remain,
                sort_files,
                button_size,
            );

            // Server Settings
            server_settings(ui, current_window, button_size);

            // Return Button
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

/// Vault management section
fn vault_management(
    ui: &mut egui::Ui,
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
    egui::CollapsingHeader::new(RichText::new("Vaults").strong()).show(ui, |ui| {
        // Create a new vault
        create_new_vault(
            ui,
            nw_vault_str,
            show,
            folder,
            error,
            button,
            vaults,
            button_size,
        );

        // Manage existing vaults
        manage_existing_vaults(ui, vaults, vault, vault_changed, button_size);

        // Add an existing vault
        add_existing_vault(ui, vaults, vault, button_size);
    });
}

fn create_new_vault(
    ui: &mut egui::Ui,
    nw_vault_str: &mut String,
    show: &mut bool,
    folder: &mut String,
    error: &mut String,
    button: &mut bool,
    vaults: &mut Vec<String>,
    button_size: [f32; 2],
) {
    if ui
        .add_sized(button_size, egui::Button::new("Create a New Vault"))
        .clicked()
    {
        let files = FileDialog::new().set_title("Select a Folder").pick_folder();
        if let Some(x) = files {
            *show = true;
            *folder = String::from(x.to_str().unwrap());
        } else {
            *show = false;
        }
    }
    if *show {
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
                *button = true;
            }
        }
    }
    if *button {
        if ui
            .add_sized(button_size, egui::Button::new("Create!"))
            .clicked()
        {
            let full_path = format!("{}/{}", folder, nw_vault_str);
            if fs::create_dir(&full_path).is_ok() {
                vaults.push(full_path.clone());
                if fs::create_dir(format!("{}/.obsidian/", full_path)).is_err() {
                    *error = "Failed to create .obsidian folder".to_string();
                }
            } else {
                *error = "Failed to create vault folder".to_string();
            }
            *nw_vault_str = String::new();
            *button = false;
            *show = false;
        }
    }
    ui.label(RichText::new(error.as_str()).color(Color32::RED));
}

fn manage_existing_vaults(
    ui: &mut egui::Ui,
    vaults: &mut Vec<String>,
    vault: &mut String,
    vault_changed: &mut bool,
    button_size: [f32; 2],
) {
    egui::CollapsingHeader::new(RichText::new("Manage Vault").strong()).show(ui, |ui| {
        let mut new_vaults = vaults.clone();
        let mut changed = false;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in &mut *vaults {
                let stri = i.as_str();
                if stri == vault {
                    ui.label(stri);
                } else {
                    let btn = Button::new(stri);
                    let menu =
                        |ui: &mut egui::Ui| remove_vault(ui, stri, &mut new_vaults, &mut changed);
                    let response = ui.add_sized(button_size, btn);
                    if response.clicked() {
                        *vault = String::from(stri);
                        *vault_changed = true;
                    }
                    response.context_menu(menu);
                }
            }
            if changed {
                *vaults = new_vaults;
            }
        });
    });
}

fn add_existing_vault(
    ui: &mut egui::Ui,
    vaults: &mut Vec<String>,
    vault: &mut String,
    button_size: [f32; 2],
) {
    if ui
        .add_sized(button_size, egui::Button::new("Add a Existing Vault"))
        .clicked()
    {
        if let Some(x) = FileDialog::new().set_title("Select a Folder").pick_folder() {
            let selected_vault = x.to_str().unwrap().to_owned();
            if !vaults.contains(&selected_vault) {
                vaults.push(selected_vault.to_owned());
                *vault = selected_vault;
            };
        }
    }
}

/// Appearance settings section
fn appearance_settings(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    font_size: &mut f32,
    center_size: &mut f32,
    center_size_remain: &mut f32,
    sort_files: &mut bool,
    button_size: [f32; 2],
) {
    ui.add_space(10.0);
    egui::CollapsingHeader::new(RichText::new("Appearance").strong()).show(ui, |ui| {
        // Theme selection
        if ui
            .add_sized(button_size, egui::Button::new("Select theme"))
            .clicked()
        {}
        ui.add_space(10.0);

        // File sorting
        ui.checkbox(sort_files, "Show files sorted");
        ui.add_space(10.0);

        // Line length
        if ui
            .add(egui::Slider::new(center_size, 0.35..=0.9).text("Line lenght"))
            .changed()
        {
            *center_size_remain = (1.0 - *center_size) / 2.0;
        };
        ui.add_space(10.0);

        // Font size
        if ui
            .add(egui::Slider::new(font_size, 10.0..=80.0).text("Font size"))
            .changed()
        {
            let mut style = (*ctx.style()).clone();
            let font_id = FontId::proportional(*font_size);
            style.override_font_id = Some(font_id);
            ctx.set_style(style);
        }
    });
}

/// Server settings section
fn server_settings(ui: &mut egui::Ui, current_window: &mut Screen, button_size: [f32; 2]) {
    ui.add_space(10.0);
    egui::CollapsingHeader::new(RichText::new("Server").strong()).show(ui, |ui| {
        // Configure backup server
        if ui
            .add_sized(button_size, egui::Button::new("Configure Backup Server"))
            .clicked()
        {
            *current_window = Screen::Server;
        };
    });
}

fn remove_vault(ui: &mut egui::Ui, s: &str, vec: &mut Vec<String>, changed: &mut bool) {
    if ui.button("Delete").clicked() {
        vec.retain(|x| x != &s);
        *changed = true;
    }
    ui.label("This doens't delete the folder from your system, just from the program acces");
}

pub fn set_server(_ctx: &egui::Context) {}
