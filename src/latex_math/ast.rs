//! AST del motor matemático de Tinta, portado a Rust.
//!
//! Basado en `src/math_parser.h` de Tinta (https://github.com/oipoistar/tinta),
//! MIT, Copyright (c) 2024 oipoistar. Los nodos son equivalentes uno a uno con los del original.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Row,
    Sym,
    Frac,
    Script,
    Delim,
    Deco,
    Space,
    Grid,
    Stack,
    Phantom,
    Middle,
}

#[derive(Clone, Debug)]
pub struct MNode {
    pub kind: Kind,
    pub text: String,
    /// Slots opcionales: en Tinta son `MNodePtr` que pueden ser null
    /// (p. ej. el subíndice o el denominador ausente de un `Script`).
    pub kids: Vec<Option<Box<MNode>>>,
    pub open: char,
    pub close: char,
    pub deco_kind: i32,
    pub roman: bool,
    pub bold: bool,
    pub literal_text: bool,
    pub force_italic: bool,
    pub spacing: f32,
    pub large_op: bool,
    pub limit_op: bool,
    pub limits: i32,
    pub style: i32,
    pub no_bar: bool,
    pub delimiter_size: f32,
    pub space: f32,
    pub cells: Vec<Vec<MNode>>,
    pub alignment: String,
    pub aligned: bool,
    pub compact: bool,
    pub uniform_alignment: bool,
    pub vertical_rules: Vec<usize>,
    pub horizontal_rules: Vec<usize>,
}

impl Default for MNode {
    fn default() -> Self {
        Self {
            kind: Kind::Row,
            text: String::new(),
            kids: Vec::new(),
            open: '\0',
            close: '\0',
            deco_kind: 0,
            roman: false,
            bold: false,
            literal_text: false,
            force_italic: false,
            spacing: -1.0,
            large_op: false,
            limit_op: false,
            limits: -1,
            style: -1,
            no_bar: false,
            delimiter_size: 0.0,
            space: 0.0,
            cells: Vec::new(),
            alignment: String::new(),
            aligned: false,
            compact: false,
            uniform_alignment: false,
            vertical_rules: Vec::new(),
            horizontal_rules: Vec::new(),
        }
    }
}

impl MNode {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }

    pub fn sym(text: impl Into<String>) -> Self {
        Self {
            kind: Kind::Sym,
            text: text.into(),
            ..Default::default()
        }
    }

    pub fn kid(&self, index: usize) -> Option<&MNode> {
        self.kids.get(index).and_then(|k| k.as_deref())
    }
}
