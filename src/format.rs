use std::sync::LazyLock;

fn buildrel_regex() -> &'static regex::Regex {
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?s)\\buildrel\s*(.*?)\s*\\over\s*(\{.*\}|\S+)").unwrap()
    });
    &RE
}

fn pmod_braced_regex() -> &'static regex::Regex {
    static RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"\\pmod\s*\{([^{}]*)\}").unwrap());
    &RE
}

fn pmod_simple_regex() -> &'static regex::Regex {
    static RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"\\pmod\s+([^\s\\{]+)").unwrap());
    &RE
}

fn join_casing_regex() -> &'static regex::Regex {
    // tex2typst es sensible a mayúsculas: solo reconoce los comandos de join en
    // minúsculas (`\leftouterjoin`, ...). Normalizamos cualquier mayúscula.
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"(?i)\\(leftouterjoin|rightouterjoin|fullouterjoin|bowtie)").unwrap()
    });
    &RE
}

pub fn indent(content:&str)->String {
    let last=content.lines().last().unwrap_or("");
    let mut ind=String::new();
    for char in last.chars(){
        if ![' ','[',']','-','+','x'].contains(&char){
            break;
        }else{
            ind.push(char);
        }
    }
    format!("{}{}",content,ind)
}

pub fn normalizar_latex(latex: &str) -> String {
    let s = latex.replace("\\stackrel", "\\overset");
    let s = buildrel_regex()
        .replace_all(&s, "\\overset{$1}{$2}")
        .into_owned();

    // Símbolos no cubiertos por tex2typst con su nombre corto. Se mapean a
    // nombres que tex2typst sí reconoce:
    let s = s.replace("\\blacksquare", "\\mdlgblksquare"); // -> square.filled
    let s = s.replace("\\square", "\\mdlgwhtsquare"); // -> square.stroked

    // \pmod{X} -> (mod X). \operatorname{mod} -> op("mod") en Typst.
    let s = pmod_braced_regex()
        .replace_all(&s, r"(\operatorname{mod}\ $1)")
        .into_owned();
    let s = pmod_simple_regex()
        .replace_all(&s, r"(\operatorname{mod}\ $1)")
        .into_owned();

    // Operadores de join de álgebra relacional. tex2typst solo los reconoce en
    // minúsculas, así que normalizamos cualquier variación de mayúsculas.
    let s = join_casing_regex()
        .replace_all(&s, |caps: &regex::Captures| {
            format!("\\{}", caps[1].to_lowercase())
        })
        .into_owned();
    let s = s.replace("\\Join", "\\bowtie");
    let s = s.replace("\\join", "\\bowtie");

    // Comandos de estilo: se eliminan aquí porque se manejan en latex_a_typst
    // (typst no los reconoce y tex2typst los pasaría como texto).
    let s = s.replace("\\displaystyle", "");
    let s = s.replace("\\textstyle", "");
    let s = s.replace("\\scriptscriptstyle", "");
    let s = s.replace("\\scriptstyle", "");

    s
}

/// Palabras de tamaño que tex2typst deja como texto literal tras un `\big`,
/// `\Big`, `\bigg`, `\Bigg` (y variantes `l`/`r`). Las más largas primero.
const BIG_WORDS: &[&str] = &[
    "Biggl", "Biggr", "biggl", "biggr", "Bigl", "Bigr", "bigl", "bigr", "Bigg",
    "bigg", "Big", "big",
];

/// Convierte la familia `\big`, `\Big`, `\bigg`, `\Bigg` sobre delimitadores
/// emparejados en `lr(...)` de Typst (delimitadores dimensionados al contenido).
/// Los `\big` que preceden a un operador/símbolo (p. ej. `\Big\bowtie`) se
/// eliminan dejando el símbolo, evitando el texto literal "Big" que producía
/// tex2typst.
fn convertir_big_familia_typst(typst: &str) -> String {
    let b = typst.as_bytes();

    fn is_open(c: u8) -> bool {
        matches!(c, b'(' | b'[' | b'{' | b'|')
    }
    fn is_close(c: u8) -> bool {
        matches!(c, b')' | b']' | b'}' | b'|')
    }
    fn close_for(c: u8) -> u8 {
        match c {
            b'(' => b')',
            b'[' => b']',
            b'{' => b'}',
            _ => b'|',
        }
    }

    enum Tok {
        Text(String),
        Del(u8),
    }

    let n = b.len();
    let mut toks: Vec<Tok> = Vec::new();
    let mut i = 0;
    let mut cur = String::new();
    let mut flush = |cur: &mut String, toks: &mut Vec<Tok>| {
        if !cur.is_empty() {
            toks.push(Tok::Text(std::mem::take(cur)));
        }
    };

    while i < n {
        let mut word: Option<&str> = None;
        for w in BIG_WORDS {
            if i + w.len() <= n && &b[i..i + w.len()] == w.as_bytes() {
                word = Some(w);
                break;
            }
        }
        if let Some(w) = word {
            flush(&mut cur, &mut toks);
            let mut k = i + w.len();
            while k < n && b[k] == b' ' {
                k += 1;
            }
            if k < n && (is_open(b[k]) || is_close(b[k])) {
                toks.push(Tok::Del(b[k]));
                i = k + 1;
            } else {
                // Palabra de tamaño suelta (antes de un operador/símbolo): se descarta.
                i = k;
            }
        } else {
            let ch = typst[i..].chars().next().unwrap();
            cur.push(ch);
            i += ch.len_utf8();
        }
    }
    flush(&mut cur, &mut toks);

    // Emparejar delimitadores de tamaño.
    let len = toks.len();
    let mut open_to_close: Vec<Option<usize>> = vec![None; len];
    let mut stack: Vec<(usize, u8, u8)> = Vec::new(); // (idx_token, delim_abre, delim_cierra)
    for (idx, tok) in toks.iter().enumerate() {
        let Tok::Del(c) = tok else { continue };
        let c = *c;
        if c == b'|' {
            // `|` abre y cierra a la vez: cierra si lo más reciente abierto es un `|`.
            if let Some(top) = stack.last() {
                if top.2 == b'|' {
                    let top = stack.pop().unwrap();
                    open_to_close[top.0] = Some(idx);
                    continue;
                }
            }
            stack.push((idx, c, b'|'));
            continue;
        }
        if is_open(c) {
            stack.push((idx, c, close_for(c)));
        } else if is_close(c) {
            if let Some(top) = stack.last() {
                if top.2 == c {
                    let top = stack.pop().unwrap();
                    open_to_close[top.0] = Some(idx);
                }
            }
        }
    }

    // Construir la salida con `lr(...)` en los pares.
    let mut out = String::new();
    for idx in 0..len {
        match &toks[idx] {
            Tok::Text(t) => out.push_str(t),
            Tok::Del(c) => {
                let c = *c;
                let is_open_del = open_to_close[idx].is_some();
                let is_close_del = open_to_close.iter().any(|&oc| oc == Some(idx));
                if is_open_del {
                    out.push_str("lr(");
                    out.push(c as char);
                } else if is_close_del {
                    out.push(c as char);
                    out.push(')');
                } else {
                    out.push(c as char);
                }
            }
        }
    }
    out
}

pub fn latex_a_typst(latex: &str) -> String {
    // \displaystyle fuerza estilo de visualización: en Typst se traduce a la
    // función math `display(...)` envolviendo toda la expresión.
    let has_displaystyle = latex.contains("\\displaystyle");

    let norm = normalizar_latex(latex);
    let out = tex2typst_rs::tex2typst(&norm).unwrap_or(norm);
    let out = out.replace("limits: true", "limits: #true");
    let out = convertir_big_familia_typst(&out);

    if has_displaystyle {
        format!("display({})", out)
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_family_convierte_sin_panico_con_unicode() {
        // 'ó' es multibyte: antes esto hacía panic por indexar a mitad de carácter.
        let s = latex_a_typst("\\Big( información \\Big)");
        assert!(s.contains("lr(("));
    }

    #[test]
    fn joins_normalizan_mayusculas() {
        assert_eq!(latex_a_typst("\\Leftouterjoin"), "join.l");
        assert_eq!(latex_a_typst("\\Rightouterjoin"), "join.r");
        assert_eq!(latex_a_typst("\\FullouterJoin"), "join.l.r");
        assert_eq!(latex_a_typst("\\bowtie"), "join");
    }

    #[test]
    fn big_familia_operador_se_descarta() {
        assert_eq!(latex_a_typst("\\Big\\bowtie"), "join");
    }

    #[test]
    fn big_familia_delimitadores_a_lr() {
        assert_eq!(latex_a_typst("\\Big[ x \\Big]"), "lr([x ])");
        assert_eq!(latex_a_typst("\\Big| y \\Big|"), "lr(|y |)");
    }
}
