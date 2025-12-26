use super::easy_mark_parser::{self, Heading};

/// Highlight easymark, memoizing previous output to save CPU.
///
/// In practice, the highlighter is fast enough not to need any caching.
#[derive(Default)]
pub struct MemoizedEasymarkHighlighter {
    style: egui::Style,
    code: String,
    output: egui::text::LayoutJob,
    cursor_index: Option<usize>,
}

impl MemoizedEasymarkHighlighter {
    pub fn highlight(&mut self, egui_style: &egui::Style, code: &str, cursor_index: Option<usize>) -> egui::text::LayoutJob {
        if (&self.style, self.code.as_str(), self.cursor_index) != (egui_style, code, cursor_index) {
            self.style = egui_style.clone();
            code.clone_into(&mut self.code);
            self.cursor_index = cursor_index;
            self.output = highlight_easymark(egui_style, code, cursor_index);
        }
        self.output.clone()
    }
}

pub fn highlight_easymark(
    egui_style: &egui::Style, 
    mut text: &str, 
    cursor_index: Option<usize>
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let mut style = easy_mark_parser::Style::default();
    let mut start_of_line = true;
    let mut current_index = 0;

    while !text.is_empty() {
        if start_of_line && text.starts_with("```") {
            let end = text.find("\n```").map_or_else(|| text.len(), |i| i + 4);
            job.append(
                &text[..end],
                0.0,
                format_from_style(
                    egui_style,
                    &easy_mark_parser::Style {
                        code: true,
                        ..Default::default()
                    },
                ),
            );
            text = &text[end..];
            current_index += end;
            style = Default::default();
            continue;
        }

        if text.starts_with('`') {
            style.code = true;
            let end = text[1..]
                .find(&['`', '\n'][..])
                .map_or_else(|| text.len(), |i| i + 2);
            job.append(&text[..end], 0.0, format_from_style(egui_style, &style));
            text = &text[end..];
            current_index += end;
            style.code = false;
            continue;
        }

        let mut skip;

        if text.starts_with('\\') && text.len() >= 2 {
            skip = 2;
        } else if start_of_line && text.starts_with('|') {
            // Table highlighting
            let line_end = text.find('\n').unwrap_or(text.len());
            job.append(
                &text[..line_end],
                0.0,
                egui::text::TextFormat {
                    color: egui_style.visuals.weak_text_color(),
                    ..Default::default()
                },
            );
            text = &text[line_end..];
            current_index += line_end;
            start_of_line = true;
            continue;
        } else if start_of_line && text.starts_with(' ') {
            // we don't preview indentation, because it is confusing
            skip = 1;
        } else if start_of_line && text.starts_with("#### ") {
            style.heading = Heading::H4;
            skip = 5;
        } else if start_of_line && text.starts_with("### ") {
            style.heading = Heading::H3;
            skip = 4;
        } else if start_of_line && text.starts_with("## ") {
            style.heading = Heading::H2;
            skip = 3;
        } else if start_of_line && text.starts_with("# ") {
            style.heading = Heading::H1;
            skip = 2;
        } else if start_of_line && text.starts_with("> ") {
            style.quoted = true;
            skip = 2;
            // we don't preview indentation, because it is confusing
        } else if start_of_line && text.starts_with("- ") {
            skip = 2;
            // we don't preview indentation, because it is confusing
        } else if text.starts_with("![ ") || text.starts_with('[') {
            let is_image = text.starts_with('!');
            let offset = if is_image { 1 } else { 0 };
            let sub_text = &text[offset..];
            
            if let Some(bracket_end) = sub_text.find(']') {
                if sub_text.len() > bracket_end + 1 && sub_text[bracket_end + 1..].starts_with('(') {
                    if let Some(parens_end) = sub_text[bracket_end + 2..].find(')') {
                        let full_len = offset + bracket_end + 2 + parens_end + 1;
                        let is_near = cursor_index.map_or(false, |idx| {
                            idx >= current_index && idx <= current_index + full_len
                        });

                        let accent_color = egui_style.visuals.hyperlink_color;
                        
                        if is_near {
                            // Highlight full [text](url)
                            job.append(
                                &text[..full_len],
                                0.0,
                                egui::text::TextFormat {
                                    color: accent_color,
                                    background: egui_style.visuals.code_bg_color,
                                    ..Default::default()
                                },
                            );
                        } else {
                            // Show [text], hide (url)
                            job.append(
                                &text[..offset + bracket_end + 1],
                                0.0,
                                egui::text::TextFormat {
                                    color: accent_color,
                                    ..Default::default()
                                },
                            );
                            // URL part: transparent and tiny
                            job.append(
                                &text[offset + bracket_end + 1..full_len],
                                0.0,
                                egui::text::TextFormat {
                                    color: egui::Color32::TRANSPARENT,
                                    font_id: egui::FontId::new(0.1, egui::FontFamily::Monospace),
                                    ..Default::default()
                                },
                            );
                        }
                        text = &text[full_len..];
                        current_index += full_len;
                        start_of_line = false;
                        continue;
                    }
                }
            }
            skip = 0;
        } else if text.starts_with("**") {
            skip = 2;
            if style.strong {
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &style));
                text = &text[skip..];
                current_index += skip;
                skip = 0;
            }
            style.strong ^= true;
        } else if text.starts_with('*') {
            skip = 1;
            if style.italics {
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &style));
                text = &text[skip..];
                current_index += skip;
                skip = 0;
            }
            style.italics ^= true;
        } else if text.starts_with('~') {
            skip = 1;
            if style.strikethrough {
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &style));
                text = &text[skip..];
                current_index += skip;
                skip = 0;
            }
            style.strikethrough ^= true;
        } else if text.starts_with('_') {
            skip = 1;
            if style.underline {
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &style));
                text = &text[skip..];
                current_index += skip;
                skip = 0;
            }
            style.underline ^= true;
        } else if text.starts_with('$') {
            skip = 1;
            if style.small {
                // Include the character that is ending this style:
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &style));
                text = &text[skip..];
                current_index += skip;
                skip = 0;
            }
            style.small ^= true;
        } else if text.starts_with('^') {
            skip = 1;
            if style.raised {
                // Include the character that is ending this style:
                job.append(&text[..skip], 0.0, format_from_style(egui_style, &style));
                text = &text[skip..];
                current_index += skip;
                skip = 0;
            }
            style.raised ^= true;
        } else {
            skip = 0;
        }

        // Swallow everything up to the next special character:
        let line_end = text[skip..]
            .find('\n')
            .map_or_else(|| text.len(), |i| skip + i + 1);
        let end = text[skip..]
            .find(&['*', '`', '~', '_', '/', '$', '^', '\\', '<', '[', '!', '|'][..])
            .map_or_else(|| text.len(), |i| (skip + i).max(1));

        if line_end <= end {
            job.append(
                &text[..line_end],
                0.0,
                format_from_style(egui_style, &style),
            );
            text = &text[line_end..];
            current_index += line_end;
            start_of_line = true;
            style = Default::default();
        } else {
            job.append(&text[..end], 0.0, format_from_style(egui_style, &style));
            text = &text[end..];
            current_index += end;
            start_of_line = false;
        }
    }

    job
}

fn format_from_style(
    egui_style: &egui::Style,
    emark_style: &easy_mark_parser::Style,
) -> egui::text::TextFormat {
    use egui::{Align, Color32, Stroke, TextStyle};

    let color = if emark_style.heading != Heading::None || emark_style.strong {
        egui_style.visuals.strong_text_color()
    } else if emark_style.quoted {
        egui_style.visuals.weak_text_color()
    } else {
        egui_style.visuals.text_color()
    };

    let mut font_id = match emark_style.heading {
        Heading::H1 => {
            let mut id = TextStyle::Heading.resolve(egui_style);
            id.size = 20.0;
            id
        },
        Heading::H2 => {
            let mut id = TextStyle::Body.resolve(egui_style);
            id.size = 18.0;
            id
        },
        Heading::H3 => {
            let mut id = TextStyle::Body.resolve(egui_style);
            id.size = 16.0;
            id
        },
        Heading::H4 => {
            let mut id = TextStyle::Body.resolve(egui_style);
            id.size = 14.0;
            id
        },
        Heading::None => {
            if emark_style.code {
                TextStyle::Monospace.resolve(egui_style)
            } else if emark_style.small | emark_style.raised {
                TextStyle::Small.resolve(egui_style)
            } else {
                TextStyle::Body.resolve(egui_style)
            }
        }
    };

    if emark_style.heading != Heading::None {
        font_id.family = egui::FontFamily::Proportional;
    }

    let background = if emark_style.code {
        egui_style.visuals.code_bg_color
    } else {
        Color32::TRANSPARENT
    };

    let underline = if emark_style.underline {
        Stroke::new(1.0, color)
    } else {
        Stroke::NONE
    };

    let strikethrough = if emark_style.strikethrough {
        Stroke::new(1.0, color)
    } else {
        Stroke::NONE
    };

    let valign = if emark_style.raised {
        Align::TOP
    } else {
        Align::BOTTOM
    };

    egui::text::TextFormat {
        font_id,
        color,
        background,
        italics: emark_style.italics,
        underline,
        strikethrough,
        valign,
        ..Default::default()
    }
}
