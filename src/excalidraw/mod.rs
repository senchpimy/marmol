use base64::{engine::general_purpose, Engine as _};
use egui::UiBuilder;
use egui::{
    ColorImage, Context, PointerButton, Pos2, Rect,
    Sense, Stroke, TextureHandle, TextureOptions, Ui, Vec2,
};
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

pub mod data;
pub mod utils;
pub mod render;
pub mod ui_panel;

use data::{ExcalidrawElement, ExcalidrawFile, ExcalidrawScene, Tool};
use utils::{is_point_inside, move_element_group};
use render::{draw_element, draw_selection_border};
use ui_panel::show_properties_panel;

#[derive(Serialize, Deserialize)]
pub struct ExcalidrawGui {
    path: String,
    #[serde(skip)]
    scene: Option<ExcalidrawScene>,
    #[serde(skip)]
    error_msg: Option<String>,
    pan: Vec2,
    scale: f32,
    #[serde(skip)]
    active_tool: Option<Tool>,
    #[serde(skip)]
    selected_element_idx: Option<usize>,
    #[serde(skip)]
    default_props: ExcalidrawElement,
    #[serde(skip)]
    dragged_element_idx: Option<usize>,
    #[serde(skip)]
    last_mouse_pos_world: Option<Pos2>,
    #[serde(skip)]
    drawing_start_pos: Option<Pos2>,
    #[serde(skip)]
    drawing_element: Option<ExcalidrawElement>,
    #[serde(skip)]
    is_dirty: bool,
    #[serde(skip)]
    is_panning: bool,
    #[serde(skip)]
    texture_cache: HashMap<String, TextureHandle>,
    #[serde(skip)]
    failed_textures: HashSet<String>,
}

impl Default for ExcalidrawGui {
    fn default() -> Self {
        Self {
            path: String::new(),
            scene: None,
            error_msg: None,
            pan: Vec2::ZERO,
            scale: 1.0,
            active_tool: Some(Tool::Selection),
            selected_element_idx: None,
            default_props: ExcalidrawElement::default(),
            dragged_element_idx: None,
            last_mouse_pos_world: None,
            drawing_start_pos: None,
            drawing_element: None,
            is_dirty: false,
            is_panning: false,
            texture_cache: HashMap::new(),
            failed_textures: HashSet::new(),
        }
    }
}

impl ExcalidrawGui {
    pub fn set_path(&mut self, path: &str) {
        if self.path != path || self.scene.is_none() {
            self.path = path.to_string();
            self.reload();
        }
    }

    pub fn reload(&mut self) {
        if self.path.is_empty() {
            return;
        }
        match fs::read_to_string(&self.path) {
            Ok(json) => match serde_json::from_str::<ExcalidrawScene>(&json) {
                Ok(mut sc) => {
                    for (i, el) in sc.elements.iter_mut().enumerate() {
                        if el.id.is_empty() {
                            el.id = format!("gen_{}", i);
                        }
                    }
                    self.scene = Some(sc);
                    self.error_msg = None;
                    self.is_dirty = false;
                    self.texture_cache.clear();
                    self.failed_textures.clear();
                    if self.scale == 0.0 {
                        self.scale = 1.0;
                    }
                    if self.active_tool.is_none() {
                        self.active_tool = Some(Tool::Selection);
                    }
                }
                Err(e) => self.error_msg = Some(format!("Error parsing: {}", e)),
            },
            Err(e) => self.error_msg = Some(format!("Error reading: {}", e)),
        }
    }

    fn save_file(&mut self) {
        if let Some(scene) = &self.scene {
            if let Ok(json) = serde_json::to_string_pretty(scene) {
                let _ = fs::write(&self.path, json);
                self.is_dirty = false;
            }
        }
    }

    fn get_or_load_texture(
        &mut self,
        ctx: &Context,
        fid: &str,
        files: &HashMap<String, ExcalidrawFile>,
    ) -> Option<TextureHandle> {
        if let Some(h) = self.texture_cache.get(fid) {
            return Some(h.clone());
        }
        if self.failed_textures.contains(fid) {
            return None;
        }
        if let Some(fd) = files.get(fid) {
            if fd.mime_type.contains("svg") {
                self.failed_textures.insert(fid.to_string());
                return None;
            }
            let parts: Vec<&str> = fd.data_url.split(',').collect();
            if parts.len() == 2 {
                if let Ok(b) = general_purpose::STANDARD.decode(parts[1]) {
                    if let Ok(mut img) = image::load_from_memory(&b) {
                        if img.width() > 1024 || img.height() > 1024 {
                            img = img.resize(1024, 1024, FilterType::Triangle);
                        }
                        let t = ctx.load_texture(
                            fid,
                            ColorImage::from_rgba_unmultiplied(
                                [img.width() as _, img.height() as _],
                                img.to_rgba8().as_flat_samples().as_slice(),
                            ),
                            TextureOptions::LINEAR,
                        );
                        self.texture_cache.insert(fid.to_string(), t.clone());
                        return Some(t);
                    }
                }
            }
        }
        self.failed_textures.insert(fid.to_string());
        None
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if let Some(e) = &self.error_msg {
            ui.colored_label(ui.ctx().style().visuals.error_fg_color, e);
            return;
        }
        if self.scene.is_none() {
            self.reload();
            if self.scene.is_none() {
                ui.label("Loading...");
                return;
            }
        }

        ui.horizontal(|ui| {
            ui.label("Tools:");
            if ui
                .selectable_label(self.active_tool == Some(Tool::Selection), "✋")
                .clicked()
            {
                self.active_tool = Some(Tool::Selection);
            }
            ui.separator();
            if ui
                .selectable_label(self.active_tool == Some(Tool::Rectangle), "⬜")
                .clicked()
            {
                self.active_tool = Some(Tool::Rectangle);
            }
            if ui
                .selectable_label(self.active_tool == Some(Tool::Ellipse), "⭕")
                .clicked()
            {
                self.active_tool = Some(Tool::Ellipse);
            }
            if ui
                .selectable_label(self.active_tool == Some(Tool::Diamond), "🔶")
                .clicked()
            {
                self.active_tool = Some(Tool::Diamond);
            }
            ui.separator();
            if ui
                .selectable_label(self.active_tool == Some(Tool::Line), "➖")
                .clicked()
            {
                self.active_tool = Some(Tool::Line);
            }
            if ui
                .selectable_label(self.active_tool == Some(Tool::Arrow), "➡")
                .clicked()
            {
                self.active_tool = Some(Tool::Arrow);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾").clicked() {
                    self.save_file();
                }
            });
        });

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        painter.rect_filled(response.rect, 0.0, ui.ctx().style().visuals.extreme_bg_color);

        let panel_width = 220.0;
        let panel_rect = Rect::from_min_size(
            Pos2::new(
                response.rect.max.x - panel_width - 10.0,
                response.rect.min.y + 10.0,
            ),
            Vec2::new(panel_width, 400.0),
        );

        let scroll_delta = ui.input(|i| i.raw_scroll_delta);
        let screen_rect_min = response.rect.min;
        if ui.input(|i| i.modifiers.ctrl) && scroll_delta.y != 0.0 {
            let zf = if scroll_delta.y > 0.0 { 1.1 } else { 0.9 };
            let old_s = self.scale;
            self.scale = (self.scale * zf).clamp(0.1, 10.0);
            if let Some(mp) = response.hover_pos() {
                let mc = mp - screen_rect_min;
                self.pan = mc - ((mc - self.pan) / old_s) * self.scale;
            }
        } else if scroll_delta != Vec2::ZERO {
            self.pan += scroll_delta;
        }

        if response.dragged_by(PointerButton::Middle)
            || (ui.input(|i| i.modifiers.alt) && response.dragged_by(PointerButton::Primary))
        {
            self.pan += response.drag_delta();
            self.is_panning = true;
        } else {
            self.is_panning = false;
        }

        let cp = self.pan;
        let cs = self.scale;
        let to_screen = move |pw: Pos2| screen_rect_min + cp + (pw.to_vec2() * cs);
        let to_world = move |ps: Pos2| Pos2::ZERO + (ps - screen_rect_min - cp) / cs;
        let vis_rect = Rect::from_min_max(to_world(response.rect.min), to_world(response.rect.max));

        if let Some(mut scene) = self.scene.take() {
            let mut dirty = self.is_dirty;
            let mut save = false;
            let tool = self.active_tool.unwrap_or(Tool::Selection);

            let mouse_over_panel = if let Some(mp) = response.hover_pos() {
                panel_rect.contains(mp)
            } else {
                false
            };

            if !mouse_over_panel {
                match tool {
                    Tool::Selection => {
                        if response.drag_started_by(PointerButton::Primary)
                            && !self.is_panning
                            && !ui.input(|i| i.modifiers.alt)
                        {
                            if let Some(mp) = response.interact_pointer_pos() {
                                let wp = to_world(mp);
                                let mut f = false;
                                for (i, el) in scene.elements.iter().enumerate().rev() {
                                    if is_point_inside(el, wp) {
                                        self.dragged_element_idx = Some(i);
                                        self.selected_element_idx = Some(i);
                                        self.last_mouse_pos_world = Some(wp);
                                        f = true;
                                        break;
                                    }
                                }
                                if !f {
                                    self.selected_element_idx = None;
                                }
                            }
                        }
                        if let Some(idx) = self.dragged_element_idx {
                            if response.dragged_by(PointerButton::Primary) {
                                if let Some(mp) = response.interact_pointer_pos() {
                                    let cwp = to_world(mp);
                                    if let Some(lp) = self.last_mouse_pos_world {
                                        let d = cwp - lp;
                                        if d.length_sq() > 0.0001 {
                                            move_element_group(&mut scene.elements, idx, d);
                                            dirty = true;
                                        }
                                    }
                                    self.last_mouse_pos_world = Some(cwp);
                                }
                            }
                        }
                    }
                    _ => {
                        // Dibujo
                        if response.drag_started_by(PointerButton::Primary)
                            && !self.is_panning
                            && !ui.input(|i| i.modifiers.alt)
                        {
                            if let Some(mp) = response.interact_pointer_pos() {
                                let sw = to_world(mp);
                                self.drawing_start_pos = Some(sw);
                                self.selected_element_idx = None;
                                let mut new_el = self.default_props.clone();
                                let ts = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_nanos();
                                new_el.id = format!("gen_{}", ts);
                                new_el.x = sw.x;
                                new_el.y = sw.y;
                                new_el.points = vec![];
                                match tool {
                                    Tool::Line => {
                                        new_el.element_type = "line".into();
                                        new_el.points = vec![[0.0, 0.0]];
                                    }
                                    Tool::Arrow => {
                                        new_el.element_type = "arrow".into();
                                        new_el.points = vec![[0.0, 0.0]];
                                        new_el.end_arrowhead = Some("arrow".into());
                                    }
                                    Tool::Rectangle => new_el.element_type = "rectangle".into(),
                                    Tool::Ellipse => new_el.element_type = "ellipse".into(),
                                    Tool::Diamond => new_el.element_type = "diamond".into(),
                                    _ => {}
                                }
                                self.drawing_element = Some(new_el);
                            }
                        }
                        if let Some(sp) = self.drawing_start_pos {
                            if response.dragged_by(PointerButton::Primary) {
                                if let Some(mp) = response.interact_pointer_pos() {
                                    let cw = to_world(mp);
                                    if let Some(el) = &mut self.drawing_element {
                                        if tool == Tool::Line || tool == Tool::Arrow {
                                            let dx = cw.x - sp.x;
                                            let dy = cw.y - sp.y;
                                            el.points = vec![[0.0, 0.0], [dx, dy]];
                                            el.width = dx.abs();
                                            el.height = dy.abs();
                                        } else {
                                            el.x = sp.x.min(cw.x);
                                            el.y = sp.y.min(cw.y);
                                            el.width = (sp.x - cw.x).abs();
                                            el.height = (sp.y - cw.y).abs();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if response.drag_stopped() {
                self.dragged_element_idx = None;
                self.last_mouse_pos_world = None;
                if let Some(el) = self.drawing_element.take() {
                    if el.width > 2.0 || el.height > 2.0 {
                        scene.elements.push(el);
                        self.selected_element_idx = Some(scene.elements.len() - 1);
                        dirty = true;
                        save = true;
                    }
                    self.active_tool = Some(Tool::Selection);
                }
                if dirty && self.dragged_element_idx.is_some() {
                    save = true;
                }
                self.drawing_start_pos = None;
            }

            let ctx = ui.ctx().clone();
            let files = &scene.files;
            for (i, el) in scene.elements.iter().enumerate() {
                let rect =
                    Rect::from_min_size(Pos2::new(el.x, el.y), Vec2::new(el.width, el.height))
                        .expand(100.0);
                if !vis_rect.intersects(rect) {
                    continue;
                }
                let tex = if let Some(fid) = &el.file_id {
                    self.get_or_load_texture(&ctx, fid, files)
                } else {
                    None
                };
                draw_element(&painter, el, tex, &to_screen, cs);
                if self.selected_element_idx == Some(i) {
                    draw_selection_border(&painter, el, &to_screen, cs, ui);
                }
            }
            if let Some(el) = &self.drawing_element {
                draw_element(&painter, el, None, &to_screen, cs);
            }

            ui.scope_builder(UiBuilder::new().max_rect(panel_rect), |ui| {
                egui::Frame::NONE
                    .fill(ui.ctx().style().visuals.panel_fill.linear_multiply(240.0 / 255.0))
                    .stroke(Stroke::new(1.0, ui.ctx().style().visuals.window_stroke.color))
                    .corner_radius(12.0)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.set_width(panel_rect.width() - 32.0);
                        
                        let selected_element = if let Some(idx) = self.selected_element_idx {
                            if idx < scene.elements.len() {
                                Some(&mut scene.elements[idx])
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if show_properties_panel(ui, selected_element, &mut self.default_props) {
                             dirty = true;
                             if self.selected_element_idx.is_some() {
                                 save = true;
                             }
                        }
                    });
            });
            
            self.scene = Some(scene);
            self.is_dirty = dirty;
            if save {
                self.save_file();
            }
        }
    }
}
