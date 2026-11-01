//! Parser LaTeX → AST, portado de `src/math_parser.cpp` de Tinta (MIT).
//!
//! Port de Tinta (MIT, Copyright (c) 2024 oipoistar).
//!
//! Se mantienen los nombres del original (`Parser`, `parseRow`, `parseAtom`,
//! `parseCommand`, `literalGroup`, `dimension`, ...) y las tablas de símbolos
//! tal cual. `std::wstring`/`wchar_t` se traducen a `String`/`char`; el parser
//! trabaja sobre `&[char]` para poder indexar por posición como el C++.

use super::ast::{Kind, MNode};
use super::macros::expand_latex_macros;

// ---------------------------------------------------------------------------
// Command tables
// ---------------------------------------------------------------------------

pub(crate) struct CmdSym {
    pub(crate) name: &'static str,
    pub(crate) text: Option<&'static str>,
}

/// Symbols and greek letters mapped straight to Unicode text runs
const K_SYMBOLS: &[CmdSym] = &[
    CmdSym { name: "alpha", text: Some("\u{03B1}") },
    CmdSym { name: "beta", text: Some("\u{03B2}") },
    CmdSym { name: "gamma", text: Some("\u{03B3}") },
    CmdSym { name: "delta", text: Some("\u{03B4}") },
    CmdSym { name: "epsilon", text: Some("\u{03F5}") },
    CmdSym { name: "varepsilon", text: Some("\u{03B5}") },
    CmdSym { name: "zeta", text: Some("\u{03B6}") },
    CmdSym { name: "eta", text: Some("\u{03B7}") },
    CmdSym { name: "theta", text: Some("\u{03B8}") },
    CmdSym { name: "vartheta", text: Some("\u{03D1}") },
    CmdSym { name: "iota", text: Some("\u{03B9}") },
    CmdSym { name: "kappa", text: Some("\u{03BA}") },
    CmdSym { name: "lambda", text: Some("\u{03BB}") },
    CmdSym { name: "mu", text: Some("\u{03BC}") },
    CmdSym { name: "nu", text: Some("\u{03BD}") },
    CmdSym { name: "xi", text: Some("\u{03BE}") },
    CmdSym { name: "pi", text: Some("\u{03C0}") },
    CmdSym { name: "rho", text: Some("\u{03C1}") },
    CmdSym { name: "sigma", text: Some("\u{03C3}") },
    CmdSym { name: "tau", text: Some("\u{03C4}") },
    CmdSym { name: "upsilon", text: Some("\u{03C5}") },
    CmdSym { name: "phi", text: Some("\u{03D5}") },
    CmdSym { name: "varphi", text: Some("\u{03C6}") },
    CmdSym { name: "chi", text: Some("\u{03C7}") },
    CmdSym { name: "varrho", text: Some("\u{03F1}") },
    CmdSym { name: "varpi", text: Some("\u{03D6}") },
    CmdSym { name: "varsigma", text: Some("\u{03C2}") },
    CmdSym { name: "varkappa", text: Some("\u{03F0}") },
    CmdSym { name: "digamma", text: Some("\u{03DD}") },
    CmdSym { name: "omicron", text: Some("o") },
    CmdSym { name: "psi", text: Some("\u{03C8}") },
    CmdSym { name: "omega", text: Some("\u{03C9}") },
    CmdSym { name: "Gamma", text: Some("\u{0393}") },
    CmdSym { name: "Delta", text: Some("\u{0394}") },
    CmdSym { name: "Theta", text: Some("\u{0398}") },
    CmdSym { name: "Lambda", text: Some("\u{039B}") },
    CmdSym { name: "Xi", text: Some("\u{039E}") },
    CmdSym { name: "Pi", text: Some("\u{03A0}") },
    CmdSym { name: "Sigma", text: Some("\u{03A3}") },
    CmdSym { name: "Upsilon", text: Some("\u{03A5}") },
    CmdSym { name: "Phi", text: Some("\u{03A6}") },
    CmdSym { name: "Psi", text: Some("\u{03A8}") },
    CmdSym { name: "Omega", text: Some("\u{03A9}") },
    CmdSym { name: "in", text: Some("\u{2208}") },
    CmdSym { name: "notin", text: Some("\u{2209}") },
    CmdSym { name: "ni", text: Some("\u{220B}") },
    CmdSym { name: "cup", text: Some("\u{222A}") },
    CmdSym { name: "cap", text: Some("\u{2229}") },
    CmdSym { name: "subset", text: Some("\u{2282}") },
    CmdSym { name: "supset", text: Some("\u{2283}") },
    CmdSym { name: "subseteq", text: Some("\u{2286}") },
    CmdSym { name: "supseteq", text: Some("\u{2287}") },
    CmdSym { name: "geq", text: Some("\u{2265}") },
    CmdSym { name: "ge", text: Some("\u{2265}") },
    CmdSym { name: "leq", text: Some("\u{2264}") },
    CmdSym { name: "le", text: Some("\u{2264}") },
    CmdSym { name: "neq", text: Some("\u{2260}") },
    CmdSym { name: "ne", text: Some("\u{2260}") },
    CmdSym { name: "equiv", text: Some("\u{2261}") },
    CmdSym { name: "approx", text: Some("\u{2248}") },
    CmdSym { name: "sim", text: Some("\u{223C}") },
    CmdSym { name: "propto", text: Some("\u{221D}") },
    CmdSym { name: "pm", text: Some("\u{00B1}") },
    CmdSym { name: "mp", text: Some("\u{2213}") },
    CmdSym { name: "times", text: Some("\u{00D7}") },
    CmdSym { name: "div", text: Some("\u{00F7}") },
    CmdSym { name: "cdot", text: Some("\u{22C5}") },
    CmdSym { name: "cdots", text: Some("\u{22EF}") },
    CmdSym { name: "ldots", text: Some("\u{2026}") },
    CmdSym { name: "dots", text: Some("\u{2026}") },
    CmdSym { name: "vdots", text: Some("\u{22EE}") },
    CmdSym { name: "ddots", text: Some("\u{22F1}") },
    CmdSym { name: "mid", text: Some("\u{2223}") },
    CmdSym { name: "parallel", text: Some("\u{2225}") },
    CmdSym { name: "perp", text: Some("\u{22A5}") },
    CmdSym { name: "angle", text: Some("\u{2220}") },
    CmdSym { name: "triangle", text: Some("\u{25B3}") },
    CmdSym { name: "bigtriangleup", text: Some("\u{25B3}") },
    CmdSym { name: "square", text: Some("\u{25A1}") },
    CmdSym { name: "infty", text: Some("\u{221E}") },
    CmdSym { name: "partial", text: Some("\u{2202}") },
    CmdSym { name: "nabla", text: Some("\u{2207}") },
    CmdSym { name: "forall", text: Some("\u{2200}") },
    CmdSym { name: "exists", text: Some("\u{2203}") },
    CmdSym { name: "emptyset", text: Some("\u{2205}") },
    CmdSym { name: "varnothing", text: Some("\u{2205}") },
    CmdSym { name: "because", text: Some("\u{2235}") },
    CmdSym { name: "therefore", text: Some("\u{2234}") },
    CmdSym { name: "to", text: Some("\u{2192}") },
    CmdSym { name: "rightarrow", text: Some("\u{2192}") },
    CmdSym { name: "longrightarrow", text: Some("\u{27F6}") },
    CmdSym { name: "longleftarrow", text: Some("\u{27F5}") },
    CmdSym { name: "longleftrightarrow", text: Some("\u{27F7}") },
    CmdSym { name: "Longrightarrow", text: Some("\u{27F9}") },
    CmdSym { name: "Longleftarrow", text: Some("\u{27F8}") },
    CmdSym { name: "Longleftrightarrow", text: Some("\u{27FA}") },
    CmdSym { name: "implies", text: Some("\u{27F9}") },
    CmdSym { name: "impliedby", text: Some("\u{27F8}") },
    CmdSym { name: "iff", text: Some("\u{27FA}") },
    CmdSym { name: "longmapsto", text: Some("\u{27FC}") },
    CmdSym { name: "leftarrow", text: Some("\u{2190}") },
    CmdSym { name: "Rightarrow", text: Some("\u{21D2}") },
    CmdSym { name: "Leftarrow", text: Some("\u{21D0}") },
    CmdSym { name: "Leftrightarrow", text: Some("\u{21D4}") },
    CmdSym { name: "leftrightarrow", text: Some("\u{2194}") },
    CmdSym { name: "mapsto", text: Some("\u{21A6}") },
    CmdSym { name: "circ", text: Some("\u{2218}") },
    CmdSym { name: "bullet", text: Some("\u{2219}") },
    CmdSym { name: "star", text: Some("\u{22C6}") },
    CmdSym { name: "oplus", text: Some("\u{2295}") },
    CmdSym { name: "otimes", text: Some("\u{2297}") },
    CmdSym { name: "sum", text: Some("\u{2211}") },
    CmdSym { name: "prod", text: Some("\u{220F}") },
    CmdSym { name: "int", text: Some("\u{222B}") },
    CmdSym { name: "oint", text: Some("\u{222E}") },
    // structural, handled apart
    CmdSym { name: "sqrt", text: None },
    CmdSym { name: "prime", text: Some("\u{2032}") },
    CmdSym { name: "degree", text: Some("\u{00B0}") },
    CmdSym { name: "lfloor", text: Some("\u{230A}") },
    CmdSym { name: "rfloor", text: Some("\u{230B}") },
    CmdSym { name: "lceil", text: Some("\u{2308}") },
    CmdSym { name: "rceil", text: Some("\u{2309}") },
    CmdSym { name: "langle", text: Some("\u{27E8}") },
    CmdSym { name: "rangle", text: Some("\u{27E9}") },
    CmdSym { name: "lbrack", text: Some("[") },
    CmdSym { name: "rbrack", text: Some("]") },
    CmdSym { name: "lbrace", text: Some("{") },
    CmdSym { name: "rbrace", text: Some("}") },
    CmdSym { name: "lvert", text: Some("|") },
    CmdSym { name: "rvert", text: Some("|") },
    CmdSym { name: "vert", text: Some("|") },
    CmdSym { name: "lVert", text: Some("\u{2016}") },
    CmdSym { name: "rVert", text: Some("\u{2016}") },
    CmdSym { name: "Vert", text: Some("\u{2016}") },
    CmdSym { name: "backslash", text: Some("\\") },
    CmdSym { name: "setminus", text: Some("\u{2216}") },
    CmdSym { name: "hbar", text: Some("\u{210F}") },
    CmdSym { name: "ell", text: Some("\u{2113}") },
    CmdSym { name: "Re", text: Some("\u{211C}") },
    CmdSym { name: "Im", text: Some("\u{2111}") },
    CmdSym { name: "aleph", text: Some("\u{2135}") },
    CmdSym { name: "wp", text: Some("\u{2118}") },
    CmdSym { name: "neg", text: Some("\u{00AC}") },
    CmdSym { name: "lnot", text: Some("\u{00AC}") },
    CmdSym { name: "land", text: Some("\u{2227}") },
    CmdSym { name: "wedge", text: Some("\u{2227}") },
    CmdSym { name: "lor", text: Some("\u{2228}") },
    CmdSym { name: "vee", text: Some("\u{2228}") },
    CmdSym { name: "iint", text: Some("\u{222C}") },
    CmdSym { name: "iiint", text: Some("\u{222D}") },
    CmdSym { name: "oiint", text: Some("\u{222F}") },
    CmdSym { name: "coprod", text: Some("\u{2210}") },
    CmdSym { name: "bigcup", text: Some("\u{22C3}") },
    CmdSym { name: "bigcap", text: Some("\u{22C2}") },
    CmdSym { name: "bigvee", text: Some("\u{22C1}") },
    CmdSym { name: "bigwedge", text: Some("\u{22C0}") },
    CmdSym { name: "bigoplus", text: Some("\u{2A01}") },
    CmdSym { name: "bigotimes", text: Some("\u{2A02}") },
    CmdSym { name: "biguplus", text: Some("\u{2A04}") },
    CmdSym { name: "gets", text: Some("\u{2190}") },
    CmdSym { name: "uparrow", text: Some("\u{2191}") },
    CmdSym { name: "downarrow", text: Some("\u{2193}") },
    CmdSym { name: "updownarrow", text: Some("\u{2195}") },
    CmdSym { name: "Uparrow", text: Some("\u{21D1}") },
    CmdSym { name: "Downarrow", text: Some("\u{21D3}") },
    CmdSym { name: "Updownarrow", text: Some("\u{21D5}") },
    CmdSym { name: "nearrow", text: Some("\u{2197}") },
    CmdSym { name: "searrow", text: Some("\u{2198}") },
    CmdSym { name: "nwarrow", text: Some("\u{2196}") },
    CmdSym { name: "swarrow", text: Some("\u{2199}") },
    CmdSym { name: "hookrightarrow", text: Some("\u{21AA}") },
    CmdSym { name: "hookleftarrow", text: Some("\u{21A9}") },
    CmdSym { name: "rightharpoonup", text: Some("\u{21C0}") },
    CmdSym { name: "rightharpoondown", text: Some("\u{21C1}") },
    CmdSym { name: "leftharpoonup", text: Some("\u{21BC}") },
    CmdSym { name: "leftharpoondown", text: Some("\u{21BD}") },
    CmdSym { name: "rightleftharpoons", text: Some("\u{21CC}") },
    CmdSym { name: "leftrightharpoons", text: Some("\u{21CB}") },
    CmdSym { name: "leadsto", text: Some("\u{21DD}") },
    CmdSym { name: "rightsquigarrow", text: Some("\u{21DD}") },
    CmdSym { name: "twoheadrightarrow", text: Some("\u{21A0}") },
    CmdSym { name: "twoheadleftarrow", text: Some("\u{219E}") },
    CmdSym { name: "ll", text: Some("\u{226A}") },
    CmdSym { name: "gg", text: Some("\u{226B}") },
    CmdSym { name: "simeq", text: Some("\u{2243}") },
    CmdSym { name: "cong", text: Some("\u{2245}") },
    CmdSym { name: "asymp", text: Some("\u{224D}") },
    CmdSym { name: "doteq", text: Some("\u{2250}") },
    CmdSym { name: "lesssim", text: Some("\u{2272}") },
    CmdSym { name: "gtrsim", text: Some("\u{2273}") },
    CmdSym { name: "leqslant", text: Some("\u{2A7D}") },
    CmdSym { name: "geqslant", text: Some("\u{2A7E}") },
    CmdSym { name: "prec", text: Some("\u{227A}") },
    CmdSym { name: "succ", text: Some("\u{227B}") },
    CmdSym { name: "preceq", text: Some("\u{2AAF}") },
    CmdSym { name: "succeq", text: Some("\u{2AB0}") },
    CmdSym { name: "nleq", text: Some("\u{2270}") },
    CmdSym { name: "ngeq", text: Some("\u{2271}") },
    CmdSym { name: "nless", text: Some("\u{226E}") },
    CmdSym { name: "ngtr", text: Some("\u{226F}") },
    CmdSym { name: "nsubseteq", text: Some("\u{2288}") },
    CmdSym { name: "nsupseteq", text: Some("\u{2289}") },
    CmdSym { name: "subsetneq", text: Some("\u{228A}") },
    CmdSym { name: "supsetneq", text: Some("\u{228B}") },
    CmdSym { name: "nexists", text: Some("\u{2204}") },
    CmdSym { name: "vdash", text: Some("\u{22A2}") },
    CmdSym { name: "dashv", text: Some("\u{22A3}") },
    CmdSym { name: "models", text: Some("\u{22A8}") },
    CmdSym { name: "vDash", text: Some("\u{22A8}") },
    CmdSym { name: "Vdash", text: Some("\u{22A9}") },
    CmdSym { name: "top", text: Some("\u{22A4}") },
    CmdSym { name: "bot", text: Some("\u{22A5}") },
    CmdSym { name: "complement", text: Some("\u{2201}") },
    CmdSym { name: "uplus", text: Some("\u{228E}") },
    CmdSym { name: "sqcup", text: Some("\u{2294}") },
    CmdSym { name: "sqcap", text: Some("\u{2293}") },
    CmdSym { name: "odot", text: Some("\u{2299}") },
    CmdSym { name: "ominus", text: Some("\u{2296}") },
    CmdSym { name: "oslash", text: Some("\u{2298}") },
    CmdSym { name: "circledast", text: Some("\u{229B}") },
    CmdSym { name: "ast", text: Some("\u{2217}") },
    CmdSym { name: "diamond", text: Some("\u{22C4}") },
    CmdSym { name: "ltimes", text: Some("\u{22C9}") },
    CmdSym { name: "rtimes", text: Some("\u{22CA}") },
    CmdSym { name: "wr", text: Some("\u{2240}") },
    CmdSym { name: "imath", text: Some("\u{0131}") },
    CmdSym { name: "jmath", text: Some("\u{0237}") },
    CmdSym { name: "ell", text: Some("\u{2113}") },
    CmdSym { name: "lparen", text: Some("(") },
    CmdSym { name: "rparen", text: Some(")") },
    CmdSym { name: "colon", text: Some(":") },
    CmdSym { name: "dotsc", text: Some("\u{2026}") },
    CmdSym { name: "dotsb", text: Some("\u{22EF}") },
    CmdSym { name: "dotsm", text: Some("\u{22EF}") },
];

/// Function names rendered upright with a trailing thin space
const K_FUNCTIONS: &[&str] = &[
    "log", "ln", "lg", "sin", "cos", "tan", "cot", "sec", "csc", "arcsin", "arccos", "arctan",
    "sinh", "cosh", "tanh", "coth", "lim", "limsup", "liminf", "max", "min", "sup", "inf", "gcd",
    "det", "dim", "ker", "deg", "arg", "exp", "mod", "Pr", "hom", "erf", "sgn",
];

fn lookup_symbol(cmd: &str) -> Option<&'static str> {
    K_SYMBOLS.iter().find(|s| s.name == cmd).and_then(|s| s.text)
}

fn is_function_name(cmd: &str) -> bool {
    K_FUNCTIONS.contains(&cmd)
}

/// Port de `codepoint`. En el original `wchar_t` puede ser UTF-16 (Windows);
/// aquí `char` siempre es un escalar Unicode.
fn codepoint(cp: u32) -> char {
    char::from_u32(cp).unwrap_or('\u{FFFD}')
}

/// Port de `mathAlphabet`.
fn math_alphabet(node: &mut MNode, command: &str) {
    node.roman = true;
    if node.kind == Kind::Sym {
        let mut mapped = String::new();
        for c in node.text.chars() {
            let mut cp = c as u32;
            let upper = c.is_ascii_uppercase();
            let lower = c.is_ascii_lowercase();
            if command == "mathbb" {
                if upper {
                    cp = 0x1D538 + c as u32 - 'A' as u32;
                }
                if lower {
                    cp = 0x1D552 + c as u32 - 'a' as u32;
                }
                if c.is_ascii_digit() {
                    cp = 0x1D7D8 + c as u32 - '0' as u32;
                }
                match c {
                    'C' => cp = 0x2102,
                    'H' => cp = 0x210D,
                    'N' => cp = 0x2115,
                    'P' => cp = 0x2119,
                    'Q' => cp = 0x211A,
                    'R' => cp = 0x211D,
                    'Z' => cp = 0x2124,
                    _ => {}
                }
            } else if command == "mathcal" || command == "mathscr" {
                if upper {
                    cp = 0x1D49C + c as u32 - 'A' as u32;
                }
                if lower {
                    cp = 0x1D4B6 + c as u32 - 'a' as u32;
                }
                match c {
                    'B' => cp = 0x212C,
                    'E' => cp = 0x2130,
                    'F' => cp = 0x2131,
                    'H' => cp = 0x210B,
                    'I' => cp = 0x2110,
                    'L' => cp = 0x2112,
                    'M' => cp = 0x2133,
                    'R' => cp = 0x211B,
                    'e' => cp = 0x212F,
                    'g' => cp = 0x210A,
                    'o' => cp = 0x2134,
                    _ => {}
                }
            } else if command == "mathfrak" {
                if upper {
                    cp = 0x1D504 + c as u32 - 'A' as u32;
                }
                if lower {
                    cp = 0x1D51E + c as u32 - 'a' as u32;
                }
                match c {
                    'C' => cp = 0x212D,
                    'H' => cp = 0x210C,
                    'I' => cp = 0x2111,
                    'R' => cp = 0x211C,
                    'Z' => cp = 0x2128,
                    _ => {}
                }
            } else {
                let base = if command == "mathsf" { 0x1D5A0 } else { 0x1D670 };
                if upper {
                    cp = base + c as u32 - 'A' as u32;
                }
                if lower {
                    cp = base + 26 + c as u32 - 'a' as u32;
                }
                if c.is_ascii_digit() {
                    cp = (if command == "mathsf" { 0x1D7E2 } else { 0x1D7F6 })
                        + c as u32
                        - '0' as u32;
                }
            }
            mapped.push(codepoint(cp));
        }
        node.text = mapped;
    }
    for kid in node.kids.iter_mut().flatten() {
        math_alphabet(kid, command);
    }
    for row in node.cells.iter_mut() {
        for cell in row.iter_mut() {
            math_alphabet(cell, command);
        }
    }
}

/// Port de `symbolSpacing`. (El layout calcula el espaciado real; aquí se
/// conserva por fidelidad con el original.)
#[allow(dead_code)]
pub fn symbol_spacing(t: &str, roman: bool) -> f32 {
    let chars: Vec<char> = t.chars().collect();
    if chars.len() != 1 {
        if t == "\u{22EF}" || t == "\u{2026}" {
            return 0.16;
        }
        // Upright function names (sin, log, ...) take thin spaces so
        // \sin m\alpha reads "sin mα" rather than "sinmα"
        if roman {
            let all_alpha = chars.iter().all(|c| c.is_alphabetic());
            if all_alpha {
                return 0.16;
            }
        }
        return 0.0;
    }
    let c = chars[0] as u32;
    if (0x2190..=0x21FF).contains(&c)
        || (0x27F0..=0x27FF).contains(&c)
        || (0x223C..=0x223D).contains(&c)
        || (0x2243..=0x228D).contains(&c)
        || (0x22A2..=0x22AF).contains(&c)
        || c == 0x2A7D
        || c == 0x2A7E
        || c == 0x2AAF
        || c == 0x2AB0
    {
        return 0.26;
    }
    match c {
        0x3D | 0x3C | 0x3E | 0x2264 | 0x2265 | 0x2260 | 0x2261 | 0x2248 | 0x2208 | 0x2209
        | 0x2282 | 0x2283 | 0x2286 | 0x2287 | 0x2192 | 0x21D2 | 0x2194 | 0x21D4 | 0x223C
        | 0x221D | 0x2223 | 0x2225 | 0x2190 | 0x21D0 | 0x21A6 | 0x27F5 | 0x27F6 | 0x27F7
        | 0x27F8 | 0x27F9 | 0x27FA | 0x27FC => 0.26, // relations
        0x2B | 0x2212 | 0x00B1 | 0x2213 | 0x00D7 | 0x00F7 | 0x22C5 | 0x222A | 0x2229 | 0x2227
        | 0x2228 | 0x2216 | 0x2295 | 0x2296 | 0x2297 | 0x2298 | 0x2299 | 0x229B | 0x228E
        | 0x2293 | 0x2294 | 0x22C9 | 0x22CA => 0.2, // binary operators
        0x2C | 0x3B => 0.12, // punctuation (space after only, approximated)
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Tokenizer + parser
// ---------------------------------------------------------------------------

struct Parser<'a> {
    src: &'a [char],
    pos: usize,
    failed: bool,
    depth: i32,
    atoms: usize,
    delimiter_depth: i32,
}

fn make(kind: Kind) -> Box<MNode> {
    Box::new(MNode::new(kind))
}

impl<'a> Parser<'a> {
    fn new(src: &'a [char]) -> Self {
        Self {
            src,
            pos: 0,
            failed: false,
            depth: 0,
            atoms: 0,
            delimiter_depth: 0,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() {
            if self.src[self.pos].is_whitespace() {
                self.pos += 1;
                continue;
            }
            if self.src[self.pos] != '%' {
                break;
            }
            while self.pos < self.src.len() && self.src[self.pos] != '\n' {
                self.pos += 1;
            }
        }
    }

    fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn peek(&self) -> char {
        if self.pos < self.src.len() {
            self.src[self.pos]
        } else {
            '\0'
        }
    }

    fn starts_with_at(&self, at: usize, pat: &str) -> bool {
        let mut i = at;
        for c in pat.chars() {
            if i >= self.src.len() || self.src[i] != c {
                return false;
            }
            i += 1;
        }
        true
    }

    fn at_command(&self, cmd: &str) -> bool {
        if !self.starts_with_at(self.pos, cmd) {
            return false;
        }
        let n = cmd.chars().count();
        self.pos + n == self.src.len() || !self.src[self.pos + n].is_alphabetic()
    }

    fn literal_group(&mut self) -> String {
        self.skip_ws();
        if self.peek() != '{' {
            self.failed = true;
            return String::new();
        }
        self.pos += 1;
        let start = self.pos;
        while !self.eof()
            && self.peek() != '}'
            && self.peek() != '{'
            && self.peek() != '\\'
        {
            self.pos += 1;
        }
        let value: String = self.src[start..self.pos].iter().collect();
        if self.peek() != '}' {
            self.failed = true;
        } else {
            self.pos += 1;
        }
        value
    }

    fn read_delimiter(&mut self) -> char {
        self.skip_ws();
        if self.eof() {
            self.failed = true;
            return '\0';
        }
        let mut value = self.src[self.pos];
        self.pos += 1;
        if value == '\\' {
            let name = self.read_command();
            if name == "{" || name == "}" {
                value = name.chars().next().unwrap();
            } else if name == "|" {
                value = '\u{2016}';
            } else {
                match lookup_symbol(&name) {
                    Some(symbol) if symbol.chars().count() == 1 => {
                        value = symbol.chars().next().unwrap();
                    }
                    _ => {
                        self.failed = true;
                        return '\0';
                    }
                }
            }
        }
        const DELIMITERS: &str =
            ".()[]{}|/\\\u{2016}\u{2223}\u{2225}\u{2308}\u{2309}\u{230A}\u{230B}\u{27E8}\u{27E9}\u{2191}\u{2193}\u{2195}\u{21D1}\u{21D3}\u{21D5}";
        if !DELIMITERS.contains(value) {
            self.failed = true;
        }
        if value == '.' {
            '\0'
        } else {
            value
        }
    }

    fn dimension(&mut self) -> Option<Box<MNode>> {
        self.skip_ws();
        let group = self.peek() == '{';
        if group {
            self.pos += 1;
        }
        let start = self.pos;
        let Some((number, used)) = scan_number(self.src, start) else {
            self.failed = true;
            return None;
        };
        if !number.is_finite() || number.abs() > 1000.0 {
            self.failed = true;
            return None;
        }
        self.pos += used;
        self.skip_ws();
        let unit: String = self.src[self.pos..(self.pos + 2).min(self.src.len())]
            .iter()
            .collect();
        if unit.chars().count() == 2 {
            self.pos += 2;
        }
        let mut space = make(Kind::Space);
        space.space = number as f32;
        if unit == "ex" {
            space.space *= 0.5;
        } else if unit == "mu" {
            space.space /= 18.0;
        } else if unit == "pt" || unit == "px" {
            space.text = unit;
        } else if unit != "em" {
            self.failed = true;
        }
        self.skip_ws();
        if group {
            if self.peek() != '}' {
                self.failed = true;
            } else {
                self.pos += 1;
            }
        }
        Some(space)
    }

    fn text_group(&mut self) -> Option<Box<MNode>> {
        self.depth += 1;
        let result = self.text_group_inner();
        self.depth -= 1;
        result
    }

    fn text_group_inner(&mut self) -> Option<Box<MNode>> {
        if self.depth > 96 {
            self.failed = true;
            return None;
        }
        self.skip_ws();
        if self.peek() != '{' {
            self.failed = true;
            return None;
        }
        self.pos += 1;
        let mut row = make(Kind::Row);
        row.literal_text = true;
        let mut text = String::new();
        while !self.eof() && !self.failed && self.peek() != '}' {
            if self.peek() == '{' {
                flush_text(&mut row, &mut text);
                row.kids.push(self.text_group());
                continue;
            }
            let c = self.src[self.pos];
            self.pos += 1;
            if c == '%' {
                while !self.eof() && self.peek() != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            if c == '\\' {
                let cmd = self.read_command();
                if cmd.chars().count() == 1
                    && "{}$%&#_ ".contains(cmd.chars().next().unwrap())
                {
                    text.push_str(&cmd);
                    continue;
                }
                if cmd == "textbf" || cmd == "textit" || cmd == "text" || cmd == "textrm" {
                    flush_text(&mut row, &mut text);
                    let mut child = self.text_group();
                    if let Some(child) = child.as_mut() {
                        if cmd == "textbf" {
                            mark_bold(child);
                        }
                        if cmd == "textit" {
                            mark_italic(child);
                        }
                    }
                    row.kids.push(child);
                    continue;
                }
                self.failed = true;
                break;
            }
            text.push(if c == '~' || c.is_whitespace() { ' ' } else { c });
        }
        flush_text(&mut row, &mut text);
        if self.peek() != '}' {
            self.failed = true;
        } else {
            self.pos += 1;
        }
        Some(row)
    }

    fn parse_environment(&mut self) -> Option<Box<MNode>> {
        let name = self.literal_group();
        let end_name = name.clone();
        let starred = name.ends_with('*');
        let name = if starred {
            name[..name.len() - 1].to_string()
        } else {
            name.clone()
        };
        let mut grid = make(Kind::Grid);
        let mut open = '\0';
        let mut close = '\0';
        if name == "pmatrix" {
            open = '(';
            close = ')';
        } else if name == "bmatrix" {
            open = '[';
            close = ']';
        } else if name == "Bmatrix" {
            open = '{';
            close = '}';
        } else if name == "vmatrix" {
            open = '|';
            close = '|';
        } else if name == "Vmatrix" {
            open = '\u{2016}';
            close = '\u{2016}';
        } else if name == "cases" || name == "rcases" {
            if name == "cases" {
                open = '{';
            } else {
                close = '}';
            }
            grid.alignment = "ll".to_string();
        } else if name == "aligned" || name == "split" {
            grid.aligned = true;
        } else if name == "smallmatrix" {
            grid.compact = true;
        } else if name == "array" {
            let spec = self.literal_group();
            for c in spec.chars() {
                if c == '|' {
                    let index = grid.alignment.chars().count();
                    grid.vertical_rules.push(index);
                } else if c == 'l' || c == 'c' || c == 'r' {
                    grid.alignment.push(c);
                } else if !c.is_whitespace() {
                    self.failed = true;
                }
            }
            let len = grid.alignment.chars().count();
            if len == 0 || len > 64 {
                self.failed = true;
            }
        } else if name != "matrix" && name != "gathered" {
            self.failed = true;
        }
        if starred {
            if !name.contains("matrix") || name == "smallmatrix" {
                self.failed = true;
            }
            self.skip_ws();
            if self.peek() == '[' {
                self.pos += 1;
                self.skip_ws();
                let align = self.peek();
                if !self.eof() {
                    self.pos += 1;
                }
                self.skip_ws();
                if (align != 'l' && align != 'c' && align != 'r') || self.peek() != ']' {
                    self.failed = true;
                } else {
                    self.pos += 1;
                }
                grid.alignment = align.to_string();
                grid.uniform_alignment = true;
            }
        }
        if self.failed {
            return None;
        }

        grid.cells.push(Vec::new());
        while !self.failed {
            self.skip_ws();
            while name == "array"
                && grid.cells.last().unwrap().is_empty()
                && self.at_command("\\hline")
            {
                grid.horizontal_rules.push(grid.cells.len() - 1);
                self.pos += 6;
                self.skip_ws();
            }
            if self.at_command("\\end")
                && grid.cells.last().unwrap().is_empty()
                && grid.cells.len() > 1
            {
                grid.cells.pop();
                break;
            }
            if grid.cells.len() > 64 || grid.cells.last().unwrap().len() >= 64 {
                self.failed = true;
                break;
            }
            let row = self.parse_row('\0', false, true);
            let cell = match row {
                Some(row) => *row,
                None => MNode::new(Kind::Row),
            };
            grid.cells.last_mut().unwrap().push(cell);
            self.skip_ws();
            if self.peek() == '&' {
                self.pos += 1;
                continue;
            }
            if self.starts_with_at(self.pos, "\\\\") || self.at_command("\\cr") {
                let step = if self.peek() == '\\'
                    && self.pos + 1 < self.src.len()
                    && self.src[self.pos + 1] == '\\'
                {
                    2
                } else {
                    3
                };
                self.pos += step;
                self.skip_ws();
                if self.at_command("\\end") {
                    break; // optional final row separator
                }
                grid.cells.push(Vec::new());
                continue;
            }
            break;
        }
        if !self.at_command("\\end") {
            self.failed = true;
            return None;
        }
        self.pos += 4;
        if self.literal_group() != end_name {
            self.failed = true;
        }
        for row in &grid.cells {
            let align_len = grid.alignment.chars().count();
            if (!grid.uniform_alignment && align_len != 0 && row.len() > align_len)
                || (name == "gathered" && row.len() != 1)
                || (name == "split" && row.len() > 2)
            {
                self.failed = true;
            }
        }
        if open == '\0' && close == '\0' {
            return Some(grid);
        }
        let mut delim = make(Kind::Delim);
        delim.open = open;
        delim.close = close;
        delim.kids = vec![Some(grid)];
        Some(delim)
    }

    /// Después de la barra invertida.
    fn read_command(&mut self) -> String {
        let mut cmd = String::new();
        while self.pos < self.src.len() && self.src[self.pos].is_alphabetic() {
            cmd.push(self.src[self.pos]);
            self.pos += 1;
        }
        if cmd.is_empty() && self.pos < self.src.len() {
            cmd.push(self.src[self.pos]);
            self.pos += 1;
        }
        cmd
    }

    /// One brace group or a single atom (TeX: \frac ab == \frac{a}{b})
    fn parse_arg(&mut self) -> Option<Box<MNode>> {
        self.skip_ws();
        if self.peek() == '{' {
            self.pos += 1;
            let row = self.parse_row('}', false, false);
            if self.peek() == '}' {
                self.pos += 1;
            } else {
                self.failed = true;
            }
            return row;
        }
        let atom = self.parse_atom();
        if atom.is_none() {
            self.failed = true;
        }
        atom
    }

    fn parse_atom(&mut self) -> Option<Box<MNode>> {
        self.depth += 1;
        let result = self.parse_atom_inner();
        self.depth -= 1;
        result
    }

    fn parse_atom_inner(&mut self) -> Option<Box<MNode>> {
        self.atoms += 1;
        if self.depth > 96 || self.atoms > 4096 {
            self.failed = true;
            return None;
        }
        self.skip_ws();
        if self.eof() {
            return None;
        }
        let c = self.src[self.pos];
        if c == '}' || c == '&' || c == '^' || c == '_' {
            self.failed = true;
            return None;
        }

        if c == '{' {
            self.pos += 1;
            let row = self.parse_row('}', false, false);
            if self.peek() == '}' {
                self.pos += 1;
            } else {
                self.failed = true;
            }
            return row;
        }
        if c == '\\' {
            self.pos += 1;
            return self.parse_command();
        }
        self.pos += 1;
        let mut sym = make(Kind::Sym);
        if c == '-' {
            sym.text = "\u{2212}".to_string(); // proper minus
        } else if c == '\'' {
            sym.text = "\u{2032}".to_string(); // prime
        } else if c == '*' {
            sym.text = "\u{2217}".to_string();
        } else {
            sym.text = c.to_string();
        }
        Some(sym)
    }

    fn parse_command(&mut self) -> Option<Box<MNode>> {
        let cmd = self.read_command();
        if cmd.is_empty() {
            self.failed = true;
            return None;
        }
        if cmd == "begin" {
            return self.parse_environment();
        }
        if cmd == "hspace" || cmd == "kern" || cmd == "mkern" {
            if cmd == "hspace" && self.peek() == '*' {
                self.pos += 1;
            }
            return self.dimension();
        }
        if cmd == "pmod" || cmd == "bmod" || cmd == "pod" {
            let mut row = make(Kind::Row);
            row.spacing = if cmd == "bmod" { 0.2 } else { 0.3 };
            if cmd != "pod" {
                let mut mod_sym = make(Kind::Sym);
                mod_sym.roman = true;
                mod_sym.text = "mod".to_string();
                row.kids.push(Some(mod_sym));
                let mut space = make(Kind::Space);
                space.space = 0.33;
                row.kids.push(Some(space));
            }
            if cmd != "bmod" {
                row.kids.push(self.parse_arg());
                let mut delim = make(Kind::Delim);
                delim.open = '(';
                delim.close = ')';
                delim.kids = vec![Some(row)];
                delim.spacing = 0.3;
                return Some(delim);
            }
            return Some(row);
        }
        const SIZES: &[(&str, f32)] = &[("big", 1.2), ("Big", 1.6), ("bigg", 2.0), ("Bigg", 2.4)];
        for &(name, size) in SIZES {
            if cmd == name
                || cmd == format!("{name}l")
                || cmd == format!("{name}r")
                || cmd == format!("{name}m")
            {
                let mut delimiter = make(Kind::Middle);
                delimiter.open = self.read_delimiter();
                delimiter.delimiter_size = size;
                return Some(delimiter);
            }
        }
        if cmd == "middle" {
            if self.delimiter_depth == 0 {
                self.failed = true;
                return None;
            }
            let mut delimiter = make(Kind::Middle);
            delimiter.open = self.read_delimiter();
            delimiter.spacing = 0.26;
            return Some(delimiter);
        }
        if cmd == "text" || cmd == "textrm" || cmd == "textbf" || cmd == "textit" {
            let mut text = self.text_group();
            if let Some(text) = text.as_mut() {
                if cmd == "textbf" {
                    mark_bold(text);
                }
                if cmd == "textit" {
                    mark_italic(text);
                }
            }
            return text;
        }
        if cmd == "operatorname" {
            let limits = self.peek() == '*';
            if limits {
                self.pos += 1;
            }
            let mut op = self.text_group();
            if let Some(op) = op.as_mut() {
                op.spacing = 0.16;
                op.limit_op = limits;
            }
            return op;
        }

        // Escapes and explicit spacing
        if cmd == "{" || cmd == "}" || cmd == "$" || cmd == "%" || cmd == "&" || cmd == "#"
            || cmd == "_"
        {
            let mut s = make(Kind::Sym);
            s.text = cmd;
            return Some(s);
        }
        if cmd == "|" {
            let mut s = make(Kind::Sym);
            s.text = "\u{2016}".to_string();
            return Some(s);
        }
        if cmd == "," || cmd == ":" || cmd == ";" || cmd == " " || cmd == "quad"
            || cmd == "qquad" || cmd == "!" || cmd == ">" || cmd == "thinspace"
            || cmd == "medspace" || cmd == "thickspace" || cmd == "enspace"
            || cmd == "negthinspace"
        {
            let mut s = make(Kind::Space);
            if cmd == "," || cmd == "thinspace" {
                s.space = 3.0 / 18.0;
            } else if cmd == ":" || cmd == ">" || cmd == "medspace" {
                s.space = 4.0 / 18.0;
            } else if cmd == ";" || cmd == "thickspace" {
                s.space = 5.0 / 18.0;
            } else if cmd == " " {
                s.space = 0.33;
            } else if cmd == "quad" {
                s.space = 1.0;
            } else if cmd == "qquad" {
                s.space = 2.0;
            } else if cmd == "enspace" {
                s.space = 0.5;
            } else {
                s.space = -3.0 / 18.0;
            }
            return Some(s);
        }

        if cmd == "frac"
            || cmd == "dfrac"
            || cmd == "tfrac"
            || cmd == "cfrac"
            || cmd == "binom"
            || cmd == "dbinom"
            || cmd == "tbinom"
        {
            let mut f = make(Kind::Frac);
            if cmd == "dfrac" || cmd == "dbinom" || cmd == "cfrac" {
                f.style = 0;
            }
            if cmd == "tfrac" || cmd == "tbinom" {
                f.style = 1;
            }
            f.kids.push(self.parse_arg());
            f.kids.push(self.parse_arg());
            if f.kids[0].is_none() || f.kids[1].is_none() {
                self.failed = true;
            }
            if cmd == "binom" || cmd == "dbinom" || cmd == "tbinom" {
                f.no_bar = true;
                let mut d = make(Kind::Delim);
                d.open = '(';
                d.close = ')';
                d.kids = vec![Some(f)];
                return Some(d);
            }
            return Some(f);
        }
        if cmd == "sqrt" {
            let mut d = make(Kind::Deco);
            d.deco_kind = 3;
            self.skip_ws();
            let mut index = None;
            if self.peek() == '[' {
                self.pos += 1;
                index = self.parse_row(']', false, false);
                if self.peek() != ']' {
                    self.failed = true;
                } else {
                    self.pos += 1;
                }
            }
            d.kids.push(self.parse_arg());
            d.kids.push(index);
            if d.kids[0].is_none() {
                self.failed = true;
            }
            return Some(d);
        }
        if cmd == "overset" || cmd == "underset" || cmd == "stackrel" {
            let annotation = self.parse_arg();
            let base = self.parse_arg();
            if annotation.is_none() || base.is_none() {
                self.failed = true;
            }
            let mut stack = make(Kind::Stack);
            stack.kids = vec![
                base,
                if cmd == "underset" { annotation.clone() } else { None },
                if cmd != "underset" { annotation } else { None },
            ];
            return Some(stack);
        }
        if cmd == "xrightarrow" || cmd == "xleftarrow" {
            let mut stack = make(Kind::Stack);
            stack.deco_kind = if cmd == "xrightarrow" { 1 } else { 2 };
            stack.spacing = 0.26;
            self.skip_ws();
            let mut below = None;
            if self.peek() == '[' {
                self.pos += 1;
                below = self.parse_row(']', false, false);
                if self.peek() != ']' {
                    self.failed = true;
                } else {
                    self.pos += 1;
                }
            }
            let above = self.parse_arg();
            if above.is_none() {
                self.failed = true;
            }
            stack.kids = vec![None, below, above];
            return Some(stack);
        }
        if cmd == "substack" {
            self.skip_ws();
            if self.peek() != '{' {
                self.failed = true;
                return None;
            }
            self.pos += 1;
            let mut grid = make(Kind::Grid);
            // The surrounding script already supplies the smaller math style.
            loop {
                grid.cells.push(vec![match self.parse_row('}', false, true) {
                    Some(row) => *row,
                    None => MNode::new(Kind::Row),
                }]);
                if grid.cells.len() > 64 {
                    self.failed = true;
                    break;
                }
                if !self.starts_with_at(self.pos, "\\\\") {
                    break;
                }
                self.pos += 2;
                if self.failed {
                    break;
                }
            }
            if self.peek() != '}' {
                self.failed = true;
            } else {
                self.pos += 1;
            }
            return Some(grid);
        }
        const DECORATIONS: &[(&str, i32)] = &[
            ("dot", 5),
            ("ddot", 6),
            ("tilde", 7),
            ("widetilde", 7),
            ("underline", 8),
            ("boxed", 9),
            ("cancel", 10),
            ("bcancel", 11),
            ("overbrace", 12),
            ("underbrace", 13),
            ("acute", 14),
            ("grave", 15),
            ("breve", 16),
            ("check", 17),
            ("not", 18),
            ("xcancel", 19),
        ];
        for &(name, value) in DECORATIONS {
            if cmd == name {
                let mut d = make(Kind::Deco);
                d.deco_kind = value;
                d.kids = vec![self.parse_arg()];
                d.limit_op = d.deco_kind == 12 || d.deco_kind == 13;
                if d.kids[0].is_none() {
                    self.failed = true;
                }
                return Some(d);
            }
        }
        if cmd == "phantom" || cmd == "hphantom" || cmd == "vphantom" || cmd == "smash" {
            let mut p = make(Kind::Phantom);
            p.deco_kind = if cmd == "hphantom" {
                1
            } else if cmd == "vphantom" {
                2
            } else if cmd == "smash" {
                3
            } else {
                0
            };
            p.kids = vec![self.parse_arg()];
            if p.kids[0].is_none() {
                self.failed = true;
            }
            return Some(p);
        }
        if cmd == "overline" || cmd == "bar" {
            let mut d = make(Kind::Deco);
            d.deco_kind = 1;
            d.kids.push(self.parse_arg());
            return Some(d);
        }
        if cmd == "overrightarrow" || cmd == "vec" {
            let mut d = make(Kind::Deco);
            d.deco_kind = 2;
            d.kids.push(self.parse_arg());
            return Some(d);
        }
        if cmd == "hat" || cmd == "widehat" {
            let mut d = make(Kind::Deco);
            d.deco_kind = 4;
            d.kids.push(self.parse_arg());
            return Some(d);
        }
        if cmd == "left" {
            self.delimiter_depth += 1;
            let open = self.read_delimiter();
            let content = self.parse_row('\0', true, false);
            let mut close = '\0';
            if !self.at_command("\\right") {
                self.failed = true;
            } else {
                self.pos += 6;
                close = self.read_delimiter();
            }
            self.delimiter_depth -= 1;
            let mut d = make(Kind::Delim);
            d.open = open;
            d.close = close;
            d.kids = vec![content.or_else(|| Some(make(Kind::Row)))];
            return Some(d);
        }
        if cmd == "mathrm" {
            let arg = self.parse_arg();
            if let Some(mut arg) = arg {
                mark_roman(&mut arg);
                return Some(arg);
            }
            return None;
        }
        if cmd == "mathbf" || cmd == "boldsymbol" || cmd == "bm" {
            let mut arg = self.parse_arg();
            if let Some(node) = arg.as_mut() {
                mark_bold(node);
            }
            if let Some(node) = arg.as_mut() {
                if cmd == "mathbf" {
                    mark_roman(node);
                }
            }
            return arg;
        }
        if cmd == "mathbb"
            || cmd == "mathcal"
            || cmd == "mathscr"
            || cmd == "mathfrak"
            || cmd == "mathsf"
            || cmd == "mathtt"
        {
            let mut arg = self.parse_arg();
            if let Some(node) = arg.as_mut() {
                math_alphabet(node, &cmd);
            }
            return arg;
        }
        if cmd == "mathit" {
            let mut arg = self.parse_arg();
            if let Some(node) = arg.as_mut() {
                mark_italic(node);
            }
            return arg;
        }
        // Convenience spellings remain definable with DeclareMathOperator,
        // as they are not predefined operators in amsmath.
        if is_function_name(&cmd) || cmd == "argmax" || cmd == "argmin" {
            let mut f = make(Kind::Sym);
            f.text = cmd.clone();
            f.roman = true;
            f.spacing = 0.16;
            f.limit_op = cmd == "lim"
                || cmd == "limsup"
                || cmd == "liminf"
                || cmd == "max"
                || cmd == "min"
                || cmd == "sup"
                || cmd == "inf"
                || cmd == "argmax"
                || cmd == "argmin";
            return Some(f);
        }
        if let Some(sym) = lookup_symbol(&cmd) {
            let mut s = make(Kind::Sym);
            s.text = sym.to_string();
            s.large_op = cmd == "sum"
                || cmd == "prod"
                || cmd == "coprod"
                || cmd == "int"
                || cmd == "iint"
                || cmd == "iiint"
                || cmd == "oint"
                || cmd == "oiint"
                || cmd == "bigcup"
                || cmd == "bigcap"
                || cmd == "bigvee"
                || cmd == "bigwedge"
                || cmd == "bigoplus"
                || cmd == "bigotimes"
                || cmd == "biguplus";
            s.limit_op = s.large_op && !cmd.contains("int");
            if s.large_op {
                s.spacing = 0.16;
            }
            return Some(s);
        }

        self.failed = true; // unknown command: whole span falls back to source
        None
    }

    fn parse_row(
        &mut self,
        terminator: char,
        stop_at_right: bool,
        cell: bool,
    ) -> Option<Box<MNode>> {
        self.depth += 1;
        let result = self.parse_row_inner(terminator, stop_at_right, cell);
        self.depth -= 1;
        result
    }

    fn parse_row_inner(
        &mut self,
        terminator: char,
        stop_at_right: bool,
        cell: bool,
    ) -> Option<Box<MNode>> {
        if self.depth > 96 {
            self.failed = true;
            return None;
        }
        let mut row = make(Kind::Row);
        let mut active_style: i32 = -1;
        while !self.eof() && !self.failed {
            self.skip_ws();
            if self.eof() {
                break;
            }
            if terminator != '\0' && self.peek() == terminator {
                break;
            }
            if cell
                && (self.peek() == '&'
                    || self.starts_with_at(self.pos, "\\\\")
                    || self.at_command("\\cr")
                    || self.at_command("\\end"))
            {
                break;
            }
            if stop_at_right && self.at_command("\\right") {
                break;
            }
            const STYLES: &[&str] = &[
                "\\displaystyle",
                "\\textstyle",
                "\\scriptstyle",
                "\\scriptscriptstyle",
            ];
            let mut directive = false;
            for (style, name) in STYLES.iter().enumerate() {
                if self.at_command(name) {
                    self.pos += name.chars().count();
                    active_style = style as i32;
                    directive = true;
                    break;
                }
            }
            if directive {
                continue;
            }
            if self.at_command("\\limits") || self.at_command("\\nolimits") {
                let limits = self.at_command("\\limits");
                self.pos += if limits { 7 } else { 9 };
                let Some(base) = row.kids.pop() else {
                    self.failed = true;
                    break;
                };
                let mut base = base.unwrap();
                if base.kind == Kind::Script {
                    base = base.kids[0].take().unwrap();
                }
                if !base.large_op && !base.limit_op {
                    self.failed = true;
                    break;
                }
                base.limits = if limits { 1 } else { 0 };
                row.kids.push(Some(base));
                continue;
            }
            if self.peek() == '^' || self.peek() == '_' {
                // Attach scripts to the previous atom (or an empty base)
                let base = if row.kids.is_empty() {
                    make(Kind::Row)
                } else {
                    row.kids.pop().unwrap().unwrap()
                };
                let mut script = if base.kind == Kind::Script {
                    base // x_1^m: add the second script
                } else {
                    let mut script = make(Kind::Script);
                    script.kids = vec![Some(base), None, None];
                    script.style = active_style;
                    script
                };
                while !self.eof() && (self.peek() == '^' || self.peek() == '_') {
                    let which = self.src[self.pos];
                    self.pos += 1;
                    let arg = self.parse_arg();
                    let Some(arg) = arg else {
                        self.failed = true;
                        break;
                    };
                    let slot = if which == '^' { 2 } else { 1 };
                    if script.kids[slot].is_some() {
                        self.failed = true;
                        break;
                    }
                    script.kids[slot] = Some(arg);
                    self.skip_ws();
                }
                row.kids.push(Some(script));
                continue;
            }
            let atom = self.parse_atom();
            if let Some(mut atom) = atom {
                if atom.style < 0 {
                    atom.style = active_style;
                }
                row.kids.push(Some(atom));
            }
        }
        Some(row)
    }
}

fn flush_text(row: &mut MNode, text: &mut String) {
    if text.is_empty() {
        return;
    }
    let mut run = MNode::new(Kind::Sym);
    run.text = std::mem::take(text);
    run.roman = true;
    run.literal_text = true;
    row.kids.push(Some(Box::new(run)));
}

fn mark_roman(n: &mut MNode) {
    n.roman = true;
    for k in n.kids.iter_mut().flatten() {
        mark_roman(k);
    }
    for row in n.cells.iter_mut() {
        for cell in row.iter_mut() {
            mark_roman(cell);
        }
    }
}

fn mark_bold(n: &mut MNode) {
    n.bold = true;
    for k in n.kids.iter_mut().flatten() {
        mark_bold(k);
    }
    for row in n.cells.iter_mut() {
        for cell in row.iter_mut() {
            mark_bold(cell);
        }
    }
}

fn mark_italic(n: &mut MNode) {
    n.force_italic = true;
    for k in n.kids.iter_mut().flatten() {
        mark_italic(k);
    }
    for row in n.cells.iter_mut() {
        for cell in row.iter_mut() {
            mark_italic(cell);
        }
    }
}

/// Escanea un número al estilo de `wcstod` (signo, dígitos, decimal y
/// exponente). Devuelve el valor y cuántos caracteres consumió.
fn scan_number(src: &[char], start: usize) -> Option<(f64, usize)> {
    let n = src.len();
    let mut i = start;
    if i < n && (src[i] == '+' || src[i] == '-') {
        i += 1;
    }
    let digits_start = i;
    while i < n && src[i].is_ascii_digit() {
        i += 1;
    }
    let mut digits = i - digits_start;
    if i < n && src[i] == '.' {
        i += 1;
        while i < n && src[i].is_ascii_digit() {
            i += 1;
        }
        digits = i - digits_start;
    }
    if digits == 0 {
        return None;
    }
    if i < n && (src[i] == 'e' || src[i] == 'E') {
        let mut j = i + 1;
        if j < n && (src[j] == '+' || src[j] == '-') {
            j += 1;
        }
        let exp_start = j;
        while j < n && src[j].is_ascii_digit() {
            j += 1;
        }
        if j > exp_start {
            i = j;
        }
    }
    let text: String = src[start..i].iter().collect();
    let value: f64 = text.parse().ok()?;
    Some((value, i - start))
}

// ---------------------------------------------------------------------------
// API pública
// ---------------------------------------------------------------------------

/// Símbolos comunes de LaTeX que el parser original de Tinta no incluye.
/// Se sustituyen por su carácter Unicode antes de parsear.
const EXTRA_SYMBOLS: &[(&str, &str)] = &[
    ("blacksquare", "\u{25A0}"),
    ("checkmark", "\u{2713}"),
    ("qed", "\u{25A0}"),
    ("blacktriangle", "\u{25B2}"),
    ("Box", "\u{25A1}"),
    ("Diamond", "\u{25C7}"),
    ("lozenge", "\u{25CA}"),
    ("blacklozenge", "\u{25C6}"),
    ("bigstar", "\u{2605}"),
    ("dagger", "\u{2020}"),
    ("ddagger", "\u{2021}"),
    ("nRightarrow", "\u{21CF}"),
    ("S", "\u{00A7}"),
    ("P", "\u{00B6}"),
    ("copyright", "\u{00A9}"),
    ("pounds", "\u{00A3}"),
    ("euro", "\u{20AC}"),
    ("mdlgblksquare", "\u{25A0}"),
    ("mdlgwhtsquare", "\u{25A1}"),
];

fn replace_extra_symbols(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && chars[end].is_alphabetic() {
                end += 1;
            }
            let name: String = chars[start..end].iter().collect();
            match EXTRA_SYMBOLS.iter().find(|(n, _)| *n == name) {
                Some((_, symbol)) => out.push_str(symbol),
                None => {
                    out.push('\\');
                    out.push_str(&name);
                }
            }
            i = end;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// ¿Hay un salto de línea `\\` fuera de cualquier entorno `\begin...\end`?
fn has_top_level_line_break(source: &str) -> bool {
    let chars: Vec<char> = source.chars().collect();
    let mut env_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' {
            if i + 1 < chars.len() && chars[i + 1] == '\\' {
                if env_depth == 0 && brace_depth == 0 {
                    return true;
                }
                i += 2;
                continue;
            }
            if i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
                let start = i + 1;
                let mut end = start;
                while end < chars.len() && chars[end].is_alphabetic() {
                    end += 1;
                }
                let name: String = chars[start..end].iter().collect();
                match name.as_str() {
                    "begin" => env_depth += 1,
                    "end" => env_depth = env_depth.saturating_sub(1),
                    _ => {}
                }
                i = end;
                continue;
            }
            // Comando no alfabético: `\{`, `\}`, `\%`... no cuentan como llaves.
            i += 2;
            continue;
        }
        match chars[i] {
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }
        i += 1;
    }
    false
}

/// `$$ ... \\ ... $$` no es válido sin un entorno, pero es muy común en
/// apuntes. Lo envolvemos en `gathered` para que cada línea sea una fila.
fn wrap_top_level_line_breaks(source: String) -> String {
    if has_top_level_line_break(&source) {
        format!("\\begin{{gathered}}{source}\\end{{gathered}}")
    } else {
        source
    }
}

/// Port de `tinta_math::parseLatex`.
pub fn parse_latex(source: &str) -> Option<MNode> {
    if source.chars().count() > 32768 {
        return None;
    }
    let expanded = expand_latex_macros(source)?;
    let normalized = wrap_top_level_line_breaks(replace_extra_symbols(&expanded));
    let chars: Vec<char> = normalized.chars().collect();
    let mut parser = Parser::new(&chars);
    let root = parser.parse_row('\0', false, false);
    let root = root?;
    if parser.failed || root.kids.is_empty() {
        None
    } else {
        Some(*root)
    }
}

/// Port de `tinta_math::isBuiltinMathCommand`.
pub fn is_builtin_math_command(command: &str) -> bool {
    if lookup_symbol(command).is_some() || is_function_name(command) {
        return true;
    }
    const NAMES: &str = " begin end frac dfrac tfrac cfrac binom dbinom tbinom sqrt \
left right middle big Big bigg Bigg bigl bigr Bigl Bigr biggl biggr Biggl Biggr \
text textrm textbf textit operatorname mathrm mathit mathbb mathcal mathscr mathfrak mathsf mathtt \
mathbf boldsymbol bm displaystyle textstyle scriptstyle scriptscriptstyle limits nolimits \
overline bar overrightarrow vec hat widehat dot ddot tilde widetilde underline boxed \
cancel bcancel xcancel overbrace underbrace acute grave breve check not \
phantom hphantom vphantom smash hspace kern mkern quad qquad thinspace medspace thickspace enspace \
overset underset stackrel substack xrightarrow xleftarrow pmod bmod pod newcommand renewcommand \
providecommand def DeclareMathOperator ";
    NAMES.contains(&format!(" {command} "))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Volcado equivalente al harness de referencia de Tinta, para comparar
    /// con `/tmp/opencode/dump_ast`.
    fn kind_name(k: Kind) -> &'static str {
        match k {
            Kind::Row => "Row",
            Kind::Sym => "Sym",
            Kind::Frac => "Frac",
            Kind::Script => "Script",
            Kind::Delim => "Delim",
            Kind::Deco => "Deco",
            Kind::Space => "Space",
            Kind::Grid => "Grid",
            Kind::Stack => "Stack",
            Kind::Phantom => "Phantom",
            Kind::Middle => "Middle",
        }
    }

    fn esc(s: &str) -> String {
        let mut out = String::new();
        for c in s.chars() {
            if c == '"' || c == '\\' {
                out.push('\\');
                out.push(c);
            } else if c == '\n' {
                out.push_str("\\n");
            } else {
                out.push(c);
            }
        }
        out
    }

    fn dump_into(n: Option<&MNode>, o: &mut String) {
        let Some(n) = n else {
            o.push_str("null");
            return;
        };
        o.push('(');
        o.push_str(kind_name(n.kind));
        if !n.text.is_empty() {
            o.push_str(&format!(" text=\"{}\"", esc(&n.text)));
        }
        if n.open != '\0' || n.close != '\0' {
            o.push_str(&format!(" open={} close={}", n.open as u32, n.close as u32));
        }
        if n.deco_kind != 0 {
            o.push_str(&format!(" deco={}", n.deco_kind));
        }
        if n.roman {
            o.push_str(" roman");
        }
        if n.bold {
            o.push_str(" bold");
        }
        if n.literal_text {
            o.push_str(" literal");
        }
        if n.force_italic {
            o.push_str(" italic");
        }
        if n.spacing >= 0.0 {
            o.push_str(&format!(" spacing={}", n.spacing));
        }
        if n.large_op {
            o.push_str(" largeop");
        }
        if n.limit_op {
            o.push_str(" limitop");
        }
        if n.limits >= 0 {
            o.push_str(&format!(" limits={}", n.limits));
        }
        if n.style >= 0 {
            o.push_str(&format!(" style={}", n.style));
        }
        if n.no_bar {
            o.push_str(" nobar");
        }
        if n.delimiter_size != 0.0 {
            o.push_str(&format!(" delims={}", n.delimiter_size));
        }
        if n.space != 0.0 {
            o.push_str(&format!(" space={}", n.space));
        }
        if !n.alignment.is_empty() {
            o.push_str(&format!(" align=\"{}\"", esc(&n.alignment)));
        }
        if n.aligned {
            o.push_str(" aligned");
        }
        if n.compact {
            o.push_str(" compact");
        }
        if n.uniform_alignment {
            o.push_str(" uniform");
        }
        if !n.vertical_rules.is_empty() {
            o.push_str(" vrules=");
            for v in &n.vertical_rules {
                o.push_str(&format!("{v},"));
            }
        }
        if !n.horizontal_rules.is_empty() {
            o.push_str(" hrules=");
            for v in &n.horizontal_rules {
                o.push_str(&format!("{v},"));
            }
        }
        if !n.kids.is_empty() {
            o.push_str(" kids=[");
            for (i, k) in n.kids.iter().enumerate() {
                if i > 0 {
                    o.push(' ');
                }
                dump_into(k.as_deref(), o);
            }
            o.push(']');
        }
        if !n.cells.is_empty() {
            o.push_str(" cells=[");
            for (r, row) in n.cells.iter().enumerate() {
                if r > 0 {
                    o.push(' ');
                }
                o.push('[');
                for (c, cell) in row.iter().enumerate() {
                    if c > 0 {
                        o.push(' ');
                    }
                    dump_into(Some(cell), o);
                }
                o.push(']');
            }
            o.push(']');
        }
        o.push(')');
    }

    fn dump(n: Option<&MNode>) -> String {
        let mut out = String::new();
        dump_into(n, &mut out);
        out
    }


    /// (entrada, S-expression esperada del binario de referencia
    /// `/tmp/opencode/dump_ast`).
    const GOLDEN: &[(&str, &str)] = &[
        (r"\frac{a}{b}", r#"(Row kids=[(Frac kids=[(Row kids=[(Sym text="a")]) (Row kids=[(Sym text="b")])])])"#),
        (r"\sqrt{x}", r#"(Row kids=[(Deco deco=3 kids=[(Row kids=[(Sym text="x")]) null])])"#),
        (r"\sqrt[3]{x}", r#"(Row kids=[(Deco deco=3 kids=[(Row kids=[(Sym text="x")]) (Row kids=[(Sym text="3")])])])"#),
        (r"x^2", r#"(Row kids=[(Script kids=[(Sym text="x") null (Sym text="2")])])"#),
        (r"x_i^2", r#"(Row kids=[(Script kids=[(Sym text="x") (Sym text="i") (Sym text="2")])])"#),
        (r"\left(\frac{a}{b}\right)", r#"(Row kids=[(Delim open=40 close=41 kids=[(Row kids=[(Frac kids=[(Row kids=[(Sym text="a")]) (Row kids=[(Sym text="b")])])])])])"#),
        (r"\begin{pmatrix}a&b\\c&d\end{pmatrix}", r#"(Row kids=[(Delim open=40 close=41 kids=[(Grid cells=[[(Row kids=[(Sym text="a")]) (Row kids=[(Sym text="b")])] [(Row kids=[(Sym text="c")]) (Row kids=[(Sym text="d")])]])])])"#),
        (r"\begin{bmatrix}a\end{bmatrix}", r#"(Row kids=[(Delim open=91 close=93 kids=[(Grid cells=[[(Row kids=[(Sym text="a")])]])])])"#),
        (r"\begin{matrix}a&b\\c&d\end{matrix}", r#"(Row kids=[(Grid cells=[[(Row kids=[(Sym text="a")]) (Row kids=[(Sym text="b")])] [(Row kids=[(Sym text="c")]) (Row kids=[(Sym text="d")])]])])"#),
        (r"\begin{cases}x&1\\y&2\end{cases}", r#"(Row kids=[(Delim open=123 close=0 kids=[(Grid align="ll" cells=[[(Row kids=[(Sym text="x")]) (Row kids=[(Sym text="1")])] [(Row kids=[(Sym text="y")]) (Row kids=[(Sym text="2")])]])])])"#),
        (r"\begin{aligned}a&=b\\c&=d\end{aligned}", r#"(Row kids=[(Grid aligned cells=[[(Row kids=[(Sym text="a")]) (Row kids=[(Sym text="=") (Sym text="b")])] [(Row kids=[(Sym text="c")]) (Row kids=[(Sym text="=") (Sym text="d")])]])])"#),
        (r"\text{hello world}", r#"(Row kids=[(Row literal kids=[(Sym text="hello world" roman literal)])])"#),
        (r"\mathrm{abc}", r#"(Row kids=[(Row roman kids=[(Sym text="a" roman) (Sym text="b" roman) (Sym text="c" roman)])])"#),
        (r"\mathbf{Ab1}", r#"(Row kids=[(Row roman bold kids=[(Sym text="A" roman bold) (Sym text="b" roman bold) (Sym text="1" roman bold)])])"#),
        (r"\overline{x}", r#"(Row kids=[(Deco deco=1 kids=[(Row kids=[(Sym text="x")])])])"#),
        (r"\vec{v}", r#"(Row kids=[(Deco deco=2 kids=[(Row kids=[(Sym text="v")])])])"#),
        (r"\hat{x}", r#"(Row kids=[(Deco deco=4 kids=[(Row kids=[(Sym text="x")])])])"#),
        (r"\binom{n}{k}", r#"(Row kids=[(Delim open=40 close=41 kids=[(Frac nobar kids=[(Row kids=[(Sym text="n")]) (Row kids=[(Sym text="k")])])])])"#),
        (r"\operatorname{foo}", r#"(Row kids=[(Row literal spacing=0.16 kids=[(Sym text="foo" roman literal)])])"#),
        (r"\lim_{x\to0}", r#"(Row kids=[(Script kids=[(Sym text="lim" roman spacing=0.16 limitop) (Row kids=[(Sym text="x") (Sym text="→") (Sym text="0")]) null])])"#),
        (r"\sum_{i=1}^n", r#"(Row kids=[(Script kids=[(Sym text="∑" spacing=0.16 largeop limitop) (Row kids=[(Sym text="i") (Sym text="=") (Sym text="1")]) (Sym text="n")])])"#),
        (r"\int_0^1", r#"(Row kids=[(Script kids=[(Sym text="∫" spacing=0.16 largeop) (Sym text="0") (Sym text="1")])])"#),
        (r"\hspace{1em}", r#"(Row kids=[(Space space=1)])"#),
        (r"\phantom{x}", r#"(Row kids=[(Phantom kids=[(Row kids=[(Sym text="x")])])])"#),
        (r"\overset{a}{b}", r#"(Row kids=[(Stack kids=[(Row kids=[(Sym text="b")]) null (Row kids=[(Sym text="a")])])])"#),
        (r"\xrightarrow[below]{above}", r#"(Row kids=[(Stack deco=1 spacing=0.26 kids=[null (Row kids=[(Sym text="b") (Sym text="e") (Sym text="l") (Sym text="o") (Sym text="w")]) (Row kids=[(Sym text="a") (Sym text="b") (Sym text="o") (Sym text="v") (Sym text="e")])])])"#),
        (r"\newcommand{\R}{\mathbb{R}} \R", r#"(Row kids=[(Row roman kids=[(Sym text="ℝ" roman)])])"#),
        (r"\def\a#1{#1^2}\a{x}", r#"(Row kids=[(Script kids=[(Sym text="x") null (Sym text="2")])])"#),
        (r"\displaystyle x^2", r#"(Row kids=[(Script style=0 kids=[(Sym text="x" style=0) null (Sym text="2")])])"#),
        (r"\left. x \right|", r#"(Row kids=[(Delim open=0 close=124 kids=[(Row kids=[(Sym text="x")])])])"#),
        (r"\bigl( x \bigr)", r#"(Row kids=[(Middle open=40 close=0 delims=1.2) (Sym text="x") (Middle open=41 close=0 delims=1.2)])"#),
        (r"\pmod{n}", r#"(Row kids=[(Delim open=40 close=41 spacing=0.3 kids=[(Row spacing=0.3 kids=[(Sym text="mod" roman) (Space space=0.33) (Row kids=[(Sym text="n")])])])])"#),
        (r"\substack{a\\b}", r#"(Row kids=[(Grid cells=[[(Row kids=[(Sym text="a")])] [(Row kids=[(Sym text="b")])]])])"#),
        (r"\begin{array}{c|c}a&b\\\hline c&d\end{array}", r#"(Row kids=[(Grid align="cc" vrules=1, hrules=1, cells=[[(Row kids=[(Sym text="a")]) (Row kids=[(Sym text="b")])] [(Row kids=[(Sym text="c")]) (Row kids=[(Sym text="d")])]])])"#),
        (r"\begin{pmatrix*}[r]a&b\end{pmatrix*}", r#"(Row kids=[(Delim open=40 close=41 kids=[(Grid align="r" uniform cells=[[(Row kids=[(Sym text="a")]) (Row kids=[(Sym text="b")])]])])])"#),
        (r"\underbrace{x}_{y}", r#"(Row kids=[(Script kids=[(Deco deco=13 limitop kids=[(Row kids=[(Sym text="x")])]) (Row kids=[(Sym text="y")]) null])])"#),
        (r"\frac12", r#"(Row kids=[(Frac kids=[(Sym text="1") (Sym text="2")])])"#),
        (r"a-b*c'd", r#"(Row kids=[(Sym text="a") (Sym text="−") (Sym text="b") (Sym text="∗") (Sym text="c") (Sym text="′") (Sym text="d")])"#),
        (r"\sin x", r#"(Row kids=[(Sym text="sin" roman spacing=0.16) (Sym text="x")])"#),
        (r"\mathbf{\alpha}", r#"(Row kids=[(Row roman bold kids=[(Sym text="α" roman bold)])])"#),
        (r"\mathbb{RNCZ}", r#"(Row kids=[(Row roman kids=[(Sym text="ℝ" roman) (Sym text="ℕ" roman) (Sym text="ℂ" roman) (Sym text="ℤ" roman)])])"#),
    ];

    #[test]
    fn ast_matches_reference_dumps() {
        for (src, expected) in GOLDEN {
            let ast = parse_latex(src);
            assert_eq!(&dump(ast.as_ref()), expected, "entrada: {src}");
        }
    }

    fn root(src: &str) -> MNode {
        parse_latex(src).unwrap_or_else(|| panic!("falló el parseo de {src}"))
    }

    fn first_kid(n: &MNode) -> &MNode {
        n.kid(0).expect("nodo sin hijos")
    }

    #[test]
    fn frac() {
        // \frac, \dfrac, \tfrac
        let ast = root(r"\frac{a}{b}");
        let frac = first_kid(&ast);
        assert_eq!(frac.kind, Kind::Frac);
        assert_eq!(frac.style, -1);
        assert_eq!(frac.kid(0).unwrap().kid(0).unwrap().text, "a");
        assert_eq!(frac.kid(1).unwrap().kid(0).unwrap().text, "b");
        assert_eq!(first_kid(&root(r"\dfrac{a}{b}")).style, 0);
        assert_eq!(first_kid(&root(r"\tfrac{a}{b}")).style, 1);
    }

    #[test]
    fn sqrt_plain_and_indexed() {
        let ast = root(r"\sqrt{x}");
        let deco = first_kid(&ast);
        assert_eq!(deco.kind, Kind::Deco);
        assert_eq!(deco.deco_kind, 3);
        assert!(deco.kid(1).is_none());

        let ast = root(r"\sqrt[3]{x}");
        let deco = first_kid(&ast);
        assert_eq!(deco.kid(1).unwrap().kid(0).unwrap().text, "3");
    }

    #[test]
    fn scripts() {
        let ast = root(r"x^2");
        let script = first_kid(&ast);
        assert_eq!(script.kind, Kind::Script);
        assert_eq!(script.kid(0).unwrap().text, "x");
        assert!(script.kid(1).is_none());
        assert_eq!(script.kid(2).unwrap().text, "2");

        // x_i^2: ambos scripts en el mismo nodo Script
        let ast = root(r"x_i^2");
        let script = first_kid(&ast);
        assert_eq!(script.kind, Kind::Script);
        assert_eq!(script.kid(1).unwrap().text, "i");
        assert_eq!(script.kid(2).unwrap().text, "2");

        // x^2^3 falla porque el slot ya está ocupado
        assert!(parse_latex(r"x^2^3").is_none());
    }

    #[test]
    fn left_right() {
        let ast = root(r"\left(\frac{a}{b}\right)");
        let delim = first_kid(&ast);
        assert_eq!(delim.kind, Kind::Delim);
        assert_eq!(delim.open, '(');
        assert_eq!(delim.close, ')');
        assert_eq!(first_kid(delim).kind, Kind::Row);
        assert_eq!(first_kid(delim).kid(0).unwrap().kind, Kind::Frac);

        // \left. oculta el delimitador de apertura
        let ast = root(r"\left. x \right|");
        let delim = first_kid(&ast);
        assert_eq!(delim.open, '\0');
        assert_eq!(delim.close, '|');
    }

    #[test]
    fn matrices() {
        let p = root(r"\begin{pmatrix}a&b\\c&d\end{pmatrix}");
        let delim = first_kid(&p);
        assert_eq!(delim.kind, Kind::Delim);
        assert_eq!((delim.open, delim.close), ('(', ')'));
        let grid = first_kid(delim);
        assert_eq!(grid.kind, Kind::Grid);
        assert_eq!(grid.cells.len(), 2);
        assert_eq!(grid.cells[1][1].kid(0).unwrap().text, "d");

        let b = root(r"\begin{bmatrix}a\end{bmatrix}");
        let delim = first_kid(&b);
        assert_eq!((delim.open, delim.close), ('[', ']'));

        // matrix no lleva delimitadores
        let m = root(r"\begin{matrix}a&b\\c&d\end{matrix}");
        let grid = first_kid(&m);
        assert_eq!(grid.kind, Kind::Grid);
        assert_eq!(grid.cells.len(), 2);

        // pmatrix* con alineación uniforme
        let s = root(r"\begin{pmatrix*}[r]a&b\end{pmatrix*}");
        let grid = first_kid(first_kid(&s));
        assert!(grid.uniform_alignment);
        assert_eq!(grid.alignment, "r");
    }

    #[test]
    fn cases_and_aligned() {
        let ast = root(r"\begin{cases}x&1\\y&2\end{cases}");
        let delim = first_kid(&ast);
        assert_eq!((delim.open, delim.close), ('{', '\0'));
        let grid = first_kid(delim);
        assert_eq!(grid.kind, Kind::Grid);
        assert_eq!(grid.alignment, "ll");
        assert_eq!(grid.cells.len(), 2);

        let ast = root(r"\begin{aligned}a&=b\\c&=d\end{aligned}");
        let grid = first_kid(&ast);
        assert_eq!(grid.kind, Kind::Grid);
        assert!(grid.aligned);
        assert_eq!(grid.cells[0].len(), 2);
    }

    #[test]
    fn text_and_fonts() {
        let ast = root(r"\text{hello world}");
        let row = first_kid(&ast);
        assert_eq!(row.kind, Kind::Row);
        assert!(row.literal_text);
        let run = row.kid(0).unwrap();
        assert_eq!(run.kind, Kind::Sym);
        assert_eq!(run.text, "hello world");
        assert!(run.roman && run.literal_text);

        let ast = root(r"\mathrm{abc}");
        let row = first_kid(&ast);
        assert!(row.roman);
        for k in row.kids.iter().flatten() {
            assert!(k.roman);
        }

        let ast = root(r"\mathbf{Ab1}");
        let row = first_kid(&ast);
        assert!(row.roman && row.bold);
        assert_eq!(row.kid(0).unwrap().text, "A");
        assert_eq!(row.kid(2).unwrap().text, "1");

        // \textit fuerza cursiva
        let ast = root(r"\textit{ab}");
        let row = first_kid(&ast);
        assert!(row.force_italic);
    }

    #[test]
    fn decorations() {
        let expected = [
            (r"\overline{x}", 1, Kind::Deco),
            (r"\vec{v}", 2, Kind::Deco),
            (r"\hat{x}", 4, Kind::Deco),
            (r"\tilde{x}", 7, Kind::Deco),
            (r"\underline{x}", 8, Kind::Deco),
            (r"\boxed{x}", 9, Kind::Deco),
            (r"\overbrace{x}", 12, Kind::Deco),
        ];
        for (src, deco, kind) in expected {
            let ast = root(src);
            let node = first_kid(&ast);
            assert_eq!(node.kind, kind, "{src}");
            assert_eq!(node.deco_kind, deco, "{src}");
        }
        // \underbrace marca limitOp
        let ast = root(r"\underbrace{x}");
        assert!(first_kid(&ast).limit_op);
    }

    #[test]
    fn binomial() {
        let ast = root(r"\binom{n}{k}");
        let delim = first_kid(&ast);
        assert_eq!(delim.kind, Kind::Delim);
        let frac = first_kid(delim);
        assert_eq!(frac.kind, Kind::Frac);
        assert!(frac.no_bar);
    }

    #[test]
    fn operatorname() {
        let ast = root(r"\operatorname{foo}");
        let op = first_kid(&ast);
        assert_eq!(op.kind, Kind::Row);
        assert_eq!(op.spacing, 0.16);
        assert!(!op.limit_op);

        let ast = root(r"\operatorname*{argmax}");
        let op = first_kid(&ast);
        assert!(op.limit_op);
    }

    #[test]
    fn functions_and_large_operators() {
        let ast = root(r"\lim_{x\to0}");
        let script = first_kid(&ast);
        let base = script.kid(0).unwrap();
        assert_eq!(base.kind, Kind::Sym);
        assert_eq!(base.text, "lim");
        assert!(base.roman && base.limit_op);
        assert_eq!(base.spacing, 0.16);

        let ast = root(r"\sum_{i=1}^n");
        let base = first_kid(&ast).kid(0).unwrap();
        assert_eq!(base.text, "∑");
        assert!(base.large_op && base.limit_op);

        // \int es operador grande pero los límites van al lado
        let ast = root(r"\int_0^1");
        let base = first_kid(&ast).kid(0).unwrap();
        assert_eq!(base.text, "∫");
        assert!(base.large_op);
        assert!(!base.limit_op);

        let ast = root(r"\sin x");
        let base = first_kid(&ast);
        assert_eq!(base.text, "sin");
        assert!(base.roman);
        assert!(!base.limit_op);
    }

    #[test]
    fn spacing_and_phantoms() {
        let ast = root(r"\hspace{1em}");
        let space = first_kid(&ast);
        assert_eq!(space.kind, Kind::Space);
        assert_eq!(space.space, 1.0);

        let ast = root(r"\hspace{2ex}");
        assert_eq!(first_kid(&ast).space, 1.0);

        let ast = root(r"\hspace{18mu}");
        assert_eq!(first_kid(&ast).space, 1.0);

        let ast = root(r"\quad");
        assert_eq!(first_kid(&ast).space, 1.0);
        let ast = root(r"\qquad");
        assert_eq!(first_kid(&ast).space, 2.0);

        let ast = root(r"\phantom{x}");
        assert_eq!(first_kid(&ast).kind, Kind::Phantom);
        assert_eq!(first_kid(&ast).deco_kind, 0);
        let ast = root(r"\hphantom{x}");
        assert_eq!(first_kid(&ast).deco_kind, 1);
        let ast = root(r"\vphantom{x}");
        assert_eq!(first_kid(&ast).deco_kind, 2);
        let ast = root(r"\smash{x}");
        assert_eq!(first_kid(&ast).deco_kind, 3);
    }

    #[test]
    fn stacks_and_arrows() {
        let ast = root(r"\overset{a}{b}");
        let stack = first_kid(&ast);
        assert_eq!(stack.kind, Kind::Stack);
        assert_eq!(stack.kid(0).unwrap().kid(0).unwrap().text, "b");
        assert!(stack.kid(1).is_none());
        assert_eq!(stack.kid(2).unwrap().kid(0).unwrap().text, "a");

        let ast = root(r"\underset{a}{b}");
        let stack = first_kid(&ast);
        assert_eq!(stack.kid(1).unwrap().kid(0).unwrap().text, "a");
        assert!(stack.kid(2).is_none());

        let ast = root(r"\xrightarrow[below]{above}");
        let stack = first_kid(&ast);
        assert_eq!(stack.kind, Kind::Stack);
        assert_eq!(stack.deco_kind, 1);
        assert!(stack.kid(0).is_none());
        assert_eq!(stack.kid(1).unwrap().kid(0).unwrap().text, "b");
        assert_eq!(stack.kid(2).unwrap().kid(0).unwrap().text, "a");

        let ast = root(r"\xleftarrow{above}");
        assert_eq!(first_kid(&ast).deco_kind, 2);
    }

    #[test]
    fn macros_newcommand_and_def() {
        let ast = root(r"\newcommand{\R}{\mathbb{R}} \R");
        let row = first_kid(&ast);
        assert!(row.roman);
        assert_eq!(row.kid(0).unwrap().text, "ℝ");

        let ast = root(r"\def\a#1{#1^2}\a{x}");
        let script = first_kid(&ast);
        assert_eq!(script.kind, Kind::Script);
        assert_eq!(script.kid(0).unwrap().text, "x");
        assert_eq!(script.kid(2).unwrap().text, "2");
    }

    #[test]
    fn invalid_input() {
        assert!(parse_latex("").is_none());
        assert!(parse_latex(r"\unknowncommand").is_none());
        assert!(parse_latex(r"\frac{a}").is_none());
        assert!(parse_latex(r"\left( x").is_none());
        assert!(parse_latex(r"\begin{pmatrix}a").is_none());
        assert!(parse_latex(r"x^").is_none());
        assert!(parse_latex(r"{a").is_none());
        assert!(parse_latex(r"a}").is_none());
        // comandos estructurales sin argumento válido
        assert!(parse_latex(r"\sqrt").is_none());
        assert!(parse_latex(r"\hspace{x}").is_none());
    }

    #[test]
    fn length_limit() {
        // 32769 caracteres superan el límite del original (32768).
        let too_long = "x".repeat(32769);
        assert!(parse_latex(&too_long).is_none());
        // un símbolo Unicode cuenta como un carácter, no como varios bytes
        let unicode_too_long = "α".repeat(32769);
        assert!(parse_latex(&unicode_too_long).is_none());
        // Por debajo del límite se parsea (el tope de átomos es 4096).
        let ok = "x".repeat(4000);
        assert!(parse_latex(&ok).is_some());
        assert!(parse_latex(&"α".repeat(4000)).is_some());
    }

    #[test]
    fn comments_are_ignored() {
        let ast = root("x % comentario\n+ y");
        assert_eq!(ast.kids.len(), 3);
        assert_eq!(ast.kid(0).unwrap().text, "x");
        assert_eq!(ast.kid(1).unwrap().text, "+");
        assert_eq!(ast.kid(2).unwrap().text, "y");
    }

    /// Cada entrada de la tabla CmdSym produce un `Kind::Sym` con su texto.
    #[test]
    fn symbol_table_maps_to_sym_text() {
        let expected = [
            ("alpha", "α"),
            ("beta", "β"),
            ("Gamma", "Γ"),
            ("Omega", "Ω"),
            ("in", "∈"),
            ("subseteq", "⊆"),
            ("geq", "≥"),
            ("neq", "≠"),
            ("pm", "±"),
            ("times", "×"),
            ("cdot", "⋅"),
            ("cdots", "⋯"),
            ("infty", "∞"),
            ("partial", "∂"),
            ("nabla", "∇"),
            ("to", "→"),
            ("Rightarrow", "⇒"),
            ("longleftrightarrow", "⟷"),
            ("mapsto", "↦"),
            ("oplus", "⊕"),
            ("sum", "∑"),
            ("prod", "∏"),
            ("int", "∫"),
            ("oint", "∮"),
            ("lfloor", "⌊"),
            ("rceil", "⌉"),
            ("langle", "⟨"),
            ("Vert", "‖"),
            ("backslash", "\\"),
            ("hbar", "ℏ"),
            ("aleph", "ℵ"),
            ("land", "∧"),
            ("bigcup", "⋃"),
            ("uparrow", "↑"),
            ("hookrightarrow", "↪"),
            ("rightleftharpoons", "⇌"),
            ("ll", "≪"),
            ("preceq", "⪯"),
            ("vdash", "⊢"),
            ("models", "⊨"),
            ("sqcup", "⊔"),
            ("wr", "≀"),
            ("lparen", "("),
            ("colon", ":"),
        ];
        for (cmd, text) in expected {
            let ast = root(&format!(r"\{cmd}"));
            let node = first_kid(&ast);
            assert_eq!(node.kind, Kind::Sym, "\\{cmd}");
            assert_eq!(node.text, text, "\\{cmd}");
        }
        // Toda la tabla, sin excepciones, se resuelve como Sym
        for entry in K_SYMBOLS {
            let Some(text) = entry.text else { continue };
            let ast = root(&format!(r"\{}", entry.name));
            let node = first_kid(&ast);
            assert_eq!(node.kind, Kind::Sym, "\\{}", entry.name);
            assert_eq!(node.text, text, "\\{}", entry.name);
        }
    }

    #[test]
    fn math_alphabets() {
        let ast = root(r"\mathbb{RNCZ}");
        let texts: Vec<&str> = first_kid(&ast)
            .kids
            .iter()
            .flatten()
            .map(|k| k.text.as_str())
            .collect();
        assert_eq!(texts, ["ℝ", "ℕ", "ℂ", "ℤ"]);

        let ast = root(r"\mathcal{B}");
        assert_eq!(first_kid(&ast).kid(0).unwrap().text, "ℬ");

        let ast = root(r"\mathfrak{R}");
        assert_eq!(first_kid(&ast).kid(0).unwrap().text, "ℜ");

        let ast = root(r"\mathsf{A}");
        assert_eq!(first_kid(&ast).kid(0).unwrap().text, "𝖠");

        // \mathtt va por la rama por defecto (monospace)
        let ast = root(r"\mathtt{A}");
        assert_eq!(first_kid(&ast).kid(0).unwrap().text, "𝙰");
    }

    #[test]
    fn builtin_commands() {
        for cmd in [
            "frac", "sqrt", "alpha", "sum", "lim", "left", "right", "begin", "end",
            "newcommand", "DeclareMathOperator", "hspace", "overrightarrow", "operatorname",
        ] {
            assert!(is_builtin_math_command(cmd), "{cmd}");
        }
        for cmd in ["", "foo", "unknowncmd", "sinx", "alphabet", "fracx"] {
            assert!(!is_builtin_math_command(cmd), "{cmd}");
        }
        // "int" es símbolo y no función; "sin" es función
        assert!(is_builtin_math_command("int"));
        assert!(is_builtin_math_command("sin"));
    }

    #[test]
    fn symbol_spacing_table() {
        assert_eq!(symbol_spacing("=", false), 0.26);
        assert_eq!(symbol_spacing("+", false), 0.2);
        assert_eq!(symbol_spacing(",", false), 0.12);
        assert_eq!(symbol_spacing("x", false), 0.0);
        // El original solo aplica el espaciado de puntos suspensivos a
        // cadenas de más de un carácter (un único wchar cae en el switch).
        assert_eq!(symbol_spacing("⋯", false), 0.0);
        assert_eq!(symbol_spacing("sin", true), 0.16);
        assert_eq!(symbol_spacing("sin", false), 0.0);
    }

    #[test]
    fn extra_symbols_are_rendered() {
        for (latex, expected) in [
            (r"\blacksquare", "■"),
            (r"\square", "□"),
            (r"\checkmark", "✓"),
            (r"\therefore", "∴"),
            (r"\qed", "■"),
            (r"\Box", "□"),
            (r"\dagger", "†"),
            (r"\nRightarrow", "⇏"),
        ] {
            let ast = root(latex);
            assert_eq!(ast.kid(0).unwrap().text, expected, "{latex}");
        }
    }

    #[test]
    fn top_level_line_breaks_are_wrapped() {
        // `\\` fuera de un entorno se envuelve en `gathered`.
        assert!(parse_latex(r"a = b \\ c = d").is_some());
        assert!(parse_latex("a = b \\\\[6pt] c = d").is_some());
        // Dentro de un entorno no se envuelve dos veces.
        assert!(parse_latex(
            r"\begin{aligned} a &= b \\ c &= d \end{aligned}"
        )
        .is_some());
    }

    #[test]
    fn reported_formulas_parse() {
        let formulas = [
            r"v - (e - 1) + (r - 1) = 2 \implies v - e + 1 + r - 1 = 2 \implies v - e + r = 2 \quad \blacksquare",
            r"v - e + r = v - (v - 1) + 1 = v - v + 1 + 1 = 2 \quad \checkmark",
            r"\mathbb{P}(E_1 \cup \dots \cup E_n) = \sum_{i=1}^{n} \mathbb{P}(E_i) - \sum_{i_1 < i_2} \mathbb{P}(E_{i_1} \cap E_{i_2}) \\
\\ + \dots + (-1)^{1+r} \sum_{i_1 < i_2 < \dots < i_r} \mathbb{P}(E_{i_1} \cap E_{i_2} \cap \dots \cap E_{i_r}) \\
\\ + \dots + (-1)^{n+1} \mathbb{P}(E_1 \cap \dots \cap E_n)",
            r"\begin{aligned}
\sum_{i} g(a_i) \, p(a_i) &= \sum_{j} \sum_{\{i \mid g(a_i) = y_j\}} g(a_i) \, p(a_i) \\[6pt]
&= \sum_{j} y_j \left( \sum_{\{i \mid g(a_i) = y_j\}} p(a_i) \right) \\[6pt]
&= \sum_{j} y_j \cdot \mathbb{P}\big(g(X) = y_j\big) \\[6pt]
&= \sum_{j} y_j \cdot \mathbb{P}(Y = y_j) \\[6pt]
&= \mathbb{E}[Y] = \mathbb{E}[g(X)] \quad \blacksquare
\end{aligned}",
        ];
        for formula in formulas {
            assert!(parse_latex(formula).is_some(), "no parsea: {formula}");
        }
    }
}
