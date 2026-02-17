use crate::egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use egui::{
    Color32, DragPanButtons, Id, Image, PointerButton, Pos2, Rect, Scene, Sense, Stroke,
    StrokeKind, Ui, Vec2,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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

impl CanvasNode {
    #[inline]
    fn id(&self) -> &str {
        match self {
            Self::File { id, .. } | Self::Text { id, .. } | Self::Group { id, .. } => id,
        }
    }

    #[inline]
    fn rect(&self) -> Rect {
        let (x, y, w, h) = match self {
            Self::File {
                x,
                y,
                width,
                height,
                ..
            }
            | Self::Text {
                x,
                y,
                width,
                height,
                ..
            }
            | Self::Group {
                x,
                y,
                width,
                height,
                ..
            } => (*x, *y, *width, *height),
        };
        Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h))
    }

    #[inline]
    fn color(&self) -> Option<&str> {
        match self {
            Self::File { color, .. } | Self::Text { color, .. } | Self::Group { color, .. } => {
                color.as_deref()
            }
        }
    }

    #[inline]
    fn set_color(&mut self, c: Option<String>) {
        match self {
            Self::File { color, .. } | Self::Text { color, .. } | Self::Group { color, .. } => {
                *color = c;
            }
        }
    }

    #[inline]
    fn translate(&mut self, delta: Vec2) {
        match self {
            Self::File { x, y, .. } | Self::Text { x, y, .. } | Self::Group { x, y, .. } => {
                *x += delta.x;
                *y += delta.y;
            }
        }
    }

    #[inline]
    fn side_pos(&self, side: &str) -> Pos2 {
        let rect = self.rect();
        match side {
            "top" => Pos2::new(rect.center().x, rect.top()),
            "bottom" => Pos2::new(rect.center().x, rect.bottom()),
            "left" => Pos2::new(rect.left(), rect.center().y),
            "right" => Pos2::new(rect.right(), rect.center().y),
            _ => rect.center(),
        }
    }
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

impl CanvasData {
    fn find_node(&self, id: &str) -> Option<&CanvasNode> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    fn find_node_mut(&mut self, id: &str) -> Option<&mut CanvasNode> {
        self.nodes.iter_mut().find(|n| n.id() == id)
    }
}

#[derive(PartialEq, Clone)]
pub enum PickerType {
    Markdown,
    Media,
}

// Cache para colores parseados
struct ColorCache {
    cache: HashMap<String, Color32>,
}

impl ColorCache {
    fn new() -> Self {
        let mut cache = HashMap::new();
        cache.insert("1".to_string(), Color32::from_rgb(255, 0, 0));
        cache.insert("2".to_string(), Color32::from_rgb(0, 255, 0));
        cache.insert("3".to_string(), Color32::from_rgb(0, 0, 255));
        Self { cache }
    }

    fn get(&mut self, color: &str) -> Option<Color32> {
        if let Some(&c) = self.cache.get(color) {
            return Some(c);
        }

        if color.starts_with('#') && color.len() == 7 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&color[1..3], 16),
                u8::from_str_radix(&color[3..5], 16),
                u8::from_str_radix(&color[5..7], 16),
            ) {
                let parsed = Color32::from_rgb(r, g, b);
                self.cache.insert(color.to_string(), parsed);
                return Some(parsed);
            }
        }
        None
    }
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
    pub selection_start: Option<Pos2>,

    // Caches adicionales
    color_cache: ColorCache,
    node_index: HashMap<String, usize>, // Para búsquedas O(1)
    visible_nodes: Vec<usize>,          // Nodos visibles en el viewport
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
            color_cache: ColorCache::new(),
            node_index: HashMap::new(),
            visible_nodes: Vec::new(),
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
                self.rebuild_node_index();
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

    fn rebuild_node_index(&mut self) {
        self.node_index.clear();
        for (idx, node) in self.data.nodes.iter().enumerate() {
            self.node_index.insert(node.id().to_string(), idx);
        }
    }

    // Culling: solo procesar nodos visibles
    fn update_visible_nodes(&mut self, viewport: Rect) {
        self.visible_nodes.clear();
        let expanded = viewport.expand(100.0); // Margen para suavizar

        for (idx, node) in self.data.nodes.iter().enumerate() {
            if node.rect().intersects(expanded) {
                self.visible_nodes.push(idx);
            }
        }
    }

    pub fn show(&mut self, ui: &mut Ui, vault: &str) {
        let unique_id = ui.id();
        let scene = Scene::new()
            .zoom_range(0.01..=10.0)
            .drag_pan_buttons(DragPanButtons::MIDDLE);

        let mut scene_rect = self.scene_rect;
        let mut moved = false;
        let mut node_to_delete = false;
        let mut zoom_to_node = None;

        ui.push_id(unique_id.with("canvas_scene_scope"), |ui| {
            scene.show(ui, &mut scene_rect, |ui| {
                // Actualizar nodos visibles basado en viewport
                self.update_visible_nodes(ui.max_rect());

                let pointer_pos = ui.input(|i| i.pointer.interact_pos());

                // 1. BACKGROUND
                let bg_resp =
                    ui.interact(ui.max_rect(), ui.id().with("bg"), Sense::click_and_drag());

                if bg_resp.drag_started_by(PointerButton::Primary) {
                    self.selection_start = pointer_pos;
                    if !ui.input(|i| i.modifiers.shift) {
                        self.selected_node_ids.clear();
                    }
                    self.editing_node_id = None;
                }

                if bg_resp.clicked_by(PointerButton::Primary) {
                    self.selected_node_ids.clear();
                    self.editing_node_id = None;
                }

                // Marquee selection
                let mut marquee_rect = None;
                if let Some(start) = self.selection_start {
                    if let Some(current) = pointer_pos {
                        let rect = Rect::from_two_pos(start, current);
                        marquee_rect = Some(rect);

                        if !ui.input(|i| i.modifiers.shift) {
                            self.selected_node_ids.clear();
                        }

                        // Solo verificar nodos visibles
                        for &idx in &self.visible_nodes {
                            let node = &self.data.nodes[idx];
                            if rect.intersects(node.rect()) {
                                self.selected_node_ids.insert(node.id().to_string());
                            }
                        }
                    }

                    if ui.input(|i| i.pointer.any_released()) {
                        self.selection_start = None;
                    }
                }

                // 2. EDGES (solo las conectadas a nodos visibles)
                self.draw_edges_optimized(ui);

                // 3. NODES (solo los visibles)
                let interactions = self.draw_nodes_optimized(ui, vault);

                if interactions.moved {
                    moved = true;
                }
                if interactions.new_drag.is_some() {
                    self.drag_edge_start = interactions.new_drag;
                }

                if let Some(new_sel_id) = interactions.new_selection {
                    if ui.input(|i| i.modifiers.shift) {
                        if self.selected_node_ids.contains(&new_sel_id) {
                            self.selected_node_ids.remove(&new_sel_id);
                        } else {
                            self.selected_node_ids.insert(new_sel_id);
                        }
                    } else {
                        if !self.selected_node_ids.contains(&new_sel_id)
                            || self.selected_node_ids.len() > 1
                        {
                            self.selected_node_ids.clear();
                            self.selected_node_ids.insert(new_sel_id);
                        }
                    }
                    self.editing_node_id = None;
                }

                // Bulk move
                if let Some(delta) = interactions.drag_delta {
                    for id in &self.selected_node_ids {
                        if let Some(node) = self.data.find_node_mut(id) {
                            node.translate(delta);
                            moved = true;
                        }
                    }
                }

                if interactions.new_editing.is_some() {
                    self.editing_node_id = interactions.new_editing;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.editing_node_id = None;
                }

                // 4. CONTEXT MENU
                if let Some(id) = self.selected_node_ids.iter().next().cloned() {
                    let result = self.draw_context_menu(ui, &id, vault);
                    if result.delete {
                        node_to_delete = true;
                    }
                    if result.zoom {
                        zoom_to_node = Some(id.clone());
                    }
                    if result.moved {
                        moved = true;
                    }
                }

                // 5. EDGE PREVIEW
                if let Some((sid, s_side)) = &self.drag_edge_start {
                    if let Some(sn) = self.data.find_node(sid) {
                        let sp = sn.side_pos(s_side);
                        if let Some(mwp) = ui.input(|i| i.pointer.interact_pos()) {
                            let (ep, target) = self.find_edge_target(mwp, sid);

                            ui.painter().line_segment(
                                [sp, ep],
                                Stroke::new(2.0, ui.visuals().widgets.active.fg_stroke.color),
                            );

                            if ui.input(|i| i.pointer.any_released()) {
                                if let Some((to_id, to_s)) = target {
                                    self.data.edges.push(CanvasEdge {
                                        id: format!("{:x}", rand::random::<u64>()),
                                        from_node: sid.clone(),
                                        from_side: s_side.clone(),
                                        to_node: to_id,
                                        to_side: to_s,
                                        label: None,
                                        color: None,
                                    });
                                    moved = true;
                                }
                                self.drag_edge_start = None;
                            }
                        }
                    }
                }

                // 6. DRAW MARQUEE
                if let Some(rect) = marquee_rect {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0, Color32::from_rgb(100, 150, 255)),
                        StrokeKind::Outside,
                    );
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(100, 150, 255, 30),
                    );
                }
            });
        });

        // Cleanup
        if node_to_delete {
            self.data
                .nodes
                .retain(|n| !self.selected_node_ids.contains(n.id()));
            self.data.edges.retain(|e| {
                !self.selected_node_ids.contains(&e.from_node)
                    && !self.selected_node_ids.contains(&e.to_node)
            });
            self.selected_node_ids.clear();
            self.rebuild_node_index();
            moved = true;
        }

        if let Some(id) = zoom_to_node {
            if let Some(node) = self.data.find_node(&id) {
                scene_rect.set_center(node.rect().center());
            }
        }

        self.scene_rect = scene_rect;

        if moved {
            self.is_dirty = true;
            self.save();
        }

        self.draw_toolbar(ui, vault);
    }

    fn draw_edges_optimized(&mut self, ui: &mut Ui) {
        let visible_set: HashSet<&str> = self
            .visible_nodes
            .iter()
            .map(|&idx| self.data.nodes[idx].id())
            .collect();

        for edge in &self.data.edges {
            // Solo dibujar si ambos nodos son visibles
            if !visible_set.contains(edge.from_node.as_str())
                && !visible_set.contains(edge.to_node.as_str())
            {
                continue;
            }

            if let (Some(from), Some(to)) = (
                self.data.find_node(&edge.from_node),
                self.data.find_node(&edge.to_node),
            ) {
                let fp = from.side_pos(&edge.from_side);
                let tp = to.side_pos(&edge.to_side);

                let color = edge
                    .color
                    .as_ref()
                    .and_then(|c| self.color_cache.get(c))
                    .unwrap_or(ui.visuals().widgets.active.fg_stroke.color);

                ui.painter().line_segment([fp, tp], Stroke::new(2.0, color));

                // Flecha
                let dir = (tp - fp).normalized();
                let angle = 30.0f32.to_radians();
                let arrow_len = 10.0;

                ui.painter().line_segment(
                    [tp, tp - Vec2::angled(dir.angle() + angle) * arrow_len],
                    Stroke::new(2.0, color),
                );
                ui.painter().line_segment(
                    [tp, tp - Vec2::angled(dir.angle() - angle) * arrow_len],
                    Stroke::new(2.0, color),
                );
            }
        }
    }

    fn draw_nodes_optimized(&mut self, ui: &mut Ui, vault: &str) -> NodeInteractions {
        let mut result = NodeInteractions::default();

        let is_editing = self.editing_node_id.is_some();
        let curr_drag = self.drag_edge_start.is_some();

        // Clonamos para evitar borrow conflict con self
        let visible_nodes = self.visible_nodes.clone();

        // Solo procesar nodos visibles
        for &idx in &visible_nodes {
            let node = &mut self.data.nodes[idx];
            let nid = node.id().to_string();
            let rect = node.rect();
            let is_selected = self.selected_node_ids.contains(&nid);
            let is_edit = self.editing_node_id.as_ref() == Some(&nid);

            let path = &self.path;
            let file_cache = &mut self.file_cache;
            let commonmark_cache = &mut self.commonmark_cache;
            let color_cache = &mut self.color_cache;
            let result_ptr = &mut result as *mut NodeInteractions;

            ui.put(rect, |ui: &mut Ui| {
                let mut frame = egui::Frame::NONE
                    .fill(ui.visuals().window_fill())
                    .stroke(ui.visuals().window_stroke())
                    .corner_radius(ui.visuals().window_corner_radius);

                if is_selected {
                    frame = frame.stroke(Stroke::new(2.0, ui.visuals().selection.stroke.color));
                } else if let Some(c) = node.color().and_then(|c| color_cache.get(c)) {
                    frame = frame.stroke(Stroke::new(2.0, c));
                }

                // SAFETY: ui.put ejecuta el closure inmediatamente, y no guardamos result_ptr.
                // Sin embargo, para evitar unsafe, tratamos de capturar result directamente si es posible.
                // Pero como estamos en un loop y el closure es FnOnce, capturar result directamente
                // lo movería. Así que usamos una referencia mutable.
                let result_ref = unsafe { &mut *result_ptr };

                let resp = frame
                    .show(ui, |ui| {
                        ui.set_min_size(rect.size());
                        Self::draw_node_content(
                            path,
                            file_cache,
                            commonmark_cache,
                            ui,
                            node,
                            vault,
                            &nid,
                            is_edit,
                            result_ref,
                        );
                    })
                    .response;

                if !is_edit {
                    let interact =
                        ui.interact(resp.rect, ui.id().with(&nid), Sense::click_and_drag());

                    if interact.clicked_by(PointerButton::Primary)
                        || interact.drag_started_by(PointerButton::Primary)
                    {
                        result_ref.new_selection = Some(nid.clone());
                    }

                    if interact.double_clicked() {
                        if matches!(node, CanvasNode::Text { .. }) {
                            result_ref.new_editing = Some(nid.clone());
                        }
                    }

                    if interact.dragged_by(PointerButton::Primary) && !curr_drag {
                        let delta = interact.drag_delta();
                        if is_selected {
                            result_ref.drag_delta = Some(delta);
                        } else {
                            node.translate(delta);
                            result_ref.moved = true;
                        }
                    }
                }

                resp
            });

            // Handles (solo si no está editando)
            if !is_editing {
                for side in ["top", "bottom", "left", "right"] {
                    let sp = node.side_pos(side);
                    let handle_id = ui.id().with(&nid).with(side);
                    let handle_rect = Rect::from_center_size(sp, Vec2::splat(20.0));
                    let handle = ui.interact(handle_rect, handle_id, Sense::click_and_drag());

                    if handle.hovered() || handle.dragged() {
                        ui.painter()
                            .circle_filled(sp, 8.0, ui.visuals().widgets.active.bg_fill);
                    } else {
                        ui.painter().circle_filled(
                            sp,
                            4.0,
                            ui.visuals().widgets.inactive.bg_fill.linear_multiply(0.5),
                        );
                    }

                    if handle.drag_started() {
                        result.new_drag = Some((nid.clone(), side.to_string()));
                    }
                }
            }
        }

        result
    }

    fn draw_node_content(
        path: &str,
        file_cache: &mut HashMap<String, String>,
        commonmark_cache: &mut CommonMarkCache,
        ui: &mut Ui,
        node: &mut CanvasNode,
        vault: &str,
        nid: &str,
        is_edit: bool,
        result: &mut NodeInteractions,
    ) {
        match node {
            CanvasNode::File { file, .. } => {
                if let Some(p) = crate::files::resolve_path(vault, path, file) {
                    if p.ends_with(".md") {
                        let content = file_cache
                            .entry(p.clone())
                            .or_insert_with(|| fs::read_to_string(&p).unwrap_or_default());

                        egui::ScrollArea::vertical().id_salt(nid).show(ui, |ui| {
                            CommonMarkViewer::new().show(ui, commonmark_cache, content);
                        });
                    } else {
                        ui.add(Image::from_uri(format!("file://{}", p)).shrink_to_fit());
                    }
                }
            }
            CanvasNode::Text { text, .. } => {
                if is_edit {
                    let edit_resp =
                        ui.add_sized(ui.available_size(), egui::TextEdit::multiline(text));
                    if edit_resp.changed() {
                        result.moved = true;
                    }
                    if edit_resp.lost_focus() {
                        result.new_editing = None;
                    }
                    if !edit_resp.has_focus() {
                        edit_resp.request_focus();
                    }
                } else {
                    egui::ScrollArea::vertical().id_salt(nid).show(ui, |ui| {
                        CommonMarkViewer::new().show(ui, commonmark_cache, text);
                    });
                }
            }
            CanvasNode::Group { label, .. } => {
                if let Some(l) = label {
                    ui.heading(l);
                }
            }
        }
    }

    fn draw_context_menu(
        &mut self,
        ui: &mut Ui,
        node_id: &str,
        vault: &str,
    ) -> ContextMenuResult {
        let mut result = ContextMenuResult::default();
        let (rect, is_text, current_color) = if let Some(node) = self.data.find_node(node_id) {
            (
                node.rect(),
                matches!(node, CanvasNode::Text { .. }),
                node.color().map(|s| s.to_string()),
            )
        } else {
            return result;
        };
        let menu_pos = Pos2::new(rect.center().x, rect.top() - 35.0);

        ui.put(
            Rect::from_center_size(menu_pos, Vec2::new(250.0, 40.0)),
            |ui: &mut Ui| {
                ui.horizontal(|ui| {
                    if ui.button("🗑").clicked() {
                        result.delete = true;
                    }

                    for c in &["1", "2", "3"] {
                        let color_val = self.color_cache.get(c).unwrap_or(Color32::WHITE);
                        if ui
                            .button(egui::RichText::new("■").color(color_val))
                            .clicked()
                        {
                            for sid in &self.selected_node_ids {
                                if let Some(n) = self.data.find_node_mut(sid) {
                                    n.set_color(Some(c.to_string()));
                                }
                            }
                            result.moved = true;
                        }
                    }

                    let mut color_rgba = current_color
                        .as_ref()
                        .and_then(|cs| self.color_cache.get(cs))
                        .unwrap_or(Color32::WHITE);

                    if ui.color_edit_button_srgba(&mut color_rgba).changed() {
                        let hex = format!(
                            "#{:02x}{:02x}{:02x}",
                            color_rgba.r(),
                            color_rgba.g(),
                            color_rgba.b()
                        );
                        for sid in &self.selected_node_ids {
                            if let Some(n) = self.data.find_node_mut(sid) {
                                n.set_color(Some(hex.clone()));
                            }
                        }
                        result.moved = true;
                    }

                    if ui.button("🔍").clicked() {
                        result.zoom = true;
                    }

                    if ui.button("📝").clicked() {
                        if is_text {
                            result.edit = true;
                        } else {
                            // Re-obtener el nodo para evitar problemas de borrow
                            if let Some(CanvasNode::File { file, .. }) = self.data.find_node(node_id)
                            {
                                if let Some(p) = crate::files::resolve_path(vault, &self.path, file)
                                {
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(Id::new("global_nav_request"), Some(p));
                                    });
                                }
                            }
                        }
                    }
                })
                .response
            },
        );

        result
    }

    fn find_edge_target(&self, pos: Pos2, exclude_id: &str) -> (Pos2, Option<(String, String)>) {
        for &idx in &self.visible_nodes {
            let node = &self.data.nodes[idx];
            if node.id() == exclude_id {
                continue;
            }

            let rect = node.rect();
            if rect.expand(25.0).contains(pos) {
                let side = get_closest_side(pos, rect);
                let ep = node.side_pos(&side);
                return (ep, Some((node.id().to_string(), side)));
            }
        }
        (pos, None)
    }

    fn draw_toolbar(&mut self, ui: &mut Ui, vault: &str) {
        let unique_id = ui.id();
        let transform = ui
            .ctx()
            .layer_transform_to_global(ui.layer_id())
            .unwrap_or_default();
        let world_center = transform.inverse() * ui.max_rect().center();

        egui::Area::new(unique_id.with("canvas_toolbar"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
            .show(ui.ctx(), |ui| {
                egui::Frame::window(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("📝 Card").clicked() {
                            let new_node = CanvasNode::Text {
                                id: format!("{:x}", rand::random::<u64>()),
                                text: "New Card".into(),
                                x: world_center.x - 100.0,
                                y: world_center.y - 50.0,
                                width: 200.0,
                                height: 100.0,
                                color: None,
                            };
                            self.data.nodes.push(new_node);
                            self.rebuild_node_index();
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

        if let Some(picker) = self.show_file_picker.clone() {
            self.show_file_picker_window(ui, vault, &picker, world_center);
        }
    }

    fn show_file_picker_window(
        &mut self,
        ui: &mut Ui,
        vault: &str,
        picker: &PickerType,
        world_center: Pos2,
    ) {
        let unique_id = ui.id();
        let mut close = false;
        let mut selected = None;

        egui::Window::new("Select File")
            .id(unique_id.with("file_picker_window"))
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
                            let ext = path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or_default();
                            let ext_dot = format!(".{}", ext.to_lowercase());

                            if extensions.iter().any(|&e| e == ext_dot) {
                                let rel = path
                                    .strip_prefix(vault)
                                    .unwrap_or(path)
                                    .to_string_lossy()
                                    .to_string();

                                if ui.selectable_label(false, &rel).clicked() {
                                    selected = Some(rel);
                                    close = true;
                                }
                            }
                        }
                    }
                });

                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });

        if let Some(file) = selected {
            let new_node = CanvasNode::File {
                id: format!("{:x}", rand::random::<u64>()),
                file,
                x: world_center.x - 150.0,
                y: world_center.y - 150.0,
                width: 300.0,
                height: 300.0,
                color: None,
            };
            self.data.nodes.push(new_node);
            self.rebuild_node_index();
            self.is_dirty = true;
            self.save();
        }

        if close {
            self.show_file_picker = None;
        }
    }
}

// Structs para retornar múltiples valores
#[derive(Default)]
struct NodeInteractions {
    moved: bool,
    new_drag: Option<(String, String)>,
    new_selection: Option<String>,
    new_editing: Option<String>,
    drag_delta: Option<Vec2>,
}

#[derive(Default)]
struct ContextMenuResult {
    delete: bool,
    zoom: bool,
    edit: bool,
    moved: bool,
}

// Helper functions
fn get_closest_side(pos: Pos2, rect: Rect) -> String {
    let distances = [
        (
            pos.distance_sq(Pos2::new(rect.center().x, rect.top())),
            "top",
        ),
        (
            pos.distance_sq(Pos2::new(rect.center().x, rect.bottom())),
            "bottom",
        ),
        (
            pos.distance_sq(Pos2::new(rect.left(), rect.center().y)),
            "left",
        ),
        (
            pos.distance_sq(Pos2::new(rect.right(), rect.center().y)),
            "right",
        ),
    ];

    distances
        .iter()
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, side)| side.to_string())
        .unwrap_or_else(|| "top".to_string())
}
