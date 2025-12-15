use crate::files;
use crate::main_area;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use yaml_rust::YamlLoader;

use egui::*;
use egui_plot::{Line, MarkerShape, Plot, PlotPoint, Points, Text};

#[derive(Clone)]
struct MarmolPoint {
    text: String,
    tags: Vec<String>,
    links: Vec<String>,
}

pub struct Graph {
    points: Vec<MarmolPoint>,
    points_coord: Vec<(f32, f32)>,
    velocities: Vec<Vec2>,
    edges: Vec<(usize, usize)>,

    // Fuerzas
    center_force: f32,
    repel_force: f32,
    link_force: f32,
    group_force: f32, // Fuerza de agrupación por tags

    dragged_node_index: Option<usize>,
    orphan_color: Color32,
    palette: Vec<Color32>,
    tags_colors: HashMap<String, Color32>,
}

impl MarmolPoint {
    fn new(val: &str, tags: Vec<String>, links: Vec<String>) -> Self {
        Self {
            text: format!("{}", val),
            tags: tags,
            links: links,
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
        // Paleta de colores vibrantes
        let palette = vec![
            Color32::from_rgb(235, 64, 52),  // Rojo
            Color32::from_rgb(60, 100, 220), // Azul Brillante
            Color32::from_rgb(140, 40, 160), // Morado
            Color32::from_rgb(255, 140, 0),  // Naranja
            Color32::from_rgb(46, 204, 113), // Esmeralda
            Color32::from_rgb(52, 152, 219), // Celeste
            Color32::from_rgb(241, 196, 15), // Amarillo
            Color32::from_rgb(231, 76, 60),  // Coral
            Color32::from_rgb(52, 73, 94),   // Azul Grisáceo
        ];

        let mut data = vec![];
        let mut coords = vec![];
        let mut total_entries = 0;

        get_data(&Path::new(vault), &mut data, &mut total_entries, vault);
        get_coords(&mut coords, total_entries);

        let edges = build_edges(&data);

        println!("--- GRAPH LOADED ---");
        println!("Nodos: {}", data.len());
        println!("Conexiones: {}", edges.len());

        let velocities = vec![Vec2::ZERO; coords.len()];

        Self {
            points: data,
            points_coord: coords,
            velocities,
            edges,
            center_force: 0.15,
            repel_force: 30.0,
            link_force: 0.8,
            group_force: 1.5,

            dragged_node_index: None,
            orphan_color: Color32::from_rgb(100, 110, 120),
            palette,
            tags_colors,
        }
    }
}

impl Graph {
    pub fn update_vault(&mut self, vault: &Path) {
        let mut new_points = vec![];
        let mut new_coords = vec![];
        let mut elements = 0;

        get_data(
            vault,
            &mut new_points,
            &mut elements,
            vault.to_str().unwrap(),
        );
        get_coords(&mut new_coords, elements);

        self.edges = build_edges(&new_points);
        self.points = new_points;
        self.points_coord = new_coords;
        self.velocities = vec![Vec2::ZERO; self.points_coord.len()];
        self.dragged_node_index = None;
    }

    pub fn controls(&mut self, ui: &mut Ui) {
        ui.label("Física");
        ui.add(egui::Slider::new(&mut self.repel_force, 1.0..=100.0).text("Repulsión (Separar)"));
        ui.add(egui::Slider::new(&mut self.link_force, 0.1..=3.0).text("Links (Cuerdas)"));
        ui.add(egui::Slider::new(&mut self.group_force, 0.0..=5.0).text("Agrupación Tags"));
        ui.add(egui::Slider::new(&mut self.center_force, 0.01..=1.0).text("Gravedad Centro"));

        ui.separator();
        ui.horizontal(|ui| {
            color_picker::color_edit_button_srgba(
                ui,
                &mut self.orphan_color,
                egui::widgets::color_picker::Alpha::Opaque,
            );
            ui.label("Color Huérfanos");
        });
    }

    fn simulate_physics(&mut self) {
        let dt = 0.016;
        let damping = 0.92;
        let max_speed = 200.0;

        let repulsion_k = self.repel_force * 5.0;
        let center_k = self.center_force;
        let spring_k = self.link_force;
        let group_k = self.group_force;

        let count = self.points_coord.len();

        let mut tag_centers: HashMap<String, Vec2> = HashMap::new();
        let mut tag_counts: HashMap<String, f32> = HashMap::new();

        for (i, point) in self.points.iter().enumerate() {
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
            if self.dragged_node_index == Some(i) {
                self.velocities[i] = Vec2::ZERO;
                continue;
            }

            let pos_i = Vec2::new(self.points_coord[i].0, self.points_coord[i].1);
            let mut force = Vec2::ZERO;

            force -= pos_i * center_k;

            let main_tag = if self.points[i].tags.is_empty() {
                "Orphan"
            } else {
                &self.points[i].tags[0]
            };

            if let Some(&group_center) = tag_centers.get(main_tag) {
                let dist_to_group = group_center - pos_i;
                force += dist_to_group * group_k;
            }

            for j in 0..count {
                if i == j {
                    continue;
                }
                let pos_j = Vec2::new(self.points_coord[j].0, self.points_coord[j].1);
                let delta = pos_i - pos_j;
                let dist_sq = delta.length_sq();

                if dist_sq > 25000.0 {
                    continue;
                }

                if dist_sq < 1.0 {
                    force += Vec2::new(1.0, 1.0) * repulsion_k;
                } else {
                    force += delta.normalized() * (repulsion_k / dist_sq);
                }
            }
            self.velocities[i] += force * dt;
        }

        for &(idx_a, idx_b) in &self.edges {
            if idx_a >= count || idx_b >= count {
                continue;
            }

            let dragging_a = self.dragged_node_index == Some(idx_a);
            let dragging_b = self.dragged_node_index == Some(idx_b);

            let pos_a = Vec2::new(self.points_coord[idx_a].0, self.points_coord[idx_a].1);
            let pos_b = Vec2::new(self.points_coord[idx_b].0, self.points_coord[idx_b].1);

            let delta = pos_b - pos_a;
            let dist = delta.length();
            let force = delta.normalized() * (dist * spring_k);

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
            let min_speed_threshold = 1.0;

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
            .show_grid([false, false]);

        let res = markers_plot
            .show(ui, |plot_ui| {
                if self.points.is_empty() {
                    return;
                }

                let line_color = Color32::from_rgba_unmultiplied(100, 100, 100, 100);

                for &(idx_a, idx_b) in &self.edges {
                    if idx_a < self.points_coord.len() && idx_b < self.points_coord.len() {
                        let p1 = self.points_coord[idx_a];
                        let p2 = self.points_coord[idx_b];

                        let line = Line::new(
                            "",
                            vec![[p1.0 as f64, p1.1 as f64], [p2.0 as f64, p2.1 as f64]],
                        )
                        .color(line_color)
                        .width(2.0);
                        plot_ui.line(line);
                    }
                }

                let pointer_pos = plot_ui.pointer_coordinate();
                let is_drag_started = plot_ui.response().drag_started();
                let is_drag_released = plot_ui.ctx().input(|i| i.pointer.any_released());

                if is_drag_released {
                    self.dragged_node_index = None;
                }

                for (index, point) in self.points.iter().enumerate() {
                    let point_color = self.get_color_for_tags(&point.tags);

                    let coords = [
                        self.points_coord[index].0 as f64,
                        self.points_coord[index].1 as f64,
                    ];

                    let punto = Points::new("", coords)
                        .radius(7.0)
                        .color(point_color)
                        .shape(MarkerShape::Circle);

                    plot_ui.points(punto);

                    // LOD Text Visibility
                    let bounds = plot_ui.plot_bounds();
                    let diff = bounds.max()[1] - bounds.min()[1];

                    if diff < 500.0 {
                        let texto = Text::new(
                            "",
                            PlotPoint::new(
                                self.points_coord[index].0,
                                (self.points_coord[index].1) - (diff * 0.02) as f32,
                            ),
                            RichText::new(&point.text).size(12.0),
                        );
                        plot_ui.text(texto);
                    }

                    if let Some(ptr) = pointer_pos {
                        let node_pos = self.points_coord[index];
                        if is_close(ptr, node_pos, 1.2) {
                            if plot_ui.response().double_clicked()
                                && self.dragged_node_index.is_none()
                            {
                                *current_file = format!("{}/{}", vault, &point.text);
                                *content = main_area::Content::View;
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

        let plot_rect = res.rect;
        let overlay_width = 250.0;
        let margin = 10.0;
        let overlay_rect = Rect::from_min_size(
            plot_rect.min + Vec2::new(margin, margin),
            Vec2::new(overlay_width, plot_rect.height() - (margin * 2.0)),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(overlay_rect), |ui| {
            egui::Frame::popup(ui.style())
                .stroke(Stroke::new(1.0, Color32::from_gray(60)))
                //.fill(Color32::from_black_alpha(200))
                .show(ui, |ui| {
                    ui.set_max_width(overlay_width - 20.0);
                    ui.collapsing("Opciones del Grafo", |ui| {
                        self.controls(ui);
                    });
                });
        });
        res
    }

    fn get_color_for_tags(&self, tags: &[String]) -> Color32 {
        let valid_tags: Vec<&String> = tags.iter().filter(|t| *t != "Orphan").collect();

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

fn build_edges(points: &Vec<MarmolPoint>) -> Vec<(usize, usize)> {
    let mut edges = vec![];
    let mut name_to_index = HashMap::new();

    for (idx, point) in points.iter().enumerate() {
        let clean = point.text.trim().to_lowercase();
        name_to_index.insert(clean, idx);
    }

    for (i, point) in points.iter().enumerate() {
        for link in &point.links {
            let link_clean = link.trim().to_lowercase();
            if let Some(&target_idx) = name_to_index.get(&link_clean) {
                if i != target_idx {
                    edges.push((i, target_idx));
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
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        *total_entries += 1;
                        let node_name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown")
                            .to_string();

                        let raw_content = files::read_file(path.to_str().unwrap());

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
                                let target = link_content.split('|').next().unwrap_or(link_content);
                                links_vec.push(target.trim().to_string());
                            }
                        }

                        let point = MarmolPoint::new(&node_name, tag_vecs, links_vec);
                        marmol_vec.push(point);
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
