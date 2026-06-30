use base64::{engine::general_purpose, Engine as _};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::fs;

/// Декодирует blueprint string в JSON (serde_json::Value)
fn decode_blueprint(bp_string: &str) -> Result<Value, Box<dyn std::error::Error>> {
    // первый символ - версия формата, отбрасываем
    let data = &bp_string[1..];
    let decoded = general_purpose::STANDARD.decode(data)?;

    let mut decoder = ZlibDecoder::new(&decoded[..]);
    let mut json_str = String::new();
    decoder.read_to_string(&mut json_str)?;

    let value: Value = serde_json::from_str(&json_str)?;
    Ok(value)
}

/// Кодирует JSON обратно в blueprint string
fn encode_blueprint(value: &Value) -> Result<String, Box<dyn std::error::Error>> {
    let json_str = serde_json::to_string(value)?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(json_str.as_bytes())?;
    let compressed = encoder.finish()?;

    let encoded = general_purpose::STANDARD.encode(&compressed);
    Ok(format!("0{}", encoded))
}

/// Список сигналов для таблицы кодов: (тип сигнала, имя, код)
/// тип: "item", "fluid", "virtual"
struct SignalEntry {
    sig_type: &'static str,
    name: &'static str,
    code: i64,
}

/// Генерирует JSON одного константного комбинатора с заданным списком сигналов.
/// Учитывает что в одном константном комбинаторе ограниченное число слотов
/// (в новых версиях Factorio лимит большой за счёт нескольких секций, но
/// раньше было ограничение ~20 на "страницу" - проверь актуальный лимит у себя).
fn build_constant_combinator(entries: &[SignalEntry], x: f64, y: f64) -> Value {
    let filters: Vec<Value> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            json!({
                "index": i + 1,
                "signal": {
                    "type": e.sig_type,
                    "name": e.name
                },
                "count": e.code
            })
        })
        .collect();

    json!({
        "entity_number": 1,
        "name": "constant-combinator",
        "position": { "x": x, "y": y },
        "control_behavior": {
            "filters": filters
        }
    })
}

/// Собирает полный blueprint JSON с одним константным комбинатором
fn build_blueprint(entries: &[SignalEntry]) -> Value {
    let entity = build_constant_combinator(entries, 0.0, 0.0);

    json!({
        "blueprint": {
            "icons": [
                { "signal": { "name": "constant-combinator" }, "index": 1 }
            ],
            "entities": [entity],
            "item": "blueprint",
            "version": 562949958402048u64
        }
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let input_file = args.get(1).map(String::as_str).unwrap_or("data/blueprint.txt");
    let output_file = args.get(2).map(String::as_str).unwrap_or("data/base_display.json");

    let bp_string = fs::read_to_string(input_file)
        .map_err(|e| format!("Не удалось прочитать '{}': {}", input_file, e))?;
    let bp_string = bp_string.trim();

    let decoded = decode_blueprint(bp_string)
        .map_err(|e| format!("Ошибка декодирования blueprint: {}", e))?;

    let json_str = serde_json::to_string_pretty(&decoded)?;
    fs::write(output_file, &json_str)
        .map_err(|e| format!("Не удалось записать '{}': {}", output_file, e))?;

    println!("Готово: '{}' -> '{}'", input_file, output_file);
    Ok(())
}
