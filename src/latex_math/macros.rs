//! Expansión de macros locales, portada de `src/math_macros.cpp` de Tinta (MIT).
//!
//! Port de Tinta (MIT, Copyright (c) 2024 oipoistar).
//!
//! Mantiene los mismos nombres que el original (`Expander`, `define`, `run`,
//! `group`, `argument`, `joinTokens`...) para facilitar la comparación.
//! El original usa `std::wstring`; aquí usamos `Vec<char>` / `String`.

use std::collections::HashMap;

use super::parser::is_builtin_math_command;

// ---------------------------------------------------------------------------
// Helpers (idénticos a los del original)
// ---------------------------------------------------------------------------

fn letter(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase()
}

/// Port de `controlWordAtEnd`: ¿el texto acaba en una palabra de control
/// (`\foo`) con un número impar de barras invertidas?
fn control_word_at_end(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = bytes.len();
    while i > 0 && letter(bytes[i - 1] as char) {
        i -= 1;
    }
    if i == bytes.len() || i == 0 || bytes[i - 1] != b'\\' {
        return false;
    }
    let mut slashes = 0usize;
    while i > 0 {
        i -= 1;
        if bytes[i] == b'\\' {
            slashes += 1;
        } else {
            break;
        }
    }
    slashes % 2 == 1
}

/// Port de `joinTokens`: separa con un espacio cuando unir una palabra
/// alfabética tras una palabra de control cambiaría el significado.
fn join_tokens(left: &mut String, right: &str) {
    if !right.is_empty()
        && letter(right.chars().next().unwrap())
        && control_word_at_end(left)
    {
        left.push(' ');
    }
    left.push_str(right);
}

fn chars_to_string(chars: &[char]) -> String {
    chars.iter().collect()
}

// ---------------------------------------------------------------------------
// Expander
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
struct Macro {
    body: Vec<char>,
    default_argument: String,
    arguments: usize,
    has_default: bool,
}

type Definitions = HashMap<String, Macro>;

struct Expander {
    text: Vec<char>,
    pos: usize,
    expansions: usize,
    failed: bool,
    macros: Definitions,
    scopes: Vec<Definitions>,
}

impl Expander {
    /// Port de `Expander::whitespace`: salta espacios y comentarios `%`.
    fn whitespace(&mut self) {
        while self.pos < self.text.len() {
            if self.text[self.pos].is_whitespace() {
                self.pos += 1;
                continue;
            }
            if self.text[self.pos] != '%' {
                break;
            }
            while self.pos < self.text.len() && self.text[self.pos] != '\n' {
                self.pos += 1;
            }
        }
    }

    /// Port de `Expander::command` (después de la barra invertida).
    fn command(&mut self) -> String {
        if self.pos == self.text.len() || self.text[self.pos] != '\\' {
            self.failed = true;
            return String::new();
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.text.len() && letter(self.text[self.pos]) {
            self.pos += 1;
        }
        if start == self.pos && self.pos < self.text.len() {
            self.pos += 1;
        }
        chars_to_string(&self.text[start..self.pos])
    }

    /// Port de `Expander::group`.
    fn group(&mut self, open: char, close: char) -> String {
        self.whitespace();
        if self.pos == self.text.len() || self.text[self.pos] != open {
            self.failed = true;
            return String::new();
        }
        self.pos += 1;
        let start = self.pos;
        let mut depth = 1i32;
        let mut braces = 0i32;
        while self.pos < self.text.len() {
            let c = self.text[self.pos];
            self.pos += 1;
            if c == '\\' {
                if self.pos < self.text.len() {
                    self.pos += 1;
                }
                continue;
            }
            if c == '%' {
                while self.pos < self.text.len() && self.text[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            if open == '[' {
                if c == '{' {
                    braces += 1;
                }
                if c == '}' {
                    braces -= 1;
                }
                if braces != 0 {
                    continue;
                }
            }
            if c == open {
                depth += 1;
            }
            if c == close {
                depth -= 1;
                if depth == 0 {
                    return chars_to_string(&self.text[start..self.pos - 1]);
                }
            }
        }
        self.failed = true;
        String::new()
    }

    /// Port de `Expander::argument`: un grupo entre llaves o un solo token.
    fn argument(&mut self) -> String {
        self.whitespace();
        if self.pos == self.text.len() {
            self.failed = true;
            return String::new();
        }
        if self.text[self.pos] == '{' {
            return self.group('{', '}');
        }
        let start = self.pos;
        if self.text[self.pos] == '\\' {
            self.command();
        } else {
            self.pos += 1;
        }
        chars_to_string(&self.text[start..self.pos])
    }

    /// Port de `Expander::replace`: sustituye `text[start..pos]` por
    /// `replacement` y deja `pos` al principio del texto insertado.
    fn replace(&mut self, start: usize, replacement: &str) {
        let mut result = chars_to_string(&self.text[..start]);
        join_tokens(&mut result, replacement);
        let resume = start;
        let suffix = chars_to_string(&self.text[self.pos..]);
        join_tokens(&mut result, &suffix);
        if result.chars().count() > 32768 {
            self.failed = true;
            return;
        }
        self.text = result.chars().collect();
        self.pos = resume;
    }

    /// Port de `Expander::name`: nombre de macro, con o sin llaves.
    fn name(&mut self, braced: bool) -> String {
        self.whitespace();
        if braced && self.pos < self.text.len() && self.text[self.pos] == '{' {
            let value = self.group('{', '}');
            let chars: Vec<char> = value.chars().collect();
            if chars.len() < 2 || chars[0] != '\\' {
                self.failed = true;
                return String::new();
            }
            for &c in &chars[1..] {
                if !letter(c) {
                    self.failed = true;
                }
            }
            return chars_to_string(&chars[1..]);
        }
        let value = self.command();
        if value.is_empty() || !letter(value.chars().next().unwrap()) {
            self.failed = true;
        }
        value
    }

    /// Port de `Expander::define`.
    fn define(&mut self, kind: &str, start: usize) {
        self.expansions += 1;
        if self.expansions > 512 {
            self.failed = true;
            return;
        }
        let starred = self.pos < self.text.len() && self.text[self.pos] == '*';
        if starred {
            self.pos += 1;
        }
        let key = self.name(kind != "def");
        let mut macro_def = Macro::default();
        self.whitespace();
        if kind == "def" {
            while self.pos < self.text.len() && self.text[self.pos] == '#' {
                self.pos += 1;
                macro_def.arguments += 1;
                let expected = char::from_digit(macro_def.arguments as u32, 10).unwrap();
                if macro_def.arguments > 9
                    || self.pos == self.text.len()
                    || self.text[self.pos] != expected
                {
                    self.failed = true;
                    return;
                }
                self.pos += 1;
                self.whitespace();
            }
        } else if kind != "DeclareMathOperator"
            && self.pos < self.text.len()
            && self.text[self.pos] == '['
        {
            let count = self.group('[', ']');
            let count_chars: Vec<char> = count.chars().collect();
            if count_chars.len() != 1 || !count_chars[0].is_ascii_digit() {
                self.failed = true;
                return;
            }
            macro_def.arguments = count_chars[0].to_digit(10).unwrap() as usize;
            self.whitespace();
            if self.pos < self.text.len() && self.text[self.pos] == '[' {
                macro_def.default_argument = self.group('[', ']');
                macro_def.has_default = true;
                if macro_def.arguments == 0 {
                    self.failed = true;
                }
            }
        }
        let body = self.group('{', '}');
        macro_def.body = body.chars().collect();
        if kind == "DeclareMathOperator" {
            let prefix = if starred {
                "\\operatorname*{"
            } else {
                "\\operatorname{"
            };
            macro_def.body = format!("{prefix}{body}}}").chars().collect();
        }
        let exists = self.macros.contains_key(&key) || is_builtin_math_command(&key);
        if (kind == "newcommand" || kind == "DeclareMathOperator") && exists {
            self.failed = true;
        }
        if kind == "renewcommand" && !exists {
            self.failed = true;
        }
        if self.failed {
            return;
        }
        if kind != "providecommand" || !exists {
            self.macros.insert(key, macro_def);
        }
        self.replace(start, "");
    }

    /// Port de `Expander::run`.
    fn run(&mut self) -> bool {
        while self.pos < self.text.len() && !self.failed {
            let c = self.text[self.pos];
            if c == '%' {
                self.whitespace();
                continue;
            }
            if c == '{' {
                if self.scopes.len() >= 96 {
                    return false;
                }
                self.scopes.push(self.macros.clone());
                self.pos += 1;
                continue;
            }
            if c == '}' {
                match self.scopes.pop() {
                    Some(scope) => self.macros = scope,
                    None => return false,
                }
                self.pos += 1;
                continue;
            }
            if c != '\\' {
                self.pos += 1;
                continue;
            }
            let start = self.pos;
            let cmd = self.command();
            if cmd == "newcommand"
                || cmd == "renewcommand"
                || cmd == "providecommand"
                || cmd == "def"
                || cmd == "DeclareMathOperator"
            {
                self.define(&cmd, start);
                continue;
            }
            let Some(found) = self.macros.get(&cmd).cloned() else {
                continue;
            };
            self.expansions += 1;
            if self.expansions > 512 {
                return false;
            }
            let macro_def = found;
            let mut args: Vec<String> = Vec::new();
            self.whitespace();
            if macro_def.has_default {
                self.whitespace();
                if self.pos < self.text.len() && self.text[self.pos] == '[' {
                    args.push(self.group('[', ']'));
                } else {
                    args.push(macro_def.default_argument.clone());
                }
            }
            while !self.failed && args.len() < macro_def.arguments {
                args.push(self.argument());
            }
            let mut replacement = String::new();
            let body = &macro_def.body;
            let mut chunk_start = 0usize;
            let mut i = 0usize;
            while i < body.len() && !self.failed {
                if body[i] == '\\' && i + 1 < body.len() {
                    i += 2;
                    continue;
                }
                if body[i] == '#' {
                    join_tokens(&mut replacement, &chars_to_string(&body[chunk_start..i]));
                    i += 1;
                    if i == body.len() {
                        self.failed = true;
                        break;
                    }
                    let digit = body[i];
                    if digit == '#' {
                        replacement.push('#');
                    } else if digit >= '1'
                        && digit
                            <= char::from_digit(macro_def.arguments as u32, 10).unwrap()
                    {
                        let index = digit as u8 - b'1';
                        join_tokens(&mut replacement, &args[index as usize]);
                    } else {
                        self.failed = true;
                    }
                    chunk_start = i + 1;
                }
                i += 1;
                if replacement.chars().count() > 32768 {
                    self.failed = true;
                }
            }
            join_tokens(&mut replacement, &chars_to_string(&body[chunk_start..]));
            if !self.failed {
                self.replace(start, &replacement);
            }
        }
        !self.failed && self.scopes.is_empty()
    }
}

/// Port de `expandLatexMacros`. Devuelve `None` si la expansión falla (igual
/// que el `false` del original).
pub fn expand_latex_macros(source: &str) -> Option<String> {
    if source.chars().count() > 32768 {
        return None;
    }
    if !source.contains("\\newcommand")
        && !source.contains("\\def")
        && !source.contains("\\renewcommand")
        && !source.contains("\\providecommand")
        && !source.contains("\\DeclareMathOperator")
    {
        return Some(source.to_string());
    }
    let mut expander = Expander {
        text: source.chars().collect(),
        pos: 0,
        expansions: 0,
        failed: false,
        macros: Definitions::new(),
        scopes: Vec::new(),
    };
    if !expander.run() {
        return None;
    }
    Some(expander.text.iter().collect())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn expand(src: &str) -> Option<String> {
        expand_latex_macros(src)
    }

    #[test]
    fn passthrough_without_definitions() {
        assert_eq!(expand(r"\frac{a}{b}"), Some(r"\frac{a}{b}".to_string()));
        assert_eq!(expand("x^2"), Some("x^2".to_string()));
    }

    #[test]
    fn newcommand_simple() {
        let out = expand(r"\newcommand{\R}{\mathbb{R}} \R").unwrap();
        assert_eq!(out, r" \mathbb{R}");
    }

    #[test]
    fn newcommand_with_arguments() {
        let out = expand(r"\newcommand{\f}[2]{\frac{#1}{#2}}\f{a}{b}").unwrap();
        assert_eq!(out, r"\frac{a}{b}");
    }

    #[test]
    fn newcommand_optional_default() {
        let out = expand(r"\newcommand{\pair}[2][a]{#1#2}\pair{b}").unwrap();
        assert_eq!(out, "ab");
        let out = expand(r"\newcommand{\pair}[2][a]{#1#2}\pair[c]{b}").unwrap();
        assert_eq!(out, "cb");
    }

    #[test]
    fn def_with_parameters() {
        let out = expand(r"\def\a#1{#1^2}\a{x}").unwrap();
        assert_eq!(out, "x^2");
        let out = expand(r"\def\a#1#2{#2#1}\a{x}{y}").unwrap();
        assert_eq!(out, "yx");
    }

    #[test]
    fn def_parameter_validation() {
        // Los parámetros de \def deben ir en orden #1, #2, ...
        assert!(expand(r"\def\a#2{x}").is_none());
        assert!(expand(r"\def\a#10{x}").is_none());
    }

    #[test]
    fn token_joining() {
        // Al sustituir una palabra de control seguida de una letra hay que
        // insertar un espacio para no fusionar los tokens.
        let out = expand(r"\def\a#1{#1}\a{\alpha}x").unwrap();
        assert_eq!(out, r"\alpha x");
    }

    #[test]
    fn comments_are_skipped() {
        let out = expand("\\newcommand{\\a}{x} % comentario\n\\a").unwrap();
        // El comentario no forma parte de la definición y se conserva; el
        // parser lo ignora después.
        assert_eq!(out, " % comentario\nx");
        assert!(super::super::parser::parse_latex(
            "\\newcommand{\\a}{x} % comentario\n\\a"
        )
        .is_some());
    }

    #[test]
    fn scopes_restore_definitions() {
        // La definición dentro de un grupo no sobrevive al cierre.
        let out = expand(r"{\newcommand{\a}{x}} \a").unwrap();
        assert_eq!(out, r"{} \a");
        // Un grupo sin cerrar hace fallar la expansión.
        assert!(expand(r"{\newcommand{\a}{x}").is_none());
        assert!(expand(r"\def\a{x}}").is_none());
    }

    #[test]
    fn newcommand_cannot_override_builtin() {
        assert!(expand(r"\newcommand{\sin}{x}").is_none());
        assert!(expand(r"\newcommand{\frac}{x}").is_none());
        // providecommand con un comando existente no hace nada
        let out = expand(r"\providecommand{\sin}{X}\sin").unwrap();
        assert_eq!(out, r"\sin");
        // renewcommand sí puede redefinirlo
        let out = expand(r"\renewcommand{\sin}{X}\sin").unwrap();
        assert_eq!(out, "X");
        // renewcommand de algo inexistente falla
        assert!(expand(r"\renewcommand{\zzz}{x}").is_none());
    }

    #[test]
    fn providecommand_defines_missing() {
        let out = expand(r"\providecommand{\zzz}{z}\zzz").unwrap();
        assert_eq!(out, "z");
    }

    #[test]
    fn declare_math_operator() {
        let out = expand(r"\DeclareMathOperator{\foo}{foo}\foo").unwrap();
        assert_eq!(out, r"\operatorname{foo}");
        let out = expand(r"\DeclareMathOperator*{\foo}{foo}\foo").unwrap();
        assert_eq!(out, r"\operatorname*{foo}");
        // No admite redefinir algo existente
        assert!(expand(r"\DeclareMathOperator{\sin}{foo}").is_none());
    }

    #[test]
    fn malformed_definitions() {
        assert!(expand(r"\newcommand{\a}").is_none());
        assert!(expand(r"\newcommand{\a}{x").is_none());
        assert!(expand(r"\newcommand{a}{x}").is_none());
        assert!(expand(r"\newcommand{\a}[x]{y}").is_none());
        assert!(expand(r"\newcommand{\a}[0][d]{y}").is_none());
        // \# al final del cuerpo no tiene dígito
        assert!(expand(r"\def\a{#}\a").is_none());
        // argumento de macro inexistente
        assert!(expand(r"\def\a{#2}\a{x}").is_none());
    }

    #[test]
    fn expansion_limit() {
        // Recursión infinita: se corta a las 512 expansiones.
        assert!(expand(r"\def\a{\a}\a").is_none());
    }

    #[test]
    fn length_limit() {
        let too_long = "x".repeat(32769);
        assert!(expand(&too_long).is_none());
        // La expansión tampoco puede pasar de 32768 caracteres.
        let body = "y".repeat(20000);
        let src = format!(r"\def\a{{{body}}}\a\a");
        assert!(src.chars().count() < 32768);
        assert!(expand(&src).is_none());
    }

    #[test]
    fn hashed_hash_in_body() {
        let out = expand(r"\def\a{##}\a").unwrap();
        assert_eq!(out, "#");
    }

    #[test]
    fn whitespace_before_arguments() {
        let out = expand("\\def\\a#1{#1}\\a  {x}").unwrap();
        assert_eq!(out, "x");
    }
}
