use serde::{Deserialize, Serialize};
use egui::{Color32, Id, Image, Pos2, Rect, Scene, Sense, Stroke, Ui, Vec2, PointerButton, StrokeKind, DragPanButtons};
use std::fs;
use std::collections::{HashMap, HashSet};
use crate::egui_commonmark::{CommonMarkCache, CommonMarkViewer};

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
    
    pub drag_edge_start: Option<(String, String)>,
    pub selected_node_ids: HashSet<String>,
    pub editing_node_id: Option<String>,
    pub show_file_picker: Option<PickerType>,
    
    // Selection marquee
    pub selection_start: Option<Pos2>,
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
            selected_node_ids: HashSet::new(),
            editing_node_id: None,
            show_file_picker: None,
            selection_start: None,
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
        let unique_id = ui.id();
        let scene = Scene::new()
            .zoom_range(0.01..=10.0)
            .drag_pan_buttons(DragPanButtons::MIDDLE);
        let mut scene_rect = self.scene_rect;
        
        let mut data = std::mem::take(&mut self.data);
        let mut commonmark_cache = std::mem::take(&mut self.commonmark_cache);
        let mut file_cache = std::mem::take(&mut self.file_cache);
        let mut drag_edge_start = self.drag_edge_start.clone();
        let mut selected_node_ids = std::mem::take(&mut self.selected_node_ids);
        let mut editing_node_id = self.editing_node_id.clone();
        let mut selection_start = self.selection_start;

        let mut moved = false;
        let mut node_to_delete = None;
        let mut zoom_to_node = None;

        let canvas_path = self.path.clone();

        ui.push_id(unique_id.with("canvas_scene_scope"), |ui| {
            scene.show(ui, &mut scene_rect, |ui| {
                let pointer_pos = ui.input(|i| i.pointer.interact_pos());

                // 1. BACKGROUND INTERACTION (Drawn first to be behind nodes)
                let bg_resp = ui.interact(ui.max_rect(), ui.id().with("bg"), Sense::click_and_drag());
                
                // Start marquee selection
                if bg_resp.drag_started_by(PointerButton::Primary) {
                    selection_start = pointer_pos;
                    if !ui.input(|i| i.modifiers.shift) {
                        selected_node_ids.clear();
                    }
                    editing_node_id = None;
                }

                // Deselect on simple click
                if bg_resp.clicked_by(PointerButton::Primary) {
                    selected_node_ids.clear();
                    editing_node_id = None;
                }

                // Selection marquee logic
                let mut marquee_to_draw = None;
                if let Some(start) = selection_start {
                    if let Some(mwp) = pointer_pos {
                        let marquee_rect = Rect::from_two_pos(start, mwp);
                        marquee_to_draw = Some(marquee_rect);

                        // Multi-select nodes inside
                        if !ui.input(|i| i.modifiers.shift) { selected_node_ids.clear(); }
                        for node in &data.nodes {
                            let (nx, ny, nw, nh) = get_node_geom(node);
                            let nr = Rect::from_min_size(Pos2::new(nx, ny), Vec2::new(nw, nh));
                            if marquee_rect.intersects(nr) {
                                selected_node_ids.insert(get_node_id(node));
                            }
                        }
                    }
                    if ui.input(|i| i.pointer.any_released()) {
                        selection_start = None;
                    }
                }

                Self::draw_edges_static(ui, &data);
                
                // 2. DRAW NODES
                let (m, d, s, e, delta) = Self::draw_nodes_static(
                    ui, vault, &mut data, &canvas_path, 
                    &mut commonmark_cache, &mut file_cache, 
                    &drag_edge_start, &selected_node_ids, &editing_node_id
                );
                
                if m { moved = true; }
                if d.is_some() { drag_edge_start = d; }
                
                if let Some(new_sel_id) = s {
                    if ui.input(|i| i.modifiers.shift) {
                        if selected_node_ids.contains(&new_sel_id) { selected_node_ids.remove(&new_sel_id); }
                        else { selected_node_ids.insert(new_sel_id); }
                    } else {
                        // Always update selection on click unless shift is held
                        if !selected_node_ids.contains(&new_sel_id) || selected_node_ids.len() > 1 {
                            selected_node_ids.clear();
                            selected_node_ids.insert(new_sel_id);
                        }
                    }
                    editing_node_id = None;
                }

                // Bulk move
                if let Some(d) = delta {
                    for nid in &selected_node_ids {
                        if let Some(n) = find_node_mut_static(&mut data, nid) {
                            move_node(n, d);
                        }
                    }
                    moved = true;
                }

                if e.is_some() { editing_node_id = e; }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) { editing_node_id = None; }

                // 3. CONTEXT MENU
                if let Some(id) = selected_node_ids.iter().next() {
                    if let Some(node) = find_node_static(&data, id) {
                        let node_info = (get_node_geom(node), matches!(node, CanvasNode::Text { .. }), get_node_color(node));
                        let ((nx, ny, nw, nh), is_text, current_color_str) = node_info;
                        let menu_pos = Pos2::new(nx + nw / 2.0, ny - 35.0);
                        
                        ui.put(Rect::from_center_size(menu_pos, Vec2::new(250.0, 40.0)), |ui: &mut Ui| {
                            ui.horizontal(|ui| {
                                if ui.button("🗑").clicked() { node_to_delete = Some(id.clone()); }
                                for c in &["1", "2", "3"] {
                                    let color_val = parse_color(c).unwrap_or(Color32::WHITE);
                                    if ui.button(egui::RichText::new("■").color(color_val)).clicked() {
                                        for sid in &selected_node_ids {
                                            if let Some(n) = find_node_mut_static(&mut data, sid) { set_node_color(n, Some(c.to_string())); }
                                        }
                                        moved = true;
                                    }
                                }
                                
                                let current_c = current_color_str.and_then(|cs| parse_color(&cs)).unwrap_or(Color32::WHITE);
                                let mut color_rgba = current_c;
                                if ui.color_edit_button_srgba(&mut color_rgba).changed() {
                                    let hex = format!("#{:02x}{:02x}{:02x}", color_rgba.r(), color_rgba.g(), color_rgba.b());
                                    for sid in &selected_node_ids {
                                        if let Some(n) = find_node_mut_static(&mut data, sid) { set_node_color(n, Some(hex.clone())); }
                                    }
                                    moved = true;
                                }

                                if ui.button("🔍").clicked() { zoom_to_node = Some(id.clone()); }
                                if ui.button("📝").clicked() {
                                    if is_text { editing_node_id = Some(id.clone()); }
                                    else {
                                        if let Some(n) = data.nodes.iter().find(|n| get_node_id(n) == *id) {
                                            if let CanvasNode::File { file, .. } = n {
                                                if let Some(p) = crate::files::resolve_path(vault, &canvas_path, file) {
                                                    ui.ctx().data_mut(|d| d.insert_temp(Id::new("global_nav_request"), Some(p)));
                                                }
                                            }
                                        }
                                    }
                                }
                            }).response
                        });
                    }
                }

                // 4. EDGE PREVIEW
                if let Some((sid, s_side)) = &drag_edge_start {
                    if let Some(sn) = find_node_static(&data, sid) {
                        let sp = get_side_pos_static(sn, s_side);
                        if let Some(mwp) = ui.input(|i| i.pointer.interact_pos()) {
                            let mut ep = mwp;
                            let mut target = None;
                            for node in &data.nodes {
                                let (nid, nx, ny, nw, nh) = get_node_info(node);
                                if nid == *sid { continue; }
                                let r = Rect::from_min_size(Pos2::new(nx, ny), Vec2::new(nw, nh));
                                if r.expand(25.0).contains(mwp) {
                                    let side = get_closest_side(mwp, r);
                                    ep = get_side_pos_static(node, &side);
                                    target = Some((nid.clone(), side));
                                    break;
                                }
                            }
                            ui.painter().line_segment([sp, ep], Stroke::new(2.0, ui.visuals().widgets.active.fg_stroke.color));
                            if ui.input(|i| i.pointer.any_released()) {
                                if let Some((to_id, to_s)) = target {
                                    data.edges.push(CanvasEdge {
                                        id: format!("{:x}", rand::random::<u64>()),
                                        from_node: sid.clone(), from_side: s_side.clone(),
                                        to_node: to_id, to_side: to_s, label: None, color: None,
                                    });
                                    moved = true;
                                }
                                drag_edge_start = None;
                            }
                        }
                    }
                }

                // 5. DRAW SELECTION MARQUEE (on top)
                if let Some(marquee_rect) = marquee_to_draw {
                    ui.painter().rect_stroke(marquee_rect, 0.0, Stroke::new(1.0, Color32::from_rgb(100, 150, 255)), StrokeKind::Outside);
                    ui.painter().rect_filled(marquee_rect, 0.0, Color32::from_rgba_unmultiplied(100, 150, 255, 30));
                }
            });
        });

        if let Some(_) = node_to_delete {
            data.nodes.retain(|n| !selected_node_ids.contains(&get_node_id(n)));
            data.edges.retain(|e| !selected_node_ids.contains(&e.from_node) && !selected_node_ids.contains(&e.to_node));
            selected_node_ids.clear();
            moved = true;
        }

        if let Some(id) = zoom_to_node {
            if let Some(node) = find_node_static(&data, &id) {
                let (nx, ny, nw, nh) = get_node_geom(node);
                scene_rect.set_center(Pos2::new(nx + nw/2.0, ny + nh/2.0));
            }
        }

        self.data = data;
        self.commonmark_cache = commonmark_cache;
        self.file_cache = file_cache;
        self.scene_rect = scene_rect;
        self.drag_edge_start = drag_edge_start;
        self.selected_node_ids = selected_node_ids;
        self.editing_node_id = editing_node_id;
        self.selection_start = selection_start;

        if moved { self.is_dirty = true; self.save(); }
        self.draw_toolbar(ui, vault);
    }

    fn draw_toolbar(&mut self, ui: &mut Ui, vault: &str) {
        let unique_id = ui.id();
        let transform = ui.ctx().layer_transform_to_global(ui.layer_id()).unwrap_or_default();
        let world_center = transform.inverse() * ui.max_rect().center();

        egui::Area::new(unique_id.with("canvas_toolbar"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
            .show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("📝 Card").clicked() {
                            self.data.nodes.push(CanvasNode::Text {
                                id: format!("{:x}", rand::random::<u64>()),
                                text: "New Card".into(),
                                x: world_center.x - 100.0, y: world_center.y - 50.0,
                                width: 200.0, height: 100.0, color: None,
                            });
                            self.is_dirty = true; self.save();
                        }
                        if ui.button("📄 Markdown").clicked() { self.show_file_picker = Some(PickerType::Markdown); }
                        if ui.button("🖼 Media").clicked() { self.show_file_picker = Some(PickerType::Media); }
                    });
                });
            });

        if let Some(picker) = self.show_file_picker.clone() {
            let mut close = false;
            let mut selected = None;
            egui::Window::new("Select File")
                .id(unique_id.with("file_picker_window"))
                .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    use walkdir::WalkDir;
                    let extensions: &[&str] = match picker {
                        PickerType::Markdown => &[".md"],
                        PickerType::Media => &[".png", ".jpg", ".jpeg", ".svg"],
                    };
                    for entry in WalkDir::new(vault).into_iter().filter_map(|e| e.ok()) {
                        if entry.file_type().is_file() {
                            let path = entry.path();
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
                            let ext_dot = format!(".{}", ext.to_lowercase());
                            if extensions.iter().any(|&e| e == ext_dot) {
                                let rel = path.strip_prefix(vault).unwrap_or(path).to_string_lossy().to_string();
                                if ui.selectable_label(false, &rel).clicked() { selected = Some(rel); close = true; }
                            }
                        }
                    }
                });
                if ui.button("Cancel").clicked() { close = true; }
            });
            if let Some(f) = selected {
                self.data.nodes.push(CanvasNode::File {
                    id: format!("{:x}", rand::random::<u64>()), file: f,
                    x: world_center.x - 150.0, y: world_center.y - 150.0,
                    width: 300.0, height: 300.0, color: None,
                });
                self.is_dirty = true; self.save();
            }
            if close { self.show_file_picker = None; }
        }
    }

    fn draw_nodes_static(
        ui: &mut Ui, vault: &str, data: &mut CanvasData, canvas_path: &str,
        cm_cache: &mut CommonMarkCache, f_cache: &mut HashMap<String, String>,
        curr_drag: &Option<(String, String)>, sel_ids: &HashSet<String>, edit_id: &Option<String>
    ) -> (bool, Option<(String, String)>, Option<String>, Option<String>, Option<Vec2>) {
        let mut moved = false;
        let mut new_drag = None;
        let mut new_sel = None;
        let mut new_edit = edit_id.clone();
        let mut delta = None;

        for node in data.nodes.iter_mut() {
            let nid = get_node_id(node);
            let (nx, ny, nw, nh) = get_node_geom(node);
            let is_sel = sel_ids.contains(&nid);
            let is_edit = edit_id.as_ref() == Some(&nid);
            
            ui.put(Rect::from_min_size(Pos2::new(nx, ny), Vec2::new(nw, nh)), |ui: &mut Ui| {
                let mut frame = egui::Frame::NONE.fill(ui.visuals().window_fill())
                    .stroke(ui.visuals().window_stroke())
                    .corner_radius(ui.visuals().window_corner_radius);
                
                if is_sel { frame = frame.stroke(Stroke::new(2.0, ui.visuals().selection.stroke.color)); }
                else if let Some(c) = get_node_color(node).and_then(|c| parse_color(&c)) { frame = frame.stroke(Stroke::new(2.0, c)); }

                let resp = frame.show(ui, |ui| {
                    ui.set_min_size(Vec2::new(nw, nh));
                    match node {
                        CanvasNode::File { file, .. } => {
                            if let Some(p) = crate::files::resolve_path(vault, canvas_path, file) {
                                if p.ends_with(".md") {
                                    let content = f_cache.entry(p.clone()).or_insert_with(|| fs::read_to_string(&p).unwrap_or_default());
                                    egui::ScrollArea::vertical().id_salt(&nid).show(ui, |ui| { CommonMarkViewer::new().show(ui, cm_cache, content); });
                                } else { ui.add(Image::from_uri(format!("file://{}", p)).shrink_to_fit()); }
                            }
                        }
                        CanvasNode::Text { text, .. } => {
                            if is_edit {
                                let edit_resp = ui.add_sized(ui.available_size(), egui::TextEdit::multiline(text));
                                if edit_resp.changed() { moved = true; }
                                if edit_resp.lost_focus() { new_edit = None; }
                                if !edit_resp.has_focus() { edit_resp.request_focus(); }
                            } else {
                                egui::ScrollArea::vertical().id_salt(&nid).show(ui, |ui| { CommonMarkViewer::new().show(ui, cm_cache, text); });
                            }
                        }
                        CanvasNode::Group { label, .. } => { if let Some(l) = label { ui.heading(&*l); } }
                    }
                }).response;

                if !is_edit {
                    let r = ui.interact(resp.rect, ui.id().with(&nid), Sense::click_and_drag());
                    if r.clicked_by(PointerButton::Primary) || r.drag_started_by(PointerButton::Primary) { new_sel = Some(nid.clone()); }
                    if r.double_clicked() {
                        if let CanvasNode::Text { .. } = node { new_edit = Some(nid.clone()); }
                    }
                    if r.dragged_by(PointerButton::Primary) && curr_drag.is_none() {
                        let d = r.drag_delta();
                        if is_sel { delta = Some(d); }
                        else { move_node(node, d); moved = true; }
                    }
                }
                resp
            });

            // Handles
            let sides = ["top", "bottom", "left", "right"];
            for s in sides {
                let sp = get_side_pos_static(node, s);
                let r = ui.interact(Rect::from_center_size(sp, Vec2::splat(20.0)), ui.id().with(&nid).with(s), Sense::click_and_drag());
                if r.hovered() || r.dragged() {
                    ui.painter().circle_filled(sp, 8.0, ui.visuals().widgets.active.bg_fill);
                } else {
                    ui.painter().circle_filled(sp, 4.0, ui.visuals().widgets.inactive.bg_fill.linear_multiply(0.5));
                }
                if r.drag_started() { new_drag = Some((nid.clone(), s.to_string())); }
            }
        }
        (moved, new_drag, new_sel, new_edit, delta)
    }

    fn draw_edges_static(ui: &mut Ui, data: &CanvasData) {
        for e in &data.edges {
            if let (Some(f), Some(t)) = (find_node_static(data, &e.from_node), find_node_static(data, &e.to_node)) {
                let fp = get_side_pos_static(f, &e.from_side);
                let tp = get_side_pos_static(t, &e.to_side);
                let c = e.color.as_ref().and_then(|c| parse_color(c)).unwrap_or(ui.visuals().widgets.active.fg_stroke.color);
                ui.painter().line_segment([fp, tp], Stroke::new(2.0, c));
                let dir = (tp - fp).normalized();
                let angle = 30.0f32.to_radians();
                ui.painter().line_segment([tp, tp - egui::Vec2::angled(dir.angle() + angle) * 10.0], Stroke::new(2.0, c));
                ui.painter().line_segment([tp, tp - egui::Vec2::angled(dir.angle() - angle) * 10.0], Stroke::new(2.0, c));
            }
        }
    }
}

// Helpers
fn get_node_id(n: &CanvasNode) -> String { match n { CanvasNode::File { id, .. } | CanvasNode::Text { id, .. } | CanvasNode::Group { id, .. } => id.clone() } }
fn get_node_geom(n: &CanvasNode) -> (f32, f32, f32, f32) { match n { CanvasNode::File { x, y, width, height, .. } | CanvasNode::Text { x, y, width, height, .. } | CanvasNode::Group { x, y, width, height, .. } => (*x, *y, *width, *height) } }
fn get_node_info(n: &CanvasNode) -> (String, f32, f32, f32, f32) { match n { CanvasNode::File { id, x, y, width, height, .. } | CanvasNode::Text { id, x, y, width, height, .. } | CanvasNode::Group { id, x, y, width, height, .. } => (id.clone(), *x, *y, *width, *height) } }
fn get_node_color(n: &CanvasNode) -> Option<String> { match n { CanvasNode::File { color, .. } | CanvasNode::Text { color, .. } | CanvasNode::Group { color, .. } => color.clone() } }
fn set_node_color(n: &mut CanvasNode, c: Option<String>) { match n { CanvasNode::File { color, .. } => { *color = c; } CanvasNode::Text { color, .. } => { *color = c; } CanvasNode::Group { color, .. } => { *color = c; } } }
fn move_node(n: &mut CanvasNode, d: Vec2) { match n { CanvasNode::File { x, y, .. } | CanvasNode::Text { x, y, .. } | CanvasNode::Group { x, y, .. } => { *x += d.x; *y += d.y; } } }

fn find_node_static<'a>(data: &'a CanvasData, id: &str) -> Option<&'a CanvasNode> { data.nodes.iter().find(|n| get_node_id(n) == id) }
fn find_node_mut_static<'a>(data: &'a mut CanvasData, id: &str) -> Option<&'a mut CanvasNode> { data.nodes.iter_mut().find(|n| get_node_id(n) == id) }

fn get_side_pos_static(n: &CanvasNode, side: &str) -> Pos2 {
    let (x, y, w, h) = get_node_geom(n);
    match side { "top" => Pos2::new(x + w/2.0, y), "bottom" => Pos2::new(x + w/2.0, y + h), "left" => Pos2::new(x, y + h / 2.0), "right" => Pos2::new(x + w, y + h / 2.0), _ => Pos2::new(x + w / 2.0, y + h / 2.0) }
}

fn get_closest_side(pos: Pos2, rect: Rect) -> String {
    let mut min = pos.distance_sq(Pos2::new(rect.center().x, rect.top())); let mut side = "top";
    let b = pos.distance_sq(Pos2::new(rect.center().x, rect.bottom())); if b < min { min = b; side = "bottom"; }
    let l = pos.distance_sq(Pos2::new(rect.left(), rect.center().y)); if l < min { min = l; side = "left"; }
    let r = pos.distance_sq(Pos2::new(rect.right(), rect.center().y)); if r < min { side = "right"; }
    side.to_string()
}

fn parse_color(color: &str) -> Option<Color32> {
    if color.starts_with('#') && color.len() == 7 {
        if let (Ok(r), Ok(g), Ok(b)) = (u8::from_str_radix(&color[1..3], 16), u8::from_str_radix(&color[3..5], 16), u8::from_str_radix(&color[5..7], 16)) {
            return Some(Color32::from_rgb(r, g, b));
        }
    }
    match color { "1" => Some(Color32::from_rgb(255, 0, 0)), "2" => Some(Color32::from_rgb(0, 255, 0)), "3" => Some(Color32::from_rgb(0, 0, 255)), _ => None }
}