use super::data::{Graph, GraphSettings, MarmolPoint, MatchType};
use crate::files;
use egui::{Color32, Vec2};
use regex::Regex;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use yaml_rust::YamlLoader;

/// Sufijos de ficheros que se consideran adjuntos en Obsidian.
const ATTACHMENT_EXTS: [&str; 12] = [
    "png", "jpg", "jpeg", "gif", "bmp", "svg", "webp", "pdf", "mp3", "wav", "mp4", "mov",
];

impl Graph {
    pub fn new(vault: &str, ctx: &egui::Context) -> Self {
        let mut tags_colors = HashMap::new();
        // Compatibilidad con el formato antiguo `test.json`.
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
            ctx.style_of(ctx.theme()).visuals.error_fg_color,
            ctx.style_of(ctx.theme()).visuals.selection.stroke.color,
            ctx.style_of(ctx.theme()).visuals.widgets.hovered.fg_stroke.color,
            ctx.style_of(ctx.theme()).visuals.warn_fg_color,
            Color32::LIGHT_GREEN,
            Color32::LIGHT_BLUE,
            Color32::YELLOW,
            Color32::LIGHT_RED,
            ctx.style_of(ctx.theme()).visuals.widgets.noninteractive.fg_stroke.color,
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
            link_distance: 30.0,
            group_force: 1.5,
            tag_force: 3.0,

            filter_query: String::new(),

            show_attachments: true,
            show_existing_only: false,
            show_orphans: true,
            show_tags: false,

            show_arrows: false,
            text_zoom_threshold: 500.0,
            node_size: 7.0,
            line_thickness: 2.0,

            dragged_node_index: None,
            hovered_node_index: None,
            selected_node_index: None,
            show_controls: true,
            pending_fit: false,

            orphan_color: ctx.style_of(ctx.theme()).visuals.widgets.inactive.fg_stroke.color.linear_multiply(0.7),
            ghost_color: ctx.style_of(ctx.theme()).visuals.widgets.noninteractive.bg_stroke.color.linear_multiply(0.2),
            attachment_color: ctx.style_of(ctx.theme()).visuals.selection.stroke.color,
            tag_node_color: ctx.style_of(ctx.theme()).visuals.warn_fg_color,
            palette,
            tags_colors,

            custom_groups: vec![],
            new_group_query: String::new(),
            new_group_col: ctx.style_of(ctx.theme()).visuals.error_fg_color,

            content_cache: HashMap::new(),
            content_cache_bytes: 0,
            vault_path: vault.to_string(),
            settings_dirty: false,
            loaded: false,
        };

        graph.load_settings();
        graph
    }

    /// Devuelve la ruta del fichero de ajustes del grafo para este vault.
    fn settings_path(&self) -> Option<std::path::PathBuf> {
        if self.vault_path.is_empty() {
            return None;
        }
        Some(Path::new(&self.vault_path).join(".marmol").join("graph.json"))
    }

    /// Carga los ajustes persistentes del grafo (si existen).
    pub fn load_settings(&mut self) {
        let Some(path) = self.settings_path() else { return };
        let Ok(data) = fs::read_to_string(&path) else { return };
        let Ok(settings) = serde_json::from_str::<GraphSettings>(&data) else { return };
        self.apply_settings(settings);
    }

    /// Persiste los ajustes actuales del grafo.
    pub fn save_settings(&self) {
        let Some(path) = self.settings_path() else { return };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let settings = self.to_settings();
        if let Ok(serialized) = serde_json::to_string_pretty(&settings) {
            let _ = fs::write(path, serialized);
        }
    }

    fn apply_settings(&mut self, s: GraphSettings) {
        self.center_force = s.center_force;
        self.repel_force = s.repel_force;
        self.link_force = s.link_force;
        self.link_distance = s.link_distance;
        self.filter_query = s.filter_query;
        self.show_attachments = s.show_attachments;
        self.show_existing_only = s.show_existing_only;
        self.show_orphans = s.show_orphans;
        self.show_tags = s.show_tags;
        self.show_arrows = s.show_arrows;
        self.text_zoom_threshold = s.text_zoom_threshold;
        self.node_size = s.node_size;
        self.line_thickness = s.line_thickness;
        self.custom_groups = s.custom_groups;
    }

    fn to_settings(&self) -> GraphSettings {
        GraphSettings {
            center_force: self.center_force,
            repel_force: self.repel_force,
            link_force: self.link_force,
            link_distance: self.link_distance,
            filter_query: self.filter_query.clone(),
            show_attachments: self.show_attachments,
            show_existing_only: self.show_existing_only,
            show_orphans: self.show_orphans,
            show_tags: self.show_tags,
            show_arrows: self.show_arrows,
            text_zoom_threshold: self.text_zoom_threshold,
            node_size: self.node_size,
            line_thickness: self.line_thickness,
            custom_groups: self.custom_groups.clone(),
        }
    }

    /// Comprueba un término individual contra un nodo.
    pub fn check_match(&self, m_type: &MatchType, val: &str, point: &MarmolPoint) -> bool {
        if val.is_empty() {
            return false;
        }
        let search = val.to_lowercase();

        match m_type {
            MatchType::Any => {
                if point.text.to_lowercase().contains(&search)
                    || point.rel_path.to_lowercase().contains(&search)
                    || point.tags.iter().any(|t| t.to_lowercase().contains(&search))
                {
                    return true;
                }
                self.match_content(&search, false, point)
            }
            MatchType::Tag => point
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&search)),
            MatchType::Filename => point.text.to_lowercase().contains(&search),
            MatchType::Path => point.rel_path.to_lowercase().contains(&search),
            MatchType::Content => self.match_content(&search, false, point),
            MatchType::Section => self.match_content(&search, true, point),
        }
    }

    fn match_content(&self, search: &str, section: bool, point: &MarmolPoint) -> bool {
        if point.is_attachment || point.is_tag {
            return false;
        }
        self.content_cache
            .get(&point.abs_path)
            .map(|c| {
                if section {
                    c.lines()
                        .any(|l| l.trim().starts_with('#') && l.to_lowercase().contains(search))
                } else {
                    c.to_lowercase().contains(search)
                }
            })
            .unwrap_or(false)
    }

    /// Evalúa una consulta completa (AND de todos los términos).
    pub fn matches_query(&self, query: &str, point: &MarmolPoint) -> bool {
        let tokens = parse_query(query);
        if tokens.is_empty() {
            return false;
        }
        tokens
            .iter()
            .all(|(mt, v)| self.check_match(mt, v, point))
    }

    pub fn update_vault(&mut self, vault: &Path) {
        self.content_cache.clear();
        self.content_cache_bytes = 0;
        self.vault_path = vault.to_str().unwrap_or("").to_string();

        let mut points: Vec<MarmolPoint> = vec![];
        let mut total = 0i32;
        let vault_str = vault.to_str().unwrap_or("");
        get_data(
            vault,
            &mut points,
            &mut total,
            vault_str,
            &mut self.content_cache,
            &mut self.content_cache_bytes,
        );

        // Índices de resolución de enlaces.
        let mut by_rel: HashMap<String, usize> = HashMap::new();
        let mut by_rel_ext: HashMap<String, usize> = HashMap::new();
        let mut by_stem: HashMap<String, usize> = HashMap::new();
        let mut by_name: HashMap<String, usize> = HashMap::new();
        for (i, p) in points.iter().enumerate() {
            if p.is_tag {
                continue;
            }
            let stem = Path::new(&p.abs_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            let name = Path::new(&p.abs_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            let rel = p
                .rel_path
                .trim_start_matches(['/', '\\'])
                .replace('\\', "/")
                .to_lowercase();
            let rel_no_ext = rel.strip_suffix(".md").unwrap_or(&rel).to_string();
            by_stem.entry(stem).or_insert(i);
            by_name.entry(name).or_insert(i);
            by_rel.entry(rel_no_ext).or_insert(i);
            by_rel_ext.entry(rel).or_insert(i);
        }

        // Resolución de enlaces + nodos fantasma.
        let mut ghost_index: HashMap<String, usize> = HashMap::new();
        let mut edges: Vec<(usize, usize)> = vec![];
        let mut extras: Vec<MarmolPoint> = vec![];
        let base_len = points.len();
        for (i, p) in points.iter().enumerate() {
            if p.is_tag {
                continue;
            }
            for link in &p.links {
                let key = link.trim().trim_start_matches("./").replace('\\', "/");
                let lower = key.to_lowercase();
                let no_ext = lower.strip_suffix(".md").unwrap_or(&lower).to_string();
                let target = by_rel
                    .get(&no_ext)
                    .or_else(|| by_rel_ext.get(&lower))
                    .or_else(|| by_stem.get(&lower))
                    .or_else(|| by_name.get(&lower))
                    .copied();
                if let Some(t) = target {
                    if t != i {
                        edges.push((i, t));
                    }
                } else {
                    let gi = match ghost_index.get(&lower) {
                        Some(&g) => g,
                        None => {
                            let g = base_len + extras.len();
                            extras.push(MarmolPoint::new(
                                &key,
                                vec![],
                                vec![],
                                String::new(),
                                String::new(),
                                false,
                                false,
                                false,
                            ));
                            ghost_index.insert(lower.clone(), g);
                            g
                        }
                    };
                    edges.push((i, gi));
                }
            }
        }
        points.append(&mut extras);

        // Nodos de tags (siempre indexados; is_visible los oculta si procede).
        let mut tag_index: HashMap<String, usize> = HashMap::new();
        let mut all_tags: Vec<String> = vec![];
        for p in &points {
            if p.is_tag || p.is_attachment {
                continue;
            }
            for t in &p.tags {
                if t == "Orphan" {
                    continue;
                }
                if !tag_index.contains_key(t) {
                    tag_index.insert(t.clone(), points.len() + all_tags.len());
                    all_tags.push(t.clone());
                }
            }
        }
        for tag in &all_tags {
            points.push(MarmolPoint::new(
                tag,
                vec![],
                vec![],
                String::new(),
                String::new(),
                false,
                true,
                true,
            ));
        }
        for (i, p) in points.iter().enumerate() {
            if p.is_tag || p.is_attachment {
                continue;
            }
            for t in &p.tags {
                if let Some(&ti) = tag_index.get(t) {
                    if ti != i {
                        edges.push((i, ti));
                    }
                }
            }
        }

        edges.sort_unstable();
        edges.dedup();

        let total_count = points.len();
        let mut new_coords = vec![];
        get_coords(&mut new_coords, total_count as i32);

        self.edges = edges;
        self.points = points;
        self.points_coord = new_coords;
        self.velocities = vec![Vec2::ZERO; total_count];

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
        self.hovered_node_index = None;
        self.selected_node_index = None;
        self.loaded = true;
        self.pending_fit = true;
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

        if !self.filter_query.trim().is_empty()
            && !self.matches_query(&self.filter_query, p)
        {
            return false;
        }
        true
    }

    pub fn simulate_physics(&mut self) {
        let dt = 0.016;
        let damping = 0.9;
        let max_speed = 200.0;
        let min_speed_threshold = 0.1;

        let repulsion_k = self.repel_force * 5.0;
        let center_k = self.center_force;
        let spring_k = self.link_force;
        let group_k = self.group_force;
        let tag_k = self.tag_force;
        let rest_len = if self.link_distance <= 0.0 {
            30.0
        } else {
            self.link_distance
        };

        let count = self.points_coord.len();
        if count == 0 {
            return;
        }

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
                if i == j || !self.is_visible(j) {
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

        // 3. Fuerzas de Enlaces (Springs) con distancia de reposo
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
            let dist = delta.length().max(0.01);

            let is_tag_conn = self.points[idx_a].is_tag || self.points[idx_b].is_tag;
            let current_k = if is_tag_conn {
                spring_k * tag_k
            } else {
                spring_k
            };

            let force = delta.normalized() * ((dist - rest_len) * current_k);

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

    /// Radio del nodo (en píxeles de pantalla). Replica el escalado por grado
    /// de Obsidian.
    pub fn node_radius(&self, index: usize) -> f32 {
        let point = &self.points[index];
        let degree = self.node_degrees[index] as f32;
        if point.is_tag {
            (self.node_size * 1.1 + degree.sqrt() * 1.5).min(self.node_size * 3.0)
        } else if point.is_attachment {
            self.node_size * 0.65
        } else if !point.exists {
            self.node_size * 0.8
        } else {
            (self.node_size * (1.0 + degree.sqrt() * 0.18)).min(self.node_size * 2.8)
        }
    }

    pub fn get_color_for_node(&self, point: &MarmolPoint) -> Color32 {
        if !point.exists {
            return self.ghost_color;
        }

        let mut matching_colors = vec![];

        for group in &self.custom_groups {
            if group.enabled && self.matches_query(&group.query, point) {
                matching_colors.push(group.color);
            }
        }
        if !self.new_group_query.is_empty() && self.matches_query(&self.new_group_query, point) {
            matching_colors.push(self.new_group_col);
        }
        if !matching_colors.is_empty() {
            return mix_colors(&matching_colors);
        }

        if point.is_attachment {
            return self.attachment_color;
        }

        if point.is_tag {
            if let Some(color) = self.tags_colors.get(&point.text) {
                return *color;
            }
            let mut hasher = DefaultHasher::new();
            point.text.hash(&mut hasher);
            return self.palette[(hasher.finish() as usize) % self.palette.len()];
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

// ---------------- Consultas ----------------

/// Parsea una consulta al estilo Obsidian en una lista de (tipo, valor).
/// Todos los términos deben cumplirse (AND).
pub fn parse_query(query: &str) -> Vec<(MatchType, String)> {
    let mut tokens: Vec<String> = vec![];
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in query.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
        .into_iter()
        .map(|token| {
            if let Some((key, value)) = token.split_once(':') {
                let mt = match key.to_lowercase().as_str() {
                    "file" | "filename" => MatchType::Filename,
                    "path" => MatchType::Path,
                    "tag" => MatchType::Tag,
                    "section" => MatchType::Section,
                    "line" | "content" | "block" => MatchType::Content,
                    _ => MatchType::Any,
                };
                // Si la clave no es un operador conocido, se busca el término
                // completo (incluyendo los dos puntos).
                if mt == MatchType::Any {
                    (MatchType::Any, token)
                } else {
                    (mt, value.to_string())
                }
            } else {
                (MatchType::Any, token)
            }
        })
        .collect()
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

/// Extrae los destinos de todos los wikilinks `[[...]]` del contenido.
/// Maneja alias (`|`), encabezados (`#`) y referencias de bloque (`^`).
pub fn extract_links(content: &str) -> Vec<String> {
    let mut out = vec![];
    let mut cursor = 0usize;
    while let Some(start) = content[cursor..].find("[[") {
        let s = cursor + start + 2;
        if let Some(end_rel) = content[s..].find("]]") {
            let inner = &content[s..s + end_rel];
            let target = inner.split('|').next().unwrap_or(inner);
            let target = target.split('#').next().unwrap_or(target);
            let target = target.split('^').next().unwrap_or(target);
            let target = target.trim();
            if !target.is_empty() {
                out.push(target.to_string());
            }
            cursor = s + end_rel + 2;
        } else {
            break;
        }
    }
    out
}

/// Extrae tags del frontmatter y del cuerpo (inline `#tag`).
pub fn extract_tags(content: &str, frontmatter_tags: &[String]) -> Vec<String> {
    let mut tags: Vec<String> = frontmatter_tags.to_vec();

    let body = strip_code_blocks(content);
    if let Ok(re) = Regex::new(r"(?:^|\s)#([A-Za-z0-9_][A-Za-z0-9_\-/]*)") {
        for cap in re.captures_iter(&body) {
            if let Some(m) = cap.get(1) {
                tags.push(m.as_str().to_string());
            }
        }
    }

    // Deduplicar preservando el orden.
    let mut seen = HashSet::new();
    tags.retain(|t| seen.insert(t.to_lowercase()));
    tags
}

/// Elimina bloques de código delimitados por ``` para no detectar tags falsos.
fn strip_code_blocks(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_fence = false;
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Presupuesto máximo de la caché de contenido del grafo (32 MiB).
const CONTENT_CACHE_BUDGET: usize = 32 * 1024 * 1024;

fn cache_content(
    cache: &mut HashMap<String, String>,
    cache_bytes: &mut usize,
    key: String,
    value: String,
) {
    let new_len = value.len();
    // Un archivo que por sí solo supera el presupuesto no se cachea.
    if new_len > CONTENT_CACHE_BUDGET {
        return;
    }
    if *cache_bytes + new_len > CONTENT_CACHE_BUDGET && !cache.is_empty() {
        cache.clear();
        *cache_bytes = 0;
    }
    if let Some(old) = cache.insert(key, value) {
        *cache_bytes = (*cache_bytes).saturating_sub(old.len());
    }
    *cache_bytes += new_len;
}

fn get_data(
    dir: &Path,
    marmol_vec: &mut Vec<MarmolPoint>,
    total_entries: &mut i32,
    vault: &str,
    content_cache: &mut HashMap<String, String>,
    content_cache_bytes: &mut usize,
) {
    if !Path::new(vault).exists() {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Ignorar directorios internos de Obsidian/Marmol.
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') {
                continue;
            }
            get_data(
                &path,
                marmol_vec,
                total_entries,
                vault,
                content_cache,
                content_cache_bytes,
            );
        } else if let Some(ext) = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
        {
            let abs_path = path.to_str().unwrap_or("").to_string();
            let rel_path = abs_path.replace(vault, "");
            let node_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            if ext == "md" {
                *total_entries += 1;
                let raw_content = files::read_file(&abs_path);

                let mut fm_tags: Vec<String> = vec![];
                if let Some(yaml_str) = extract_frontmatter(&raw_content) {
                    if let Ok(docs) = YamlLoader::load_from_str(&yaml_str) {
                        if !docs.is_empty() {
                            if let Some(tags) =
                                docs[0]["tags"].as_vec().or(docs[0]["Tags"].as_vec())
                            {
                                for tag in tags {
                                    if let Some(s) = tag.as_str() {
                                        fm_tags.push(s.trim_start_matches('#').to_owned());
                                    }
                                }
                            } else if let Some(s) =
                                docs[0]["tags"].as_str().or(docs[0]["Tags"].as_str())
                            {
                                for t in s.split([',', ' ', '\n']) {
                                    let t = t.trim().trim_start_matches('#');
                                    if !t.is_empty() {
                                        fm_tags.push(t.to_owned());
                                    }
                                }
                            }
                        }
                    }
                }

                let mut tag_vecs = extract_tags(&raw_content, &fm_tags);
                if tag_vecs.is_empty() {
                    tag_vecs.push("Orphan".to_owned());
                }

                let links_vec = extract_links(&raw_content);
                cache_content(
                    content_cache,
                    content_cache_bytes,
                    abs_path.clone(),
                    raw_content,
                );
                marmol_vec.push(MarmolPoint::new(
                    &node_name, tag_vecs, links_vec, rel_path, abs_path, false, false, true,
                ));
            } else if ATTACHMENT_EXTS.contains(&ext.as_str()) {
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

fn extract_frontmatter(content: &str) -> Option<String> {
    if let Some(rest) = content.strip_prefix("---") {
        if let Some(end_idx) = rest.find("\n---") {
            return Some(rest[..end_idx].to_string());
        }
    }
    None
}

fn get_coords(coords_vec: &mut Vec<(f32, f32)>, total_entries: i32) {
    let radio = 10.0;
    let var = std::f32::consts::TAU;
    for i in 0..total_entries {
        let a = (var / total_entries.max(1) as f32) * i as f32;
        coords_vec.push((radio * a.cos(), radio * a.sin()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_obsidian_query_operators() {
        let tokens = parse_query("file:nota path:\"mi carpeta\" tag:rust texto");
        assert_eq!(
            tokens,
            vec![
                (MatchType::Filename, "nota".to_string()),
                (MatchType::Path, "mi carpeta".to_string()),
                (MatchType::Tag, "rust".to_string()),
                (MatchType::Any, "texto".to_string()),
            ]
        );
    }

    #[test]
    fn unknown_operator_falls_back_to_any() {
        let tokens = parse_query("http://example.com");
        assert_eq!(
            tokens,
            vec![(MatchType::Any, "http://example.com".to_string())]
        );
    }

    #[test]
    fn extracts_links_with_alias_heading_and_block() {
        let content =
            "Ver [[Nota]] y [[Otra|Alias]] y [[Carpeta/Sub#Encabezado]] y [[N^bloque]].";
        assert_eq!(
            extract_links(content),
            vec![
                "Nota".to_string(),
                "Otra".to_string(),
                "Carpeta/Sub".to_string(),
                "N".to_string(),
            ]
        );
    }

    #[test]
    fn extracts_inline_tags_and_frontmatter() {
        let content = "# Encabezado\n\nTexto con #rust y #proyecto/sub.\n\n```\n#falso\n```";
        let tags = extract_tags(content, &["yaml".to_string()]);
        assert!(tags.contains(&"yaml".to_string()));
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"proyecto/sub".to_string()));
        assert!(!tags.contains(&"falso".to_string()));
    }
}
