use base64::{engine::general_purpose, Engine as _};
use lz_str;
use egui::UiBuilder;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

use egui::{
    Color32, ColorImage, Context, PointerButton, Pos2, Rect,
    Sense, Stroke, TextureHandle, TextureOptions, Ui, Vec2, Id,
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
    #[serde(skip)]
    selection_rect: Option<Rect>,
    #[serde(skip)]
    selected_indices: HashSet<usize>,
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
            selection_rect: None,
            selected_indices: HashSet::new(),
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
            Ok(content) => {
                let json_to_parse: Option<String> = if content.trim().starts_with('{') {
                    Some(content)
                } else {
                    // Intentar extraer de bloque ```json o ```compressed-json
                    if let Some(start) = content.find("```json\n") {
                        let rest = &content[start + 8..];
                        if let Some(end) = rest.find("\n```") {
                            Some(rest[..end].to_string())
                        } else {
                            None
                        }
                    } else if let Some(start) = content.find("```compressed-json\n") {
                        let rest = &content[start + 19..];
                        if let Some(end) = rest.find("\n```") {
                            let compressed = rest[..end].trim();
                            lz_str::decompress_from_base64(compressed)
                                .and_then(|v| String::from_utf16(&v).ok())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(json) = json_to_parse {
                    match serde_json::from_str::<ExcalidrawScene>(&json) {
                        Ok(mut sc) => {
                            // Ensure type is correct and not duplicated in extra
                            sc.type_ = "excalidraw".to_string();
                            sc.extra.remove("type");
                            sc.extra.remove("type_");

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
                        Err(e) => {
                            self.error_msg = Some(format!("Error parsing: {}", e));
                        }
                    }
                } else {
                    self.error_msg = Some("Could not find valid drawing data in file".to_string());
                }
            }
            Err(e) => self.error_msg = Some(format!("Error reading: {}", e)),
        }
    }

    fn save_file(&mut self) {
        if let Some(scene) = &self.scene {
            if let Ok(json) = serde_json::to_string(scene) {
                let compressed = lz_str::compress_to_base64(&json);
                let mut text_elements = String::new();
                for el in &scene.elements {
                    if el.element_type == "text" && !el.text.is_empty() {
                        text_elements.push_str(&format!("{} ^{}\n\n", el.text, el.id));
                    }
                }

                let full_content = format!(
"---

excalidraw-plugin: parsed
tags: [excalidraw]

---
==⚠  Switch to EXCALIDRAW VIEW in the MORE OPTIONS menu of this document. ⚠== You can decompress Drawing data with the command palette: 'Decompress current Excalidraw file'. For more info check in plugin settings under 'Saving'


# Excalidraw Data

## Text Elements
{}

%%
## Drawing
```compressed-json
{}
```
%%", text_elements, compressed);

                let _ = fs::write(&self.path, full_content);
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

    pub fn show(&mut self, ui: &mut Ui, vault: &str, _seed_id: Id) {
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
            let color = ui.ctx().style().visuals.widgets.noninteractive.fg_stroke.color;
            let btn_size = Vec2::splat(20.0);

            ui.label("Tools:");
            if ui
                .add(egui::Button::image(
                    egui::Image::new(egui::include_image!("../../resources/pointer.svg"))
                        .tint(color)
                        .fit_to_exact_size(btn_size),
                ).selected(self.active_tool == Some(Tool::Selection)))
                .on_hover_text("Selection (S)")
                .clicked()
            {
                self.active_tool = Some(Tool::Selection);
            }

            if ui
                .add(egui::Button::image(
                    egui::Image::new(egui::include_image!("../../resources/hand.svg"))
                        .tint(color)
                        .fit_to_exact_size(btn_size),
                ).selected(self.active_tool == Some(Tool::Hand)))
                .on_hover_text("Hand (H)")
                .clicked()
            {
                self.active_tool = Some(Tool::Hand);
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
            if ui
                .selectable_label(self.active_tool == Some(Tool::Freedraw), "✏")
                .clicked()
            {
                self.active_tool = Some(Tool::Freedraw);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾").clicked() {
                    self.save_file();
                }
            });
        });

        let mut bg_color = Color32::WHITE;
        if let Some(scene) = &self.scene {
            if let Some(app_state) = scene.extra.get("appState").and_then(|v| v.as_object()) {
                if let Some(view_bg) = app_state.get("viewBackgroundColor").and_then(|v| v.as_str()) {
                    let color = utils::hex_to_color(view_bg);
                    if color != Color32::TRANSPARENT {
                        bg_color = color;
                    }
                }
            }
        }

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::all());
        painter.rect_filled(response.rect, 0.0, bg_color);

        if response.clicked() || response.drag_started() {
            self.error_msg = None;
        }

        // Detect double-click for Wikilinks
        if response.double_clicked() {
            if let Some(mp) = response.interact_pointer_pos() {
                // Adjust for current pan/scale to get world coordinates
                let screen_rect_min = response.rect.min;
                let cp = self.pan;
                let cs = self.scale;
                let wp = Pos2::ZERO + (mp - screen_rect_min - cp) / cs;
                
                eprintln!("DEBUG: Double-click detected at screen {:?}, world {:?}", mp, wp);

                if let Some(scene) = &self.scene {
                    for el in scene.elements.iter().rev() {
                        if el.element_type == "text" {
                            // Use a very generous hit box for text
                            let hit_margin = 20.0 / cs;
                            let rect = Rect::from_min_size(Pos2::new(el.x, el.y), Vec2::new(el.width, el.height)).expand(hit_margin);
                            
                            if rect.contains(wp) {
                                eprintln!("DEBUG: Hit text element: '{}' at [{:?}]", el.text, rect);
                                
                                let mut link_target: Option<String> = None;

                                // Helper to extract from [[ ]]
                                let extract_wiki = |s: &str| -> Option<String> {
                                    let re_wiki = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
                                    re_wiki.captures(s).map(|caps| caps[1].trim().to_string())
                                };

                                // 1. Try explicit 'link' property (might be [[Link]] or a URL)
                                if let Some(l) = &el.link {
                                    if !l.is_empty() {
                                        if let Some(w) = extract_wiki(l) {
                                            link_target = Some(w);
                                        } else {
                                            link_target = Some(l.trim().to_string());
                                        }
                                    }
                                }

                                // 2. Try raw_text (Obsidian often puts the clean link here)
                                if link_target.is_none() {
                                    if let Some(rt) = &el.raw_text {
                                        if let Some(w) = extract_wiki(rt) {
                                            link_target = Some(w);
                                        }
                                    }
                                }

                                // 3. Try text or original_text
                                if link_target.is_none() {
                                    for t in &[Some(&el.text), el.original_text.as_ref()] {
                                        if let Some(text_val) = t {
                                            if let Some(w) = extract_wiki(text_val) {
                                                link_target = Some(w);
                                                break;
                                            }
                                        }
                                    }
                                }

                                // 4. Fallback: Cleaned text
                                if link_target.is_none() {
                                    let cleaned: String = el.text.chars()
                                        .filter(|c| !c.is_ascii_punctuation() && (c.is_alphanumeric() || c.is_whitespace()))
                                        .collect();
                                    let cleaned = cleaned.trim();
                                    if !cleaned.is_empty() {
                                        link_target = Some(cleaned.to_string());
                                    }
                                }

                                if let Some(target) = link_target {
                                    // Remove any remaining wikilink brackets if present (defensive)
                                    let clean_target = target.trim_start_matches("[[").trim_end_matches("]]");
                                    
                                    eprintln!("DEBUG: Attempting to resolve link target: '{}'", clean_target);
                                    let resolved = crate::files::resolve_path(vault, &self.path, clean_target);
                                    eprintln!("DEBUG: Resolved path: {:?}", resolved);

                                    if let Some(path) = resolved {
                                        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("global_nav_request"), Some(path)));
                                        return; 
                                    } else {
                                        self.error_msg = Some(format!("Could not find file: {}", clean_target));
                                    }
                                } else {
                                    eprintln!("DEBUG: No link target found in element");
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

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
            || (self.active_tool == Some(Tool::Hand) && response.dragged_by(PointerButton::Primary))
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
                                        if !self.selected_indices.contains(&i) {
                                            self.selected_indices.clear();
                                            self.selected_indices.insert(i);
                                            self.selected_element_idx = Some(i);
                                        }
                                        self.last_mouse_pos_world = Some(wp);
                                        f = true;
                                        break;
                                    }
                                }
                                if !f {
                                    self.selected_indices.clear();
                                    self.selected_element_idx = None;
                                    self.selection_rect = Some(Rect::from_min_max(wp, wp));
                                }
                            }
                        }

                        if let Some(rect) = &mut self.selection_rect {
                            if response.dragged_by(PointerButton::Primary) {
                                if let Some(mp) = response.interact_pointer_pos() {
                                    let wp = to_world(mp);
                                    rect.max = wp;
                                    
                                    // Update selection
                                    self.selected_indices.clear();
                                    let norm_rect = Rect::from_two_pos(rect.min, rect.max);
                                    for (i, el) in scene.elements.iter().enumerate() {
                                        let el_rect = Rect::from_min_size(Pos2::new(el.x, el.y), Vec2::new(el.width, el.height));
                                        if norm_rect.intersects(el_rect) {
                                            self.selected_indices.insert(i);
                                        }
                                    }
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
                                            // Move all selected elements if we are dragging one of them
                                            for &s_idx in &self.selected_indices {
                                                if let Some(el) = scene.elements.get_mut(s_idx) {
                                                    el.x += d.x;
                                                    el.y += d.y;
                                                }
                                            }
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
                                    Tool::Freedraw => {
                                        new_el.element_type = "freedraw".into();
                                        new_el.points = vec![[0.0, 0.0]];
                                    }
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
                                        } else if tool == Tool::Freedraw {
                                            let dx = cw.x - sp.x;
                                            let dy = cw.y - sp.y;
                                            el.points.push([dx, dy]);

                                            // Recalculate bounding box
                                            let mut min_x = 0.0f32;
                                            let mut min_y = 0.0f32;
                                            let mut max_x = 0.0f32;
                                            let mut max_y = 0.0f32;

                                            for p in &el.points {
                                                min_x = min_x.min(p[0]);
                                                min_y = min_y.min(p[1]);
                                                max_x = max_x.max(p[0]);
                                                max_y = max_y.max(p[1]);
                                            }

                                            if min_x != 0.0 || min_y != 0.0 {
                                                el.x += min_x;
                                                el.y += min_y;
                                                for p in &mut el.points {
                                                    p[0] -= min_x;
                                                    p[1] -= min_y;
                                                }
                                                // Adjust start pos as well to keep logic consistent
                                                self.drawing_start_pos =
                                                    Some(Pos2::new(sp.x + min_x, sp.y + min_y));
                                            }
                                            el.width = max_x - min_x;
                                            el.height = max_y - min_y;
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
                self.selection_rect = None;
                
                // If we have multiple selected, pick one for the properties panel (or just use the first)
                self.selected_element_idx = self.selected_indices.iter().next().copied();

                if let Some(el) = self.drawing_element.take() {
                    if el.width > 2.0 || el.height > 2.0 {
                        scene.elements.push(el);
                        self.selected_indices.clear();
                        self.selected_indices.insert(scene.elements.len() - 1);
                        self.selected_element_idx = Some(scene.elements.len() - 1);
                        dirty = true;
                        save = true;
                    }
                    self.active_tool = Some(Tool::Selection);
                }
                if dirty && !self.selected_indices.is_empty() {
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
                if self.selected_indices.contains(&i) {
                    draw_selection_border(&painter, el, &to_screen, cs, ui);
                }
            }

            if let Some(rect) = self.selection_rect {
                let screen_rect = Rect::from_min_max(to_screen(rect.min), to_screen(rect.max));
                painter.rect_stroke(
                    screen_rect,
                    0.0,
                    Stroke::new(1.0, Color32::from_rgb(100, 100, 255)),
                    egui::StrokeKind::Outside,
                );
                painter.rect_filled(
                    screen_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(100, 100, 255, 20),
                );
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
