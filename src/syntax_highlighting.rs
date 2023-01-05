use egui::text::LayoutJob;
use egui::{ TextFormat, FontId, Color32, Stroke, TextStyle, RichText };

pub fn word_to_canvas(s:String, job: &mut LayoutJob,ui: &mut egui::Ui){
    let text:Vec<String>=s.split("\n\n").map(|x| x.to_string()).collect();
    for mut words in text{
        words = words.replace("\n", " ");
        let words:Vec<&str> = words.split_whitespace().collect();
        for word in words{
            if word.contains("*"){
                italic(&word, job)
            }else{
                job.append(
                    &word,
                    0.0,
                    TextFormat {
                        color: Color32::WHITE,
                        ..Default::default()
                    },
                 );
            }
        }
    }
//    for &word in text{
//    }
}


fn bold(word:&str,job:&mut LayoutJob){
job.append(word,0.0,
    TextFormat {
    color: Color32::BLACK,
    ..Default::default()
    },
    );
}

fn italic(word:&str,job:&mut LayoutJob){
job.append(word,0.0,
    TextFormat {
        italics:true,
    ..Default::default()
    },
    );
}

fn strike(word:&str,job:&mut LayoutJob){
job.append(word,0.0,
    TextFormat {
        strikethrough:Stroke{width:10.,color:Color32::WHITE},
    ..Default::default()
    },
    );
}
