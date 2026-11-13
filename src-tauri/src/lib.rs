pub mod config;
pub mod pdf_generator;

use config::AppConfig;
use pdf_generator::{generar_pdf_bytes_con_config, ItemCodigo};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
fn analizar_codigo(codigo: &str) -> Result<String, String> {
    let limpio = codigo.trim();
    if limpio.is_empty() {
        return Err("El código no puede estar vacío".into());
    }

    Ok(format!(
        "Código validado en Rust: {} ({} caracteres)",
        limpio,
        limpio.len()
    ))
}

#[tauri::command]
fn cargar_configuracion(app: tauri::AppHandle) -> Result<AppConfig, String> {
    config::cargar_config_internal(&app)
}

#[tauri::command]
fn guardar_configuracion(app: tauri::AppHandle, nuevos_ajustes: AppConfig) -> Result<String, String> {
    config::guardar_config_internal(&app, &nuevos_ajustes)?;
    Ok("Ajustes guardados correctamente en el sistema".to_string())
}

#[tauri::command]
async fn generar_pdf_guardar(
    app: tauri::AppHandle,
    items: Vec<ItemCodigo>,
) -> Result<String, String> {
    if items.is_empty() {
        return Err("No hay códigos de barras en la lista para exportar a PDF".into());
    }

    let config = config::cargar_config_internal(&app)?;
    let pdf_bytes = generar_pdf_bytes_con_config(&items, &config)?;

    let file_path = app
        .dialog()
        .file()
        .set_title("Guardar Listado de Códigos de Barras en PDF")
        .set_file_name("codigos_barras_carta.pdf")
        .add_filter("Documento PDF (*.pdf)", &["pdf"])
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let path_buf = path
                .into_path()
                .map_err(|e| format!("Ruta de archivo no válida: {}", e))?;

            std::fs::write(&path_buf, pdf_bytes)
                .map_err(|e| format!("Error al escribir el archivo PDF en disco: {}", e))?;

            Ok(format!("PDF guardado exitosamente en: {}", path_buf.display()))
        }
        None => Err("Operación de guardado cancelada".into()),
    }
}

#[tauri::command]
async fn generar_pdf_imprimir(
    app: tauri::AppHandle,
    items: Vec<ItemCodigo>,
) -> Result<String, String> {
    if items.is_empty() {
        return Err("No hay códigos de barras en la lista para imprimir".into());
    }

    let config = config::cargar_config_internal(&app)?;
    let pdf_bytes = generar_pdf_bytes_con_config(&items, &config)?;

    let temp_path = std::env::temp_dir().join("codigos_barras_impresion.pdf");
    std::fs::write(&temp_path, pdf_bytes)
        .map_err(|e| format!("Error al guardar archivo temporal para impresión: {}", e))?;

    let path_str = temp_path.to_string_lossy().to_string();

    app.opener()
        .open_path(&path_str, None::<&str>)
        .map_err(|e| format!("Error al abrir el visor de impresión del sistema: {}", e))?;

    Ok(format!("Documento enviado al visor de impresión: {}", path_str))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            analizar_codigo,
            cargar_configuracion,
            guardar_configuracion,
            generar_pdf_guardar,
            generar_pdf_imprimir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
