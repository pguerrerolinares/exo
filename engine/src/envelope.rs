use serde_json::Value;

/// Versión del contrato de `data` para consumidores de exo (independiente
/// del `schema_version` de kbx). Cambio breaking en la forma de `data` ⇒
/// bump; campos aditivos no lo suben (spec §4).
pub const SCHEMA_VERSION: u32 = 1;

/// Emite `{"schema_version":1,"command":<command>,"data":<data>}` como una
/// única línea JSON, newline-terminada, a **stdout** — stdout es exclusivo
/// del envelope; todo lo humano/warnings va a stderr (spec §4, adopción del
/// patrón `envelope.Write` de kbx). Los consumidores gatean por exit code,
/// jamás por campos de `data`.
pub fn emite(command: &str, data: Value) {
    let envoltorio = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "command": command,
        "data": data,
    });
    println!("{envoltorio}");
}
