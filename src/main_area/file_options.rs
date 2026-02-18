use std::fs;
use std::path::Path;

use crate::files;
use eframe::egui::{Button, RichText};

pub fn file_options(
    ui: &mut egui::Ui,
    s: &str,
    _path: &str,
    rename: &mut String,
    renaming_path: &mut Option<String>,
    error: &mut String,
    vault: &str,
) {
    ui.label(RichText::new(&*error).color(ui.ctx().style().visuals.error_fg_color));
    let copy = egui::Button::new("Copy file").frame(false);
    let star = egui::Button::new("Star this file").frame(false);
    let path_s = Path::new(s).file_name().unwrap();

    if ui.add(copy).clicked() {
        let tmp = s.to_owned() + ".copy";
        files::copy_file(s, &tmp);
    }

    if ui.add(star).clicked() {
        files::add_starred(vault, s);
        ui.close();
    }

    if ui.button("Rename").clicked() {
        //TODO Test Funtionality
        *renaming_path = Some(s.to_string());
        *rename = Path::new(s)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        ui.close();
    }

    ui.separator();

    let id = ui.make_persistent_id(format!("del_{}", s));

    if ui.data(|d| d.get_temp(id).unwrap_or(false)) {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("Are you sure?")
                    .strong()
                    .color(ui.ctx().style().visuals.error_fg_color),
            );
            ui.label(RichText::new(path_s.to_str().unwrap()).italics().weak());
            ui.add_space(5.);

            ui.horizontal(|ui| {
                if ui.button("No").clicked() {
                    ui.data_mut(|d| d.insert_temp(id, false));
                }

                let btn_yes = egui::Button::new(
                    RichText::new("Yes, Delete")
                        .color(ui.ctx().style().visuals.widgets.active.fg_stroke.color),
                )
                .fill(ui.ctx().style().visuals.error_fg_color);

                if ui.add(btn_yes).clicked() {
                    let res = files::delete_file(s);
                    if res {
                        *error = String::new();
                        ui.data_mut(|d| d.remove_temp::<bool>(id));
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("file_deleted_signal"), Some(s.to_string()))
                        });
                        ui.close();
                    }
                }
            });
        });
    } else {
        let btn_delete = Button::selectable(
            false,
            RichText::new("Delete file").color(ui.ctx().style().visuals.error_fg_color),
        );

        if ui.add(btn_delete).clicked() {
            ui.data_mut(|d| d.insert_temp(id, true));
        }
    }
}

pub fn folder_options(ui: &mut egui::Ui, s: &str, _path: &str, error: &mut String) {
    ui.label(RichText::new(&*error).color(ui.ctx().style().visuals.error_fg_color));
    let path_s = Path::new(s).file_name().unwrap();

    let id = ui.make_persistent_id(format!("del_dir_{}", s));

    if ui.data(|d| d.get_temp(id).unwrap_or(false)) {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("Are you sure you want to delete this folder and all its contents?")
                    .strong()
                    .color(ui.ctx().style().visuals.error_fg_color),
            );
            ui.label(RichText::new(path_s.to_str().unwrap()).italics().weak());
            ui.add_space(5.);

            ui.horizontal(|ui| {
                if ui.button("No").clicked() {
                    ui.data_mut(|d| d.insert_temp(id, false));
                }

                let btn_yes = egui::Button::new(
                    RichText::new("Yes, Delete Everything")
                        .color(ui.ctx().style().visuals.widgets.active.fg_stroke.color),
                )
                .fill(ui.ctx().style().visuals.error_fg_color);

                if ui.add(btn_yes).clicked() {
                    files::delete_folder(s);
                    ui.close();
                }
            });
        });
    } else {
        let btn_delete = Button::selectable(
            false,
            RichText::new("Delete Folder").color(ui.ctx().style().visuals.error_fg_color),
        );

        if ui.add(btn_delete).clicked() {
            ui.data_mut(|d| d.insert_temp(id, true));
        }
    }
}
