use super::config::ConfigLlm;
use base64::{engine::general_purpose, Engine as _};
use serde_json::json;
use std::time::Duration;

pub fn reconocer(cfg: &ConfigLlm, png_bytes: &[u8]) -> Result<String, String> {
    let clave = cfg.api_key.trim();
    if clave.is_empty() {
        return Err("No hay API key configurada (⚙ en la barra o DEFAULT_API_KEY en config.rs)"
            .to_string());
    }

    let b64 = general_purpose::STANDARD.encode(png_bytes);
    let data_url = format!("data:image/png;base64,{}", b64);

    let body = json!({
        "model": cfg.modelo,
        "temperature": 0.0,
        "max_tokens": 1024,
        "messages": [
            {
                "role": "system",
                "content": "Eres un reconocedor de escritura matemática y texto manuscrito. Analiza la imagen del dibujo y responde ÚNICAMENTE con el código LaTeX equivalente. Sin explicaciones, sin bloques de código markdown, sin delimitadores $."
            },
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Convierte este dibujo manuscrito a LaTeX."},
                    {"type": "image_url", "image_url": {"url": data_url}}
                ]
            }
        ]
    });

    let cliente = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let cuerpo = serde_json::to_vec(&body).map_err(|e| format!("Error serializando: {}", e))?;

    let resp = cliente
        .post(normalizar_url(&cfg.url))
        .bearer_auth(clave)
        .header("Content-Type", "application/json")
        .body(cuerpo)
        .send()
        .map_err(|e| format!("Error de red: {}", e))?;

    let status = resp.status();
    let texto = resp
        .text()
        .map_err(|e| format!("Error leyendo respuesta: {}", e))?;

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status.as_u16(), recortar(&texto, 300)));
    }

    let v: serde_json::Value = serde_json::from_str(&texto)
        .map_err(|_| format!("Respuesta JSON inválida: {}", recortar(&texto, 300)))?;

    let contenido = v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| format!("Respuesta sin contenido: {}", recortar(&texto, 300)))?;

    Ok(limpiar_latex(contenido))
}

fn normalizar_url(url: &str) -> String {
    let u = url.trim().trim_end_matches('/');
    if u.ends_with("/chat/completions") {
        u.to_string()
    } else {
        format!("{}/chat/completions", u)
    }
}

fn limpiar_latex(s: &str) -> String {
    let mut t = s.trim();

    if t.starts_with("```") {
        t = t.trim_start_matches('`');
        t = t
            .trim_start_matches("latex")
            .trim_start_matches("math")
            .trim_start();
        if let Some(idx) = t.rfind("```") {
            t = t[..idx].trim_end();
        }
    }

    if t.starts_with("$$") && t.ends_with("$$") && t.len() > 4 {
        t = t[2..t.len() - 2].trim();
    } else if t.starts_with('$') && t.ends_with('$') && t.len() > 2 {
        t = t[1..t.len() - 1].trim();
    }

    t.to_string()
}

fn recortar(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut fin = max;
        while !s.is_char_boundary(fin) {
            fin -= 1;
        }
        format!("{}…", &s[..fin])
    }
}
