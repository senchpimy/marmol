use arboard::Clipboard;
use egui::{text::CCursorRange, Key, KeyboardShortcut, Modifiers, TextBuffer, TextEdit, Ui};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
        }
    }
}

impl EasyMarkEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed_programmatically = false;

        // Handle Ctrl+V before TextEdit
        let has_image = self.has_image_in_clipboard();
        if has_image {
            let ctrl_v = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::V);
            let cmd_v = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::V);
            if ui.input_mut(|i| i.consume_shortcut(&ctrl_v) || i.consume_shortcut(&cmd_v)) {
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

            if let Some(galley) = galley_out {
                if let Some(mut state) = TextEdit::load_state(ui.ctx(), response.id) {
                    if let Some(mut ccursor_range) = state.cursor.char_range() {
                        let any_change = shortcuts(ui, self, &mut ccursor_range);

                        if response.changed() || any_change {
                            let cursor_pos_rect = galley.pos_from_cursor(ccursor_range.primary);
                            let cursor_rect =
                                cursor_pos_rect.translate(response.rect.min.to_vec2());
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

            if changed_programmatically {
                response.mark_changed();
            }
            response
        }
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
