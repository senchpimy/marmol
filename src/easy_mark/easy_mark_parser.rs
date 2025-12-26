// A parser for `EasyMark`: a very simple markup language.
//
// WARNING: `EasyMark` is subject to change.
//
// # `EasyMark` design goals:
// 1. easy to parse
// 2. easy to learn
// 3. similar to markdown

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Item<'a> {
    /// `\n`
    // TODO(emilk): add Style here so empty heading still uses up the right amount of space.
    Newline,

    /// Text
    Text(Style, &'a str),

    /// title, url
    Hyperlink(Style, &'a str, &'a str),

    /// alt, url
    Image(Style, &'a str, &'a str),

    /// leading space before e.g. a [`Self::BulletPoint`].
    Indentation(usize),

    /// >
    QuoteIndent,

    /// - a point well made.
    BulletPoint,

    /// 1. numbered list. The string is the number(s).
    NumberedPoint(&'a str),

    /// ---
    Separator,

    /// language, code
    CodeBlock(&'a str, &'a str),

    /// Table rows
    Table(Vec<Vec<String>>),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum Heading {
    H1,
    H2,
    H3,
    H4,
    #[default]
    None,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Style {
    /// # heading (large text)
    pub heading: Heading,

    /// > quoted (slightly dimmer color or other font style)
    pub quoted: bool,

    /// `code` (monospace, some other color)
    pub code: bool,

    /// **bold**
    pub strong: bool,

    /// _underline_
    pub underline: bool,

    /// ~strikethrough~
    pub strikethrough: bool,

    /// *italics*
    pub italics: bool,

    /// $small$
    pub small: bool,

    /// ^raised^
    pub raised: bool,
}

/// Parser for the `EasyMark` markup language.
///
/// See the module-level documentation for details.
///
/// # Example:
/// ```
/// # use egui_demo_lib::easy_mark::parser::Parser;
/// for item in Parser::new("Hello *world*!") {
/// }
///
/// ```
pub struct Parser<'a> {
    /// The remainder of the input text
    s: &'a str,

    /// Are we at the start of a line?
    start_of_line: bool,

    /// Current self.style. Reset after a newline.
    style: Style,
}

impl<'a> Parser<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s,
            start_of_line: true,
            style: Style::default(),
        }
    }

    /// `1. `, `42. ` etc.
    fn numbered_list(&mut self) -> Option<Item<'a>> {
        let n_digits = self.s.chars().take_while(|c| c.is_ascii_digit()).count();
        if n_digits > 0 && self.s.chars().skip(n_digits).take(2).eq(". ".chars()) {
            let number = &self.s[..n_digits];
            self.s = &self.s[(n_digits + 2)..];
            self.start_of_line = false;
            return Some(Item::NumberedPoint(number));
        }
        None
    }

    // ```{language}\n{code}```
    fn code_block(&mut self) -> Option<Item<'a>> {
        if let Some(language_start) = self.s.strip_prefix("```") {
            if let Some(newline) = language_start.find('\n') {
                let language = &language_start[..newline];
                let code_start = &language_start[newline + 1..];
                if let Some(end) = code_start.find("\n```") {
                    let code = &code_start[..end].trim();
                    self.s = &code_start[end + 4..];
                    self.start_of_line = false;
                    return Some(Item::CodeBlock(language, code));
                } else {
                    self.s = "";
                    return Some(Item::CodeBlock(language, code_start));
                }
            }
        }
        None
    }

    // `code`
    fn inline_code(&mut self) -> Option<Item<'a>> {
        if let Some(rest) = self.s.strip_prefix('`') {
            self.s = rest;
            self.start_of_line = false;
            self.style.code = true;
            let rest_of_line = &self.s[..self.s.find('\n').unwrap_or(self.s.len())];
            if let Some(end) = rest_of_line.find('`') {
                let item = Item::Text(self.style, &self.s[..end]);
                self.s = &self.s[end + 1..];
                self.style.code = false;
                return Some(item);
            } else {
                let end = rest_of_line.len();
                let item = Item::Text(self.style, rest_of_line);
                self.s = &self.s[end..];
                self.style.code = false;
                return Some(item);
            }
        }
        None
    }

    /// `<url>` or `[link](url)` or `![alt](url)`
    fn url_or_image(&mut self) -> Option<Item<'a>> {
        if self.s.starts_with('<') {
            let this_line = &self.s[..self.s.find('\n').unwrap_or(self.s.len())];
            if let Some(url_end) = this_line.find('>') {
                let url = &self.s[1..url_end];
                self.s = &self.s[url_end + 1..];
                self.start_of_line = false;
                return Some(Item::Hyperlink(self.style, url, url));
            }
        }

        let is_image = self.s.starts_with('!');
        let s_slice = if is_image { &self.s[1..] } else { self.s };

        if s_slice.starts_with('[') {
            let this_line = &s_slice[..s_slice.find('\n').unwrap_or(s_slice.len())];
            if let Some(bracket_end) = this_line.find(']') {
                let text = &this_line[1..bracket_end];
                if this_line.len() > bracket_end + 1 && this_line[bracket_end + 1..].starts_with('(') {
                    if let Some(parens_end) = this_line[bracket_end + 2..].find(')') {
                        let parens_end = bracket_end + 2 + parens_end;
                        let url = &s_slice[bracket_end + 2..parens_end];
                        self.s = &s_slice[parens_end + 1..];
                        self.start_of_line = false;
                        if is_image {
                            return Some(Item::Image(self.style, text, url));
                        } else {
                            return Some(Item::Hyperlink(self.style, text, url));
                        }
                    }
                }
            }
        }
        None
    }

    fn table(&mut self) -> Option<Item<'a>> {
        if !self.s.starts_with('|') {
            return None;
        }

        let mut rows = Vec::new();
        while self.s.starts_with('|') {
            let line_end = self.s.find('\n').unwrap_or(self.s.len());
            let line = &self.s[1..line_end];
            
            // If it's a separator line like |---|---| , skip it
            if line.chars().all(|c| c == '-' || c == '|' || c == ' ') {
                self.s = &self.s[line_end..];
                if self.s.starts_with('\n') {
                    self.s = &self.s[1..];
                }
                continue;
            }

            let cells: Vec<String> = line
                .split('|')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty() || line.contains("||")) // Allow empty cells if explicitly double piped
                .collect();
            
            if !cells.is_empty() {
                rows.push(cells);
            }

            self.s = &self.s[line_end..];
            if self.s.starts_with('\n') {
                self.s = &self.s[1..];
            } else {
                break;
            }
        }

        if !rows.is_empty() {
            self.start_of_line = true;
            Some(Item::Table(rows))
        } else {
            None
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.s.is_empty() {
                return None;
            }

            // \n
            if self.s.starts_with('\n') {
                self.s = &self.s[1..];
                self.start_of_line = true;
                self.style = Style::default();
                return Some(Item::Newline);
            }

            // Ignore line break (continue on the same line)
            if self.s.starts_with("\\n") && self.s.len() >= 2 {
                self.s = &self.s[2..];
                self.start_of_line = false;
                continue;
            }

            // \ escape (to show e.g. a backtick)
            if self.s.starts_with('\\') && self.s.len() >= 2 {
                let text = &self.s[1..2];
                self.s = &self.s[2..];
                self.start_of_line = false;
                return Some(Item::Text(self.style, text));
            }

            if self.start_of_line {
                // Table
                if let Some(item) = self.table() {
                    return Some(item);
                }

                // leading space (indentation)
                if self.s.starts_with(' ') {
                    let length = self.s.find(|c| c != ' ').unwrap_or(self.s.len());
                    self.s = &self.s[length..];
                    self.start_of_line = true; // indentation doesn't count
                    return Some(Item::Indentation(length));
                }

                // # Heading
                let n_hashes = self.s.chars().take_while(|&c| c == '#').count();
                if n_hashes > 0 && self.s.chars().nth(n_hashes) == Some(' ') {
                    let level = match n_hashes {
                        1 => Heading::H1,
                        2 => Heading::H2,
                        3 => Heading::H3,
                        4 => Heading::H4,
                        _ => Heading::H4,
                    };
                    self.s = &self.s[n_hashes + 1..];
                    self.start_of_line = false;
                    self.style.heading = level;
                    continue;
                }

                // > quote
                if let Some(after) = self.s.strip_prefix("> ") {
                    self.s = after;
                    self.start_of_line = true; // quote indentation doesn't count
                    self.style.quoted = true;
                    return Some(Item::QuoteIndent);
                }

                // - bullet point
                if self.s.starts_with("- ") {
                    self.s = &self.s[2..];
                    self.start_of_line = false;
                    return Some(Item::BulletPoint);
                }

                // `1. `, `42. ` etc.
                if let Some(item) = self.numbered_list() {
                    return Some(item);
                }

                // ---
                if let Some(after) = self.s.strip_prefix("---") {
                    self.s = after.trim_start_matches('-'); // remove extra dashes
                    self.s = self.s.strip_prefix('\n').unwrap_or(self.s); // remove trailing newline
                    self.start_of_line = false;
                    return Some(Item::Separator);
                }

                // ```{language}\n{code}```
                if let Some(item) = self.code_block() {
                    return Some(item);
                }
            }

            // `code`
            if let Some(item) = self.inline_code() {
                return Some(item);
            }

            if let Some(rest) = self.s.strip_prefix("**") {
                self.s = rest;
                self.start_of_line = false;
                self.style.strong = !self.style.strong;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('*') {
                self.s = rest;
                self.start_of_line = false;
                self.style.italics = !self.style.italics;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('_') {
                self.s = rest;
                self.start_of_line = false;
                self.style.underline = !self.style.underline;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('~') {
                self.s = rest;
                self.start_of_line = false;
                self.style.strikethrough = !self.style.strikethrough;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('/') {
                self.s = rest;
                self.start_of_line = false;
                self.style.italics = !self.style.italics;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('$') {
                self.s = rest;
                self.start_of_line = false;
                self.style.small = !self.style.small;
                continue;
            }
            if let Some(rest) = self.s.strip_prefix('^') {
                self.s = rest;
                self.start_of_line = false;
                self.style.raised = !self.style.raised;
                continue;
            }

            // `<url>` or `[link](url)` or `![alt](url)`
            if let Some(item) = self.url_or_image() {
                return Some(item);
            }

            // Swallow everything up to the next special character:
            let end = self
                .s
                .find(&['*', '`', '~', '_', '/', '$', '^', '\\', '<', '[', '!', '|', '\n'][..])
                .map_or_else(|| self.s.len(), |special| special.max(1));

            let item = Item::Text(self.style, &self.s[..end]);
            self.s = &self.s[end..];
            self.start_of_line = false;
            return Some(item);
        }
    }
}