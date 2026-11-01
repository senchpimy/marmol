//! Utilidades de formato de texto.
//!
//! La conversión LaTeX → Typst que vivía aquí se eliminó junto con Typst; el
//! render de matemáticas usa ahora el motor nativo de `crate::latex_math`.

pub fn indent(content: &str) -> String {
    let last = content.lines().last().unwrap_or("");
    let mut ind = String::new();
    for char in last.chars() {
        if ![' ', '[', ']', '-', '+', 'x'].contains(&char) {
            break;
        } else {
            ind.push(char);
        }
    }
    format!("{}{}", content, ind)
}
