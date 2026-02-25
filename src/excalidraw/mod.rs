use base64::{engine::general_purpose, Engine as _};
use egui::UiBuilder;
use lz_str;
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

use egui::{
    Color32, ColorImage, Context, Id, PointerButton, Pos2, Rect, Sense, Stroke, TextureHandle,
    TextureOptions, Ui, Vec2,
};
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

pub mod data;
pub mod render;
pub mod ui_panel;
pub mod utils;

use data::{ExcalidrawElement, ExcalidrawFile, ExcalidrawScene, Tool};
use render::{draw_element, draw_selection_border};
use ui_panel::show_properties_panel;
use utils::{is_point_inside, move_element_group, normalize_element};

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
    dragged_point_idx: Option<usize>,
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
    #[serde(skip)]
    undo_stack: Vec<ExcalidrawScene>,
    #[serde(skip)]
    redo_stack: Vec<ExcalidrawScene>,
    #[serde(skip)]
    clipboard: Vec<ExcalidrawElement>,
    #[serde(skip)]
    active_guides: Vec<Rect>,
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
            dragged_point_idx: None,
            last_mouse_pos_world: None,
            drawing_start_pos: None,
            drawing_element: None,
            is_dirty: false,
            is_panning: false,
            texture_cache: HashMap::new(),
            failed_textures: HashSet::new(),
            selection_rect: None,
            selected_indices: HashSet::new(),
            undo_stack: vec![],
            redo_stack: vec![],
            clipboard: vec![],
            active_guides: vec![],
        }
    }
}

impl ExcalidrawGui {
    fn move_selection_in_z(&mut self, scene: &mut ExcalidrawScene, direction: i32) -> bool {
        if self.selected_indices.is_empty() {
            return false;
        }

        let mut indices: Vec<usize> = self.selected_indices.iter().copied().collect();
        indices.sort_unstable();

        let mut changed = false;
        
        // Use IDs to track selection through reordering
        let selected_ids: HashSet<String> = indices.iter()
            .filter_map(|&i| scene.elements.get(i).map(|el| el.id.clone()))
            .collect();

        match direction {
            2 => { // Bring to Front
                self.push_undo(scene);
                let mut moved = Vec::new();
                let mut remaining = Vec::new();
                for el in scene.elements.drain(..) {
                    if selected_ids.contains(&el.id) {
                        moved.push(el);
                    } else {
                        remaining.push(el);
                    }
                }
                remaining.extend(moved);
                scene.elements = remaining;
                changed = true;
            }
            -2 => { // Send to Back
                self.push_undo(scene);
                let mut moved = Vec::new();
                let mut remaining = Vec::new();
                for el in scene.elements.drain(..) {
                    if selected_ids.contains(&el.id) {
                        moved.push(el);
                    } else {
                        remaining.push(el);
                    }
                }
                moved.extend(remaining);
                scene.elements = moved;
                changed = true;
            }
            1 => { // Forward
                if indices.last().map_or(false, |&i| i < scene.elements.len() - 1) {
                    self.push_undo(scene);
                    for i in (0..scene.elements.len() - 1).rev() {
                        if selected_ids.contains(&scene.elements[i].id) && !selected_ids.contains(&scene.elements[i+1].id) {
                            scene.elements.swap(i, i + 1);
                        }
                    }
                    changed = true;
                }
            }
            -1 => { // Backward
                if indices.first().map_or(false, |&i| i > 0) {
                    self.push_undo(scene);
                    for i in 1..scene.elements.len() {
                        if selected_ids.contains(&scene.elements[i].id) && !selected_ids.contains(&scene.elements[i-1].id) {
                            scene.elements.swap(i, i - 1);
                        }
                    }
                    changed = true;
                }
            }
            _ => {}
        }

        if changed {
            self.selected_indices.clear();
            for (i, el) in scene.elements.iter().enumerate() {
                if selected_ids.contains(&el.id) {
                    self.selected_indices.insert(i);
                }
            }
            self.selected_element_idx = self.selected_indices.iter().next().copied();
        }

        changed
    }

    pub fn set_path(&mut self, path: &str) {
        if self.path != path || self.scene.is_none() {
            self.path = path.to_string();
            self.undo_stack.clear();
            self.redo_stack.clear();
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
                                normalize_element(el);
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

    fn push_undo(&mut self, scene: &ExcalidrawScene) {
        self.undo_stack.push(scene.clone());
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            if let Some(current) = self.scene.take() {
                self.redo_stack.push(current);
            }
            self.scene = Some(prev);
            self.is_dirty = true;
            self.save_file();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            if let Some(current) = self.scene.take() {
                self.undo_stack.push(current);
            }
            self.scene = Some(next);
            self.is_dirty = true;
            self.save_file();
        }
    }

    fn save_file(&mut self) {
        if let Some(scene) = &self.scene {
            if let Ok(json) = serde_json::to_string(scene) {
                let compressed = lz_str::compress_to_base64(&json);
                let mut text_elements = String::new();
                for el in &scene.elements {
                    if el.is_deleted {
                        continue;
                    }
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

    pub fn generate_svg(&self) -> Option<String> {
        let scene = self.scene.as_ref()?;
        if scene.elements.is_empty() {
            return None;
        }

        // Calculate bounding box
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        let mut has_visible = false;
        for el in &scene.elements {
            if el.is_deleted {
                continue;
            }
            has_visible = true;
            min_x = min_x.min(el.x);
            min_y = min_y.min(el.y);
            max_x = max_x.max(el.x + el.width);
            max_y = max_y.max(el.y + el.height);
        }

        if !has_visible {
            return None;
        }

        let width = max_x - min_x + 40.0;
        let height = max_y - min_y + 40.0;
        let offset_x = -min_x + 20.0;
        let offset_y = -min_y + 20.0;

        let bg_color = &scene.app_state.view_background_color;
        let mut svg = format!(
            r#"<svg viewBox="0 0 {} {}" width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            width, height, width, height
        );
        svg.push_str(&format!(
            r#"<rect x="0" y="0" width="{}" height="{}" fill="{}"/>"#,
            width, height, bg_color
        ));

        for el in &scene.elements {
            if el.is_deleted {
                continue;
            }

            let x = el.x + offset_x;
            let y = el.y + offset_y;
            let stroke = &el.stroke_color;
            let stroke_width = el.stroke_width;
            let fill = if el.background_color == "transparent" {
                "none".to_string()
            } else {
                el.background_color.clone()
            };
            let opacity = el.opacity as f32 / 100.0;
            let angle_deg = el.angle.to_degrees();
            let transform = format!(
                r#"transform="rotate({} {} {})""#,
                angle_deg,
                x + el.width / 2.0,
                y + el.height / 2.0
            );

            let dash_array = match el.stroke_style.as_str() {
                "dashed" => format!(r#"stroke-dasharray="{},{}""#, 10, 10),
                "dotted" => format!(r#"stroke-dasharray="{},{}""#, 2, 6),
                _ => "".to_string(),
            };

            match el.element_type.as_str() {
                "rectangle" => {
                    let r = el
                        .roundness
                        .as_ref()
                        .map(|x| if x.round_type == 3 { 20.0 } else { 4.0 })
                        .unwrap_or(0.0);
                    svg.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}" {} {} />"#,
                        x, y, el.width, el.height, r, r, fill, stroke, stroke_width, opacity, dash_array, transform
                    ));
                }
                "ellipse" => {
                    svg.push_str(&format!(
                        r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}" {} {} />"#,
                        x + el.width / 2.0,
                        y + el.height / 2.0,
                        el.width / 2.0,
                        el.height / 2.0,
                        fill,
                        stroke,
                        stroke_width,
                        opacity,
                        dash_array,
                        transform
                    ));
                }
                "diamond" => {
                    let pts = format!(
                        "{},{} {},{} {},{} {},{}",
                        x + el.width / 2.0, y,
                        x + el.width, y + el.height / 2.0,
                        x + el.width / 2.0, y + el.height,
                        x, y + el.height / 2.0
                    );
                    svg.push_str(&format!(
                        r#"<polygon points="{}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}" {} {} />"#,
                        pts, fill, stroke, stroke_width, opacity, dash_array, transform
                    ));
                }
                "line" | "arrow" | "draw" | "freedraw" => {
                    if !el.points.is_empty() {
                        let mut pts_str = String::new();
                        for p in &el.points {
                            pts_str.push_str(&format!("{},{} ", x + p[0], y + p[1]));
                        }
                        svg.push_str(&format!(
                            r#"<polyline points="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}" stroke-linecap="round" stroke-linejoin="round" {} {} />"#,
                            pts_str, stroke, stroke_width, opacity, dash_array, transform
                        ));
                        
                        if el.element_type == "arrow" && el.points.len() >= 2 {
                            let end = el.points[el.points.len() - 1];
                            let prev = el.points[el.points.len() - 2];
                            let dx = end[0] - prev[0];
                            let dy = end[1] - prev[1];
                            let angle = dy.atan2(dx);
                            let l = 20.0;
                            let sp = 0.52; // 30deg
                            
                            let p1x = x + end[0] + l * (angle + std::f32::consts::PI - sp).cos();
                            let p1y = y + end[1] + l * (angle + std::f32::consts::PI - sp).sin();
                            let p2x = x + end[0] + l * (angle + std::f32::consts::PI + sp).cos();
                            let p2y = y + end[1] + l * (angle + std::f32::consts::PI + sp).sin();

                            svg.push_str(&format!(
                                r#"<polyline points="{},{} {},{} {},{}" fill="none" stroke="{}" stroke-width="{}" opacity="{}" stroke-linecap="round" stroke-linejoin="round" {} />"#,
                                p1x, p1y, x + end[0], y + end[1], p2x, p2y, stroke, stroke_width, opacity, transform
                            ));
                        }
                    }
                }
                "text" => {
                    let font_size = el.font_size.unwrap_or(20.0);
                    svg.push_str(&format!(
                        r#"<text x="{}" y="{}" font-family="sans-serif" font-size="{}" fill="{}" opacity="{}" {} dominant-baseline="hanging">{}</text>"#,
                        x, y, font_size, stroke, opacity, transform, el.text
                    ));
                }
                "image" => {
                    if let Some(fid) = &el.file_id {
                        if let Some(file) = scene.files.get(fid) {
                            svg.push_str(&format!(
                                r#"<image x="{}" y="{}" width="{}" height="{}" href="{}" opacity="{}" {} />"#,
                                x, y, el.width, el.height, file.data_url, opacity, transform
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        svg.push_str("</svg>");
        Some(svg)
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

    pub fn render_static(&mut self, ui: &mut Ui, size: Option<f32>) {
        if let Some(e) = &self.error_msg {
            ui.colored_label(ui.ctx().style().visuals.error_fg_color, e);
            return;
        }
        if self.scene.is_none() {
            self.reload();
            if self.scene.is_none() {
                ui.label("Loading drawing...");
                return;
            }
        }

        if let Some(scene) = self.scene.take() {
            if scene.elements.is_empty() {
                self.scene = Some(scene);
                return;
            }

            // Calculate bounding box of all elements
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;

            for el in &scene.elements {
                if el.is_deleted {
                    continue;
                }
                min_x = min_x.min(el.x);
                min_y = min_y.min(el.y);
                max_x = max_x.max(el.x + el.width);
                max_y = max_y.max(el.y + el.height);
            }

            let drawing_width = max_x - min_x;
            let drawing_height = max_y - min_y;

            if drawing_width <= 0.0 || drawing_height <= 0.0 {
                self.scene = Some(scene);
                return;
            }

            let available_width = ui.available_width();
            let max_w = 600.0;
            // Height reduced by 30% means max_height = available_width * 0.7 (if we consider a square as baseline)
            // but more accurately, it usually means limit growth.
            let max_h = available_width * 0.7;

            let mut target_width = size
                .unwrap_or(available_width)
                .min(available_width)
                .min(max_w);
            let mut scale = target_width / drawing_width;

            if drawing_height * scale > max_h {
                scale = max_h / drawing_height;
                target_width = drawing_width * scale;
            }

            let padding_y = 10.0;
            let target_height = drawing_height * scale + padding_y * 2.0;

            let x_offset = (available_width - target_width) / 2.0;
            let (response, painter) =
                ui.allocate_painter(Vec2::new(available_width, target_height), Sense::hover());
            let drawing_rect = Rect::from_min_size(
                response.rect.min + Vec2::new(x_offset, 0.0),
                Vec2::new(target_width, target_height),
            );

            let bg_color =
                crate::excalidraw::utils::hex_to_color(&scene.app_state.view_background_color);
            if bg_color != Color32::TRANSPARENT {
                painter.rect_filled(drawing_rect, 0.0, bg_color);
            }

            let screen_min = drawing_rect.min;
            let to_screen = |pw: Pos2| {
                screen_min
                    + Vec2::new(0.0, padding_y)
                    + (Vec2::new(pw.x - min_x, pw.y - min_y) * scale)
            };

            let ctx = ui.ctx().clone();
            for el in &scene.elements {
                if el.is_deleted {
                    continue;
                }
                let tex = if let Some(fid) = &el.file_id {
                    self.get_or_load_texture(&ctx, fid, &scene.files)
                } else {
                    None
                };
                draw_element(&painter, el, tex, &to_screen, scale);
            }
            self.scene = Some(scene);
        }
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

        // Capture keyboard shortcuts at the very beginning
        let mut do_copy = false;
        let mut do_paste = false;

        ui.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Copy => do_copy = true,
                    egui::Event::Paste(_) => do_paste = true,
                    egui::Event::Key {
                        key: egui::Key::C,
                        pressed: true,
                        modifiers: egui::Modifiers { command: true, .. },
                        ..
                    } => do_copy = true,
                    egui::Event::Key {
                        key: egui::Key::V,
                        pressed: true,
                        modifiers: egui::Modifiers { command: true, .. },
                        ..
                    } => do_paste = true,
                    _ => {}
                }
            }
        });

        ui.horizontal(|ui| {
            let color = ui
                .ctx()
                .style()
                .visuals
                .widgets
                .noninteractive
                .fg_stroke
                .color;
            let btn_size = Vec2::splat(20.0);

            ui.label("Tools:");
            if ui
                .add(
                    egui::Button::image(
                        egui::Image::new(egui::include_image!("../../resources/pointer.svg"))
                            .tint(color)
                            .fit_to_exact_size(btn_size),
                    )
                    .selected(self.active_tool == Some(Tool::Selection)),
                )
                .on_hover_text("Selection (S)")
                .clicked()
            {
                self.active_tool = Some(Tool::Selection);
            }

            if ui
                .add(
                    egui::Button::image(
                        egui::Image::new(egui::include_image!("../../resources/hand.svg"))
                            .tint(color)
                            .fit_to_exact_size(btn_size),
                    )
                    .selected(self.active_tool == Some(Tool::Hand)),
                )
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

            ui.separator();
            let (mut sg, mut sn) = if let Some(scene) = &self.scene {
                (scene.app_state.show_grid, scene.app_state.snap_enabled)
            } else {
                (true, true)
            };

            if ui
                .selectable_label(sg, "▦")
                .on_hover_text("Show Grid")
                .clicked()
            {
                sg = !sg;
                if let Some(scene) = &mut self.scene {
                    scene.app_state.show_grid = sg;
                    self.save_file();
                }
            }
            if ui
                .selectable_label(sn, "🧲")
                .on_hover_text("Enable Snapping")
                .clicked()
            {
                sn = !sn;
                if let Some(scene) = &mut self.scene {
                    scene.app_state.snap_enabled = sn;
                    self.save_file();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾").on_hover_text("Save (Manual)").clicked() {
                    self.save_file();
                }
                if ui.button("🖼 SVG").on_hover_text("Export as SVG").clicked() {
                    if let Some(svg_content) = self.generate_svg() {
                        #[cfg(not(target_os = "android"))]
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("SVG Image", &["svg"])
                            .set_file_name("drawing.svg")
                            .save_file()
                        {
                            let _ = fs::write(path, svg_content);
                        }
                    }
                }
                ui.separator();
                if ui
                    .add_enabled(!self.redo_stack.is_empty(), egui::Button::new("↪"))
                    .on_hover_text("Redo (Ctrl+Shift+Z)")
                    .clicked()
                {
                    self.redo();
                }
                if ui
                    .add_enabled(!self.undo_stack.is_empty(), egui::Button::new("↩"))
                    .on_hover_text("Undo (Ctrl+Z)")
                    .clicked()
                {
                    self.undo();
                }
            });
        });

        let mut bg_color = Color32::WHITE;
        if let Some(scene) = &self.scene {
            let color = utils::hex_to_color(&scene.app_state.view_background_color);
            if color != Color32::TRANSPARENT {
                bg_color = color;
            }
        }

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::all());
        painter.rect_filled(response.rect, 0.0, bg_color);

        if response.clicked() {
            response.request_focus();
        }

        if response.clicked() || response.drag_started() {
            self.error_msg = None;
        }

        // Detect double-click for Wikilinks
        let screen_rect_min = response.rect.min;
        let cp_pre = self.pan;
        let cs_pre = self.scale;
        let to_world_pre = |ps: Pos2| Pos2::ZERO + (ps - screen_rect_min - cp_pre) / cs_pre;

        if response.double_clicked() {
            if let Some(mp) = response.interact_pointer_pos() {
                let wp = to_world_pre(mp);

                eprintln!(
                    "DEBUG: Double-click detected at screen {:?}, world {:?}",
                    mp, wp
                );

                if let Some(scene) = &self.scene {
                    for el in scene.elements.iter().rev() {
                        if el.is_deleted {
                            continue;
                        }
                        if el.element_type == "text" {
                            // Use a very generous hit box for text
                            let hit_margin = 20.0 / cs_pre;
                            let rect = Rect::from_min_size(
                                Pos2::new(el.x, el.y),
                                Vec2::new(el.width, el.height),
                            )
                            .expand(hit_margin);

                            if rect.contains(wp) {
                                eprintln!("DEBUG: Hit text element: '{}' at [{:?}]", el.text, rect);

                                let mut link_target: Option<String> = None;

                                // Helper to extract from [[ ]]
                                let extract_wiki = |s: &str| -> Option<String> {
                                    let re_wiki =
                                        Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
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
                                    let cleaned: String = el
                                        .text
                                        .chars()
                                        .filter(|c| {
                                            !c.is_ascii_punctuation()
                                                && (c.is_alphanumeric() || c.is_whitespace())
                                        })
                                        .collect();
                                    let cleaned = cleaned.trim();
                                    if !cleaned.is_empty() {
                                        link_target = Some(cleaned.to_string());
                                    }
                                }

                                if let Some(target) = link_target {
                                    // Remove any remaining wikilink brackets if present (defensive)
                                    let clean_target =
                                        target.trim_start_matches("[[").trim_end_matches("]]");

                                    eprintln!(
                                        "DEBUG: Attempting to resolve link target: '{}'",
                                        clean_target
                                    );
                                    let resolved =
                                        crate::files::resolve_path(vault, &self.path, clean_target);
                                    eprintln!("DEBUG: Resolved path: {:?}", resolved);

                                    if let Some(path) = resolved {
                                        ui.ctx().data_mut(|d| {
                                            d.insert_temp(
                                                egui::Id::new("global_nav_request"),
                                                Some(path),
                                            )
                                        });
                                        return;
                                    } else {
                                        self.error_msg =
                                            Some(format!("Could not find file: {}", clean_target));
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

        let (show_grid, snap_enabled) = if let Some(scene) = &self.scene {
            (scene.app_state.show_grid, scene.app_state.snap_enabled)
        } else {
            (true, true)
        };

        // Grid Rendering
        let grid_size = 20.0;
        if show_grid {
            let dot_color = Color32::from_rgb(22, 23, 23);
            //if ui.visuals().dark_mode {
            //    Color32::from_white_alpha(40)
            //} else {
            //    Color32::from_black_alpha(40)
            //};

            let start_x = (to_world(response.rect.min).x / grid_size).floor() * grid_size;
            let start_y = (to_world(response.rect.min).y / grid_size).floor() * grid_size;
            let end_x = (to_world(response.rect.max).x / grid_size).ceil() * grid_size;
            let end_y = (to_world(response.rect.max).y / grid_size).ceil() * grid_size;

            let steps_x = ((end_x - start_x) / grid_size).ceil() as i32 + 1;
            let steps_y = ((end_y - start_y) / grid_size).ceil() as i32 + 1;

            if steps_x > 0 && steps_x < 1000 && steps_y > 0 && steps_y < 1000 {
                let dot_radius = (1.0 * self.scale).clamp(0.8, 1.5);
                for ix in 0..steps_x {
                    let x = start_x + ix as f32 * grid_size;
                    for iy in 0..steps_y {
                        let y = start_y + iy as f32 * grid_size;
                        let p = to_screen(Pos2::new(x, y));
                        painter.circle_filled(p, dot_radius, dot_color);
                    }
                }
            }
        }

        // Keyboard shortcuts for Undo/Redo
        if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Z)) {
            if ui.input(|i| i.modifiers.shift) {
                self.redo();
            } else {
                self.undo();
            }
        } else if ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Y)) {
            self.redo();
        }

        // Layer shortcuts
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::CloseBracket)) {
            if let Some(mut scene) = self.scene.take() {
                let dir = if ui.input(|i| i.modifiers.shift) { 2 } else { 1 };
                if self.move_selection_in_z(&mut scene, dir) {
                    self.save_file();
                }
                self.scene = Some(scene);
            }
        } else if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::OpenBracket)) {
            if let Some(mut scene) = self.scene.take() {
                let dir = if ui.input(|i| i.modifiers.shift) { -2 } else { -1 };
                if self.move_selection_in_z(&mut scene, dir) {
                    self.save_file();
                }
                self.scene = Some(scene);
            }
        }

        if let Some(mut scene) = self.scene.take() {
            let mut dirty = self.is_dirty;
            let mut save = false;

            // Better Copy (Ctrl+C)
            if do_copy && !self.selected_indices.is_empty() {
                self.clipboard.clear();
                for &idx in &self.selected_indices {
                    if let Some(el) = scene.elements.get(idx) {
                        if !el.is_deleted {
                            self.clipboard.push(el.clone());
                        }
                    }
                }
            }

            // Better Paste (Ctrl+V)
            if do_paste && !self.clipboard.is_empty() {
                self.push_undo(&scene);
                let offset = 20.0;
                let mut new_selected = HashSet::new();
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis();

                for (i, el) in self.clipboard.clone().into_iter().enumerate() {
                    let mut new_el = el;
                    new_el.id = format!("copy_{}_{}", timestamp, i);
                    new_el.x += offset;
                    new_el.y += offset;
                    new_el.is_deleted = false;
                    scene.elements.push(new_el);
                    new_selected.insert(scene.elements.len() - 1);
                }

                self.selected_indices = new_selected;
                self.selected_element_idx = self.selected_indices.iter().next().copied();
                dirty = true;
                save = true;
            }

            let tool = self.active_tool.unwrap_or(Tool::Selection);

            let mouse_over_panel = if let Some(mp) = response.hover_pos() {
                panel_rect.contains(mp)
            } else {
                false
            };

            if !mouse_over_panel {
                match tool {
                    Tool::Selection => {
                        let click_or_drag_start = response.clicked_by(PointerButton::Primary)
                            || response.drag_started_by(PointerButton::Primary);

                        if response.clicked_by(PointerButton::Secondary) {
                            if let Some(mp) = response.interact_pointer_pos() {
                                let wp = to_world(mp);
                                let mut hit_index = None;
                                for (i, el) in scene.elements.iter().enumerate().rev() {
                                    if el.is_deleted {
                                        continue;
                                    }
                                    if is_point_inside(el, wp) {
                                        hit_index = Some(i);
                                        break;
                                    }
                                }

                                if let Some(i) = hit_index {
                                    if !self.selected_indices.contains(&i) {
                                        self.push_undo(&scene);
                                        self.selected_indices.clear();
                                        self.selected_indices.insert(i);
                                        self.selected_element_idx = Some(i);
                                    }
                                }
                            }
                        }

                        response.context_menu(|ui| {
                            if !self.selected_indices.is_empty() {
                                if ui.button("⏫ Bring to Front").clicked() {
                                    if self.move_selection_in_z(&mut scene, 2) {
                                        dirty = true;
                                        save = true;
                                    }
                                    ui.close();
                                }
                                if ui.button("🔼 Bring Forward").clicked() {
                                    if self.move_selection_in_z(&mut scene, 1) {
                                        dirty = true;
                                        save = true;
                                    }
                                    ui.close();
                                }
                                if ui.button("🔽 Send Backward").clicked() {
                                    if self.move_selection_in_z(&mut scene, -1) {
                                        dirty = true;
                                        save = true;
                                    }
                                    ui.close();
                                }
                                if ui.button("⏬ Send to Back").clicked() {
                                    if self.move_selection_in_z(&mut scene, -2) {
                                        dirty = true;
                                        save = true;
                                    }
                                    ui.close();
                                }
                                ui.separator();
                                if ui.button("🗑 Delete").clicked() {
                                    self.push_undo(&scene);
                                    for &idx in &self.selected_indices {
                                        if let Some(el) = scene.elements.get_mut(idx) {
                                            el.is_deleted = true;
                                        }
                                    }
                                    self.selected_indices.clear();
                                    self.selected_element_idx = None;
                                    dirty = true;
                                    save = true;
                                    ui.close();
                                }
                            } else {
                                ui.label("No element selected");
                            }
                        });

                        if click_or_drag_start && !self.is_panning
                        {
                            if let Some(mp) = response.interact_pointer_pos() {
                                let wp = to_world(mp);

                                // Check if we clicked on a point handle of a single selected line/arrow
                                let mut point_hit = None;
                                let mut resize_hit = None;
                                let mut rotate_hit = None;

                                if self.selected_indices.len() == 1 {
                                    let idx = *self.selected_indices.iter().next().unwrap();
                                    if let Some(el) = scene.elements.get(idx) {
                                        let center = Pos2::new(
                                            el.x + el.width / 2.0,
                                            el.y + el.height / 2.0,
                                        );
                                        let cl = Vec2::new(el.width / 2.0, el.height / 2.0);
                                        let rot = egui::emath::Rot2::from_angle(el.angle);
                                        let handle_radius = 16.0 / cs;

                                        if (el.element_type == "line" || el.element_type == "arrow")
                                            && el.points.len() >= 2
                                        {
                                            let handles = [0, el.points.len() - 1];
                                            for &p_idx in &handles {
                                                let p = el.points[p_idx];
                                                let p_world = center
                                                    + rot * (Pos2::new(p[0], p[1]) - cl).to_vec2();
                                                if p_world.distance(wp) < handle_radius {
                                                    point_hit = Some((idx, p_idx));
                                                    break;
                                                }
                                            }
                                        } else {
                                            // Corner resize handles
                                            let padding = 4.0;
                                            let corners = [
                                                Pos2::new(-padding, -padding),                      // 0: TL
                                                Pos2::new(el.width + padding, -padding), // 1: TR
                                                Pos2::new(el.width + padding, el.height + padding), // 2: BR
                                                Pos2::new(-padding, el.height + padding), // 3: BL
                                            ];
                                            for (c_idx, &p) in corners.iter().enumerate() {
                                                let p_world = center + rot * (p - cl).to_vec2();
                                                if p_world.distance(wp) < handle_radius {
                                                    resize_hit = Some((idx, c_idx));
                                                    break;
                                                }
                                            }

                                            // Rotation handle
                                            if rotate_hit.is_none() {
                                                let rot_p =
                                                    Pos2::new(el.width / 2.0, -padding - 20.0);
                                                let rot_p_world =
                                                    center + rot * (rot_p - cl).to_vec2();
                                                if rot_p_world.distance(wp) < handle_radius {
                                                    rotate_hit = Some(idx);
                                                }
                                            }
                                        }
                                    }
                                }

                                if let Some((el_idx, p_idx)) = point_hit {
                                    self.push_undo(&scene);
                                    self.dragged_element_idx = Some(el_idx);
                                    self.dragged_point_idx = Some(p_idx);
                                    self.last_mouse_pos_world = Some(wp);
                                } else if let Some((el_idx, c_idx)) = resize_hit {
                                    self.push_undo(&scene);
                                    self.dragged_element_idx = Some(el_idx);
                                    // Use 100+ for corners
                                    self.dragged_point_idx = Some(100 + c_idx);
                                    self.last_mouse_pos_world = Some(wp);
                                } else if let Some(el_idx) = rotate_hit {
                                    self.push_undo(&scene);
                                    self.dragged_element_idx = Some(el_idx);
                                    // Use 200 for rotation
                                    self.dragged_point_idx = Some(200);
                                    self.last_mouse_pos_world = Some(wp);
                                } else {
                                    // If Alt is held, we only want to select if we actually hit an element.
                                    // If we hit nothing, we don't clear selection, to allow Alt+Drag to pan
                                    // from outside the element if we didn'thit anything.
                                    // But wait, our new pan logic already checks self.dragged_element_idx.is_none().
                                    
                                    let mut hit_index = None;
                                    for (i, el) in scene.elements.iter().enumerate().rev() {
                                        if el.is_deleted {
                                            continue;
                                        }
                                        if is_point_inside(el, wp) {
                                            hit_index = Some(i);
                                            break;
                                        }
                                    }

                                    if let Some(i) = hit_index {
                                        if !self.selected_indices.contains(&i) {
                                            self.push_undo(&scene);
                                            self.selected_indices.clear();
                                            self.selected_indices.insert(i);
                                            self.selected_element_idx = Some(i);
                                        }
                                        if response.drag_started_by(PointerButton::Primary) {
                                            self.dragged_element_idx = Some(i);
                                            self.dragged_point_idx = None;
                                            self.last_mouse_pos_world = Some(wp);
                                        }
                                    } else {
                                        // Hit nothing. 
                                        // If Alt is held, don't clear selection or start selection rect, 
                                        // so the Pan logic can take over.
                                        if !ui.input(|i| i.modifiers.alt) {
                                            self.selected_indices.clear();
                                            self.selected_element_idx = None;
                                            if response.drag_started_by(PointerButton::Primary) {
                                                self.selection_rect = Some(Rect::from_min_max(wp, wp));
                                            }
                                        }
                                    }
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
                                        if el.is_deleted {
                                            continue;
                                        }
                                        let el_rect = Rect::from_min_size(
                                            Pos2::new(el.x, el.y),
                                            Vec2::new(el.width, el.height),
                                        );
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

                                                            let mut cwp = to_world(mp);

                                                            let mut d = response.drag_delta() / cs;

                        

                                                            // Grid Snap (20px) - we only snap the "final" position/delta if needed

                                                            // But for smooth resizing, using raw drag_delta is better.

                                                            let grid_size = 20.0;

                                                            if snap_enabled {

                                                                cwp.x = (cwp.x / grid_size).round() * grid_size;

                                                                cwp.y = (cwp.y / grid_size).round() * grid_size;

                                                            }

                        

                                                            if d.length_sq() > 0.00001 {

                                                                self.active_guides.clear();

                                                                if let Some(p_idx) = self.dragged_point_idx {

                                                                    if p_idx == 200 {

                                                                        // Rotation - use snapped mouse pos for better control

                                                                        if let Some(el) = scene.elements.get_mut(idx) {

                                                                            let center = Pos2::new(

                                                                                el.x + el.width / 2.0,

                                                                                el.y + el.height / 2.0,

                                                                            );

                                                                            let v = cwp - center;

                                                                            // Adjust by PI/2 because rotation handle is at the top

                                                                            el.angle = v.y.atan2(v.x)

                                                                                + std::f32::consts::FRAC_PI_2;

                                                                            // Snap rotation to 15 degrees

                                                                            let snap_angle = 15.0f32.to_radians();

                                                                            el.angle = (el.angle / snap_angle).round()

                                                                                * snap_angle;

                                                                            dirty = true;

                                                                        }

                                                                                                                    } else if p_idx >= 100 && p_idx <= 103 {

                                                                                                                        // Corner Resize - absolute logic for stability

                                                                                                                        if let Some(el) = scene.elements.get_mut(idx) {

                                                                                                                            let c_idx = p_idx - 100;

                                                                                                                            let rot =

                                                                                                                                egui::emath::Rot2::from_angle(el.angle);

                                                                                                                            let rot_inv = egui::emath::Rot2::from_angle(

                                                                                                                                -el.angle,

                                                                                                                            );

                                                                    

                                                                                                                            let center = Pos2::new(

                                                                                                                                el.x + el.width / 2.0,

                                                                                                                                el.y + el.height / 2.0,

                                                                                                                            );

                                                                    

                                                                                                                            // opposite corner

                                                                                                                            let opp_idx = (c_idx + 2) % 4;

                                                                                                                            let opp_local = match opp_idx {

                                                                                                                                0 => Vec2::new(

                                                                                                                                    -el.width / 2.0,

                                                                                                                                    -el.height / 2.0,

                                                                                                                                ),

                                                                                                                                1 => Vec2::new(

                                                                                                                                    el.width / 2.0,

                                                                                                                                    -el.height / 2.0,

                                                                                                                                ),

                                                                                                                                2 => Vec2::new(

                                                                                                                                    el.width / 2.0,

                                                                                                                                    el.height / 2.0,

                                                                                                                                ),

                                                                                                                                3 => Vec2::new(

                                                                                                                                    -el.width / 2.0,

                                                                                                                                    el.height / 2.0,

                                                                                                                                ),

                                                                                                                                _ => Vec2::ZERO,

                                                                                                                            };

                                                                                                                            let opp_world = center + rot * opp_local;

                                                                    

                                                                                                                            let resize_from_center =

                                                                                                                                ui.input(|i| i.modifiers.alt);

                                                                                                                            let mut new_width;

                                                                                                                            let mut new_height;

                                                                    

                                                                                                                            if resize_from_center {

                                                                                                                                let mouse_local =

                                                                                                                                    rot_inv * (cwp - center);

                                                                                                                                new_width = (mouse_local.x.abs() * 2.0)

                                                                                                                                    .max(1.0);

                                                                                                                                new_height = (mouse_local.y.abs() * 2.0)

                                                                                                                                    .max(1.0);

                                                                                                                            } else {

                                                                                                                                let mouse_rel_opp =

                                                                                                                                    rot_inv * (cwp - opp_world);

                                                                                                                                new_width = match c_idx {

                                                                                                                                    0 | 3 => {

                                                                                                                                        (-mouse_rel_opp.x).max(1.0)

                                                                                                                                    }

                                                                                                                                    1 | 2 => (mouse_rel_opp.x).max(1.0),

                                                                                                                                    _ => el.width,

                                                                                                                                };

                                                                                                                                new_height = match c_idx {

                                                                                                                                    0 | 1 => {

                                                                                                                                        (-mouse_rel_opp.y).max(1.0)

                                                                                                                                    }

                                                                                                                                    2 | 3 => (mouse_rel_opp.y).max(1.0),

                                                                                                                                    _ => el.height,

                                                                                                                                };

                                                                                                                            }

                                                                    

                                                                                                                            if ui.input(|i| i.modifiers.shift) {

                                                                                                                                let ratio = el.width / el.height;

                                                                                                                                if new_width / ratio > new_height {

                                                                                                                                    new_height = new_width / ratio;

                                                                                                                                } else {

                                                                                                                                    new_width = new_height * ratio;

                                                                                                                                }

                                                                                                                            }

                                                                    

                                                                                                                            el.width = new_width;

                                                                                                                            el.height = new_height;

                                                                    

                                                                                                                            let new_center = if resize_from_center {

                                                                                                                                center

                                                                                                                            } else {

                                                                                                                                let new_opp_local = match opp_idx {

                                                                                                                                    0 => Vec2::new(

                                                                                                                                        -el.width / 2.0,

                                                                                                                                        -el.height / 2.0,

                                                                                                                                    ),

                                                                                                                                    1 => Vec2::new(

                                                                                                                                        el.width / 2.0,

                                                                                                                                        -el.height / 2.0,

                                                                                                                                    ),

                                                                                                                                    2 => Vec2::new(

                                                                                                                                        el.width / 2.0,

                                                                                                                                        el.height / 2.0,

                                                                                                                                    ),

                                                                                                                                    3 => Vec2::new(

                                                                                                                                        -el.width / 2.0,

                                                                                                                                        el.height / 2.0,

                                                                                                                                    ),

                                                                                                                                    _ => Vec2::ZERO,

                                                                                                                                };

                                                                                                                                opp_world - rot * new_opp_local

                                                                                                                            };

                                                                                                                            el.x = new_center.x - el.width / 2.0;

                                                                                                                            el.y = new_center.y - el.height / 2.0;

                                                                                                                            dirty = true;

                                                                                                                        }

                                                                    } else {

                                                                        // Dragging a specific point

                                                                        if let Some(el) = scene.elements.get_mut(idx) {

                                                                            let rot_inv = egui::emath::Rot2::from_angle(

                                                                                -el.angle,

                                                                            );

                                                                            let d_local = rot_inv * d;

                                                                            if p_idx < el.points.len() {

                                                                                el.points[p_idx][0] += d_local.x;

                                                                                el.points[p_idx][1] += d_local.y;

                                                                                normalize_element(el);

                                                                            }

                                                                        }

                                                                    }

                                                                } else {

                                                                    // Move all selected elements

                                                                    // Alignment snap only if moving one element

                                                                    if self.selected_indices.len() == 1 && snap_enabled

                                                                    {

                                                                        if let Some(el) = scene.elements.get_mut(idx) {

                                                                            let my_bounds = [

                                                                                el.x + d.x,                  // Left

                                                                                el.x + d.x + el.width / 2.0, // Center X

                                                                                el.x + d.x + el.width,       // Right

                                                                            ];

                                                                            let my_bounds_y = [

                                                                                el.y + d.y,                   // Top

                                                                                el.y + d.y + el.height / 2.0, // Center Y

                                                                                el.y + d.y + el.height,       // Bottom

                                                                            ];

                        

                                                                            let snap_dist = 5.0;

                                                                            for (other_i, other_el) in

                                                                                scene.elements.iter().enumerate()

                                                                            {

                                                                                if other_el.is_deleted || other_i == idx {

                                                                                    continue;

                                                                                }

                        

                                                                                let other_bounds = [

                                                                                    other_el.x,

                                                                                    other_el.x + other_el.width / 2.0,

                                                                                    other_el.x + other_el.width,

                                                                                ];

                                                                                let other_bounds_y = [

                                                                                    other_el.y,

                                                                                    other_el.y + other_el.height / 2.0,

                                                                                    other_el.y + other_el.height,

                                                                                ];

                        

                                                                                // X Snapping

                                                                                for &mb in &my_bounds {

                                                                                    for &ob in &other_bounds {

                                                                                        if (mb - ob).abs() < snap_dist {

                                                                                            let diff = ob - mb;

                                                                                            d.x += diff;

                                                                                            self.active_guides.push(

                                                                                                Rect::from_x_y_ranges(

                                                                                                    ob..=ob,

                                                                                                    -10000.0..=10000.0,

                                                                                                ),

                                                                                            );

                                                                                        }

                                                                                    }

                                                                                }

                                                                                // Y Snapping

                                                                                for &mb in &my_bounds_y {

                                                                                    for &ob in &other_bounds_y {

                                                                                        if (mb - ob).abs() < snap_dist {

                                                                                            let diff = ob - mb;

                                                                                            d.y += diff;

                                                                                            self.active_guides.push(

                                                                                                Rect::from_x_y_ranges(

                                                                                                    -10000.0..=10000.0,

                                                                                                    ob..=ob,

                                                                                                ),

                                                                                            );

                                                                                        }

                                                                                    }

                                                                                }

                                                                            }

                                                                        }

                                                                    }

                        

                                                                    for &s_idx in &self.selected_indices {

                                                                        if let Some(el) = scene.elements.get_mut(s_idx)

                                                                        {

                                                                            el.x += d.x;

                                                                            el.y += d.y;

                                                                        }

                                                                    }

                                                                }

                                                                dirty = true;

                                                            }

                                                            self.last_mouse_pos_world = Some(cwp);

                                                        }

                                                    }

                                                }
                    }
                    _ => {
                        // Drawing snap
                        if response.drag_started_by(PointerButton::Primary)
                            && !self.is_panning
                            && !ui.input(|i| i.modifiers.alt)
                        {
                            if let Some(mp) = response.interact_pointer_pos() {
                                self.push_undo(&scene);
                                let mut sw = to_world(mp);
                                let grid_size = 20.0;
                                if snap_enabled && tool != Tool::Freedraw {
                                    sw.x = (sw.x / grid_size).round() * grid_size;
                                    sw.y = (sw.y / grid_size).round() * grid_size;
                                }

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
                                    let mut cw = to_world(mp);
                                    let grid_size = 20.0;
                                    // Disable snap for Freedraw
                                    if snap_enabled && tool != Tool::Freedraw {
                                        cw.x = (cw.x / grid_size).round() * grid_size;
                                        cw.y = (cw.y / grid_size).round() * grid_size;
                                    }

                                    if let Some(el) = &mut self.drawing_element {
                                        if tool == Tool::Line || tool == Tool::Arrow {
                                            let dx = cw.x - sp.x;
                                            let dy = cw.y - sp.y;
                                            el.x = sp.x;
                                            el.y = sp.y;
                                            el.points = vec![[0.0, 0.0], [dx, dy]];
                                            normalize_element(el);
                                        } else if tool == Tool::Freedraw {
                                            let dx = cw.x - sp.x;
                                            let dy = cw.y - sp.y;
                                            el.points.push([dx, dy]);
                                            // Don't normalize during drag for freedraw to avoid jitter/offset
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
                self.active_guides.clear();

                // If we have multiple selected, pick one for the properties panel (or just use the first)
                self.selected_element_idx = self.selected_indices.iter().next().copied();

                if let Some(mut el) = self.drawing_element.take() {
                    if el.width > 2.0 || el.height > 2.0 || !el.points.is_empty() {
                        if el.element_type == "freedraw" {
                            normalize_element(&mut el);
                        }
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

            if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
                && !self.selected_indices.is_empty()
            {
                self.push_undo(&scene); // Push before deleting
                for &idx in &self.selected_indices {
                    if let Some(el) = scene.elements.get_mut(idx) {
                        el.is_deleted = true;
                    }
                }
                self.selected_indices.clear();
                self.selected_element_idx = None;
                dirty = true;
                save = true;
            }

            // Keyboard arrow movement
            let mut move_vec = Vec2::ZERO;
            let step = if ui.input(|i| i.modifiers.shift) {
                10.0
            } else {
                1.0
            };

            if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                move_vec.x -= step;
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                move_vec.x += step;
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                move_vec.y -= step;
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                move_vec.y += step;
            }

            if move_vec != Vec2::ZERO && !self.selected_indices.is_empty() {
                self.push_undo(&scene);
                for &idx in &self.selected_indices {
                    if let Some(el) = scene.elements.get_mut(idx) {
                        el.x += move_vec.x;
                        el.y += move_vec.y;
                    }
                }
                dirty = true;
                save = true;
            }

            let ctx = ui.ctx().clone();
            let files = &scene.files;
            for (i, el) in scene.elements.iter().enumerate() {
                if el.is_deleted {
                    continue;
                }
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
                let screen_rect = Rect::from_two_pos(to_screen(rect.min), to_screen(rect.max));
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

            // Render Guides
            for guide in &self.active_guides {
                let p1 = to_screen(guide.min);
                let p2 = to_screen(guide.max);
                painter.line_segment([p1, p2], Stroke::new(1.0, Color32::from_rgb(255, 0, 255)));
            }

            ui.scope_builder(UiBuilder::new().max_rect(panel_rect), |ui| {
                egui::Frame::NONE
                    .fill(
                        ui.ctx()
                            .style()
                            .visuals
                            .panel_fill
                            .linear_multiply(240.0 / 255.0),
                    )
                    .stroke(Stroke::new(
                        1.0,
                        ui.ctx().style().visuals.window_stroke.color,
                    ))
                    .corner_radius(12.0)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.set_width(panel_rect.width() - 32.0);

                        if !self.selected_indices.is_empty() {
                            ui.label("Layers:");
                            ui.horizontal(|ui| {
                                if ui.button("⏫").on_hover_text("Bring to Front (Ctrl+Shift+])").clicked() {
                                    if self.move_selection_in_z(&mut scene, 2) {
                                        dirty = true;
                                        save = true;
                                    }
                                }
                                if ui.button("🔼").on_hover_text("Bring Forward (Ctrl+])").clicked() {
                                    if self.move_selection_in_z(&mut scene, 1) {
                                        dirty = true;
                                        save = true;
                                    }
                                }
                                if ui.button("🔽").on_hover_text("Send Backward (Ctrl+[)").clicked() {
                                    if self.move_selection_in_z(&mut scene, -1) {
                                        dirty = true;
                                        save = true;
                                    }
                                }
                                if ui.button("⏬").on_hover_text("Send to Back (Ctrl+Shift+[)").clicked() {
                                    if self.move_selection_in_z(&mut scene, -2) {
                                        dirty = true;
                                        save = true;
                                    }
                                }
                            });
                            ui.separator();
                        }

                        // Capture undo state when starting to interact with the properties panel
                        if ui.input(|i| i.pointer.any_pressed())
                            && ui.rect_contains_pointer(ui.max_rect())
                        {
                            self.push_undo(&scene);
                        }

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
            if dirty {
                self.is_dirty = true;
            }
            if save {
                self.save_file();
            }
        }
    }
}
