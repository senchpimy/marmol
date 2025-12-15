use crate::files;
use crate::main_area;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use yaml_rust::YamlLoader;

use egui::*;
use egui_plot::{Arrows, Line, MarkerShape, Plot, PlotPoint, Points, Text};

#[derive(Clone)]
struct MarmolPoint {
    text: String,
    tags: Vec<String>,
    links: Vec<String>,
    rel_path: String,
    abs_path: String,
    is_attachment: bool,
    is_tag: bool, // NUEVO CAMPO
    exists: bool,
}

#[derive(Clone, PartialEq)]
enum MatchType {
    Filename,
    Tag,
    Path,
    Content,
    Section,
}

#[derive(Clone)]
struct CustomGroup {
    match_type: MatchType,
    value: String,
    color: Color32,
}

pub struct Graph {
    points: Vec<MarmolPoint>,
    points_coord: Vec<(f32, f32)>,
    velocities: Vec<Vec2>,
    edges: Vec<(usize, usize)>,
    node_degrees: Vec<usize>,

    // Fuerzas
    center_force: f32,
    repel_force: f32,
    link_force: f32,
    group_force: f32,
    tag_force: f32, // NUEVO CAMPO: Fuerza de atracción extra para tags

    // Filtros de Texto
    filter_filename: String,
    filter_tag: String,
    filter_path: String,
    filter_line: String,
    filter_section: String,

    // Toggles
    show_attachments: bool,
    show_existing_only: bool,
    show_orphans: bool,
    show_tags: bool, // NUEVO TOGGLE

    // Visualización
    show_arrows: bool,
    text_zoom_threshold: f64,
    node_size: f32,
    line_thickness: f32,

    dragged_node_index: Option<usize>,
    orphan_color: Color32,
    ghost_color: Color32,
    attachment_color: Color32,
    palette: Vec<Color32>,
    tags_colors: HashMap<String, Color32>,

    custom_groups: Vec<CustomGroup>,
    new_group_type: MatchType,
    new_group_val: String,
    new_group_col: Color32,
}

impl MarmolPoint {
    fn new(
        val: &str,
        tags: Vec<String>,
        links: Vec<String>,
        rel_path: String,
        abs_path: String,
        is_attachment: bool,
        is_tag: bool, // NUEVO ARGUMENTO
        exists: bool,
    ) -> Self {
        Self {
            text: format!("{}", val),
            tags,
            links,
            rel_path,
            abs_path,
            is_attachment,
            is_tag,
            exists,
        }
    }
}

impl Graph {
    pub fn new(vault: &str) -> Self {
        let mut tags_colors = HashMap::new();
        if let Ok(file_content) = fs::read_to_string("./test.json") {
            if let Ok(parsed) = json::parse(&file_content) {
                if let json::JsonValue::Array(color_groups) = &parsed["colorGroups"] {
                    for group in color_groups {
                        if let (Some(tag), rgb_array) =
                            (group["tag"].as_str(), group["rgb"].members())
                        {
                            let rgb: Vec<u8> = rgb_array.map(|j| j.as_u8().unwrap_or(0)).collect();
                            if rgb.len() == 3 {
                                tags_colors.insert(
                                    tag.to_string(),
                                    Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                                );
                            }
                        }
                    }
                }
            }
        }

        let palette = vec![
            Color32::from_rgb(235, 64, 52),
            Color32::from_rgb(60, 100, 220),
            Color32::from_rgb(140, 40, 160),
            Color32::from_rgb(255, 140, 0),
            Color32::from_rgb(46, 204, 113),
            Color32::from_rgb(52, 152, 219),
            Color32::from_rgb(241, 196, 15),
            Color32::from_rgb(231, 76, 60),
            Color32::from_rgb(52, 73, 94),
        ];

        let mut graph = Self {
            points: vec![],
            points_coord: vec![],
            velocities: vec![],
            edges: vec![],
            node_degrees: vec![],
            center_force: 0.15,
            repel_force: 30.0,
            link_force: 0.8,
            group_force: 1.5,
            tag_force: 3.0, // Fuerza extra por defecto

            filter_filename: String::new(),
            filter_tag: String::new(),
            filter_path: String::new(),
            filter_line: String::new(),
            filter_section: String::new(),

            show_attachments: true,
            show_existing_only: false,
            show_orphans: true,
            show_tags: false, // Apagado por defecto

            show_arrows: false,
            text_zoom_threshold: 500.0,
            node_size: 7.0,
            line_thickness: 2.0,

            dragged_node_index: None,
            orphan_color: Color32::from_rgb(100, 110, 120),
            ghost_color: Color32::from_rgb(60, 60, 60),
            attachment_color: Color32::from_rgb(100, 200, 100),
            palette,
            tags_colors,

            custom_groups: vec![],
            new_group_type: MatchType::Tag,
            new_group_val: String::new(),
            new_group_col: Color32::from_rgb(255, 0, 0),
        };

        graph.update_vault(Path::new(vault));
        graph
    }

    fn check_match(&self, m_type: &MatchType, val: &str, point: &MarmolPoint) -> bool {
        if val.is_empty() {
            return false;
        }
        let search = val.to_lowercase();

        match m_type {
            MatchType::Tag => point
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&search)),
            MatchType::Filename => point.text.to_lowercase().contains(&search),
            MatchType::Path => point.rel_path.to_lowercase().contains(&search),
            MatchType::Content => {
                if point.is_attachment || point.is_tag {
                    return false;
                }
                if let Ok(c) = fs::read_to_string(&point.abs_path) {
                    c.to_lowercase().contains(&search)
                } else {
                    false
                }
            }
            MatchType::Section => {
                if point.is_attachment || point.is_tag {
                    return false;
                }
                if let Ok(c) = fs::read_to_string(&point.abs_path) {
                    c.lines()
                        .any(|l| l.trim().starts_with('#') && l.to_lowercase().contains(&search))
                } else {
                    false
                }
            }
        }
    }

    pub fn update_vault(&mut self, vault: &Path) {
        let mut new_points = vec![];
        let mut elements = 0;

        // 1. Obtener Archivos
        get_data(
            vault,
            &mut new_points,
            &mut elements,
            vault.to_str().unwrap(),
        );

        // 2. Generar Nodos Fantasma (Links rotos)
        let mut existing_names: HashSet<String> = HashSet::new();
        for p in &new_points {
            existing_names.insert(p.text.to_lowercase());
        }

        let mut ghost_points = vec![];
        for p in &new_points {
            for link in &p.links {
                let link_lower = link.to_lowercase();
                if !existing_names.contains(&link_lower) {
                    let already_added = ghost_points
                        .iter()
                        .any(|gp: &MarmolPoint| gp.text.to_lowercase() == link_lower);
                    if !already_added {
                        ghost_points.push(MarmolPoint::new(
                            link,
                            vec![],
                            vec![],
                            "".to_string(),
                            "".to_string(),
                            false,
                            false, // is_tag
                            false, // exists
                        ));
                    }
                }
            }
        }

        // Eliminar duplicados de fantasmas
        let mut seen_ghosts = HashSet::new();
        for g in ghost_points {
            if seen_ghosts.insert(g.text.to_lowercase()) {
                new_points.push(g);
            }
        }

        // 3. Generar Nodos de TAGS (Si el toggle está activo)
        if self.show_tags {
            let mut tag_map: HashMap<String, usize> = HashMap::new();

            // Primero identificamos todos los tags únicos y creamos sus nodos
            // Nota: Iteramos sobre una copia temporal de los tags para no borrow new_points
            let mut all_tags = HashSet::new();
            for p in &new_points {
                if p.is_tag {
                    continue;
                }
                for tag in &p.tags {
                    if tag != "Orphan" {
                        all_tags.insert(tag.clone());
                    }
                }
            }

            // Crear puntos para los tags
            let start_idx = new_points.len();
            for (i, tag) in all_tags.iter().enumerate() {
                let tag_point = MarmolPoint::new(
                    tag,
                    vec![],
                    vec![],
                    "".to_string(),
                    "".to_string(),
                    false, // attachment
                    true,  // is_tag
                    true,  // exists
                );
                new_points.push(tag_point);
                tag_map.insert(tag.clone(), start_idx + i);
            }

            // Calcular edges para tags (se hace en build_edges, pero necesitamos pasar info o hacerlo aqui)
            // Dado que build_edges usa links, y los tags no son links textuales,
            // necesitamos inyectar estas conexiones manualmente después.
            // Para simplificar, añadimos los indices de los tags a una lista auxiliar en Graph,
            // pero mejor modificamos build_edges para soportar tags o lo hacemos aquí.

            // Vamos a hacerlo en build_edges modificado abajo.
        }

        let total_count = new_points.len();

        let mut new_coords = vec![];
        get_coords(&mut new_coords, total_count as i32);

        // Construir Edges (Links + Conexiones a Tags)
        self.edges = build_edges(&new_points, self.show_tags);

        self.points = new_points;
        self.points_coord = new_coords;
        self.velocities = vec![Vec2::ZERO; self.points_coord.len()];

        self.node_degrees = vec![0; total_count];
        for &(a, b) in &self.edges {
            if a < total_count {
                self.node_degrees[a] += 1;
            }
            if b < total_count {
                self.node_degrees[b] += 1;
            }
        }

        self.dragged_node_index = None;
    }

    fn is_visible(&self, index: usize) -> bool {
        let p = &self.points[index];

        if !self.show_attachments && p.is_attachment {
            return false;
        }
        if self.show_existing_only && !p.exists {
            return false;
        }
        if !self.show_orphans && self.node_degrees[index] == 0 {
            return false;
        }
        // Si el toggle de tags está apagado, los puntos is_tag no deberían existir en el array,
        // pero por seguridad filtramos.
        if !self.show_tags && p.is_tag {
            return false;
        }

        if !self.filter_filename.is_empty() {
            if !p
                .text
                .to_lowercase()
                .contains(&self.filter_filename.to_lowercase())
            {
                return false;
            }
        }
        if !self.filter_tag.is_empty() {
            let search = self.filter_tag.to_lowercase();
            // Si es un nodo TAG, filtramos por su propio nombre
            if p.is_tag {
                if !p.text.to_lowercase().contains(&search) {
                    return false;
                }
            } else {
                if !p.tags.iter().any(|t| t.to_lowercase().contains(&search)) {
                    return false;
                }
            }
        }
        if !self.filter_path.is_empty() {
            if !p
                .rel_path
                .to_lowercase()
                .contains(&self.filter_path.to_lowercase())
            {
                return false;
            }
        }

        // Filtros de contenido (excluyen tags y ghosts)
        if !self.filter_line.is_empty() {
            if !p.exists || p.is_attachment || p.is_tag {
                return false;
            }
            if let Ok(content) = fs::read_to_string(&p.abs_path) {
                if !content
                    .to_lowercase()
                    .contains(&self.filter_line.to_lowercase())
                {
                    return false;
                }
            } else {
                return false;
            }
        }
        if !self.filter_section.is_empty() {
            if !p.exists || p.is_attachment || p.is_tag {
                return false;
            }
            if let Ok(content) = fs::read_to_string(&p.abs_path) {
                let search = self.filter_section.to_lowercase();
                let has_header = content.lines().any(|line| {
                    let l = line.trim();
                    l.starts_with('#') && l.to_lowercase().contains(&search)
                });
                if !has_header {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn controls(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.collapsing("Configuración Física", |ui| {
            ui.add(egui::Slider::new(&mut self.repel_force, 1.0..=100.0).text("Repulsión"));
            ui.add(egui::Slider::new(&mut self.link_force, 0.1..=3.0).text("Links"));
            ui.add(egui::Slider::new(&mut self.group_force, 0.0..=5.0).text("Agrupación"));
            ui.add(egui::Slider::new(&mut self.center_force, 0.01..=1.0).text("Gravedad"));
            ui.add(egui::Slider::new(&mut self.tag_force, 0.1..=10.0).text("Atracción Tags"));

            ui.horizontal(|ui| {
                color_picker::color_edit_button_srgba(
                    ui,
                    &mut self.orphan_color,
                    egui::widgets::color_picker::Alpha::Opaque,
                );
                ui.label("Color Huérfanos");
            });
        });

        ui.collapsing("Visualización", |ui| {
            ui.checkbox(&mut self.show_arrows, "Mostrar Flechas (Dirección)");

            ui.add(egui::Slider::new(&mut self.node_size, 2.0..=20.0).text("Tamaño Nodo"));
            ui.add(egui::Slider::new(&mut self.line_thickness, 0.5..=10.0).text("Grosor Línea"));

            ui.label("Visibilidad de Texto (Zoom):");
            ui.add(
                egui::Slider::new(&mut self.text_zoom_threshold, 10.0..=2000.0)
                    .text("Umbral")
                    .logarithmic(true),
            );
        });

        ui.collapsing("Crear Grupos (Colores)", |ui| {
            ui.label("Define reglas para colorear nodos.");

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("group_type_combo")
                    .selected_text(match self.new_group_type {
                        MatchType::Tag => "Tag",
                        MatchType::Filename => "Filename",
                        MatchType::Path => "Path",
                        MatchType::Content => "Content",
                        MatchType::Section => "Section",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_group_type, MatchType::Tag, "Tag");
                        ui.selectable_value(
                            &mut self.new_group_type,
                            MatchType::Filename,
                            "Filename",
                        );
                        ui.selectable_value(&mut self.new_group_type, MatchType::Path, "Path");
                        ui.selectable_value(
                            &mut self.new_group_type,
                            MatchType::Content,
                            "Content",
                        );
                        ui.selectable_value(
                            &mut self.new_group_type,
                            MatchType::Section,
                            "Section",
                        );
                    });

                color_picker::color_edit_button_srgba(
                    ui,
                    &mut self.new_group_col,
                    egui::widgets::color_picker::Alpha::Opaque,
                );
            });

            ui.text_edit_singleline(&mut self.new_group_val)
                .on_hover_text("Valor a buscar");

            if ui.button("Agregar Grupo").clicked() {
                if !self.new_group_val.is_empty() {
                    self.custom_groups.push(CustomGroup {
                        match_type: self.new_group_type.clone(),
                        value: self.new_group_val.clone(),
                        color: self.new_group_col,
                    });
                    self.new_group_val.clear();
                }
            }

            ui.separator();
            ui.label("Grupos Activos:");
            let mut index_to_remove = None;
            for (i, group) in self.custom_groups.iter().enumerate() {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::hover());
                    ui.painter().rect_filled(rect, 2.0, group.color);
                    let type_str = match group.match_type {
                        MatchType::Tag => "Tag",
                        MatchType::Filename => "File",
                        MatchType::Path => "Path",
                        MatchType::Content => "Cont",
                        MatchType::Section => "Sect",
                    };
                    ui.label(format!("{}: '{}'", type_str, group.value));
                    if ui.button("x").clicked() {
                        index_to_remove = Some(i);
                    }
                });
            }
            if let Some(i) = index_to_remove {
                self.custom_groups.remove(i);
            }
        });

        ui.collapsing("Filtros", |ui| {
            ui.label("Filename (Nombre):");
            ui.text_edit_singleline(&mut self.filter_filename);
            ui.label("Tag (Etiqueta):");
            ui.text_edit_singleline(&mut self.filter_tag);
            ui.label("Path (Carpeta):");
            ui.text_edit_singleline(&mut self.filter_path);
            ui.label("Line (Contenido):");
            ui.text_edit_singleline(&mut self.filter_line);
            ui.label("Section (Heading #):");
            ui.text_edit_singleline(&mut self.filter_section);

            ui.separator();
            ui.checkbox(&mut self.show_attachments, "Mostrar Adjuntos");
            ui.checkbox(&mut self.show_existing_only, "Ocultar Nodos Fantasma");
            ui.checkbox(&mut self.show_orphans, "Mostrar Huérfanos");

            if ui
                .checkbox(&mut self.show_tags, "Mostrar Nodos de Tags")
                .changed()
            {
                changed = true;
            }

            if ui.button("Limpiar Todo").clicked() {
                self.filter_filename.clear();
                self.filter_tag.clear();
                self.filter_path.clear();
                self.filter_line.clear();
                self.filter_section.clear();
            }
        });

        changed
    }

    fn simulate_physics(&mut self) {
        let dt = 0.016;
        let damping = 0.95;
        let max_speed = 200.0;
        let min_speed_threshold = 0.1;

        let repulsion_k = self.repel_force * 5.0;
        let center_k = self.center_force;
        let spring_k = self.link_force;
        let group_k = self.group_force;
        let tag_k = self.tag_force;

        let count = self.points_coord.len();
        let mut tag_centers: HashMap<String, Vec2> = HashMap::new();
        let mut tag_counts: HashMap<String, f32> = HashMap::new();

        for i in 0..count {
            if !self.is_visible(i) {
                continue;
            }
            let point = &self.points[i];
            if point.is_tag {
                continue;
            }

            let main_tag = if point.tags.is_empty() {
                "Orphan"
            } else {
                &point.tags[0]
            };
            let pos = Vec2::new(self.points_coord[i].0, self.points_coord[i].1);
            let entry_sum = tag_centers
                .entry(main_tag.to_string())
                .or_insert(Vec2::ZERO);
            *entry_sum += pos;
            let entry_count = tag_counts.entry(main_tag.to_string()).or_insert(0.0);
            *entry_count += 1.0;
        }

        for (tag, center) in tag_centers.iter_mut() {
            if let Some(&count) = tag_counts.get(tag) {
                if count > 0.0 {
                    *center /= count;
                }
            }
        }

        for i in 0..count {
            if !self.is_visible(i) || self.dragged_node_index == Some(i) {
                self.velocities[i] = Vec2::ZERO;
                continue;
            }

            let pos_i = Vec2::new(self.points_coord[i].0, self.points_coord[i].1);
            let mut force = Vec2::ZERO;
            force -= pos_i * center_k;

            if !self.points[i].is_tag {
                let main_tag = if self.points[i].tags.is_empty() {
                    "Orphan"
                } else {
                    &self.points[i].tags[0]
                };
                if let Some(&group_center) = tag_centers.get(main_tag) {
                    let dist_to_group = group_center - pos_i;
                    force += dist_to_group * group_k;
                }
            }

            for j in 0..count {
                if i == j {
                    continue;
                }
                if !self.is_visible(j) {
                    continue;
                }

                let pos_j = Vec2::new(self.points_coord[j].0, self.points_coord[j].1);
                let delta = pos_i - pos_j;
                let dist_sq = delta.length_sq();

                if dist_sq > 25000.0 {
                    continue;
                }
                let safe_dist_sq = dist_sq.max(10.0);

                let multiplier = if self.points[i].is_tag || self.points[j].is_tag {
                    2.0
                } else {
                    1.0
                };
                force += delta.normalized() * (repulsion_k * multiplier / safe_dist_sq);
            }
            self.velocities[i] += force * dt;
        }

        for &(idx_a, idx_b) in &self.edges {
            if idx_a >= count || idx_b >= count {
                continue;
            }
            if !self.is_visible(idx_a) || !self.is_visible(idx_b) {
                continue;
            }

            let dragging_a = self.dragged_node_index == Some(idx_a);
            let dragging_b = self.dragged_node_index == Some(idx_b);
            let pos_a = Vec2::new(self.points_coord[idx_a].0, self.points_coord[idx_a].1);
            let pos_b = Vec2::new(self.points_coord[idx_b].0, self.points_coord[idx_b].1);

            let delta = pos_b - pos_a;
            let dist = delta.length();

            let is_tag_connection = self.points[idx_a].is_tag || self.points[idx_b].is_tag;
            let current_k = if is_tag_connection {
                spring_k * tag_k
            } else {
                spring_k
            };

            let force = delta.normalized() * (dist * current_k);

            if !dragging_a {
                self.velocities[idx_a] += force * dt;
            }
            if !dragging_b {
                self.velocities[idx_b] -= force * dt;
            }
        }

        for i in 0..count {
            if self.dragged_node_index == Some(i) {
                continue;
            }
            if !self.is_visible(i) {
                continue;
            }

            self.velocities[i] *= damping;
            let speed = self.velocities[i].length();

            if !self.velocities[i].is_finite() {
                self.velocities[i] = Vec2::ZERO;
            } else if speed > max_speed {
                self.velocities[i] = self.velocities[i].normalized() * max_speed;
            } else if speed < min_speed_threshold {
                self.velocities[i] = Vec2::ZERO;
            }

            self.points_coord[i].0 += self.velocities[i].x * dt;
            self.points_coord[i].1 += self.velocities[i].y * dt;
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        current_file: &mut String,
        content: &mut main_area::Content,
        vault: &str,
    ) -> Response {
        self.simulate_physics();
        ui.ctx().request_repaint();

        let markers_plot = Plot::new("Graph")
            .data_aspect(1.0)
            .allow_drag(self.dragged_node_index.is_none())
            .show_axes([false, false])
            .show_grid([false, false])
            .include_x(100.0)
            .include_x(-100.0)
            .include_y(100.0)
            .include_y(-100.0);

        let response = markers_plot
            .show(ui, |plot_ui| {
                if self.points.is_empty() {
                    return;
                }

                let line_color = Color32::from_rgba_unmultiplied(100, 100, 100, 100);

                if self.show_arrows {
                    let mut origins = vec![];
                    let mut tips = vec![];

                    for &(idx_a, idx_b) in &self.edges {
                        if self.is_visible(idx_a)
                            && self.is_visible(idx_b)
                            && idx_a < self.points_coord.len()
                            && idx_b < self.points_coord.len()
                        {
                            let p1 = self.points_coord[idx_a];
                            let p2 = self.points_coord[idx_b];
                            origins.push([p1.0 as f64, p1.1 as f64]);
                            tips.push([p2.0 as f64, p2.1 as f64]);
                        }
                    }

                    let arrows = Arrows::new("".to_string(), origins, tips)
                        .color(line_color)
                        .tip_length(25.0);
                    plot_ui.arrows(arrows);
                } else {
                    for &(idx_a, idx_b) in &self.edges {
                        if self.is_visible(idx_a)
                            && self.is_visible(idx_b)
                            && idx_a < self.points_coord.len()
                            && idx_b < self.points_coord.len()
                        {
                            let p1 = self.points_coord[idx_a];
                            let p2 = self.points_coord[idx_b];
                            let line = Line::new(
                                "".to_string(),
                                vec![[p1.0 as f64, p1.1 as f64], [p2.0 as f64, p2.1 as f64]],
                            )
                            .color(line_color)
                            .width(self.line_thickness);
                            plot_ui.line(line);
                        }
                    }
                }

                let pointer_pos = plot_ui.pointer_coordinate();
                let is_double_click = plot_ui.response().double_clicked();
                let is_drag_started = plot_ui.response().drag_started();
                let is_drag_released = plot_ui.ctx().input(|i| i.pointer.any_released());

                if is_drag_released {
                    self.dragged_node_index = None;
                }

                for (index, point) in self.points.iter().enumerate() {
                    if !self.is_visible(index) {
                        continue;
                    }

                    let point_color = self.get_color_for_node(point);
                    let coords = [
                        self.points_coord[index].0 as f64,
                        self.points_coord[index].1 as f64,
                    ];

                    let mut radius = self.node_size;

                    if point.is_tag {
                        let degree = self.node_degrees[index] as f32;
                        radius = (self.node_size * 1.5) + (degree * 0.5);
                        if radius > 50.0 {
                            radius = 50.0;
                        }
                    } else if point.is_attachment {
                        radius = self.node_size * 0.7;
                    } else if !point.exists {
                        radius = self.node_size * 0.85;
                    }

                    let mut punto = Points::new("".to_string(), coords)
                        .radius(radius)
                        .color(point_color)
                        .shape(MarkerShape::Circle);

                    if point.is_attachment {
                        punto = punto.shape(MarkerShape::Square);
                    }
                    if point.is_tag {
                        punto = punto.shape(MarkerShape::Diamond);
                    }

                    plot_ui.points(punto);

                    let bounds = plot_ui.plot_bounds();
                    let diff = bounds.max()[1] - bounds.min()[1];

                    if diff < self.text_zoom_threshold {
                        let texto = Text::new(
                            "".to_string(),
                            PlotPoint::new(
                                self.points_coord[index].0 as f64,
                                (self.points_coord[index].1 as f64) - (diff * 0.02),
                            ),
                            RichText::new(&point.text).size(12.0),
                        );
                        plot_ui.text(texto);
                    }

                    if let Some(ptr) = pointer_pos {
                        let node_pos = self.points_coord[index];
                        if is_close(ptr, node_pos, 1.2) {
                            if is_double_click && self.dragged_node_index.is_none() {
                                if point.exists && !point.is_attachment && !point.is_tag {
                                    *current_file = point.abs_path.clone();
                                    *content = main_area::Content::View;
                                }
                            }
                            if is_drag_started {
                                self.dragged_node_index = Some(index);
                            }
                        }
                    }
                }

                if let Some(idx) = self.dragged_node_index {
                    if let Some(ptr) = pointer_pos {
                        self.points_coord[idx].0 = ptr.x as f32;
                        self.points_coord[idx].1 = ptr.y as f32;
                        self.velocities[idx] = Vec2::ZERO;
                    }
                }
            })
            .response;

        let plot_rect = response.rect;
        let mut controls_changed = false;

        egui::Area::new("graph_controls_overlay".into())
            .fixed_pos(plot_rect.min + egui::vec2(10.0, 10.0))
            .order(Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .shadow(Shadow::default())
                    .fill(ui.style().visuals.window_fill().linear_multiply(0.95))
                    .stroke(ui.style().visuals.window_stroke())
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_max_width(220.0);
                        ui.set_max_height(plot_rect.height() - 40.0);

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if self.controls(ui) {
                                controls_changed = true;
                            }
                        });
                    });
            });

        if controls_changed {
            self.update_vault(Path::new(vault));
        }

        response
    }

    fn get_color_for_node(&self, point: &MarmolPoint) -> Color32 {
        if !point.exists {
            return self.ghost_color;
        }

        let mut matching_colors = vec![];

        for group in &self.custom_groups {
            if self.check_match(&group.match_type, &group.value, point) {
                matching_colors.push(group.color);
            }
        }

        if !self.new_group_val.is_empty() {
            if self.check_match(&self.new_group_type, &self.new_group_val, point) {
                matching_colors.push(self.new_group_col);
            }
        }

        if !matching_colors.is_empty() {
            return mix_colors(&matching_colors);
        }

        if point.is_attachment {
            return self.attachment_color;
        }

        if point.is_tag {
            let mut hasher = DefaultHasher::new();
            point.text.hash(&mut hasher);
            let hash = hasher.finish();
            let index = (hash as usize) % self.palette.len();
            return self.palette[index].linear_multiply(1.2); // Más brillante
        }

        let valid_tags: Vec<&String> = point.tags.iter().filter(|t| *t != "Orphan").collect();

        if valid_tags.is_empty() {
            return self.orphan_color;
        }

        let tag = valid_tags[0];
        if let Some(color) = self.tags_colors.get(tag) {
            return *color;
        }

        let mut hasher = DefaultHasher::new();
        tag.hash(&mut hasher);
        let hash = hasher.finish();

        let index = (hash as usize) % self.palette.len();
        self.palette[index]
    }
}

fn is_close(delta: PlotPoint, point_pos: (f32, f32), tol: f32) -> bool {
    let dx = (delta.x as f32 - point_pos.0).abs();
    let dy = (delta.y as f32 - point_pos.1).abs();
    dx < tol && dy < tol
}

fn build_edges(points: &Vec<MarmolPoint>, show_tags: bool) -> Vec<(usize, usize)> {
    let mut edges = vec![];
    let mut name_to_index = HashMap::new();
    let mut tag_to_index = HashMap::new();

    for (idx, point) in points.iter().enumerate() {
        if point.is_tag {
            tag_to_index.insert(point.text.clone(), idx);
        } else {
            let clean = point.text.trim().to_lowercase();
            name_to_index.insert(clean, idx);
        }
    }

    for (i, point) in points.iter().enumerate() {
        if point.is_tag {
            continue;
        }

        for link in &point.links {
            let link_clean = link.trim().to_lowercase();
            if let Some(&target_idx) = name_to_index.get(&link_clean) {
                if i != target_idx {
                    edges.push((i, target_idx));
                }
            }
        }

        if show_tags {
            for tag in &point.tags {
                if tag != "Orphan" {
                    if let Some(&tag_idx) = tag_to_index.get(tag) {
                        edges.push((i, tag_idx));
                    }
                }
            }
        }
    }
    edges
}

fn extract_frontmatter(content: &str) -> Option<String> {
    if content.starts_with("---") {
        if let Some(end_idx) = content[3..].find("---") {
            return Some(content[0..end_idx + 6].to_string());
        }
    }
    None
}

fn get_data(dir: &Path, marmol_vec: &mut Vec<MarmolPoint>, total_entries: &mut i32, vault: &str) {
    if !Path::new(vault).exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                get_data(&path, marmol_vec, total_entries, vault);
            } else {
                if let Some(ext_os) = path.extension() {
                    if let Some(ext) = ext_os.to_str() {
                        let ext = ext.to_lowercase();
                        let abs_path = path.to_str().unwrap().to_string();
                        let rel_path = abs_path.replace(vault, "");

                        let node_name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        if ext == "md" {
                            *total_entries += 1;
                            let raw_content = files::read_file(&abs_path);
                            let mut tag_vecs = vec![];
                            if let Some(yaml_str) = extract_frontmatter(&raw_content) {
                                match YamlLoader::load_from_str(&yaml_str) {
                                    Ok(docs) => {
                                        if !docs.is_empty() {
                                            let doc = &docs[0];
                                            let tags_opt =
                                                doc["tags"].as_vec().or(doc["Tags"].as_vec());
                                            if let Some(tags) = tags_opt {
                                                for tag in tags {
                                                    if let Some(s) = tag.as_str() {
                                                        tag_vecs.push(s.to_owned());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            if tag_vecs.is_empty() {
                                tag_vecs.push("Orphan".to_owned());
                            }

                            let mut links_vec = vec![];
                            let parts: Vec<&str> = raw_content.split("[[").collect();
                            for (i, part) in parts.iter().enumerate() {
                                if i == 0 {
                                    continue;
                                }
                                if let Some(end) = part.find("]]") {
                                    let link_content = &part[..end];
                                    let target =
                                        link_content.split('|').next().unwrap_or(link_content);
                                    links_vec.push(target.trim().to_string());
                                }
                            }

                            marmol_vec.push(MarmolPoint::new(
                                &node_name, tag_vecs, links_vec, rel_path, abs_path, false, false,
                                true,
                            ));
                        } else if ["png", "jpg", "jpeg", "pdf", "gif"].contains(&ext.as_str()) {
                            *total_entries += 1;
                            marmol_vec.push(MarmolPoint::new(
                                &node_name,
                                vec!["Attachment".to_string()],
                                vec![],
                                rel_path,
                                abs_path,
                                true,
                                false,
                                true,
                            ));
                        }
                    }
                }
            }
        }
    }
}

fn get_coords(coords_vec: &mut Vec<(f32, f32)>, total_entries: i32) {
    let elementos = total_entries as f32;
    let radio = 10.0;
    let var = std::f32::consts::TAU;

    for i in 0..total_entries {
        let a = (var / elementos) * i as f32;
        let x: f32 = radio * a.cos();
        let y: f32 = radio * a.sin();
        coords_vec.push((x, y));
    }
}

fn mix_colors(colors: &[Color32]) -> Color32 {
    if colors.is_empty() {
        return Color32::WHITE;
    }
    let mut r = 0u32;
    let mut g = 0u32;
    let mut b = 0u32;
    let mut a = 0u32;

    for c in colors {
        r += c.r() as u32;
        g += c.g() as u32;
        b += c.b() as u32;
        a += c.a() as u32;
    }
    let count = colors.len() as u32;
    Color32::from_rgba_premultiplied(
        (r / count) as u8,
        (g / count) as u8,
        (b / count) as u8,
        (a / count) as u8,
    )
}
