use base64::{engine::general_purpose, Engine as _};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde_json::Value;
use std::io::{Read, Write};
use std::path::Path;

fn bp_decode(bp_string: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let decoded = general_purpose::STANDARD.decode(&bp_string[1..])?;
    let mut decoder = ZlibDecoder::new(&decoded[..]);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str)?;
    Ok(serde_json::from_str(&json_str)?)
}

fn bp_encode(value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let json_str = serde_json::to_string(value)?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(json_str.as_bytes())?;
    let compressed = encoder.finish()?;
    Ok(format!("0{}", general_purpose::STANDARD.encode(&compressed)))
}

#[tauri::command]
fn decode_blueprint(bp: String) -> Result<String, String> {
    bp_decode(&bp)
        .map_err(|e| e.to_string())
        .and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string()))
}

#[tauri::command]
fn encode_blueprint(json: String) -> Result<String, String> {
    let value: Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    bp_encode(&value).map_err(|e| e.to_string())
}

// CARGO_MANIFEST_DIR = app/src-tauri/ → ../../ = project root
const PROJECT_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

#[derive(serde::Serialize)]
pub struct DocEntry {
    pub id: String,
    pub title: String,
    pub content: String,
    pub path: String,
}

#[tauri::command]
fn get_docs() -> Vec<DocEntry> {
    let root = Path::new(PROJECT_ROOT);
    let files = [
        ("readme",    "README",        "README.md"),
        ("readme_en", "README EN",     "README_EN.md"),
        ("docs",      "Documentation", "docs/documentation.md"),
    ];
    files.iter().filter_map(|(id, title, rel)| {
        std::fs::read_to_string(root.join(rel)).ok().map(|content| DocEntry {
            id: id.to_string(),
            title: title.to_string(),
            content,
            path: rel.to_string(),
        })
    }).collect()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![decode_blueprint, encode_blueprint, get_docs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
