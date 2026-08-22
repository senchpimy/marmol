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
    let re =
        regex::Regex::new(r"(?s)\\buildrel\s*(.*?)\s*\\over\s*(\{.*\}|\S+)").unwrap();
    re.replace_all(&s, "\\overset{$1}{$2}").into_owned()
}

pub fn latex_a_typst(latex: &str) -> String {
    let norm = normalizar_latex(latex);
    let out = tex2typst_rs::tex2typst(&norm).unwrap_or(norm);
    out.replace("limits: true", "limits: #true")
}
