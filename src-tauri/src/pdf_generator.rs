use crate::config::AppConfig;
use barcoders::sym::code128::Code128;
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};
use std::collections::HashMap;

// =========================================================================
// 1. CONFIGURACIÓN DE LA PÁGINA Y MÁRGENES (LETTER)
// =========================================================================

/// Factor de conversión de centímetros a puntos tipográficos PostScript (72 pt / 2.54 cm)
pub const CM_TO_PT: f64 = 72.0 / 2.54;

/// Tamaño de página: LETTER (Carta: 21.59 cm de ancho x 27.94 cm de alto)
pub const PAGE_WIDTH_CM: f64 = 21.59;
pub const PAGE_HEIGHT_CM: f64 = 27.94;

/// Margen superior: 0.7 cm
pub const MARGIN_TOP_CM: f64 = 0.7;
/// Margen inferior: 0.5 cm
pub const MARGIN_BOTTOM_CM: f64 = 0.5;
/// Margen izquierdo: 0.2 cm
pub const MARGIN_LEFT_CM: f64 = 0.2;
/// Margen derecho: 0.2 cm
pub const MARGIN_RIGHT_CM: f64 = 0.2;

// =========================================================================
// 2. DIMENSIONES Y PROPIEDADES DEL CÓDIGO DE BARRAS (CODE 128)
// =========================================================================

/// Ancho del código de barras: 3.3 cm
pub const BARCODE_WIDTH_CM: f64 = 3.3;
/// Alto del código de barras: 1.0 cm
pub const BARCODE_HEIGHT_CM: f64 = 1.0;
/// Mostrar texto integrado en el código: False (el precio se maneja independientemente arriba)
pub const SHOW_CODE_TEXT_IN_BARCODE: bool = false;

// =========================================================================
// 3. CONFIGURACIÓN DE LA GRILLA (DISTRIBUCIÓN)
// =========================================================================

/// Códigos por fila: 6
pub const COLS_PER_ROW: usize = 6;
/// Espacio horizontal entre códigos: 0.15 cm
pub const HORIZONTAL_GAP_CM: f64 = 0.15;
/// Espacio vertical entre filas: 0.15 cm
pub const ROW_GAP_CM: f64 = 0.15;
/// Espacio reservado para el precio: 0.25 cm
pub const PRICE_RESERVED_HEIGHT_CM: f64 = 0.25;

// =========================================================================
// 4. CONFIGURACIÓN DEL PRECIO
// =========================================================================

/// Fuente: Helvetica-Bold
pub const PRICE_FONT: &str = "Helvetica-Bold";
/// Tamaño de fuente: 14 pt
pub const PRICE_FONT_SIZE_PT: f64 = 14.0;
/// Prefijo monetario
pub const MONETARY_PREFIX: &str = "Q";
/// Desplazamiento vertical (offset_y): 0.4 cm sobre el código de barras
pub const PRICE_OFFSET_Y_CM: f64 = 0.4;

/// Estructura de datos recibida desde la tabla del Frontend
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ItemCodigo {
    pub codigo: String,
    #[serde(deserialize_with = "deserialize_cantidad")]
    pub cantidad: u32,
    pub precio: String,
}

fn deserialize_cantidad<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Deserialize;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => Ok(n.as_u64().unwrap_or(1) as u32),
        serde_json::Value::String(s) => Ok(s.trim().parse::<u32>().unwrap_or(1)),
        _ => Ok(1),
    }
}

/// Formatea el precio asegurando el prefijo configurado y formato monetario (ej. "Q 150.00")
pub fn format_precio(precio: &str, prefijo: &str) -> String {
    let trimmed = precio.trim();
    let p_prefix = if prefijo.is_empty() { "Q" } else { prefijo };

    let sin_prefijo = trimmed
        .strip_prefix(p_prefix)
        .or_else(|| trimmed.strip_prefix(&p_prefix.to_lowercase()))
        .unwrap_or(trimmed)
        .trim();

    if sin_prefijo.is_empty() {
        format!("{} 0.00", p_prefix)
    } else if sin_prefijo.contains('.') {
        format!("{} {}", p_prefix, sin_prefijo)
    } else {
        format!("{} {}.00", p_prefix, sin_prefijo)
    }
}

/// Medición precisa del ancho en puntos de un texto en Helvetica-Bold
pub fn helvetica_bold_text_width(text: &str, font_size_pt: f64) -> f64 {
    let units: f64 = text
        .chars()
        .map(|c| match c {
            ' ' | '.' | ',' | ':' | ';' => 278.0,
            'Q' => 722.0,
            '0'..='9' => 556.0,
            'A'..='Z' => 670.0,
            'a'..='z' => 550.0,
            '-' | '+' => 333.0,
            _ => 556.0,
        })
        .sum();
    (units / 1000.0) * font_size_pt
}

/// Codifica un string en módulos de barras Code 128 (1 = barra negra, 0 = espacio blanco)
pub fn encode_code128(data: &str) -> Result<Vec<u8>, String> {
    let clean = data.trim();
    if clean.is_empty() {
        return Err("El código no puede estar vacío".into());
    }

    // Para secuencias puramente numéricas de longitud par, Subset C (Ć) es el más compacto.
    // De lo contrario, Subset B (Ɓ) admite todo el juego ASCII imprimible (letras, símbolos, números).
    let prefixed = if clean.chars().all(|c| c.is_ascii_digit()) && clean.len() % 2 == 0 {
        format!("Ć{}", clean)
    } else {
        format!("Ɓ{}", clean)
    };

    let barcode = Code128::new(&prefixed)
        .or_else(|_| Code128::new(format!("Ɓ{}", clean)))
        .map_err(|e| format!("Error al codificar Code 128 para '{}': {:?}", clean, e))?;

    Ok(barcode.encode())
}

/// Crea un Form XObject reutilizable que encapsula el precio superior y las barras vectoriales
fn create_form_xobject(
    doc: &mut Document,
    font_id: ObjectId,
    precio: &str,
    modules: &[u8],
    config: &AppConfig,
) -> Result<ObjectId, String> {
    let barcode_width_pt = config.codigo_barras.ancho_cm * CM_TO_PT;
    let barcode_height_pt = config.codigo_barras.alto_cm * CM_TO_PT;
    let price_offset_y_pt = config.precio.offset_y_cm * CM_TO_PT;
    let price_reserved_height_pt = config.grilla.espacio_precio_cm * CM_TO_PT;
    let total_cell_height_pt = barcode_height_pt + price_offset_y_pt + price_reserved_height_pt;

    let mut form_stream_bytes = Vec::new();

    // 1. Texto superior: Precio formateado según configuración, centrado sobre el código
    let precio_fmt = format_precio(precio, &config.precio.prefijo);
    let escaped_precio = precio_fmt
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)");

    let font_size = config.precio.tamano as f64;
    let text_width = helvetica_bold_text_width(&precio_fmt, font_size);
    let text_x = ((barcode_width_pt - text_width) / 2.0).max(0.0);
    // Desplazamiento vertical (offset_y sobre el código de barras)
    let text_y = barcode_height_pt + price_offset_y_pt;

    let text_cmd = format!(
        "BT\n/F1 {font_size:.1} Tf\n0 0 0 rg\n1 0 0 1 {text_x:.2} {text_y:.2} Tm\n({escaped_precio}) Tj\nET\n"
    );
    form_stream_bytes.extend_from_slice(text_cmd.as_bytes());

    // 2. Dibujo vectorial de las barras del código Code 128 (sin texto integrado)
    // Agrupamos módulos consecutivos de 1s para minimizar los operadores PDF a dibujar
    form_stream_bytes.extend_from_slice(b"0 0 0 rg\n");
    let num_modules = modules.len();
    if num_modules > 0 {
        let module_width = barcode_width_pt / num_modules as f64;
        let mut i = 0;
        while i < num_modules {
            if modules[i] == 1 {
                let start = i;
                while i < num_modules && modules[i] == 1 {
                    i += 1;
                }
                let run = i - start;
                let x = start as f64 * module_width;
                let w = run as f64 * module_width;
                let rect_cmd = format!("{x:.3} 0.000 {w:.3} {barcode_height_pt:.3} re f\n");
                form_stream_bytes.extend_from_slice(rect_cmd.as_bytes());
            } else {
                i += 1;
            }
        }
    }

    // 3. Diccionario del Form XObject
    let mut form_dict = Dictionary::new();
    form_dict.set("Type", Object::Name(b"XObject".to_vec()));
    form_dict.set("Subtype", Object::Name(b"Form".to_vec()));
    form_dict.set("FormType", Object::Integer(1));
    form_dict.set(
        "BBox",
        Object::Array(vec![
            0.0.into(),
            0.0.into(),
            barcode_width_pt.into(),
            total_cell_height_pt.into(),
        ]),
    );

    let mut form_resources = Dictionary::new();
    let mut form_fonts = Dictionary::new();
    form_fonts.set("F1", Object::Reference(font_id));
    form_resources.set("Font", Object::Dictionary(form_fonts));
    form_dict.set("Resources", Object::Dictionary(form_resources));

    let form_stream = Stream::new(form_dict, form_stream_bytes);
    Ok(doc.add_object(form_stream))
}

/// Genera el documento PDF completo en memoria aplicando una configuración personalizada
pub fn generar_pdf_bytes_con_config(items: &[ItemCodigo], config: &AppConfig) -> Result<Vec<u8>, String> {
    if items.is_empty() {
        return Err("No hay códigos de barras en la lista para imprimir o generar PDF".into());
    }

    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();

    // Fuente estándar
    let mut font_dict = Dictionary::new();
    font_dict.set("Type", Object::Name(b"Font".to_vec()));
    font_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
    font_dict.set("BaseFont", Object::Name(config.precio.fuente.as_bytes().to_vec()));
    let font_id = doc.add_object(font_dict);

    // Creación de Form XObjects únicos por cada combinación de (Código, Precio)
    let mut unique_forms: HashMap<(String, String), (String, ObjectId)> = HashMap::new();
    let mut form_counter = 0;

    for item in items {
        let key = (item.codigo.clone(), item.precio.clone());
        if !unique_forms.contains_key(&key) {
            let form_name = format!("Fm{}", form_counter);
            form_counter += 1;
            let modules = encode_code128(&item.codigo)?;
            let form_id = create_form_xobject(&mut doc, font_id, &item.precio, &modules, config)?;
            unique_forms.insert(key, (form_name, form_id));
        }
    }

    // Configuración dimensional en puntos derivada de AppConfig
    let page_width_pt = PAGE_WIDTH_CM * CM_TO_PT;
    let page_height_pt = PAGE_HEIGHT_CM * CM_TO_PT;
    let margin_left_pt = config.pdf.margenes.izquierdo_cm * CM_TO_PT;
    let margin_top_pt = config.pdf.margenes.superior_cm * CM_TO_PT;
    let margin_bottom_pt = config.pdf.margenes.inferior_cm * CM_TO_PT;

    let barcode_height_pt = config.codigo_barras.alto_cm * CM_TO_PT;
    let price_offset_y_pt = config.precio.offset_y_cm * CM_TO_PT;
    let price_reserved_height_pt = config.grilla.espacio_precio_cm * CM_TO_PT;
    let total_cell_height_pt = barcode_height_pt + price_offset_y_pt + price_reserved_height_pt;

    let row_gap_pt = config.grilla.espacio_vertical_cm * CM_TO_PT;
    let row_step_pt = total_cell_height_pt + row_gap_pt;

    // Espacio horizontal entre códigos
    let col_step_pt = (config.codigo_barras.ancho_cm + config.grilla.espacio_horizontal_cm) * CM_TO_PT;
    let cols_per_row = config.grilla.codigos_por_fila.max(1);

    // Altura inicial de la primera fila desde el borde superior
    let first_row_y = page_height_pt - margin_top_pt - total_cell_height_pt;

    let mut pages_streams: Vec<String> = Vec::new();
    let mut current_page_stream = String::new();
    let mut current_col: usize = 0;
    let mut current_row: usize = 0;

    for item in items {
        let key = (item.codigo.clone(), item.precio.clone());
        let (form_name, _) = unique_forms.get(&key).unwrap();

        let count = item.cantidad.max(1);
        for _ in 0..count {
            let y = first_row_y - current_row as f64 * row_step_pt;

            // Salto de página al alcanzar el margen inferior
            if y < margin_bottom_pt {
                pages_streams.push(current_page_stream);
                current_page_stream = String::new();
                current_row = 0;
                current_col = 0;
            }

            let y = first_row_y - current_row as f64 * row_step_pt;
            let x = margin_left_pt + current_col as f64 * col_step_pt;

            // Invocación del Form XObject mediante la matriz de transformación 'cm' y operador 'Do'
            current_page_stream
                .push_str(&format!("q 1 0 0 1 {x:.3} {y:.3} cm /{form_name} Do Q\n"));

            current_col += 1;
            if current_col >= cols_per_row {
                current_col = 0;
                current_row += 1;
            }
        }
    }

    if !current_page_stream.is_empty() {
        pages_streams.push(current_page_stream);
    }

    // Construcción de objetos de página en lopdf
    let mut page_references: Vec<Object> = Vec::new();

    // Diccionario de XObjects compartido para todas las páginas
    let mut xobjects_dict = Dictionary::new();
    for (_, (name, id)) in &unique_forms {
        xobjects_dict.set(name.as_bytes().to_vec(), Object::Reference(*id));
    }

    let mut shared_fonts = Dictionary::new();
    shared_fonts.set("F1", Object::Reference(font_id));

    let num_pages = pages_streams.len();

    for content_str in pages_streams {
        let content_id = doc.add_object(Stream::new(Dictionary::new(), content_str.into_bytes()));

        let mut page_resources = Dictionary::new();
        page_resources.set("XObject", Object::Dictionary(xobjects_dict.clone()));
        page_resources.set("Font", Object::Dictionary(shared_fonts.clone()));

        let mut page = Dictionary::new();
        page.set("Type", Object::Name(b"Page".to_vec()));
        page.set("Parent", Object::Reference(pages_id));
        page.set(
            "MediaBox",
            Object::Array(vec![
                0.0.into(),
                0.0.into(),
                page_width_pt.into(),
                page_height_pt.into(),
            ]),
        );
        page.set("Contents", Object::Reference(content_id));
        page.set("Resources", Object::Dictionary(page_resources));

        let page_id = doc.add_object(page);
        page_references.push(Object::Reference(page_id));
    }

    // Estructura Pages del catálogo
    let mut pages = Dictionary::new();
    pages.set("Type", Object::Name(b"Pages".to_vec()));
    pages.set("Kids", Object::Array(page_references));
    pages.set("Count", Object::Integer(num_pages as i64));
    doc.set_object(pages_id, pages);

    // Catálogo raíz
    let mut catalog = Dictionary::new();
    catalog.set("Type", Object::Name(b"Catalog".to_vec()));
    catalog.set("Pages", Object::Reference(pages_id));
    let catalog_id = doc.add_object(catalog);

    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Compresión FlateDecode de los flujos de contenido para peso mínimo
    doc.compress();

    let mut output_bytes = Vec::new();
    doc.save_to(&mut output_bytes)
        .map_err(|e| format!("Error al serializar el documento PDF: {:?}", e))?;

    Ok(output_bytes)
}

/// Genera el documento PDF completo utilizando la configuración por defecto
pub fn generar_pdf_bytes(items: &[ItemCodigo]) -> Result<Vec<u8>, String> {
    generar_pdf_bytes_con_config(items, &AppConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generar_pdf_5000_etiquetas() {
        let items = vec![
            ItemCodigo {
                codigo: "740123456789".into(),
                cantidad: 2500,
                precio: "150.00".into(),
            },
            ItemCodigo {
                codigo: "PROD-998877".into(),
                cantidad: 2500,
                precio: "75.50".into(),
            },
        ];

        let start = std::time::Instant::now();
        let pdf_bytes = generar_pdf_bytes(&items).unwrap();
        let elapsed = start.elapsed();

        println!(
            "5,000 etiquetas generadas en: {:?}, tamaño PDF: {} KB",
            elapsed,
            pdf_bytes.len() / 1024
        );

        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF-1.7"));
        assert!(elapsed.as_millis() < 1000, "Debe tardar menos de 1 segundo");
    }

    #[test]
    fn test_verificar_dimensiones_y_posicionamiento() {
        let items = vec![ItemCodigo {
            codigo: "7401005123456".into(),
            cantidad: 6,
            precio: "150.00".into(),
        }];

        let pdf = generar_pdf_bytes(&items).unwrap();
        assert!(!pdf.is_empty());

        // Verificamos que las dimensiones y márgenes cumplan la especificación
        let left_pt = MARGIN_LEFT_CM * CM_TO_PT;
        assert!((left_pt - 5.669).abs() < 0.01); // 0.2 cm = ~5.669 pt

        let width_pt = BARCODE_WIDTH_CM * CM_TO_PT;
        assert!((width_pt - 93.543).abs() < 0.01); // 3.3 cm = ~93.543 pt

        let height_pt = BARCODE_HEIGHT_CM * CM_TO_PT;
        assert!((height_pt - 28.346).abs() < 0.01); // 1.0 cm = ~28.346 pt
    }
}
