use base64::{engine::general_purpose, Engine as _};
use eframe::egui;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{Read, Write};

// ── Модели ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub index: u32,
    pub name: String,
    pub count: i64,
    pub comparator: String,
    #[serde(rename = "quality", skip_serializing_if = "Option::is_none")]
    pub signal_quality: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub signal_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub name: String,
    #[serde(rename = "quality", skip_serializing_if = "Option::is_none")]
    pub signal_quality: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub signal_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub comparator: String,
    pub constant: i64,
    pub first_signal: Signal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub condition: Condition,
    pub icon: Signal,
}

impl From<&Filter> for Parameter {
    fn from(f: &Filter) -> Self {
        Parameter {
            condition: Condition {
                comparator: f.comparator.clone(),
                constant: f.count,
                first_signal: Signal {
                    name: format!("parameter-{}", f.index - 1),
                    signal_quality: None,
                    signal_type: None,
                },
            },
            icon: Signal {
                name: f.name.clone(),
                signal_quality: f.signal_quality.clone(),
                signal_type: f.signal_type.clone(),
            },
        }
    }
}

// ── Утилиты ───────────────────────────────────────────────────────────────────

fn decode_blueprint(bp_string: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let data = &bp_string[1..];
    let decoded = general_purpose::STANDARD.decode(data)?;
    let mut decoder = ZlibDecoder::new(&decoded[..]);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str)?;
    Ok(serde_json::from_str(&json_str)?)
}

fn encode_blueprint(value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let json_str = serde_json::to_string(value)?;
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(json_str.as_bytes())?;
    let compressed = encoder.finish()?;
    Ok(format!("0{}", general_purpose::STANDARD.encode(&compressed)))
}

#[allow(dead_code)]
fn extract_filters(blueprint: &Value) -> Vec<Filter> {
    let mut result = Vec::new();
    let Some(entities) = blueprint["blueprint"]["entities"].as_array() else {
        return result;
    };
    for entity in entities {
        let Some(sections) = entity["control_behavior"]["sections"]["sections"].as_array() else {
            continue;
        };
        for section in sections {
            let Some(filters) = section["filters"].as_array() else { continue };
            for f in filters {
                if let Ok(filter) = serde_json::from_value::<Filter>(f.clone()) {
                    result.push(filter);
                }
            }
        }
    }
    result
}

#[allow(dead_code)]
fn build_blueprint(filters: &[Filter]) -> Value {
    let filters_json: Vec<Value> = filters.iter().map(|f| json!(f)).collect();
    json!({
        "blueprint": {
            "icons": [{ "signal": { "name": "constant-combinator" }, "index": 1 }],
            "entities": [{
                "entity_number": 1,
                "name": "constant-combinator",
                "position": { "x": 0.0, "y": 0.0 },
                "control_behavior": {
                    "sections": { "sections": [{ "index": 1, "filters": filters_json }] }
                }
            }],
            "item": "blueprint",
            "version": 562949958402048u64
        }
    })
}

// ── GUI ───────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct App {
    blueprint: String,
    json: String,
    status: String,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.label(&self.status);
        });

        egui::SidePanel::left("bp_panel")
            .resizable(true)
            .default_width(500.0)
            .show(ctx, |ui| {
                ui.heading("Blueprint");
                if ui.button("Decode  →  JSON").clicked() {
                    match decode_blueprint(self.blueprint.trim()) {
                        Ok(v) => {
                            self.json = serde_json::to_string_pretty(&v).unwrap_or_default();
                            self.status = "Декодировано".to_string();
                        }
                        Err(e) => self.status = format!("Ошибка: {e}"),
                    }
                }
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut self.blueprint)
                        .font(egui::TextStyle::Monospace),
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("JSON");
            if ui.button("Encode  →  Blueprint").clicked() {
                match serde_json::from_str::<Value>(&self.json) {
                    Ok(v) => match encode_blueprint(&v) {
                        Ok(s) => {
                            self.blueprint = s;
                            self.status = "Закодировано".to_string();
                        }
                        Err(e) => self.status = format!("Ошибка кодирования: {e}"),
                    },
                    Err(e) => self.status = format!("Ошибка парсинга JSON: {e}"),
                }
            }
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut self.json)
                    .font(egui::TextStyle::Monospace),
            );
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Factorio Blueprint Tool"),
        ..Default::default()
    };
    eframe::run_native(
        "Factorio Blueprint Tool",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
