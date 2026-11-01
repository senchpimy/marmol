use arboard::Clipboard;
use egui::{text::CCursorRange, Key, KeyboardShortcut, Modifiers, TextBuffer, TextEdit, Ui};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct EasyMarkEditor {
    pub code: String,
    pub show_rendered: bool,
    pub show_hotkey_editor: bool,

    pub shortcut_bold: KeyboardShortcut,
    pub shortcut_code: KeyboardShortcut,
    pub shortcut_italics: KeyboardShortcut,

    pub vault_path: String,
    pub file_path: String,

    pub collapsed_blocks: HashSet<usize>,

    #[serde(skip)]
    highlighter: super::MemoizedEasymarkHighlighter,

    #[serde(skip)]
    latex_preview_cache: Option<(String, Result<crate::latex_math::MathBox, String>)>,

    #[serde(skip)]
    wiki_files: Vec<String>,
    #[serde(skip)]
    wiki_scanned_vault: String,
    #[serde(skip)]
    wiki_open: bool,
    #[serde(skip)]
    wiki_query: String,
    #[serde(skip)]
    wiki_open_byte: usize,
    #[serde(skip)]
    wiki_close_byte: Option<usize>,
    #[serde(skip)]
    wiki_cursor_byte: usize,
    #[serde(skip)]
    wiki_selected: usize,
    #[serde(skip)]
    wiki_scroll: bool,
    #[serde(skip)]
    pending_cursor: Option<usize>,
}

impl PartialEq for EasyMarkEditor {
    fn eq(&self, other: &Self) -> bool {
        (
            &self.code,
            self.show_rendered,
            self.show_hotkey_editor,
            &self.collapsed_blocks,
        ) == (
            &other.code,
            other.show_rendered,
            other.show_hotkey_editor,
            &other.collapsed_blocks,
        ) && self.shortcut_bold == other.shortcut_bold
            && self.shortcut_code == other.shortcut_code
            && self.shortcut_italics == other.shortcut_italics
    }
}

impl Default for EasyMarkEditor {
    fn default() -> Self {
        Self {
            code: String::new(),
            show_rendered: false,
            show_hotkey_editor: false,
            shortcut_bold: SHORTCUT_BOLD,
            shortcut_code: SHORTCUT_CODE,
            shortcut_italics: SHORTCUT_ITALICS,
            vault_path: String::new(),
            file_path: String::new(),
            collapsed_blocks: HashSet::new(),
            highlighter: Default::default(),
            latex_preview_cache: None,
            wiki_files: Vec::new(),
            wiki_scanned_vault: String::new(),
            wiki_open: false,
            wiki_query: String::new(),
            wiki_open_byte: 0,
            wiki_close_byte: None,
            wiki_cursor_byte: 0,
            wiki_selected: 0,
            wiki_scroll: false,
            pending_cursor: None,
        }
    }
}

impl EasyMarkEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed_programmatically = false;

        // Autocompletado de wikilinks: procesar teclas antes del TextEdit para
        // que Enter/Flechas/Tab no lleguen al editor.
        if self.wiki_manejar_teclas(ui) {
            changed_programmatically = true;
        }

        // Handle Ctrl+V before TextEdit. Solo consultar el portapapeles cuando
        // realmente se presiona el atajo (antes se hacía cada frame).
        let paste_pressed = ui.input(|i| i.modifiers.command && i.key_pressed(Key::V));
        if paste_pressed {
            if self.has_image_in_clipboard() {
                let ctrl_v = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::V);
                let cmd_v = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::V);
                ui.input_mut(|i| {
                    i.consume_shortcut(&ctrl_v);
                    i.consume_shortcut(&cmd_v);
                });
                if self.try_paste_image(ui.ctx()) {
                    changed_programmatically = true;
                }
            }
        }

        // --- BARRA DE HERRAMIENTAS FIJA ---
        ui.horizontal(|ui| {
            if ui
                .button("📋")
                .on_hover_text("Paste image from clipboard")
                .clicked()
            {
                if self.try_paste_image(ui.ctx()) {
                    changed_programmatically = true;
                }
            }
            if ui.button("Hotkeys").clicked() {
                self.show_hotkey_editor = !self.show_hotkey_editor;
            }

            // Botón para colapsar el bloque actual
            if ui
                .button("⮩")
                .on_hover_text("Toggle folding of code block at cursor")
                .clicked()
            {
                let text = self.code.as_str();
                let editor_id = ui.id().with("easymark_edit");
                if let Some(state) = TextEdit::load_state(ui.ctx(), editor_id) {
                    if let Some(range) = state.cursor.char_range() {
                        let cursor_idx: usize = range.primary.index.into();

                        let mut count_before = 0;
                        let mut temp_text = &text[..cursor_idx];
                        let mut found_start = None;
                        while let Some(pos) = temp_text.rfind("```") {
                            if pos == 0 || temp_text.get(pos - 1..pos) == Some("\n") {
                                count_before += 1;
                                if count_before % 2 != 0 {
                                    found_start = Some(pos);
                                    break;
                                }
                            }
                            temp_text = &temp_text[..pos];
                        }

                        if let Some(start_pos) = found_start {
                            if self.collapsed_blocks.contains(&start_pos) {
                                self.collapsed_blocks.remove(&start_pos);
                            } else {
                                self.collapsed_blocks.insert(start_pos);
                            }
                            changed_programmatically = true;
                        }
                    }
                }
            }

            // Botón para colapsar/expandir TODO
            if ui
                .button("↕")
                .on_hover_text("Toggle all code blocks")
                .clicked()
            {
                if self.collapsed_blocks.is_empty() {
                    let mut text = self.code.as_str();
                    let mut idx = 0;
                    while !text.is_empty() {
                        if text.starts_with("```") {
                            self.collapsed_blocks.insert(idx);
                            let end = text.find("\n```").map_or_else(|| text.len(), |i| i + 4);
                            text = &text[end..];
                            idx += end;
                        } else {
                            let next_line = text.find('\n').map_or(text.len(), |i| i + 1);
                            text = &text[next_line..];
                            idx += next_line;
                        }
                    }
                } else {
                    self.collapsed_blocks.clear();
                }
                changed_programmatically = true;
            }

            // Abrir pestaña Sketch (escribir a mano → LaTeX)
            if ui
                .button("Latex")
                .on_hover_text("Abrir Sketch: escribir a mano y convertir a LaTeX")
                .clicked()
            {
                let editor_id = ui.id().with("easymark_edit");
                let cursor_index = TextEdit::load_state(ui.ctx(), editor_id)
                    .and_then(|state| state.cursor.char_range())
                    .map(|range| usize::from(range.primary.index))
                    .unwrap_or_else(|| self.code.len());
                ui.ctx().data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new("open_sketch_signal"),
                        Some((self.file_path.clone(), cursor_index)),
                    )
                });
            }
        });

        ui.separator();

        if self.show_hotkey_editor {
            egui::Window::new("Hotkey Editor")
                .open(&mut self.show_hotkey_editor)
                .show(ui.ctx(), |ui| {
                    egui::Grid::new("hotkey_grid").show(ui, |ui| {
                        ui.label("Bold:");
                        shortcut_ui(ui, &mut self.shortcut_bold);
                        ui.end_row();

                        ui.label("Code:");
                        shortcut_ui(ui, &mut self.shortcut_code);
                        ui.end_row();

                        ui.label("Italics:");
                        shortcut_ui(ui, &mut self.shortcut_italics);
                        ui.end_row();
                    });
                });
        }

        // --- ÁREA DE EDICIÓN CON SCROLL PROPIO ---
        let editor_id = ui.id().with("easymark_edit");
        let cursor_index = TextEdit::load_state(ui.ctx(), editor_id)
            .and_then(|state| state.cursor.char_range())
            .map(|range| usize::from(range.primary.index));

        let mut galley_out = None;
        let mut preview_anchor: Option<(egui::Rect, usize)> = None;
        {
            let highlighter = &mut self.highlighter;
            let collapsed_blocks = &self.collapsed_blocks;
            let mut layouter = |ui: &egui::Ui, easymark: &dyn TextBuffer, wrap_width: f32| {
                let mut layout_job = highlighter.highlight(
                    ui.style(),
                    easymark.as_str(),
                    cursor_index,
                    collapsed_blocks,
                );
                layout_job.wrap.max_width = wrap_width;
                let galley = ui.fonts_mut(|f| f.layout_job(layout_job));
                galley_out = Some(galley.clone());
                galley
            };

            ui.spacing_mut().item_spacing.y = 0.0;

            let mut response = ui.add(
                egui::TextEdit::multiline(&mut self.code)
                    .id(editor_id)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace)
                    .layouter(&mut layouter)
                    .frame(egui::Frame::NONE),
            );

            // Reposicionar el cursor tras una inserción programática (wikilink).
            if let Some(byte) = self.pending_cursor.take() {
                let char_idx = byte_a_char(&self.code, byte);
                if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
                    state
                        .cursor
                        .set_char_range(Some(CCursorRange::one(egui::text::CCursor::new(
                            char_idx,
                        ))));
                    state.store(ui.ctx(), response.id);
                    response.mark_changed();
                }
            }

            if let Some(galley) = galley_out {
                if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
                    if let Some(mut ccursor_range) = state.cursor.char_range() {
                        let any_change = shortcuts(ui, self, &mut ccursor_range);

                        let cursor_pos_rect = galley.pos_from_cursor(ccursor_range.primary);
                        let cursor_rect = cursor_pos_rect.translate(response.rect.min.to_vec2());
                        preview_anchor =
                            Some((cursor_rect, usize::from(ccursor_range.primary.index)));

                        if response.changed() || any_change {
                            ui.scroll_to_rect(cursor_rect, Some(egui::Align::Center));
                        }

                        if any_change {
                            state.cursor.set_char_range(Some(ccursor_range));
                            state.store(ui.ctx(), response.id);
                            response.mark_changed();
                        }
                    }
                }
            }

            if self.wiki_actualizar_y_mostrar(ui, &response, preview_anchor) {
                changed_programmatically = true;
            }

            self.mostrar_preview_latex(ui, &response, preview_anchor);

            if changed_programmatically {
                response.mark_changed();
            }

            response
        }
    }

    /// Detecta si el cursor está dentro de un wikilink `[[...` y, en ese caso,
    /// muestra el popup de autocompletado con los archivos del vault.
    fn wiki_actualizar_y_mostrar(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        anchor: Option<(egui::Rect, usize)>,
    ) -> bool {
        if !response.has_focus() {
            self.wiki_open = false;
            return false;
        }
        let Some((cursor_rect, char_idx)) = anchor else {
            self.wiki_open = false;
            return false;
        };

        let cursor_byte = char_a_byte(&self.code, char_idx);
        self.wiki_actualizar(cursor_byte);
        if !self.wiki_open {
            return false;
        }

        let resultados = self.wiki_resultados();
        if resultados.is_empty() {
            return false;
        }
        self.wiki_selected = self.wiki_selected.min(resultados.len() - 1);

        let pos = egui::pos2(cursor_rect.left(), cursor_rect.bottom() + 4.0);
        let mut elegido: Option<String> = None;
        egui::Popup::new(
            egui::Id::new("easymark_wiki_autocomplete"),
            ui.ctx().clone(),
            egui::PopupAnchor::Position(pos),
            ui.layer_id(),
        )
        .open(true)
        .align(egui::RectAlign::BOTTOM_START)
        .gap(2.0)
        .show(|ui| {
            ui.set_min_width(240.0);
            ui.set_max_width(420.0);
            let hacer_scroll = self.wiki_scroll;
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for (i, ruta) in resultados.iter().enumerate() {
                        let resp = ui.selectable_label(i == self.wiki_selected, ruta);
                        if resp.hovered() {
                            self.wiki_selected = i;
                        }
                        if resp.clicked() {
                            elegido = Some(ruta.clone());
                        }
                        if i == self.wiki_selected && hacer_scroll {
                            resp.scroll_to_me(Some(egui::Align::Center));
                        }
                    }
                });
        });
        self.wiki_scroll = false;

        if let Some(nombre) = elegido {
            return self.wiki_aceptar(&nombre);
        }
        false
    }

    /// Actualiza el estado del autocompletado a partir de la posición del cursor.
    fn wiki_actualizar(&mut self, cursor_byte: usize) {
        if let Some((open, query, close)) = wiki_context(&self.code, cursor_byte) {
            if !self.wiki_open || self.wiki_open_byte != open {
                self.wiki_selected = 0;
                self.wiki_scroll = true;
            }
            self.wiki_escanear();
            self.wiki_open = true;
            self.wiki_open_byte = open;
            self.wiki_query = query;
            self.wiki_close_byte = close;
            self.wiki_cursor_byte = cursor_byte;
        } else {
            self.wiki_open = false;
        }
    }

    /// Escanea el vault una sola vez por vault para listar los archivos enlazables.
    fn wiki_escanear(&mut self) {
        if self.wiki_scanned_vault == self.vault_path && !self.wiki_files.is_empty() {
            return;
        }
        self.wiki_files.clear();
        if self.vault_path.is_empty() {
            return;
        }
        // No descender en carpetas ocultas (.obsidian, .git, .venv, ...) ni en
        // carpetas de dependencias, para no inundar el autocompletado.
        let walker = WalkDir::new(&self.vault_path)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let name = e.file_name().to_string_lossy();
                    !name.starts_with('.') && name != "node_modules" && name != "target"
                } else {
                    true
                }
            });
        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let p = entry.path().to_string_lossy().to_string();
            let lower = p.to_lowercase();
            if lower.ends_with(".md")
                || lower.ends_with(".markdown")
                || lower.ends_with(".canvas")
                || lower.ends_with(".graph")
                || lower.ends_with(".inc")
            {
                self.wiki_files.push(p);
            }
        }
        self.wiki_scanned_vault = self.vault_path.clone();
    }

    /// Archivos filtrados por la consulta actual, como su ruta relativa al vault
    /// sin extensión (p. ej. `Base De Datos/Readme`). La consulta se compara tanto
    /// con el nombre como con la ruta, de modo que escribir la carpeta también filtra.
    fn wiki_resultados(&self) -> Vec<String> {
        let q = self.wiki_query.to_lowercase();
        let mut v: Vec<String> = Vec::new();
        for p in &self.wiki_files {
            let rel = Path::new(p)
                .strip_prefix(&self.vault_path)
                .unwrap_or_else(|_| Path::new(p));
            let nombre = wiki_nombre(p);
            let carpeta = rel
                .parent()
                .map(|d| d.to_string_lossy().replace('\\', "/"))
                .filter(|s| !s.is_empty())
                .unwrap_or_default();
            let ruta_rel = if carpeta.is_empty() {
                nombre.clone()
            } else {
                format!("{}/{}", carpeta, nombre)
            };
            if q.is_empty()
                || nombre.to_lowercase().contains(&q)
                || ruta_rel.to_lowercase().contains(&q)
            {
                v.push(ruta_rel);
            }
        }
        // Prioriza los nombres que empiezan con la consulta, luego alfabético.
        v.sort_by(|a, b| {
            let na = nombre_de_ruta(a);
            let nb = nombre_de_ruta(b);
            let pa = !na.starts_with(&q);
            let pb = !nb.starts_with(&q);
            pa.cmp(&pb)
                .then_with(|| na.cmp(&nb))
                .then_with(|| a.to_lowercase().cmp(&b.to_lowercase()))
        });
        v.dedup();
        v.truncate(30);
        v
    }

    /// Reemplaza el wikilink en curso por `[[nombre]]`.
    fn wiki_aceptar(&mut self, nombre: &str) -> bool {
        let open = self.wiki_open_byte;
        if open > self.code.len() || !self.code.is_char_boundary(open) {
            self.wiki_open = false;
            return false;
        }
        let end = ajustar_a_limite(
            &self.code,
            self.wiki_close_byte
                .unwrap_or(self.wiki_cursor_byte)
                .max(open + 2)
                .min(self.code.len()),
        );
        let reemplazo = format!("[[{}]]", nombre);
        self.code = format!("{}{}{}", &self.code[..open], reemplazo, &self.code[end..]);
        self.pending_cursor = Some(open + reemplazo.len());
        self.wiki_open = false;
        true
    }

    /// Maneja Flechas/Enter/Tab/Escape mientras el popup está abierto.
    fn wiki_manejar_teclas(&mut self, ui: &egui::Ui) -> bool {
        if !self.wiki_open {
            return false;
        }
        let editor_id = ui.id().with("easymark_edit");
        if !ui.memory(|m| m.has_focus(editor_id)) {
            return false;
        }
        let resultados = self.wiki_resultados();
        if resultados.is_empty() {
            return false;
        }

        let (bajar, subir, aceptar, escapar) = ui.input(|i| {
            (
                i.key_pressed(Key::ArrowDown),
                i.key_pressed(Key::ArrowUp),
                (i.key_pressed(Key::Enter) || i.key_pressed(Key::Tab)) && i.modifiers.is_none(),
                i.key_pressed(Key::Escape),
            )
        });

        if bajar || subir {
            if bajar {
                self.wiki_selected = (self.wiki_selected + 1) % resultados.len();
            } else {
                self.wiki_selected = (self.wiki_selected + resultados.len() - 1) % resultados.len();
            }
            self.wiki_scroll = true;
            ui.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::ArrowDown);
                i.consume_key(Modifiers::NONE, Key::ArrowUp);
            });
        }

        if aceptar {
            let ruta = resultados[self.wiki_selected.min(resultados.len() - 1)].clone();
            ui.input_mut(|i| {
                i.consume_key(Modifiers::NONE, Key::Enter);
                i.consume_key(Modifiers::NONE, Key::Tab);
            });
            return self.wiki_aceptar(&ruta);
        }

        if escapar {
            self.wiki_open = false;
            ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
        }

        false
    }

    /// Mientras el cursor está dentro de una fórmula (`$...$` o `$$...$$`),
    /// muestra una previsualización de cómo se verá justo debajo del cursor.
    fn mostrar_preview_latex(
        &mut self,
        ui: &egui::Ui,
        response: &egui::Response,
        anchor: Option<(egui::Rect, usize)>,
    ) {
        if !response.has_focus() {
            return;
        }
        let Some((cursor_rect, char_idx)) = anchor else {
            return;
        };
        let byte_pos = char_a_byte(&self.code, char_idx);
        let Some(formula) = latex_bajo_cursor(&self.code, byte_pos) else {
            return;
        };
        if formula.trim().is_empty() {
            return;
        }

        let pos = egui::pos2(cursor_rect.left(), cursor_rect.bottom() + 6.0);
        egui::Area::new(egui::Id::new("easymark_latex_preview"))
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .constrain(true)
            .interactable(false)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .inner_margin(egui::Margin::same(6))
                    .show(ui, |ui| {
                        ui.set_max_width(420.0);
                        crate::sketch::preview::mostrar_preview(
                            ui,
                            &mut self.latex_preview_cache,
                            &formula,
                        );
                    });
            });
    }

    fn has_image_in_clipboard(&self) -> bool {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(_) => return false,
        };

        if clipboard.get_image().is_ok() {
            return true;
        }

        if let Ok(text) = clipboard.get_text() {
            let path_str = text.trim().trim_start_matches("file://");
            let path = PathBuf::from(path_str);
            if path.exists() && path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    return matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp");
                }
            }
        }
        false
    }

    pub fn try_paste_image(&mut self, ctx: &egui::Context) -> bool {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(_) => return false,
        };

        let image = if let Ok(image_data) = clipboard.get_image() {
            let width = image_data.width as u32;
            let height = image_data.height as u32;
            let rgba_data = image_data.bytes.into_owned();

            let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
                ImageBuffer::from_raw(width, height, rgba_data)
                    .expect("Failed to create image buffer");

            Some(DynamicImage::ImageRgba8(img_buffer))
        } else if let Ok(text) = clipboard.get_text() {
            let path_str = text.trim().trim_start_matches("file://");
            let path = PathBuf::from(path_str);
            if path.exists() && path.is_file() {
                image::open(&path).ok()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(img) = image {
            let attachments_dir = if !self.vault_path.is_empty() {
                PathBuf::from(&self.vault_path).join("attachments")
            } else {
                PathBuf::from("attachments")
            };

            if !attachments_dir.exists() {
                let _ = std::fs::create_dir_all(&attachments_dir);
            }

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let filename = format!("pasted_{}.png", timestamp);
            let file_path = attachments_dir.join(&filename);

            if img.save(&file_path).is_ok() {
                let markdown_ref = format!("\n![[{}]]\n", filename);

                self.code.push_str(&markdown_ref);
                ctx.request_repaint();
                return true;
            }
        }
        false
    }
}

fn shortcut_ui(ui: &mut egui::Ui, shortcut: &mut KeyboardShortcut) {
    ui.horizontal(|ui| {
        let mut modifiers = shortcut.modifiers;
        ui.checkbox(&mut modifiers.command, "Cmd/Ctrl");
        ui.checkbox(&mut modifiers.shift, "Shift");
        ui.checkbox(&mut modifiers.alt, "Alt");
        shortcut.modifiers = modifiers;

        let has_focus = ui.memory(|m| m.has_focus(ui.id()));

        let btn_text = if has_focus {
            "Press key...".to_string()
        } else {
            format!("{:?}", shortcut.logical_key)
        };

        if ui.button(btn_text).clicked() {
            ui.memory_mut(|m| m.request_focus(ui.id()));
        }

        if has_focus {
            let key = ui.input(|i| {
                for key in Key::ALL {
                    if i.key_pressed(*key) {
                        return Some(*key);
                    }
                }
                None
            });
            if let Some(key) = key {
                shortcut.logical_key = key;
                ui.memory_mut(|m| m.surrender_focus(ui.id()));
            }
        }
    });
}

pub const SHORTCUT_BOLD: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::B);
pub const SHORTCUT_CODE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
pub const SHORTCUT_ITALICS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::I);
pub const SHORTCUT_SUBSCRIPT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::L);
pub const SHORTCUT_SUPERSCRIPT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND, Key::Y);
pub const SHORTCUT_STRIKETHROUGH: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::Q);
pub const SHORTCUT_UNDERLINE: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::W);
pub const SHORTCUT_INDENT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::E);
pub const SHORTCUT_INDENT_INC: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND, Key::CloseBracket);
pub const SHORTCUT_INDENT_DEC: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::COMMAND, Key::OpenBracket);

fn shortcuts(ui: &Ui, editor: &mut EasyMarkEditor, ccursor_range: &mut CCursorRange) -> bool {
    let mut any_change = false;
    let code = &mut editor.code;

    if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_INDENT))
        || ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_INDENT_INC))
    {
        any_change = true;
        let [primary, _secondary] = ccursor_range.sorted_cursors();

        // Find start of line
        let mut line_start = primary.index;
        while line_start > egui::text::CharIndex::ZERO && code.char_range(line_start - 1..line_start) != "\n" {
            line_start -= 1;
        }

        let advance = code.insert_text("  ", line_start);
        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }

    if ui.input_mut(|i| i.consume_shortcut(&SHORTCUT_INDENT_DEC)) {
        let [primary, _secondary] = ccursor_range.sorted_cursors();

        // Find start of line
        let mut line_start = primary.index;
        while line_start > egui::text::CharIndex::ZERO && code.char_range(line_start - 1..line_start) != "\n" {
            line_start -= 1;
        }

        if code.char_range(line_start..line_start + 1) == " " {
            any_change = true;
            let mut remove_count = 1;
            if code.char_range(line_start + 1..line_start + 2) == " " {
                remove_count = 2;
            }
            code.delete_char_range(line_start..line_start + remove_count);
            ccursor_range.primary.index = ccursor_range.primary.index.saturating_sub(remove_count);
            ccursor_range.secondary.index =
                ccursor_range.secondary.index.saturating_sub(remove_count);
        }
    }

    // Auto-continue lists
    if ui.input(|i| i.key_pressed(Key::Enter) && i.modifiers.is_none()) {
        let [primary, _secondary] = ccursor_range.sorted_cursors();
        let index = primary.index;
        if index > egui::text::CharIndex::ZERO && code.char_range(index - 1..index) == "\n" {
            let mut line_start = index - 1;
            while line_start > egui::text::CharIndex::ZERO && code.char_range(line_start - 1..line_start) != "\n" {
                line_start -= 1;
            }

            let line = code.char_range(line_start..index - 1);
            let mut indentation_char_count = 0;
            for c in line.chars() {
                if c.is_whitespace() {
                    indentation_char_count += 1;
                } else {
                    break;
                }
            }
            let indentation = code.char_range(line_start..line_start + indentation_char_count);
            let trimmed_line = code.char_range(line_start + indentation_char_count..index - 1);

            let markers = ["- [ ] ", "- [x] ", "- ", "+ ", "* "];
            for marker in markers {
                if trimmed_line.starts_with(marker) {
                    any_change = true;
                    if trimmed_line.trim() == marker.trim() {
                        // Clear the whole line including indentation and the newline
                        code.delete_char_range(line_start..index);
                        ccursor_range.primary.index = line_start;
                        ccursor_range.secondary.index = line_start;
                    } else {
                        let marker_to_add = if marker == "- [x] " { "- [ ] " } else { marker };
                        let full_marker = format!("{}{}", indentation, marker_to_add);
                        let advance = code.insert_text(&full_marker, index);
                        ccursor_range.primary.index += advance;
                        ccursor_range.secondary.index += advance;
                    }
                    break;
                }
            }
        }
    }

    for (shortcut, surrounding) in [
        (editor.shortcut_bold, "**"),
        (editor.shortcut_code, "`"),
        (editor.shortcut_italics, "*"),
        (SHORTCUT_SUBSCRIPT, "$"),
        (SHORTCUT_SUPERSCRIPT, "^"),
        (SHORTCUT_STRIKETHROUGH, "~"),
        (SHORTCUT_UNDERLINE, "_"),
    ] {
        if ui.input_mut(|i| i.consume_shortcut(&shortcut)) {
            any_change = true;
            toggle_surrounding(code, ccursor_range, surrounding);
        }
    }

    any_change
}

/// E.g. toggle *strong* with `toggle_surrounding(&mut text, &mut cursor, "*")`
fn toggle_surrounding(
    code: &mut dyn TextBuffer,
    ccursor_range: &mut CCursorRange,
    surrounding: &str,
) {
    let [primary, secondary] = ccursor_range.sorted_cursors();

    let surrounding_ccount = surrounding.chars().count();

    let prefix_crange = primary.index.saturating_sub(surrounding_ccount)..primary.index;
    let suffix_crange = secondary.index..secondary.index.saturating_add(surrounding_ccount);
    let already_surrounded = code.char_range(prefix_crange.clone()) == surrounding
        && code.char_range(suffix_crange.clone()) == surrounding;

    if already_surrounded {
        code.delete_char_range(suffix_crange);
        code.delete_char_range(prefix_crange);
        ccursor_range.primary.index -= surrounding_ccount;
        ccursor_range.secondary.index -= surrounding_ccount;
    } else {
        code.insert_text(surrounding, secondary.index);
        let advance = code.insert_text(surrounding, primary.index);

        ccursor_range.primary.index += advance;
        ccursor_range.secondary.index += advance;
    }
}

/// Convierte un índice de carácter a un desplazamiento en bytes.
fn char_a_byte(code: &str, char_idx: usize) -> usize {
    code.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(code.len())
}

/// Si el cursor está dentro de una fórmula delimitada por `$...$` o `$$...$$`,
/// devuelve el contenido de la fórmula (aunque todavía no se haya cerrado).
fn latex_bajo_cursor(code: &str, cursor_byte: usize) -> Option<String> {
    let bytes = code.as_bytes();
    let mut i = 0;
    let mut abierto: Option<(usize, bool)> = None; // (inicio del contenido, es_bloque)
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i;
            while i < bytes.len() && bytes[i] == b'$' {
                i += 1;
            }
            let run = i - start;
            match abierto {
                Some((content_start, es_bloque)) => {
                    let cierra = if es_bloque { run >= 2 } else { run >= 1 };
                    if cierra {
                        if cursor_byte >= content_start && cursor_byte <= start {
                            return Some(code[content_start..start].to_string());
                        }
                        abierto = None;
                    }
                }
                None => {
                    abierto = Some((i, run >= 2));
                }
            }
        } else {
            i += 1;
        }
    }
    if let Some((content_start, _)) = abierto {
        if cursor_byte >= content_start {
            return Some(code[content_start..].to_string());
        }
    }
    None
}

/// Convierte un desplazamiento en bytes a un índice de carácter.
fn byte_a_char(code: &str, byte: usize) -> usize {
    code[..byte.min(code.len())].chars().count()
}

/// Ajusta un desplazamiento en bytes al siguiente límite de carácter válido.
fn ajustar_a_limite(code: &str, mut byte: usize) -> usize {
    while byte < code.len() && !code.is_char_boundary(byte) {
        byte += 1;
    }
    byte
}

/// Nombre visible de un archivo para un wikilink (sin la extensión).
fn wiki_nombre(path: &str) -> String {
    let fname = Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    for sufijo in [
        ".excalidraw.md",
        ".markdown",
        ".canvas",
        ".graph",
        ".inc",
        ".md",
    ] {
        if let Some(stripped) = fname.strip_suffix(sufijo) {
            return stripped.to_string();
        }
    }
    fname
}

/// Nombre (minúsculas) de una ruta relativa ya sin extensión. Para ordenar.
fn nombre_de_ruta(ruta_rel: &str) -> String {
    Path::new(ruta_rel)
        .file_name()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// Si el cursor está dentro de un wikilink `[[...`, devuelve
/// `(inicio_del_wikilink, consulta, fin_tras_]] )`.
fn wiki_context(code: &str, cursor_byte: usize) -> Option<(usize, String, Option<usize>)> {
    let before = &code[..cursor_byte];
    let open = before.rfind("[[")?;
    let between = &code[open + 2..cursor_byte];
    if between.contains("]]") || between.contains('\n') {
        return None;
    }
    let query = between.to_string();
    let after = &code[cursor_byte..];
    let close = after.find("]]").map(|i| cursor_byte + i + 2);
    Some((open, query, close))
}

#[cfg(test)]
mod tests_wikilink {
    use super::*;

    #[test]
    fn detecta_wikilink_sin_cerrar() {
        let code = "texto [[No";
        let ctx = wiki_context(code, code.len()).unwrap();
        assert_eq!(ctx.0, 6);
        assert_eq!(ctx.1, "No");
        assert_eq!(ctx.2, None);
    }

    #[test]
    fn detecta_wikilink_cerrado_con_cursor_dentro() {
        let code = "[[Nota]]";
        let ctx = wiki_context(code, 4).unwrap();
        assert_eq!(ctx.0, 0);
        assert_eq!(ctx.1, "No");
        assert_eq!(ctx.2, Some(8));
    }

    #[test]
    fn no_detecta_fuera_de_wikilink() {
        assert!(wiki_context("texto normal", 5).is_none());
        assert!(wiki_context("[[Uno]] y [[Dos]]", 8).is_none());
    }

    #[test]
    fn nombre_sin_extension() {
        assert_eq!(wiki_nombre("/v/Nota.md"), "Nota");
        assert_eq!(wiki_nombre("/v/Dibujo.excalidraw.md"), "Dibujo");
        assert_eq!(wiki_nombre("/v/tablero.canvas"), "tablero");
    }
}
