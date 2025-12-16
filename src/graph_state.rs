// graph_state.rs
use crate::files;
// Asegúrate de importar Content donde sea necesario o pasarlo como genérico si prefieres desacoplarlo
// use crate::main_area::content_enum::Content;

use egui::{Color32, Vec2};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use yaml_rust::YamlLoader;

#[derive(Clone)]
pub struct MarmolPoint {
    pub text: String,
    pub tags: Vec<String>,
    pub links: Vec<String>,
    pub rel_path: String,
    pub abs_path: String,
    pub is_attachment: bool,
    pub is_tag: bool,
    pub exists: bool,
}

#[derive(Clone, PartialEq)]
pub enum MatchType {
    Filename,
    Tag,
    Path,
    Content,
    Section,
}

#[derive(Clone)]
pub struct CustomGroup {
    pub match_type: MatchType,
    pub value: String,
    pub color: Color32,
}

pub struct Graph {
    // Datos principales (Hacemos pub para que la UI pueda leerlos)
    pub points: Vec<MarmolPoint>,
    pub points_coord: Vec<(f32, f32)>,
    pub velocities: Vec<Vec2>,
    pub edges: Vec<(usize, usize)>,
    pub node_degrees: Vec<usize>,

    // Fuerzas Físicas (pub para los sliders)
    pub center_force: f32,
    pub repel_force: f32,
    pub link_force: f32,
    pub group_force: f32,
    pub tag_force: f32,

    // Filtros de Texto
    pub filter_filename: String,
    pub filter_tag: String,
    pub filter_path: String,
    pub filter_line: String,
    pub filter_section: String,

    // Toggles
    pub show_attachments: bool,
    pub show_existing_only: bool,
    pub show_orphans: bool,
    pub show_tags: bool,

    // Visualización
    pub show_arrows: bool,
    pub text_zoom_threshold: f64,
    pub node_size: f32,
    pub line_thickness: f32,

    // Estado interno UI (Drag & Drop)
    pub dragged_node_index: Option<usize>,

    // Colores
    pub orphan_color: Color32,
    pub ghost_color: Color32,
    pub attachment_color: Color32,
    pub palette: Vec<Color32>,
    pub tags_colors: HashMap<String, Color32>,

    // Grupos personalizados (UI State)
    pub custom_groups: Vec<CustomGroup>,
    pub new_group_type: MatchType,
    pub new_group_val: String,
    pub new_group_col: Color32,
    pub hovered_node_index: Option<usize>,
}

impl MarmolPoint {
    fn new(
        val: &str,
        tags: Vec<String>,
        links: Vec<String>,
        rel_path: String,
        abs_path: String,
        is_attachment: bool,
        is_tag: bool,
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
    pub fn new(vault: &str, ctx: &egui::Context) -> Self {
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
            ctx.style().visuals.error_fg_color,
            ctx.style().visuals.selection.stroke.color,
            ctx.style().visuals.widgets.hovered.fg_stroke.color,
            ctx.style().visuals.warn_fg_color,
            Color32::LIGHT_GREEN,
            Color32::LIGHT_BLUE,
            Color32::YELLOW,
            Color32::LIGHT_RED,
            ctx.style().visuals.widgets.noninteractive.fg_stroke.color,
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
            tag_force: 3.0,

            filter_filename: String::new(),
            filter_tag: String::new(),
            filter_path: String::new(),
            filter_line: String::new(),
            filter_section: String::new(),

            show_attachments: true,
            show_existing_only: false,
            show_orphans: true,
            show_tags: false,

            show_arrows: false,
            text_zoom_threshold: 500.0,
            node_size: 7.0,
            line_thickness: 2.0,

            dragged_node_index: None,
            orphan_color: ctx.style().visuals.widgets.inactive.fg_stroke.color.linear_multiply(0.7),
            ghost_color: ctx.style().visuals.widgets.noninteractive.bg_stroke.color.linear_multiply(0.2),
            attachment_color: ctx.style().visuals.selection.stroke.color,
            palette,
            tags_colors,

            custom_groups: vec![],
            new_group_type: MatchType::Tag,
            new_group_val: String::new(),
            new_group_col: ctx.style().visuals.error_fg_color,
            hovered_node_index: None,
        };

        graph.update_vault(Path::new(vault));
        graph
    }

    pub fn check_match(&self, m_type: &MatchType, val: &str, point: &MarmolPoint) -> bool {
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

        // 2. Generar Nodos Fantasma
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
                            false,
                            false,
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

        // 3. Generar Nodos de TAGS
        if self.show_tags {
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
            for tag in all_tags {
                new_points.push(MarmolPoint::new(
                    &tag,
                    vec![],
                    vec![],
                    "".to_string(),
                    "".to_string(),
                    false,
                    true,
                    true,
                ));
            }
        }

        let total_count = new_points.len();
        let mut new_coords = vec![];
        get_coords(&mut new_coords, total_count as i32);

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

    pub fn is_visible(&self, index: usize) -> bool {
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
        if !self.show_tags && p.is_tag {
            return false;
        }

        if !self.filter_filename.is_empty()
            && !p
                .text
                .to_lowercase()
                .contains(&self.filter_filename.to_lowercase())
        {
            return false;
        }

        if !self.filter_tag.is_empty() {
            let search = self.filter_tag.to_lowercase();
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

        if !self.filter_path.is_empty()
            && !p
                .rel_path
                .to_lowercase()
                .contains(&self.filter_path.to_lowercase())
        {
            return false;
        }

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
                if !content
                    .lines()
                    .any(|l| l.trim().starts_with('#') && l.to_lowercase().contains(&search))
                {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn simulate_physics(&mut self) {
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

        // 1. Calcular centros de grupos
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
            *tag_centers
                .entry(main_tag.to_string())
                .or_insert(Vec2::ZERO) += pos;
            *tag_counts.entry(main_tag.to_string()).or_insert(0.0) += 1.0;
        }

        for (tag, center) in tag_centers.iter_mut() {
            if let Some(&count) = tag_counts.get(tag) {
                if count > 0.0 {
                    *center /= count;
                }
            }
        }

        // 2. Fuerzas Nodos y Repulsión
        for i in 0..count {
            if !self.is_visible(i) || self.dragged_node_index == Some(i) {
                self.velocities[i] = Vec2::ZERO;
                continue;
            }

            let pos_i = Vec2::new(self.points_coord[i].0, self.points_coord[i].1);
            let mut force = Vec2::ZERO;
            force -= pos_i * center_k; // Gravedad central

            if !self.points[i].is_tag {
                let main_tag = if self.points[i].tags.is_empty() {
                    "Orphan"
                } else {
                    &self.points[i].tags[0]
                };
                if let Some(&group_center) = tag_centers.get(main_tag) {
                    force += (group_center - pos_i) * group_k; // Atracción de grupo
                }
            }

            // Repulsión
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

        // 3. Fuerzas de Enlaces (Springs)
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

            let is_tag_conn = self.points[idx_a].is_tag || self.points[idx_b].is_tag;
            let current_k = if is_tag_conn {
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

        // 4. Integración y Amortiguación
        for i in 0..count {
            if self.dragged_node_index == Some(i) || !self.is_visible(i) {
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

    pub fn get_color_for_node(&self, point: &MarmolPoint) -> Color32 {
        if !point.exists {
            return self.ghost_color;
        }

        let mut matching_colors = vec![];

        for group in &self.custom_groups {
            if self.check_match(&group.match_type, &group.value, point) {
                matching_colors.push(group.color);
            }
        }
        if !self.new_group_val.is_empty()
            && self.check_match(&self.new_group_type, &self.new_group_val, point)
        {
            matching_colors.push(self.new_group_col);
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
            return self.palette[(hasher.finish() as usize) % self.palette.len()]
                .linear_multiply(1.2);
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
        self.palette[(hasher.finish() as usize) % self.palette.len()]
    }
}

// ---------------- Helpers Internos ----------------

fn mix_colors(colors: &[Color32]) -> Color32 {
    if colors.is_empty() {
        return Color32::WHITE;
    }
    let (mut r, mut g, mut b, mut a) = (0u32, 0u32, 0u32, 0u32);
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

fn build_edges(points: &Vec<MarmolPoint>, show_tags: bool) -> Vec<(usize, usize)> {
    let mut edges = vec![];
    let mut name_to_index = HashMap::new();
    let mut tag_to_index = HashMap::new();

    for (idx, point) in points.iter().enumerate() {
        if point.is_tag {
            tag_to_index.insert(point.text.clone(), idx);
        } else {
            name_to_index.insert(point.text.trim().to_lowercase(), idx);
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
                if let Some(ext) = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_lowercase())
                {
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
                            if let Ok(docs) = YamlLoader::load_from_str(&yaml_str) {
                                if !docs.is_empty() {
                                    if let Some(tags) =
                                        docs[0]["tags"].as_vec().or(docs[0]["Tags"].as_vec())
                                    {
                                        for tag in tags {
                                            if let Some(s) = tag.as_str() {
                                                tag_vecs.push(s.to_owned());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if tag_vecs.is_empty() {
                            tag_vecs.push("Orphan".to_owned());
                        }

                        let mut links_vec = vec![];
                        for part in raw_content.split("[[").skip(1) {
                            if let Some(end) = part.find("]]") {
                                let target = part[..end].split('|').next().unwrap_or(&part[..end]);
                                links_vec.push(target.trim().to_string());
                            }
                        }
                        marmol_vec.push(MarmolPoint::new(
                            &node_name, tag_vecs, links_vec, rel_path, abs_path, false, false, true,
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

fn extract_frontmatter(content: &str) -> Option<String> {
    if content.starts_with("---") {
        if let Some(end_idx) = content[3..].find("---") {
            return Some(content[0..end_idx + 6].to_string());
        }
    }
    None
}

fn get_coords(coords_vec: &mut Vec<(f32, f32)>, total_entries: i32) {
    let radio = 10.0;
    let var = std::f32::consts::TAU;
    for i in 0..total_entries {
        let a = (var / total_entries as f32) * i as f32;
        coords_vec.push((radio * a.cos(), radio * a.sin()));
    }
}
