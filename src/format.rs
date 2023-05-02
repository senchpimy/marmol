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
