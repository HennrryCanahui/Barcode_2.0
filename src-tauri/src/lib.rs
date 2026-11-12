#[tauri::command]
fn analizar_codigo(codigo: &str) -> Result<String, String> {
    let limpio = codigo.trim();
    if limpio.is_empty() {
        return Err("El código no puede estar vacío".into());
    }

    // Aquí irá tu lógica en Rust (validar EAN-13, guardar en SQLite, etc.)
    Ok(format!(
        "Código validado en Rust: {} ({} caracteres)",
        limpio,
        limpio.len()
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![analizar_codigo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
