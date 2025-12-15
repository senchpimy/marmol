use base64::{engine::general_purpose, Engine as _};
use egui::{
    emath::Rot2, Color32, ColorImage, Context, FontFamily, FontId, PointerButton, Pos2, Rect,
    Sense, Shape, Stroke, TextureHandle, TextureOptions, Ui, Vec2,
};
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExcalidrawRoundness {
    #[serde(rename = "type")]
    round_type: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BoundElement {
    id: String,
    #[serde(rename = "type")]
    element_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExcalidrawFile {
    #[serde(rename = "mimeType")]
    mime_type: String,
    id: String,
    #[serde(rename = "dataURL")]
    data_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExcalidrawElement {
    #[serde(default)]
    id: String,
    #[serde(rename = "type")]
    element_type: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    #[serde(default)]
    angle: f32,
    #[serde(default, rename = "strokeColor")]
    stroke_color: String,
    #[serde(default, rename = "backgroundColor")]
    background_color: String,
    #[serde(default, rename = "fillStyle")]
    fill_style: String,
    #[serde(default, rename = "strokeWidth")]
    stroke_width: f32,
    #[serde(default, rename = "strokeStyle")]
    stroke_style: String,
    #[serde(default)]
    opacity: f32,
    #[serde(default)]
    points: Vec<[f32; 2]>,
    #[serde(default)]
    text: String,
    #[serde(default, rename = "fontSize")]
    font_size: f32,
    #[serde(default)]
    roundness: Option<ExcalidrawRoundness>,
    #[serde(default, rename = "endArrowhead")]
    end_arrowhead: Option<String>,
    #[serde(default, rename = "boundElements")]
    bound_elements: Option<Vec<BoundElement>>,
    #[serde(default, rename = "containerId")]
    container_id: Option<String>,
    #[serde(default, rename = "fileId")]
    file_id: Option<String>,
    #[serde(default)]
    scale: Option<[f32; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExcalidrawScene {
    #[serde(default)]
    type_: String,
    #[serde(default)]
    version: i32,
    #[serde(default)]
    source: String,
    elements: Vec<ExcalidrawElement>,
    #[serde(default)]
    files: HashMap<String, ExcalidrawFile>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

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
    dragged_element_idx: Option<usize>,
    #[serde(skip)]
    last_mouse_pos_world: Option<Pos2>,
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
            dragged_element_idx: None,
            last_mouse_pos_world: None,
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
            Ok(json_content) => match serde_json::from_str::<ExcalidrawScene>(&json_content) {
                Ok(mut scene) => {
                    for (i, el) in scene.elements.iter_mut().enumerate() {
                        if el.id.is_empty() {
                            el.id = format!("gen_id_{}", i);
                        }
                    }
                    self.scene = Some(scene);
                    self.error_msg = None;
                    self.is_dirty = false;
                    self.texture_cache.clear();
                    self.failed_textures.clear();
                    if self.scale == 0.0 {
                        self.scale = 1.0;
                    }
                }
                Err(e) => {
                    self.error_msg = Some(format!("Error parsing JSON: {}", e));
                }
            },
            Err(e) => {
                self.error_msg = Some(format!("Error reading file: {}", e));
            }
        }
    }

    fn save_file(&mut self) {
        if let Some(scene) = &self.scene {
            if let Ok(json) = serde_json::to_string_pretty(scene) {
                if let Err(e) = fs::write(&self.path, json) {
                    self.error_msg = Some(format!("Error saving file: {}", e));
                } else {
                    self.is_dirty = false;
                }
            }
        }
    }

    fn get_or_load_texture(
        &mut self,
        ctx: &Context,
        file_id: &str,
        files: &HashMap<String, ExcalidrawFile>,
    ) -> Option<TextureHandle> {
        if let Some(handle) = self.texture_cache.get(file_id) {
            return Some(handle.clone());
        }
        if self.failed_textures.contains(file_id) {
            return None;
        }

        if let Some(file_data) = files.get(file_id) {
            if file_data.mime_type.contains("svg") {
                self.failed_textures.insert(file_id.to_string());
                return None;
            }

            let parts: Vec<&str> = file_data.data_url.split(',').collect();
            if parts.len() == 2 {
                let base64_data = parts[1];
                if let Ok(bytes) = general_purpose::STANDARD.decode(base64_data) {
                    if let Ok(mut dynamic_image) = image::load_from_memory(&bytes) {
                        const MAX_SIZE: u32 = 1024;
                        if dynamic_image.width() > MAX_SIZE || dynamic_image.height() > MAX_SIZE {
                            dynamic_image =
                                dynamic_image.resize(MAX_SIZE, MAX_SIZE, FilterType::Triangle);
                        }

                        let size = [dynamic_image.width() as _, dynamic_image.height() as _];
                        let image_buffer = dynamic_image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        let color_image =
                            ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

                        let texture =
                            ctx.load_texture(file_id, color_image, TextureOptions::LINEAR);
                        self.texture_cache
                            .insert(file_id.to_string(), texture.clone());
                        return Some(texture);
                    }
                }
            }
        }
        self.failed_textures.insert(file_id.to_string());
        None
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if let Some(err) = &self.error_msg {
            ui.colored_label(Color32::RED, err);
            return;
        }
        if self.scene.is_none() {
            self.reload();
            if self.scene.is_none() {
                ui.label("Loading...");
                return;
            }
        }

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        painter.rect_filled(response.rect, 0.0, Color32::WHITE);

        let screen_rect_min = response.rect.min;
        let screen_rect_max = response.rect.max;

        // ZOOM & PAN
        let scroll_delta = ui.input(|i| i.raw_scroll_delta);
        if ui.input(|i| i.modifiers.ctrl) && scroll_delta.y != 0.0 {
            let zoom_factor = if scroll_delta.y > 0.0 { 1.1 } else { 0.9 };
            let old_scale = self.scale;
            self.scale *= zoom_factor;
            self.scale = self.scale.clamp(0.1, 10.0);

            if let Some(mouse_pos) = response.hover_pos() {
                let mouse_in_canvas = mouse_pos - screen_rect_min;
                let mouse_world_relative = (mouse_in_canvas - self.pan) / old_scale;
                self.pan = mouse_in_canvas - (mouse_world_relative * self.scale);
            }
        } else if scroll_delta.y != 0.0 || scroll_delta.x != 0.0 {
            self.pan += scroll_delta;
        }

        if response.dragged_by(PointerButton::Middle)
            || response.dragged_by(PointerButton::Secondary)
            || (ui.input(|i| i.modifiers.alt) && response.dragged_by(PointerButton::Primary))
        {
            self.pan += response.drag_delta();
            self.is_panning = true;
        } else {
            self.is_panning = false;
        }

        let current_pan = self.pan;
        let current_scale = self.scale;

        let to_screen = move |pos_world: Pos2| -> Pos2 {
            screen_rect_min + current_pan + (pos_world.to_vec2() * current_scale)
        };

        let to_world = move |pos_screen: Pos2| -> Pos2 {
            let rel = pos_screen - screen_rect_min - current_pan;
            Pos2::ZERO + rel / current_scale
        };

        let visible_world_rect =
            Rect::from_min_max(to_world(screen_rect_min), to_world(screen_rect_max));

        if let Some(mut scene) = self.scene.take() {
            let mut dirty = self.is_dirty;
            let mut should_save = false;

            if response.drag_started_by(PointerButton::Primary)
                && !self.is_panning
                && !ui.input(|i| i.modifiers.alt)
            {
                if let Some(mouse_pos) = response.interact_pointer_pos() {
                    let world_pos = to_world(mouse_pos);
                    for (i, el) in scene.elements.iter().enumerate().rev() {
                        if is_point_inside(el, world_pos) {
                            self.dragged_element_idx = Some(i);
                            self.last_mouse_pos_world = Some(world_pos);
                            break;
                        }
                    }
                }
            }

            if let Some(idx) = self.dragged_element_idx {
                if response.dragged_by(PointerButton::Primary) {
                    if let Some(mouse_pos) = response.interact_pointer_pos() {
                        let current_world_pos = to_world(mouse_pos);
                        if let Some(last_pos) = self.last_mouse_pos_world {
                            let delta = current_world_pos - last_pos;
                            if delta.length_sq() > 0.0001 {
                                move_element_group(&mut scene.elements, idx, delta);
                                dirty = true;
                            }
                        }
                        self.last_mouse_pos_world = Some(current_world_pos);
                    }
                }
            }

            if response.drag_stopped() {
                if self.dragged_element_idx.is_some() && dirty {
                    should_save = true;
                }
                self.dragged_element_idx = None;
                self.last_mouse_pos_world = None;
            }

            let ctx = ui.ctx().clone();
            let files_ref = &scene.files;

            for el in &scene.elements {
                let el_rect =
                    Rect::from_min_size(Pos2::new(el.x, el.y), Vec2::new(el.width, el.height))
                        .expand(100.0);

                if !visible_world_rect.intersects(el_rect) {
                    continue;
                }

                let texture = if let Some(fid) = &el.file_id {
                    self.get_or_load_texture(&ctx, fid, files_ref)
                } else {
                    None
                };

                draw_element(&painter, el, texture, &to_screen, current_scale);
            }

            self.scene = Some(scene);
            self.is_dirty = dirty;
            if should_save {
                self.save_file();
            }
        }
    }
}

fn move_element_group(elements: &mut Vec<ExcalidrawElement>, root_idx: usize, delta: Vec2) {
    let mut indices_to_move = vec![root_idx];
    let root_id = elements[root_idx].id.clone();
    let root_container_id = elements[root_idx].container_id.clone();

    if let Some(container_id) = root_container_id {
        if let Some(parent_idx) = elements.iter().position(|e| e.id == container_id) {
            indices_to_move.push(parent_idx);
        }
    }

    if let Some(bounds) = &elements[root_idx].bound_elements {
        for bound in bounds {
            if let Some(child_idx) = elements.iter().position(|e| e.id == bound.id) {
                indices_to_move.push(child_idx);
            }
        }
    }

    for (i, el) in elements.iter().enumerate() {
        if let Some(cid) = &el.container_id {
            if *cid == root_id && !indices_to_move.contains(&i) {
                indices_to_move.push(i);
            }
        }
    }

    indices_to_move.sort_unstable();
    indices_to_move.dedup();

    for idx in indices_to_move {
        if let Some(el) = elements.get_mut(idx) {
            el.x += delta.x;
            el.y += delta.y;
        }
    }
}

fn is_point_inside(el: &ExcalidrawElement, p: Pos2) -> bool {
    let margin = 10.0;
    let rect =
        Rect::from_min_size(Pos2::new(el.x, el.y), Vec2::new(el.width, el.height)).expand(margin);
    rect.contains(p)
}

fn parse_color(hex: &str, alpha: u8) -> Color32 {
    if hex == "transparent" {
        return Color32::TRANSPARENT;
    }
    let hex_clean = hex.trim_start_matches('#');
    if hex_clean.len() == 6 {
        if let Ok(num) = u32::from_str_radix(hex_clean, 16) {
            return Color32::from_rgba_unmultiplied(
                ((num >> 16) & 0xFF) as u8,
                ((num >> 8) & 0xFF) as u8,
                (num & 0xFF) as u8,
                alpha,
            );
        }
    } else if hex_clean.len() == 3 {
        if let Ok(num) = u16::from_str_radix(hex_clean, 16) {
            let r = ((num >> 8) & 0xF) as u8;
            let g = ((num >> 4) & 0xF) as u8;
            let b = (num & 0xF) as u8;
            return Color32::from_rgba_unmultiplied(r * 17, g * 17, b * 17, alpha);
        }
    }
    Color32::from_rgba_unmultiplied(0, 0, 0, alpha)
}

fn draw_stroke(
    painter: &egui::Painter,
    points: Vec<Pos2>,
    stroke: Stroke,
    style: &str,
    scale: f32,
    closed: bool,
) {
    if points.len() < 2 {
        return;
    }

    match style {
        "dashed" | "dotted" => {
            let (dash, gap) = if style == "dashed" {
                (10.0 * scale, 10.0 * scale)
            } else {
                (2.0 * scale, 6.0 * scale)
            };

            let mut final_points = points.clone();
            if closed {
                final_points.push(points[0]); // Cerrar el loop para shapes
            }

            painter.add(Shape::dashed_line(&final_points, stroke, dash, gap));
        }
        _ => {
            if closed {
                painter.add(Shape::closed_line(points, stroke));
            } else {
                painter.add(Shape::line(points, stroke));
            }
        }
    }
}

fn draw_arrow_head(painter: &egui::Painter, p_end: Pos2, p_prev: Pos2, scale: f32, stroke: Stroke) {
    let vec = p_end - p_prev;
    let angle = vec.angle();
    let arrow_len = 20.0 * scale;
    let spread = std::f32::consts::PI / 6.0;

    let angle_left = angle + std::f32::consts::PI - spread;
    let angle_right = angle + std::f32::consts::PI + spread;

    let p_left = p_end + Vec2::new(arrow_len * angle_left.cos(), arrow_len * angle_left.sin());
    let p_right = p_end + Vec2::new(arrow_len * angle_right.cos(), arrow_len * angle_right.sin());

    painter.add(Shape::line(vec![p_left, p_end, p_right], stroke));
}

fn draw_element<F>(
    painter: &egui::Painter,
    el: &ExcalidrawElement,
    texture: Option<TextureHandle>,
    to_screen: &F,
    scale: f32,
) where
    F: Fn(Pos2) -> Pos2,
{
    if el.opacity == 0.0 {
        return;
    }

    let alpha = ((el.opacity / 100.0) * 255.0) as u8;
    let stroke_color = parse_color(&el.stroke_color, alpha);
    let bg_color = parse_color(&el.background_color, alpha);

    let stroke = Stroke::new(el.stroke_width * scale, stroke_color);

    let center_world = Pos2::new(el.x + el.width / 2.0, el.y + el.height / 2.0);
    let center_local = Pos2::new(el.width / 2.0, el.height / 2.0);
    let rot = Rot2::from_angle(el.angle);

    let tr = |ps: &[Pos2]| -> Vec<Pos2> {
        ps.iter()
            .map(|&p| {
                let offset = p - center_local;
                let rotated_offset = rot * offset;
                to_screen(center_world + rotated_offset)
            })
            .collect()
    };

    match el.element_type.as_str() {
        "rectangle" | "diamond" | "ellipse" => {
            let pts = if el.element_type == "rectangle" {
                let r = el
                    .roundness
                    .as_ref()
                    .map(|x| if x.round_type == 3 { 20.0 } else { 4.0 })
                    .unwrap_or(0.0);
                discretize_rect(
                    Rect::from_min_size(Pos2::ZERO, Vec2::new(el.width, el.height)),
                    r,
                )
            } else if el.element_type == "diamond" {
                vec![
                    Pos2::new(el.width / 2.0, 0.0),
                    Pos2::new(el.width, el.height / 2.0),
                    Pos2::new(el.width / 2.0, el.height),
                    Pos2::new(0.0, el.height / 2.0),
                ]
            } else {
                discretize_ellipse(el.width, el.height)
            };

            let s_pts = tr(&pts);

            if el.background_color != "transparent" {
                painter.add(Shape::convex_polygon(s_pts.clone(), bg_color, Stroke::NONE));
            }

            draw_stroke(painter, s_pts, stroke, &el.stroke_style, scale, true);
        }
        "image" => {
            if let Some(tex) = texture {
                let rect_local = Rect::from_min_size(Pos2::ZERO, Vec2::new(el.width, el.height));
                let corners = [
                    rect_local.min,
                    rect_local.right_top(),
                    rect_local.max,
                    rect_local.left_bottom(),
                ];
                let s_pts = tr(&corners);

                let mut mesh = egui::Mesh::with_texture(tex.id());
                let tint = Color32::from_white_alpha(alpha);
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(0, 2, 3);
                let uvs = [
                    Pos2::new(0.0, 0.0),
                    Pos2::new(1.0, 0.0),
                    Pos2::new(1.0, 1.0),
                    Pos2::new(0.0, 1.0),
                ];
                for (i, p) in s_pts.iter().enumerate() {
                    mesh.vertices.push(egui::epaint::Vertex {
                        pos: *p,
                        uv: uvs[i],
                        color: tint,
                    });
                }
                painter.add(Shape::mesh(mesh));
            }
        }
        "line" | "arrow" | "draw" | "freedraw" => {
            if !el.points.is_empty() {
                let raw: Vec<Pos2> = el.points.iter().map(|p| Pos2::new(p[0], p[1])).collect();
                let s_pts = tr(&raw);

                draw_stroke(
                    painter,
                    s_pts.clone(),
                    stroke,
                    &el.stroke_style,
                    scale,
                    false,
                );

                if let Some(arrow_type) = &el.end_arrowhead {
                    if arrow_type == "arrow" && s_pts.len() >= 2 {
                        let last = s_pts[s_pts.len() - 1];
                        let prev = s_pts[s_pts.len() - 2];
                        let arrow_stroke = Stroke::new(el.stroke_width * scale, stroke_color);
                        draw_arrow_head(painter, last, prev, scale, arrow_stroke);
                    }
                }
            }
        }
        "text" => {
            let tl = tr(&[Pos2::ZERO])[0];
            painter.text(
                tl,
                egui::Align2::LEFT_TOP,
                &el.text,
                FontId::new(el.font_size * scale, FontFamily::Proportional),
                stroke_color,
            );
        }
        _ => {}
    }
}

// --- GEOMETRÍA ---
fn discretize_rect(rect: Rect, radius: f32) -> Vec<Pos2> {
    if radius <= 1.0 {
        return vec![rect.min, rect.right_top(), rect.max, rect.left_bottom()];
    }
    let mut pts = Vec::new();
    let r = radius.min(rect.width() / 2.0).min(rect.height() / 2.0);
    add_arc(
        &mut pts,
        Pos2::new(rect.max.x - r, rect.min.y + r),
        r,
        -1.57,
        0.0,
    );
    add_arc(
        &mut pts,
        Pos2::new(rect.max.x - r, rect.max.y - r),
        r,
        0.0,
        1.57,
    );
    add_arc(
        &mut pts,
        Pos2::new(rect.min.x + r, rect.max.y - r),
        r,
        1.57,
        3.14,
    );
    add_arc(
        &mut pts,
        Pos2::new(rect.min.x + r, rect.min.y + r),
        r,
        3.14,
        4.71,
    );
    pts
}
fn discretize_ellipse(w: f32, h: f32) -> Vec<Pos2> {
    let rx = w / 2.0;
    let ry = h / 2.0;
    let c = Pos2::new(rx, ry);
    (0..64)
        .map(|i| {
            let t = (i as f32 / 64.0) * 6.28;
            c + Vec2::new(rx * t.cos(), ry * t.sin())
        })
        .collect()
}
fn add_arc(pts: &mut Vec<Pos2>, c: Pos2, r: f32, start: f32, end: f32) {
    for i in 0..=8 {
        let t = i as f32 / 8.0;
        let a = start + (end - start) * t;
        pts.push(c + Vec2::new(r * a.cos(), r * a.sin()));
    }
}
