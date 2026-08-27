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

    // Comandos de estilo: se eliminan aquí porque se manejan en latex_a_typst
    // (typst no los reconoce y tex2typst los pasaría como texto).
    let s = s.replace("\\displaystyle", "");
    let s = s.replace("\\textstyle", "");
    let s = s.replace("\\scriptscriptstyle", "");
    let s = s.replace("\\scriptstyle", "");

    s
}

pub fn latex_a_typst(latex: &str) -> String {
    // \displaystyle fuerza estilo de visualización: en Typst se traduce a la
    // función math `display(...)` envolviendo toda la expresión.
    let has_displaystyle = latex.contains("\\displaystyle");

    let norm = normalizar_latex(latex);
    let out = tex2typst_rs::tex2typst(&norm).unwrap_or(norm);
    let out = out.replace("limits: true", "limits: #true");

    if has_displaystyle {
        format!("display({})", out)
    } else {
        out
    }
}
