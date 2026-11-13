use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

// =========================================================================
// ESTRUCTURAS DE CONFIGURACIÓN TIPADAS
// =========================================================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MargenesConfig {
    pub superior_cm: f64,
    pub inferior_cm: f64,
    pub izquierdo_cm: f64,
    pub derecho_cm: f64,
}

impl Default for MargenesConfig {
    fn default() -> Self {
        Self {
            superior_cm: 0.7,
            inferior_cm: 0.5,
            izquierdo_cm: 0.2,
            derecho_cm: 0.2,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PdfConfig {
    pub pagesize: String,
    pub margenes: MargenesConfig,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            pagesize: "LETTER".to_string(),
            margenes: MargenesConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CodigoBarrasConfig {
    pub tipo: String,
    pub ancho_cm: f64,
    pub alto_cm: f64,
    pub mostrar_texto: bool,
}

impl Default for CodigoBarrasConfig {
    fn default() -> Self {
        Self {
            tipo: "CODE128".to_string(),
            ancho_cm: 3.3,
            alto_cm: 1.0,
            mostrar_texto: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GrillaConfig {
    pub codigos_por_fila: usize,
    pub espacio_horizontal_cm: f64,
    pub espacio_vertical_cm: f64,
    pub espacio_precio_cm: f64,
}

impl Default for GrillaConfig {
    fn default() -> Self {
        Self {
            codigos_por_fila: 6,
            espacio_horizontal_cm: 0.15,
            espacio_vertical_cm: 0.15,
            espacio_precio_cm: 0.25,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PrecioConfig {
    pub fuente: String,
    pub tamano: u32,
    pub prefijo: String,
    pub offset_y_cm: f64,
}

impl Default for PrecioConfig {
    fn default() -> Self {
        Self {
            fuente: "Helvetica-Bold".to_string(),
            tamano: 14,
            prefijo: "Q".to_string(),
            offset_y_cm: 0.4,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AppConfig {
    pub pdf: PdfConfig,
    pub codigo_barras: CodigoBarrasConfig,
    pub grilla: GrillaConfig,
    pub precio: PrecioConfig,
}

// =========================================================================
// GESTIÓN DEL ALMACENAMIENTO OCULTO EN EL SISTEMA OPERATIVO
// =========================================================================

/// Obtiene la ruta del archivo de configuración dentro del directorio de datos de la app
pub fn get_config_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo determinar el directorio de datos del sistema: {}", e))?;

    if !dir.exists() {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("No se pudo crear el directorio de datos '{:?}': {}", dir, e))?;
    }

    Ok(dir.join("config.json"))
}

/// Carga la configuración del sistema. Si el archivo no existe, lo crea con valores por defecto óptimos.
pub fn cargar_config_internal(app: &tauri::AppHandle) -> Result<AppConfig, String> {
    let path = get_config_file_path(app)?;

    if !path.exists() {
        let default_config = AppConfig::default();
        let json_str = serde_json::to_string_pretty(&default_config)
            .map_err(|e| format!("Error al serializar configuración por defecto: {}", e))?;
        fs::write(&path, json_str)
            .map_err(|e| format!("Error al crear el archivo de configuración inicial: {}", e))?;
        return Ok(default_config);
    }

    let contenido = fs::read_to_string(&path)
        .map_err(|e| format!("Error al leer el archivo de configuración '{:?}': {}", path, e))?;

    match serde_json::from_str::<AppConfig>(&contenido) {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("Advertencia: Configuración corrupta ({:?}), regenerando valores por defecto", e);
            let default_config = AppConfig::default();
            let json_str = serde_json::to_string_pretty(&default_config)
                .map_err(|err| format!("Error al serializar configuración por defecto: {}", err))?;
            let _ = fs::write(&path, json_str);
            Ok(default_config)
        }
    }
}

/// Guarda la configuración actualizada en el archivo JSON dentro del directorio de datos
pub fn guardar_config_internal(app: &tauri::AppHandle, nuevos_ajustes: &AppConfig) -> Result<(), String> {
    let path = get_config_file_path(app)?;

    let json_str = serde_json::to_string_pretty(nuevos_ajustes)
        .map_err(|e| format!("Error al convertir la configuración a formato JSON: {}", e))?;

    fs::write(&path, json_str)
        .map_err(|e| format!("Error al escribir el archivo de configuración en '{:?}': {}", path, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_json_structure() {
        let config = AppConfig::default();
        let json_value = serde_json::to_value(&config).unwrap();

        assert_eq!(json_value["pdf"]["pagesize"], "LETTER");
        assert_eq!(json_value["pdf"]["margenes"]["superior_cm"], 0.7);
        assert_eq!(json_value["pdf"]["margenes"]["inferior_cm"], 0.5);
        assert_eq!(json_value["pdf"]["margenes"]["izquierdo_cm"], 0.2);
        assert_eq!(json_value["pdf"]["margenes"]["derecho_cm"], 0.2);

        assert_eq!(json_value["codigo_barras"]["tipo"], "CODE128");
        assert_eq!(json_value["codigo_barras"]["ancho_cm"], 3.3);
        assert_eq!(json_value["codigo_barras"]["alto_cm"], 1.0);
        assert_eq!(json_value["codigo_barras"]["mostrar_texto"], false);

        assert_eq!(json_value["grilla"]["codigos_por_fila"], 6);
        assert_eq!(json_value["grilla"]["espacio_horizontal_cm"], 0.15);
        assert_eq!(json_value["grilla"]["espacio_vertical_cm"], 0.15);
        assert_eq!(json_value["grilla"]["espacio_precio_cm"], 0.25);

        assert_eq!(json_value["precio"]["fuente"], "Helvetica-Bold");
        assert_eq!(json_value["precio"]["tamano"], 14);
        assert_eq!(json_value["precio"]["prefijo"], "Q");
        assert_eq!(json_value["precio"]["offset_y_cm"], 0.4);
    }

    #[test]
    fn test_custom_config_deserialization() {
        let json_input = r#"{
            "pdf": {
                "pagesize": "LETTER",
                "margenes": {
                    "superior_cm": 1.0,
                    "inferior_cm": 0.8,
                    "izquierdo_cm": 0.3,
                    "derecho_cm": 0.3
                }
            },
            "codigo_barras": {
                "tipo": "CODE128",
                "ancho_cm": 3.5,
                "alto_cm": 1.2,
                "mostrar_texto": true
            },
            "grilla": {
                "codigos_por_fila": 5,
                "espacio_horizontal_cm": 0.2,
                "espacio_vertical_cm": 0.2,
                "espacio_precio_cm": 0.3
            },
            "precio": {
                "fuente": "Helvetica-Bold",
                "tamano": 12,
                "prefijo": "$",
                "offset_y_cm": 0.5
            }
        }"#;

        let parsed: AppConfig = serde_json::from_str(json_input).expect("Debe deserializar correctamente");
        assert_eq!(parsed.pdf.margenes.superior_cm, 1.0);
        assert_eq!(parsed.codigo_barras.mostrar_texto, true);
        assert_eq!(parsed.grilla.codigos_por_fila, 5);
        assert_eq!(parsed.precio.prefijo, "$");
        assert_eq!(parsed.precio.tamano, 12);
    }
}
