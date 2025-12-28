use crate::emojis::emojis;
use eframe::egui;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

// --- ESTRUCTURAS DE CONFIGURACIÓN ---

#[derive(Clone)]
pub struct IconPack {
    pub name: &'static str,
    pub display_name: &'static str,
    pub internal_path: &'static str,
    pub download_link: &'static str,
}

pub fn get_predefined_packs() -> Vec<IconPack> {
    vec![
        IconPack {
            name: "font-awesome-brands",
            display_name: "FontAwesome Brands",
            internal_path: "fontawesome-free-6.5.1-web/svgs/brands/",
            download_link: "https://github.com/FortAwesome/Font-Awesome/releases/download/6.5.1/fontawesome-free-6.5.1-web.zip",
        },
        IconPack {
            name: "font-awesome-regular",
            display_name: "FontAwesome Regular",
            internal_path: "fontawesome-free-6.5.1-web/svgs/regular/",
            download_link: "https://github.com/FortAwesome/Font-Awesome/releases/download/6.5.1/fontawesome-free-6.5.1-web.zip",
        },
        IconPack {
            name: "font-awesome-solid",
            display_name: "FontAwesome Solid",
            internal_path: "fontawesome-free-6.5.1-web/svgs/solid/",
            download_link: "https://github.com/FortAwesome/Font-Awesome/releases/download/6.5.1/fontawesome-free-6.5.1-web.zip",
        },
        IconPack {
            name: "remix-icons",
            display_name: "Remix Icons",
            internal_path: "",
            download_link: "https://github.com/Remix-Design/RemixIcon/releases/download/v4.2.0/RemixIcon_Svg_v4.2.0.zip",
        },
        IconPack {
            name: "icon-brew",
            display_name: "Icon Brew",
            internal_path: "",
            download_link: "https://github.com/FlorianWoelki/obsidian-iconize/raw/main/iconPacks/icon-brew.zip",
        },
        IconPack {
            name: "simple-icons",
            display_name: "Simple Icons",
            internal_path: "simple-icons-11.10.0/icons/",
            download_link: "https://github.com/simple-icons/simple-icons/archive/refs/tags/11.10.0.zip",
        },
        IconPack {
            name: "lucide-icons",
            display_name: "Lucide",
            internal_path: "",
            download_link: "https://github.com/lucide-icons/lucide/releases/download/0.363.0/lucide-icons-0.363.0.zip",
        },
        IconPack {
            name: "tabler-icons",
            display_name: "Tabler Icons",
            internal_path: "svg",
            download_link: "https://github.com/tabler/tabler-icons/releases/download/v3.1.0/tabler-icons-3.1.0.zip",
        },
        IconPack {
            name: "boxicons",
            display_name: "Boxicons",
            internal_path: "svg",
            download_link: "https://github.com/FlorianWoelki/obsidian-iconize/raw/main/iconPacks/boxicons.zip",
        },
        IconPack {
            name: "rpg-awesome",
            display_name: "RPG Awesome",
            internal_path: "",
            download_link: "https://github.com/FlorianWoelki/obsidian-iconize/raw/main/iconPacks/rpg-awesome.zip",
        },
        IconPack {
            name: "coolicons",
            display_name: "Coolicons",
            internal_path: "cooliocns SVG",
            download_link: "https://github.com/krystonschwarze/coolicons/releases/download/v4.1/coolicons.v4.1.zip",
        },
        IconPack {
            name: "feather-icons",
            display_name: "Feather Icons",
            internal_path: "feather-4.29.1/icons/",
            download_link: "https://github.com/feathericons/feather/archive/refs/tags/v4.29.1.zip",
        },
        IconPack {
            name: "octicons",
            display_name: "Octicons",
            internal_path: "octicons-19.8.0/icons/",
            download_link: "https://github.com/primer/octicons/archive/refs/tags/v19.8.0.zip",
        },
    ]
}

pub struct IconPackInstaller {
    pub is_open: bool,
    pub packs: Vec<IconPack>,
    pub status_message: String,
    pub is_downloading: bool,
}

impl Default for IconPackInstaller {
    fn default() -> Self {
        Self {
            is_open: false,
            packs: get_predefined_packs(),
            status_message: String::new(),
            is_downloading: false,
        }
    }
}

impl IconPackInstaller {
    pub fn ui(&mut self, ctx: &egui::Context, vault_path: &str, icon_manager: &mut IconManager) {
        if !self.is_open {
            return;
        }

        let mut pack_to_install = None;

        egui::Window::new("Install Icon Pack")
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .fixed_size([400.0, 500.0])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Select an icon pack to download and install:");
                ui.add_space(10.0);

                if self.is_downloading {
                    ui.spinner();
                    ui.label("Downloading and installing...");
                } else {
                    egui::ScrollArea::vertical().max_height(350.0).show(ui, |ui| {
                        for pack in &self.packs {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(pack.display_name).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Install").clicked() {
                                        pack_to_install = Some(pack.clone());
                                    }
                                });
                            });
                            ui.separator();
                        }
                    });
                }

                if !self.status_message.is_empty() {
                    ui.add_space(10.0);
                    ui.label(&self.status_message);
                }

                ui.add_space(10.0);
                if ui.button("Close").clicked() {
                    self.is_open = false;
                }
            });

        if let Some(pack) = pack_to_install {
            self.is_downloading = true;
            self.status_message = format!("Downloading {}...", pack.display_name);
            
            // Trigger repaint to show spinner
            ctx.request_repaint();

            // Perform download in a blocking way (for simplicity, ideally async/thread)
            // Since we are in immediate mode GUI, blocking the main thread is bad.
            // But for this prototype we'll do it or spawn a thread if possible.
            // Let's spawn a thread to not freeze the UI entirely, but we need to handle state.
            // For safety and simplicity in this context, we'll block briefly or use a channel.
            // BUT eframe/egui is single threaded mostly. 
            // We will do a blocking call here for now, accepting the freeze. 
            // Better approach: Use std::thread::spawn and a channel, but that requires changing struct to hold receiver.
            
            // Let's try blocking for now to ensure it works.
            match self.install_pack(vault_path, &pack) {
                Ok(_) => {
                    self.status_message = format!("Successfully installed {}!", pack.display_name);
                    icon_manager.load_icons(vault_path); // Reload icons
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                }
            }
            self.is_downloading = false;
        }
    }

    fn install_pack(&self, vault_path: &str, pack: &IconPack) -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(pack.download_link)?.bytes()?;
        let reader = Cursor::new(resp);
        let mut archive = ZipArchive::new(reader)?;

        let icons_path = Path::new(vault_path).join(".obsidian/icons").join(pack.name);
        fs::create_dir_all(&icons_path)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name().to_string();

            if file.is_dir() {
                continue;
            }

            // Check if file matches internal path prefix
            if !pack.internal_path.is_empty() && !file_name.starts_with(pack.internal_path) {
                continue;
            }

            // Extract filename
            let extracted_name = if !pack.internal_path.is_empty() {
                file_name.strip_prefix(pack.internal_path).unwrap_or(&file_name).to_string()
            } else {
                // If no specific prefix, we might want to flatten or keep structure.
                // The original logic seemed to flatten or just take the file.
                // Let's just take the file name part to flatten it into the pack folder.
                Path::new(&file_name).file_name().unwrap().to_string_lossy().to_string()
            };
            
            if extracted_name.is_empty() || extracted_name.contains('/') || extracted_name.contains('\\') {
                 // Skip nested folders if we just want flat icons, OR handle nested creation.
                 // The predefined packs usually have a structure. 
                 // Obsidian Iconize usually flattens specific paths.
                 // Let's try to just write the file to the pack folder using the filename only.
                 // But wait, some packs have subfolders.
                 // Let's stick to simple flattening for now: only SVGs.
                 if !extracted_name.ends_with(".svg") {
                     continue;
                 }
                 
                 let target_path = icons_path.join(Path::new(&file_name).file_name().unwrap());
                 let mut outfile = fs::File::create(&target_path)?;
                 std::io::copy(&mut file, &mut outfile)?;
            } else {
                 if !extracted_name.ends_with(".svg") {
                     continue;
                 }
                 let target_path = icons_path.join(&extracted_name);
                 let mut outfile = fs::File::create(&target_path)?;
                 std::io::copy(&mut file, &mut outfile)?;
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ExtraMargin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Default for ExtraMargin {
    fn default() -> Self {
        Self {
            top: 0,
            right: 4,
            bottom: 0,
            left: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IconSettings {
    pub migrated: i32,
    pub icon_packs_path: String,
    pub font_size: f32,
    pub emoji_style: String,
    pub icon_color: Option<String>,
    pub recently_used_icons: Vec<String>,
    pub recently_used_icons_size: usize,
    pub rules: Vec<Value>,
    pub extra_margin: ExtraMargin,
    pub icon_in_tabs_enabled: bool,
    pub icon_in_title_enabled: bool,
    pub icon_in_title_position: String,
    pub icon_in_frontmatter_enabled: bool,
    pub icon_in_frontmatter_field_name: String,
    pub icon_color_in_frontmatter_field_name: String,
    pub icons_background_check_enabled: bool,
    pub icons_in_notes_enabled: bool,
    pub icons_in_links_enabled: bool,
    pub icon_identifier: String,
    pub lucide_icon_pack_type: String,
    pub debug_mode: bool,
    pub use_internal_plugins: bool,
}

impl Default for IconSettings {
    fn default() -> Self {
        Self {
            migrated: 6,
            icon_packs_path: ".obsidian/icons".to_string(),
            font_size: 16.0,
            emoji_style: "native".to_string(),
            icon_color: None,
            recently_used_icons: vec![],
            recently_used_icons_size: 5,
            rules: vec![],
            extra_margin: ExtraMargin::default(),
            icon_in_tabs_enabled: false,
            icon_in_title_enabled: false,
            icon_in_title_position: "above".to_string(),
            icon_in_frontmatter_enabled: false,
            icon_in_frontmatter_field_name: "icon".to_string(),
            icon_color_in_frontmatter_field_name: "iconColor".to_string(),
            icons_background_check_enabled: false,
            icons_in_notes_enabled: true,
            icons_in_links_enabled: true,
            icon_identifier: ":".to_string(),
            lucide_icon_pack_type: "native".to_string(),
            debug_mode: false,
            use_internal_plugins: false,
        }
    }
}

// --- GESTOR DE ICONOS ---

#[derive(Clone)]
pub enum IconSource {
    Url(String),
    Bytes(Vec<u8>),
}

pub struct IconManager {
    pub icons: HashMap<String, String>,
    pub settings: IconSettings,
    pub svg_cache: HashMap<String, PathBuf>,
    pub legacy_mappings: HashMap<String, String>,
    pub app_assets_path: PathBuf,
}

impl IconManager {
    pub fn new() -> Self {
        let app_assets_path = PathBuf::from("assets/icons");

        // Mapeos para corregir diferencias de nombres (versiones viejas vs nuevas)
        let mut legacy_mappings = HashMap::new();
        legacy_mappings.insert("LiAlertTriangle".to_string(), "LiTriangleAlert".to_string());
        legacy_mappings.insert("LiEdit".to_string(), "LiPencil".to_string());

        Self {
            icons: HashMap::new(),
            settings: IconSettings::default(),
            svg_cache: HashMap::new(),
            legacy_mappings,
            app_assets_path,
        }
    }

    pub fn load_icons(&mut self, vault_path: &str) {
        self.icons.clear();
        self.svg_cache.clear();

        let config_path =
            Path::new(vault_path).join(".obsidian/plugins/obsidian-icon-folder/data.json");

        println!("Iconize: Looking for config at {:?}", config_path);

        if config_path.exists() {
            println!("Iconize: Config file found!");
            if let Ok(data) = fs::read_to_string(config_path) {
                if let Ok(json_parsed) = serde_json::from_str::<Value>(&data) {
                    if let Value::Object(map) = json_parsed {
                        if let Some(settings_val) = map.get("settings") {
                            if let Ok(s) = serde_json::from_value(settings_val.clone()) {
                                self.settings = s;
                            }
                        }
                        for (key, value) in map {
                            if key != "settings" {
                                if let Some(icon_str) = value.as_str() {
                                    self.icons.insert(key, icon_str.to_string());
                                }
                            }
                        }
                        println!("Iconize: Loaded {} icons from config", self.icons.len());
                    }
                }
            }
        } else {
            println!("Iconize: Config file NOT found at {:?}", config_path);
        }

        if self.app_assets_path.exists() {
            self.scan_directory(&self.app_assets_path.clone(), None);
            println!("Iconize: Scanned {} SVGs in assets", self.svg_cache.len());
        }

        // Scan downloaded icons in .obsidian/icons
        let vault_icons_path = Path::new(vault_path).join(".obsidian/icons");
        if vault_icons_path.exists() {
            println!("Iconize: Scanning vault icons at {:?}", vault_icons_path);
            self.scan_directory(&vault_icons_path, None);
        }
    }

    fn scan_directory(&mut self, dir: &Path, prefix_override: Option<&str>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path.file_name().unwrap().to_str().unwrap().to_lowercase();
                    let detected_prefix = match folder_name.as_str() {
                        "lucide" | "lucide-icons" => Some("Li"),
                        "simple-icons" => Some("Si"),
                        "icon-brew" => Some("Ib"),
                        "remix-icons" | "remix" => Some("Ri"),
                        "fa-brands" | "fa-regular" | "fa-solid" | "font-awesome-brands" | "font-awesome-regular" | "font-awesome-solid" => Some("Fa"),
                        "tabler-icons" => Some("Ti"),
                        "boxicons" => Some("Bi"),
                        "rpg-awesome" => Some("Ra"),
                        "coolicons" => Some("Co"),
                        "feather-icons" => Some("Fi"),
                        "octicons" => Some("Oc"),
                        _ => None,
                    };
                    let final_prefix = detected_prefix.or(prefix_override);
                    self.scan_directory(&path, final_prefix);
                } else if let Some(ext) = path.extension() {
                    if ext == "svg" {
                        self.register_svg(&path, prefix_override);
                    }
                }
            }
        }
    }

    fn register_svg(&mut self, path: &Path, prefix: Option<&str>) {
        if let Some(stem) = path.file_stem() {
            let raw_name = stem.to_string_lossy().to_string();
            let pascal_name = self.kebab_to_pascal(&raw_name);
            let final_id = if let Some(p) = prefix {
                format!("{}{}", p, pascal_name)
            } else {
                raw_name.clone()
            };
            self.svg_cache.insert(final_id, path.to_path_buf());
        }
    }

    fn kebab_to_pascal(&self, s: &str) -> String {
        s.split(|c| c == '-' || c == '_' || c == ' ')
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut c = word.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect()
    }

    pub fn get_icon(&self, relative_path: &str) -> Option<&String> {
        self.icons.get(relative_path)
    }

    pub fn get_icon_source(&self, icon_name: &str) -> Option<IconSource> {
        if let Some(path) = self.svg_cache.get(icon_name) {
            return self.load_svg_bytes(path);
        }

        // Legacy fallback
        if let Some(new_name) = self.legacy_mappings.get(icon_name) {
            if let Some(path) = self.svg_cache.get(new_name) {
                return self.load_svg_bytes(path);
            }
        }
        None
    }

    fn load_svg_bytes(&self, path: &PathBuf) -> Option<IconSource> {
        if let Ok(content) = fs::read_to_string(path) {
            // SOLUCIÓN TRIÁNGULOS ROJOS:
            // Solo reemplazamos colores. NO inyectamos atributos en el tag <svg>
            // porque eso rompe archivos que ya tienen esos atributos.
            let whitened = content
                .replace("currentColor", "white")
                .replace("#000000", "white")
                .replace("#000", "white")
                .replace("black", "white");

            return Some(IconSource::Bytes(whitened.into_bytes()));
        }
        None
    }

    pub fn get_icon_path(&self, icon_name: &str) -> Option<String> {
        if let Some(p) = self.svg_cache.get(icon_name) {
            return Some(format!("file://{}", p.to_string_lossy()));
        }
        if let Some(new_name) = self.legacy_mappings.get(icon_name) {
            if let Some(p) = self.svg_cache.get(new_name) {
                return Some(format!("file://{}", p.to_string_lossy()));
            }
        }
        None
    }

    pub fn save_settings(&self, vault_path: &str) {
        let config_path =
            Path::new(vault_path).join(".obsidian/plugins/obsidian-icon-folder/data.json");

        let mut json_val: Value = if config_path.exists() {
            let data = fs::read_to_string(&config_path).unwrap_or_else(|_| "{}".to_string());
            serde_json::from_str(&data).unwrap_or(Value::Object(Map::new()))
        } else {
            Value::Object(Map::new())
        };

        if let Value::Object(ref mut map) = json_val {
            if let Ok(settings_json) = serde_json::to_value(&self.settings) {
                map.insert("settings".to_string(), settings_json);
            }
        }

        if let Ok(serialized) = serde_json::to_string_pretty(&json_val) {
            let _ = fs::write(config_path, serialized);
        }
    }

    pub fn rename_icon(&mut self, vault_path: &str, old_rel_path: &str, new_rel_path: &str) {
        if let Some(icon) = self.icons.remove(old_rel_path) {
            self.icons.insert(new_rel_path.to_string(), icon.clone());

            let config_path =
                Path::new(vault_path).join(".obsidian/plugins/obsidian-icon-folder/data.json");

            let mut json_val: Value = if config_path.exists() {
                let data = fs::read_to_string(&config_path).unwrap_or_else(|_| "{}".to_string());
                serde_json::from_str(&data).unwrap_or(Value::Object(Map::new()))
            } else {
                Value::Object(Map::new())
            };

            if let Value::Object(ref mut map) = json_val {
                map.remove(old_rel_path);
                map.insert(new_rel_path.to_string(), Value::String(icon));
            }

            if let Ok(serialized) = serde_json::to_string_pretty(&json_val) {
                let _ = fs::write(config_path, serialized);
            }
        }
    }
}

// --- SELECTOR DE ICONOS ---

pub struct IconSelector {
    pub is_open: bool,
    target_path: String,
    query: String,
    all_icons: Vec<String>,
    filtered_results: Vec<(String, String)>,
    selected_index: usize,
    initialized: bool,
    // Lista de emojis que TÚ llenarás
    emoji_map: Vec<(&'static str, &'static str)>,
}

impl Default for IconSelector {
    fn default() -> Self {
        Self {
            is_open: false,
            target_path: String::new(),
            query: String::new(),
            all_icons: Vec::new(),
            filtered_results: Vec::new(),
            selected_index: 0,
            initialized: false,
            emoji_map: emojis(),
        }
    }
}

impl IconSelector {
    pub fn open(&mut self, relative_path: String, icon_manager: &IconManager) {
        self.is_open = true;
        self.target_path = relative_path;
        self.query.clear();
        self.selected_index = 0;
        self.initialized = true;

        self.all_icons.clear();

        // Agregar solo SVGs (los emojis se buscan dinámicamente desde emoji_map)
        for key in icon_manager.svg_cache.keys() {
            self.all_icons.push(key.clone());
        }
        self.all_icons.sort();

        self.update_filter();
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    fn update_filter(&mut self) {
        self.filtered_results.clear();
        let limit = 200;
        let q = self.query.to_lowercase();
        let empty_query = q.is_empty();

        // 1. Buscar en SVGs
        for icon_id in &self.all_icons {
            if self.filtered_results.len() >= limit {
                break;
            }
            if empty_query || icon_id.to_lowercase().contains(&q) {
                self.filtered_results
                    .push((icon_id.clone(), icon_id.clone()));
            }
        }

        // 2. Buscar en Emojis
        if self.filtered_results.len() < limit {
            for (char, name) in &self.emoji_map {
                if self.filtered_results.len() >= limit {
                    break;
                }
                if empty_query || name.contains(&q) || char.contains(&q) {
                    self.filtered_results
                        .push((char.to_string(), name.to_string()));
                }
            }
        }

        if self.selected_index >= self.filtered_results.len() {
            self.selected_index = 0;
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, vault_path: &str, icon_manager: &mut IconManager) {
        if !self.is_open {
            return;
        }

        let mut selected_icon: Option<String> = None;
        let mut remove_icon = false;

        egui::Window::new("Icon Switcher")
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .fixed_size([550.0, 450.0])
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                // --- TOP PANEL (Header) ---
                ui.vertical(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Select Icon for: {}", self.target_path))
                                .strong(),
                        );
                    });
                    ui.add_space(8.0);

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Search icons...")
                            .lock_focus(true)
                            .desired_width(f32::INFINITY),
                    );

                    if self.initialized {
                        response.request_focus();
                        self.initialized = false;
                    }

                    if response.changed() {
                        self.update_filter();
                        self.selected_index = 0;
                    }

                    // Navegación teclado
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        if self.selected_index + 1 < self.filtered_results.len() {
                            self.selected_index += 1;
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if !self.filtered_results.is_empty() {
                            selected_icon =
                                Some(self.filtered_results[self.selected_index].0.clone());
                        } else if !self.query.is_empty() {
                            selected_icon = Some(self.query.clone());
                        }
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.close();
                    }
                    ui.add_space(8.0);
                });

                // --- CALCULO DE LAYOUT (Scroll Fix) ---
                // Calculamos cuánto espacio necesitamos abajo para los botones
                let footer_height = 40.0;
                // El espacio que queda es todo para el scroll
                let available_height = ui.available_height() - footer_height;
                let text_color = ui.visuals().text_color();

                // --- MIDDLE PANEL (Scroll) ---
                egui::ScrollArea::vertical()
                    .max_height(300.0) // Fixed height to ensure scrolling works
                    .show(ui, |ui| {
                        egui::Grid::new("icons_grid_real")
                            .num_columns(2)
                            .min_col_width(240.0)
                            .spacing([10.0, 10.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for (i, (icon_id, description)) in
                                    self.filtered_results.iter().enumerate()
                                {
                                    let is_selected = i == self.selected_index;

                                    ui.push_id(i, |ui| {
                                        let is_svg = icon_manager.svg_cache.contains_key(icon_id);

                                        // Dibujo manual de la celda
                                        let desired_size = egui::vec2(ui.available_width(), 24.0);
                                        let (rect, response) = ui.allocate_exact_size(
                                            desired_size,
                                            egui::Sense::click(),
                                        );

                                        if ui.is_rect_visible(rect) {
                                            if is_selected || response.hovered() {
                                                let bg = if is_selected {
                                                    ui.style().visuals.selection.bg_fill
                                                } else {
                                                    ui.style().visuals.widgets.hovered.bg_fill
                                                };
                                                ui.painter().rect_filled(rect, 4.0, bg);
                                            }

                                            let content_rect = rect.shrink(4.0);

                                            ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
                                                ui.horizontal_centered(|ui| {
                                                    // ICONO
                                                    if is_svg {
                                                        if let Some(IconSource::Bytes(bytes)) =
                                                            icon_manager.get_icon_source(icon_id)
                                                        {
                                                            let tint = if is_selected {
                                                                ui.style()
                                                                    .visuals
                                                                    .selection
                                                                    .stroke
                                                                    .color
                                                            } else {
                                                                text_color
                                                            };

                                                            // URI único para evitar caché cruzado erróneo: bytes://NOMBRE.svg
                                                            ui.add(
                                                                egui::Image::from_bytes(
                                                                    format!(
                                                                        "bytes://{}.svg",
                                                                        icon_id
                                                                    ),
                                                                    bytes,
                                                                )
                                                                .tint(tint)
                                                                .fit_to_exact_size(egui::vec2(
                                                                    18.0, 18.0,
                                                                )),
                                                            );
                                                        }
                                                    } else {
                                                        ui.label(
                                                            egui::RichText::new(icon_id).size(18.0),
                                                        );
                                                    }

                                                    ui.add_space(8.0);

                                                    // TEXTO
                                                    let txt_col = if is_selected {
                                                        ui.style().visuals.selection.stroke.color
                                                    } else {
                                                        text_color
                                                    };
                                                    
                                                    let label_text = if is_svg {
                                                        if let Some(path) = icon_manager.svg_cache.get(icon_id) {
                                                            if let Some(parent) = path.parent() {
                                                                if let Some(pack_name) = parent.file_name() {
                                                                    format!("{} ({})", icon_id, pack_name.to_string_lossy())
                                                                } else {
                                                                    icon_id.clone()
                                                                }
                                                            } else {
                                                                icon_id.clone()
                                                            }
                                                        } else {
                                                            icon_id.clone()
                                                        }
                                                    } else {
                                                        description.clone()
                                                    };

                                                    ui.label(
                                                        egui::RichText::new(label_text)
                                                            .color(txt_col),
                                                    );
                                                });
                                            });
                                        }

                                        if response.clicked() {
                                            selected_icon = Some(icon_id.clone());
                                        }
                                    });

                                    if (i + 1) % 2 == 0 {
                                        ui.end_row();
                                    }
                                }
                            });

                        if self.filtered_results.is_empty() {
                            ui.label("No icons found.");
                        }
                    });

                // --- BOTTOM PANEL (Footer) ---
                // Esto se empujará al fondo gracias al cálculo de altura anterior
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new("Remove Icon")
                                    .color(ui.visuals().error_fg_color),
                            )
                            .clicked()
                        {
                            remove_icon = true;
                        }
                        if ui.button("Cancel").clicked() {
                            self.close();
                        }
                    });
                    ui.separator();
                });
            });

        ctx.move_to_top(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("Icon Switcher"),
        ));

        if remove_icon {
            self.save_icon(vault_path, &self.target_path, "", icon_manager);
            self.close();
        } else if let Some(icon) = selected_icon {
            self.save_icon(vault_path, &self.target_path, &icon, icon_manager);
            self.close();
        }
    }

    fn save_icon(
        &self,
        vault_path: &str,
        relative_path: &str,
        icon: &str,
        icon_manager: &mut IconManager,
    ) {
        let config_path =
            Path::new(vault_path).join(".obsidian/plugins/obsidian-icon-folder/data.json");

        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut json_val: Value = if config_path.exists() {
            let data = fs::read_to_string(&config_path).unwrap_or_else(|_| "{}".to_string());
            serde_json::from_str(&data).unwrap_or(Value::Object(Map::new()))
        } else {
            Value::Object(Map::new())
        };

        if let Value::Object(ref mut map) = json_val {
            if let Ok(settings_json) = serde_json::to_value(&icon_manager.settings) {
                map.insert("settings".to_string(), settings_json);
            }

            if icon.is_empty() {
                map.remove(relative_path);
                icon_manager.icons.remove(relative_path);
            } else {
                map.insert(relative_path.to_string(), Value::String(icon.to_string()));
                icon_manager
                    .icons
                    .insert(relative_path.to_string(), icon.to_string());
            }
        }

        if let Ok(serialized) = serde_json::to_string_pretty(&json_val) {
            let _ = fs::write(config_path, serialized);
        }
    }
}
