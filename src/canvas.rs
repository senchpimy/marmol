use crate::egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui::{Color32, Id, Image, Margin, Pos2, Rect, Scene, Sense, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum CanvasNode {
    #[serde(rename = "file")]
    File {
        id: String,
        file: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<String>,
    },
    #[serde(rename = "text")]
    Text {
        id: String,
        text: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<String>,
    },
    #[serde(rename = "group")]
    Group {
        id: String,
        label: Option<String>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CanvasEdge {
    pub id: String,
    pub from_node: String,
    pub from_side: String,
    pub to_node: String,
    pub to_side: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CanvasData {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
}

#[derive(PartialEq, Clone)]
pub enum PickerType {
    Markdown,
    Media,
}

pub struct CanvasGui {
    pub path: String,
    pub data: CanvasData,
    pub scene_rect: Rect,
    pub is_dirty: bool,
    pub commonmark_cache: CommonMarkCache,
    pub file_cache: HashMap<String, String>,
    
    // State for creating new edges
    pub drag_edge_start: Option<(String, String)>, // (node_id, side)

    // Selection and Editing
    pub selected_node_id: Option<String>,
    pub editing_node_id: Option<String>,

    // UI State
    pub show_file_picker: Option<PickerType>,
}

impl Default for CanvasGui {
    fn default() -> Self {
        Self {
            path: String::new(),
            data: CanvasData::default(),
            scene_rect: Rect::from_center_size(Pos2::ZERO, Vec2::splat(1000.0)),
            is_dirty: false,
            commonmark_cache: CommonMarkCache::default(),
            file_cache: HashMap::new(),
            drag_edge_start: None,
            selected_node_id: None,
            editing_node_id: None,
            show_file_picker: None,
        }
    }
}

impl CanvasGui {
    pub fn set_path(&mut self, path: &str) {
        if self.path != path {
            self.path = path.to_string();
            self.reload();
        }
    }

    pub fn reload(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.path) {
            if let Ok(data) = serde_json::from_str::<CanvasData>(&content) {
                self.data = data;
                self.is_dirty = false;
            }
        }
    }

    pub fn save(&mut self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = fs::write(&self.path, json);
            self.is_dirty = false;
        }
    }

    pub fn show(&mut self, ui: &mut Ui, vault: &str) {
        let scene = Scene::new().zoom_range(0.01..=10.0);

        let mut scene_rect = self.scene_rect;

        let mut data = std::mem::take(&mut self.data);
        let mut commonmark_cache = std::mem::take(&mut self.commonmark_cache);
        let mut file_cache = std::mem::take(&mut self.file_cache);
        let mut drag_edge_start = self.drag_edge_start.take();

        let mut moved = false;

        scene.show(ui, &mut scene_rect, |ui| {
            Self::draw_edges_static(ui, &data);

            let (m, new_drag) = Self::draw_nodes_static(
                ui,
                vault,
                &mut data,
                &self.path,
                &mut commonmark_cache,
                &mut file_cache,
                &drag_edge_start,
            );
            moved = m;
            if new_drag.is_some() {
                drag_edge_start = new_drag;
            }

            if let Some((start_node_id, start_side)) = &drag_edge_start {
                if let Some(start_node) = find_node_static(&data, start_node_id) {
                    let start_pos = get_side_pos_static(start_node, start_side);

                    // Get world-space mouse position
                    let transform = ui
                        .ctx()
                        .layer_transform_to_global(ui.layer_id())
                        .unwrap_or_default();
                    if let Some(mouse_screen_pos) = ui.input(|i| i.pointer.interact_pos()) {
                        let mouse_world_pos = transform.inverse() * mouse_screen_pos;

                        let mut end_pos = mouse_world_pos;
                        let mut target_info = None;

                        for node in &data.nodes {
                            let node_id = match node {
                                CanvasNode::File { id, .. } => id,
                                CanvasNode::Text { id, .. } => id,
                                CanvasNode::Group { id, .. } => id,
                            };
                            if node_id == start_node_id {
                                continue;
                            }

                            let (x, y, w, h) = match node {
                                CanvasNode::File {
                                    x,
                                    y,
                                    width,
                                    height,
                                    ..
                                } => (*x, *y, *width, *height),
                                CanvasNode::Text {
                                    x,
                                    y,
                                    width,
                                    height,
                                    ..
                                } => (*x, *y, *width, *height),
                                CanvasNode::Group {
                                    x,
                                    y,
                                    width,
                                    height,
                                    ..
                                } => (*x, *y, *width, *height),
                            };
                            let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h));

                            if rect.expand(30.0).contains(mouse_world_pos) {
                                let side = get_closest_side(mouse_world_pos, rect);
                                end_pos = get_side_pos_static(node, &side);
                                target_info = Some((node_id.clone(), side));
                                break;
                            }
                        }

                        let painter = ui.painter();
                        let color = ui.visuals().widgets.active.fg_stroke.color;
                        painter.line_segment([start_pos, end_pos], Stroke::new(2.0, color));

                        let dir = (end_pos - start_pos).normalized();
                        let tip = end_pos;
                        let angle = 30.0f32.to_radians();
                        let length = 15.0;
                        let p1 = tip - Vec2::angled(dir.angle() + angle) * length;
                        let p2 = tip - Vec2::angled(dir.angle() - angle) * length;
                        painter.line_segment([tip, p1], Stroke::new(2.0, color));
                        painter.line_segment([tip, p2], Stroke::new(2.0, color));

                        if ui.input(|i| i.pointer.any_released()) {
                            if let Some((to_id, to_s)) = target_info {
                                let id = format!("{:x}", rand::random::<u64>());
                                data.edges.push(CanvasEdge {
                                    id,
                                    from_node: start_node_id.clone(),
                                    from_side: start_side.clone(),
                                    to_node: to_id,
                                    to_side: to_s,
                                    label: None,
                                    color: None,
                                });
                                moved = true;
                            }
                            drag_edge_start = None;
                        }
                    } else if ui.input(|i| i.pointer.any_released()) {
                        drag_edge_start = None;
                    }
                }
            }
        });

        self.drag_edge_start = drag_edge_start;
        self.scene_rect = scene_rect;
        self.data = data;
        self.commonmark_cache = commonmark_cache;
        self.file_cache = file_cache;

        if moved {
            self.is_dirty = true;
            self.save();
        }

        // Floating Toolbar
        let transform = ui.ctx().layer_transform_to_global(ui.layer_id()).unwrap_or_default();
        let world_center = transform.inverse() * ui.max_rect().center();

        egui::Area::new(Id::new("canvas_toolbar"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
            .show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("📝 Card").clicked() {
                            let id = format!("{:x}", rand::random::<u64>());
                            self.data.nodes.push(CanvasNode::Text {
                                id,
                                text: "New Card".to_string(),
                                x: world_center.x - 100.0,
                                y: world_center.y - 50.0,
                                width: 200.0,
                                height: 100.0,
                                color: None,
                            });
                            self.is_dirty = true;
                            self.save();
                        }
                        if ui.button("📄 Markdown").clicked() {
                            self.show_file_picker = Some(PickerType::Markdown);
                        }
                        if ui.button("🖼 Media").clicked() {
                            self.show_file_picker = Some(PickerType::Media);
                        }
                    });
                });
            });

        // File Picker Modal
        if let Some(picker) = self.show_file_picker.clone() {
            let mut close = false;
            let mut selected_file = None;
            
            egui::Window::new(match picker {
                PickerType::Markdown => "Select Markdown File",
                PickerType::Media => "Select Media File",
            })
            .collapsible(false)
            .resizable(true)
            .default_size([400.0, 500.0])
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let extensions: &[&str] = match picker {
                        PickerType::Markdown => &[".md"],
                        PickerType::Media => &[".png", ".jpg", ".jpeg", ".svg"],
                    };
                    
                    use walkdir::WalkDir;
                    for entry in WalkDir::new(vault).into_iter().filter_map(|e| e.ok()) {
                        if entry.file_type().is_file() {
                            let path = entry.path();
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
                            let ext_dot = format!(".{}", ext.to_lowercase());
                            
                            if extensions.iter().any(|&e| e == ext_dot) {
                                let rel_path = path.strip_prefix(vault).unwrap_or(path).to_string_lossy().to_string();
                                if ui.selectable_label(false, &rel_path).clicked() {
                                    selected_file = Some(rel_path);
                                    close = true;
                                }
                            }
                        }
                    }
                });
                ui.separator();
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
            
            if let Some(file) = selected_file {
                let id = format!("{:x}", rand::random::<u64>());
                self.data.nodes.push(CanvasNode::File {
                    id,
                    file,
                    x: world_center.x - 150.0,
                    y: world_center.y - 150.0,
                    width: 300.0,
                    height: 300.0,
                    color: None,
                });
                self.is_dirty = true;
                self.save();
            }
            
            if close {
                self.show_file_picker = None;
            }
        }
    }

    fn draw_nodes_static(
        ui: &mut Ui,
        vault: &str,
        data: &mut CanvasData,
        canvas_path: &str,
        commonmark_cache: &mut CommonMarkCache,
        file_cache: &mut HashMap<String, String>,
        current_drag: &Option<(String, String)>,
    ) -> (bool, Option<(String, String)>) {
        let mut moved = false;
        let mut new_drag = None;

        for node in data.nodes.iter_mut() {
            let (node_id, x, y, width, height, color) = match node {
                CanvasNode::File {
                    id,
                    x,
                    y,
                    width,
                    height,
                    color,
                    ..
                } => (id.clone(), x, y, width, height, color),
                CanvasNode::Text {
                    id,
                    x,
                    y,
                    width,
                    height,
                    color,
                    ..
                } => (id.clone(), x, y, width, height, color),
                CanvasNode::Group {
                    id,
                    x,
                    y,
                    width,
                    height,
                    color,
                    ..
                } => (id.clone(), x, y, width, height, color),
            };

            let node_rect = Rect::from_min_size(Pos2::new(*x, *y), Vec2::new(*width, *height));

            let border_color = color.as_ref().and_then(|c| parse_color(c));
            let fill_color = ui.visuals().window_fill();

            ui.put(node_rect, |ui: &mut Ui| {
                let mut frame = egui::Frame::NONE
                    .fill(fill_color)
                    .stroke(ui.visuals().window_stroke())
                    .corner_radius(ui.visuals().window_corner_radius)
                    .inner_margin(Margin::ZERO);

                if let Some(c) = border_color {
                    frame = frame.stroke(Stroke::new(2.0, c));
                }

                let response = frame
                    .show(ui, |ui| match node {
                        CanvasNode::File {
                            file,
                            width,
                            height,
                            ..
                        } => {
                            ui.set_min_size(Vec2::new(*width, *height));
                            let resolved = crate::files::resolve_path(vault, canvas_path, file);

                            if let Some(path) = resolved {
                                if path.ends_with(".md") {
                                    let content =
                                        file_cache.entry(path.clone()).or_insert_with(|| {
                                            fs::read_to_string(&path).unwrap_or_default()
                                        });
                                    egui::ScrollArea::vertical()
                                        .id_salt(format!("scroll_{}", path))
                                        .show(ui, |ui| {
                                            CommonMarkViewer::new().show(
                                                ui,
                                                commonmark_cache,
                                                content,
                                            );
                                        });
                                } else if path.ends_with(".png")
                                    || path.ends_with(".jpg")
                                    || path.ends_with(".jpeg")
                                {
                                    let img =
                                        Image::from_uri(format!("file://{}", path)).shrink_to_fit();
                                    ui.add(img);
                                } else {
                                    ui.vertical_centered(|ui| {
                                        ui.label(&*file);
                                        ui.weak("Unsupported preview");
                                    });
                                }
                            }
                        }
                        CanvasNode::Text {
                            text,
                            width,
                            height,
                            ..
                        } => {
                            ui.set_min_size(Vec2::new(*width, *height));
                            egui::ScrollArea::vertical()
                                .id_salt(format!("scroll_text_{}", node_id))
                                .show(ui, |ui| {
                                    CommonMarkViewer::new().show(ui, commonmark_cache, text);
                                });
                        }
                        CanvasNode::Group {
                            label,
                            width,
                            height,
                            ..
                        } => {
                            ui.set_min_size(Vec2::new(*width, *height));
                            if let Some(l) = label {
                                ui.vertical_centered(|ui| {
                                    ui.heading(&*l);
                                });
                            }
                        }
                    })
                    .response;

                let response = response.interact(Sense::click_and_drag());

                if response.double_clicked() {
                    if let CanvasNode::File { file, .. } = node {
                        let resolved = crate::files::resolve_path(vault, canvas_path, file);
                        if let Some(path) = resolved {
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(Id::new("global_nav_request"), Some(path))
                            });
                        }
                    }
                }

                if response.dragged() && current_drag.is_none() {
                    let delta = response.drag_delta();
                    match node {
                        CanvasNode::File { x, y, .. } => {
                            *x += delta.x;
                            *y += delta.y;
                        }
                        CanvasNode::Text { x, y, .. } => {
                            *x += delta.x;
                            *y += delta.y;
                        }
                        CanvasNode::Group { x, y, .. } => {
                            *x += delta.x;
                            *y += delta.y;
                        }
                    }
                    moved = true;
                }

                response
            });

            // Handles for connections - use ui.interact directly for precision
            let sides = ["top", "bottom", "left", "right"];
            for side in sides {
                let side_pos = get_side_pos_static(node, side);
                let handle_size = 20.0;
                let handle_rect = Rect::from_center_size(side_pos, Vec2::splat(handle_size));

                let response = ui.interact(
                    handle_rect,
                    Id::new(&node_id).with(side),
                    Sense::click_and_drag(),
                );

                if response.hovered() || response.dragged() {
                    ui.painter()
                        .circle_filled(side_pos, 8.0, ui.visuals().widgets.active.bg_fill);
                    ui.painter().circle_stroke(
                        side_pos,
                        8.0,
                        Stroke::new(1.0, ui.visuals().window_stroke().color),
                    );
                } else {
                    ui.painter().circle_filled(
                        side_pos,
                        4.0,
                        ui.visuals().widgets.inactive.bg_fill.linear_multiply(0.5),
                    );
                }

                if response.drag_started() {
                    new_drag = Some((node_id.clone(), side.to_string()));
                }
            }
        }

        (moved, new_drag)
    }

    fn draw_edges_static(ui: &mut Ui, data: &CanvasData) {
        let painter = ui.painter();
        for edge in &data.edges {
            if let (Some(from_node), Some(to_node)) = (
                find_node_static(data, &edge.from_node),
                find_node_static(data, &edge.to_node),
            ) {
                let from_pos = get_side_pos_static(from_node, &edge.from_side);
                let to_pos = get_side_pos_static(to_node, &edge.to_side);

                let color = edge
                    .color
                    .as_ref()
                    .and_then(|c| parse_color(c))
                    .unwrap_or(ui.visuals().widgets.active.fg_stroke.color);

                painter.line_segment([from_pos, to_pos], Stroke::new(2.0, color));
                let dir = (to_pos - from_pos).normalized();
                let tip = to_pos;
                let angle = 30.0f32.to_radians();
                let length = 12.0;

                let p1 = tip - Vec2::angled(dir.angle() + angle) * length;
                let p2 = tip - Vec2::angled(dir.angle() - angle) * length;

                painter.line_segment([tip, p1], Stroke::new(2.0, color));
                painter.line_segment([tip, p2], Stroke::new(2.0, color));
            }
        }
    }
}

fn find_node_static<'a>(data: &'a CanvasData, id: &str) -> Option<&'a CanvasNode> {
    data.nodes.iter().find(|n| match n {
        CanvasNode::File { id: nid, .. } => nid == id,
        CanvasNode::Text { id: nid, .. } => nid == id,
        CanvasNode::Group { id: nid, .. } => nid == id,
    })
}

fn get_side_pos_static(node: &CanvasNode, side: &str) -> Pos2 {
    let (x, y, w, h) = match node {
        CanvasNode::File {
            x,
            y,
            width,
            height,
            ..
        } => (*x, *y, *width, *height),
        CanvasNode::Text {
            x,
            y,
            width,
            height,
            ..
        } => (*x, *y, *width, *height),
        CanvasNode::Group {
            x,
            y,
            width,
            height,
            ..
        } => (*x, *y, *width, *height),
    };

    match side {
        "top" => Pos2::new(x + w / 2.0, y),
        "bottom" => Pos2::new(x + w / 2.0, y + h),
        "left" => Pos2::new(x, y + h / 2.0),
        "right" => Pos2::new(x + w, y + h / 2.0),
        _ => Pos2::new(x + w / 2.0, y + h / 2.0),
    }
}

fn get_closest_side(pos: Pos2, rect: Rect) -> String {
    let top = pos.distance_sq(Pos2::new(rect.center().x, rect.top()));
    let bottom = pos.distance_sq(Pos2::new(rect.center().x, rect.bottom()));
    let left = pos.distance_sq(Pos2::new(rect.left(), rect.center().y));
    let right = pos.distance_sq(Pos2::new(rect.right(), rect.center().y));

    let mut min = top;
    let mut side = "top";

    if bottom < min {
        min = bottom;
        side = "bottom";
    }
    if left < min {
        min = left;
        side = "left";
    }
    if right < min {
        side = "right";
    }

    side.to_string()
}

fn parse_color(color: &str) -> Option<Color32> {
    if color.starts_with('#') {
        if let Ok(c) = hex_to_color(color) {
            return Some(c);
        }
    }
    match color {
        "1" => Some(Color32::from_rgb(255, 0, 0)),
        "2" => Some(Color32::from_rgb(0, 255, 0)),
        "3" => Some(Color32::from_rgb(0, 0, 255)),
        _ => None,
    }
}

fn hex_to_color(hex: &str) -> Result<Color32, &str> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "invalid hex")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "invalid hex")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "invalid hex")?;
        Ok(Color32::from_rgb(r, g, b))
    } else {
        Err("invalid length")
    }
}
