import { invoke } from "@tauri-apps/api/core";

// Interfaces de configuración
export interface MargenesConfig {
  superior_cm: number;
  inferior_cm: number;
  izquierdo_cm: number;
  derecho_cm: number;
}

export interface PdfConfig {
  pagesize: string;
  margenes: MargenesConfig;
}

export interface CodigoBarrasConfig {
  tipo: string;
  ancho_cm: number;
  alto_cm: number;
  mostrar_texto: boolean;
}

export interface GrillaConfig {
  codigos_por_fila: number;
  espacio_horizontal_cm: number;
  espacio_vertical_cm: number;
  espacio_precio_cm: number;
}

export interface PrecioConfig {
  fuente: string;
  tamano: number;
  prefijo: string;
  offset_y_cm: number;
}

export interface AppConfig {
  pdf: PdfConfig;
  codigo_barras: CodigoBarrasConfig;
  grilla: GrillaConfig;
  precio: PrecioConfig;
}

export const DEFAULT_CONFIG: AppConfig = {
  pdf: {
    pagesize: "LETTER",
    margenes: {
      superior_cm: 0.7,
      inferior_cm: 0.5,
      izquierdo_cm: 0.2,
      derecho_cm: 0.2,
    },
  },
  codigo_barras: {
    tipo: "CODE128",
    ancho_cm: 3.3,
    alto_cm: 1.0,
    mostrar_texto: false,
  },
  grilla: {
    codigos_por_fila: 6,
    espacio_horizontal_cm: 0.15,
    espacio_vertical_cm: 0.15,
    espacio_precio_cm: 0.25,
  },
  precio: {
    fuente: "Helvetica-Bold",
    tamano: 14,
    prefijo: "Q",
    offset_y_cm: 0.4,
  },
};

// Comandos IPC de configuración
export async function cargarConfiguracion(): Promise<AppConfig> {
  return await invoke<AppConfig>("cargar_configuracion");
}

export async function guardarConfiguracion(nuevosAjustes: AppConfig): Promise<string> {
  return await invoke<string>("guardar_configuracion", {
    nuevosAjustes,
    nuevos_ajustes: nuevosAjustes,
  });
}

// Elementos de la interfaz principal
const inputCodigo = document.querySelector<HTMLInputElement>("#input-codigo");
const inputCantidad = document.querySelector<HTMLInputElement>("#input-cantidad");
const inputPrecio = document.querySelector<HTMLInputElement>("#input-precio");
const btnAgregar = document.querySelector<HTMLButtonElement>("#btn-agregar");
const btnGenerar = document.querySelector<HTMLButtonElement>("#btn-generar");
const btnPdf = document.querySelector<HTMLButtonElement>("#btn-pdf");
const btnImprimir = document.querySelector<HTMLButtonElement>("#btn-imprimir");
const tablaCuerpo = document.querySelector<HTMLTableSectionElement>("#tabla-cuerpo");

// Elementos del Modal de Ajustes
const btnAjustes = document.querySelector<HTMLButtonElement>("#btn-ajustes");
const modalAjustes = document.querySelector<HTMLDivElement>("#modal-ajustes");
const btnCerrarAjustes = document.querySelector<HTMLButtonElement>("#btn-cerrar-ajustes");
const btnCancelarAjustes = document.querySelector<HTMLButtonElement>("#btn-cancelar-ajustes");
const btnResetAjustes = document.querySelector<HTMLButtonElement>("#btn-reset-ajustes");
const btnGuardarAjustes = document.querySelector<HTMLButtonElement>("#btn-guardar-ajustes");

// Inputs del modal
const cfgPagesize = document.querySelector<HTMLInputElement>("#cfg-pagesize");
const cfgMargenSuperior = document.querySelector<HTMLInputElement>("#cfg-margen-superior");
const cfgMargenInferior = document.querySelector<HTMLInputElement>("#cfg-margen-inferior");
const cfgMargenIzquierdo = document.querySelector<HTMLInputElement>("#cfg-margen-izquierdo");
const cfgMargenDerecho = document.querySelector<HTMLInputElement>("#cfg-margen-derecho");

const cfgBarcodeTipo = document.querySelector<HTMLInputElement>("#cfg-barcode-tipo");
const cfgBarcodeAncho = document.querySelector<HTMLInputElement>("#cfg-barcode-ancho");
const cfgBarcodeAlto = document.querySelector<HTMLInputElement>("#cfg-barcode-alto");
const cfgBarcodeTexto = document.querySelector<HTMLInputElement>("#cfg-barcode-texto");

const cfgGrillaColumnas = document.querySelector<HTMLInputElement>("#cfg-grilla-columnas");
const cfgGrillaHgap = document.querySelector<HTMLInputElement>("#cfg-grilla-hgap");
const cfgGrillaVgap = document.querySelector<HTMLInputElement>("#cfg-grilla-vgap");
const cfgGrillaPrecioSpace = document.querySelector<HTMLInputElement>("#cfg-grilla-precio-space");

const cfgPrecioFuente = document.querySelector<HTMLInputElement>("#cfg-precio-fuente");
const cfgPrecioTamano = document.querySelector<HTMLInputElement>("#cfg-precio-tamano");
const cfgPrecioPrefijo = document.querySelector<HTMLInputElement>("#cfg-precio-prefijo");
const cfgPrecioOffsetY = document.querySelector<HTMLInputElement>("#cfg-precio-offset-y");

let filaEnEdicion: HTMLTableRowElement | null = null;
let prefijoMoneda = "Q";

interface ItemCodigo {
  codigo: string;
  cantidad: number;
  precio: string;
}

// Generador de 12 dígitos ultrarrápido
function generarCodigo12(): string {
  const min = 100000000000;
  const max = 999999999999;
  const randomNum = Math.floor(min + Math.random() * (max - min + 1));
  return randomNum.toString();
}

// Botón Generar (solo inserta el código y enfoca el input sin seleccionarlo todo)
btnGenerar?.addEventListener("click", () => {
  if (!inputCodigo) return;
  inputCodigo.value = generarCodigo12();
  inputCodigo.focus();
});

function limpiarInputs() {
  if (inputCodigo) inputCodigo.value = "";
  if (inputCantidad) inputCantidad.value = "";
  if (inputPrecio) inputPrecio.value = "";
  inputCodigo?.focus();
}

function cancelarEdicion() {
  if (filaEnEdicion) {
    filaEnEdicion.classList.remove("editing-row");
    filaEnEdicion = null;
  }
  if (btnAgregar) {
    btnAgregar.innerHTML = "＋ Agregar";
    btnAgregar.classList.remove("btn-updating");
  }
}

function activarEdicion(tr: HTMLTableRowElement) {
  if (filaEnEdicion) {
    filaEnEdicion.classList.remove("editing-row");
  }

  filaEnEdicion = tr;
  tr.classList.add("editing-row");

  const celdas = tr.querySelectorAll("td");
  if (celdas.length >= 3) {
    const codigo = celdas[0].textContent?.trim() || "";
    const cantidad = celdas[1].textContent?.trim() || "";
    const precioConQ = celdas[2].textContent?.trim() || "";
    const precio = precioConQ
      .replace(new RegExp(`^${prefijoMoneda}\\s*`, "i"), "")
      .replace(/^[^\d.]+\s*/, "")
      .trim();

    if (inputCodigo) inputCodigo.value = codigo;
    if (inputCantidad) inputCantidad.value = cantidad;
    if (inputPrecio) inputPrecio.value = precio;

    if (btnAgregar) {
      btnAgregar.innerHTML = `
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 2v6h-6"></path><path d="M3 12a9 9 0 0 1 15-6.7L21 8"></path>
          <path d="M3 22v-6h6"></path><path d="M21 12a9 9 0 0 1-15 6.7L3 16"></path>
        </svg>
        Actualizar
      `;
      btnAgregar.classList.add("btn-updating");
    }

    inputCodigo?.focus();
  }
}

function vincularAccionesFila(tr: HTMLTableRowElement) {
  const btnEliminar = tr.querySelector(".btn-eliminar");
  const btnEditar = tr.querySelector(".btn-editar");

  btnEliminar?.addEventListener("click", () => {
    if (filaEnEdicion === tr) {
      cancelarEdicion();
      limpiarInputs();
    }
    tr.remove();
  });

  btnEditar?.addEventListener("click", () => {
    activarEdicion(tr);
  });
}

// Botón Agregar / Actualizar
btnAgregar?.addEventListener("click", () => {
  if (!inputCodigo || !inputCodigo.value.trim()) return;

  const codigo = inputCodigo.value.trim();
  const cantidad = inputCantidad?.value.trim() || "1";
  const precio = inputPrecio?.value.trim() || "0.00";

  if (filaEnEdicion) {
    // Modo Actualizar
    const celdas = filaEnEdicion.querySelectorAll("td");
    if (celdas.length >= 3) {
      celdas[0].textContent = codigo;
      celdas[1].textContent = cantidad;
      celdas[2].textContent = `${prefijoMoneda} ${precio}`;
    }
    cancelarEdicion();
    limpiarInputs();
  } else if (tablaCuerpo) {
    // Modo Agregar
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td>${codigo}</td>
      <td>${cantidad}</td>
      <td>${prefijoMoneda} ${precio}</td>
      <td>
        <div class="table-actions">
          <button class="btn-table-action btn-eliminar" title="Eliminar" aria-label="Eliminar">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 6h18"></path>
              <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
              <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
              <line x1="10" y1="11" x2="10" y2="17"></line>
              <line x1="14" y1="11" x2="14" y2="17"></line>
            </svg>
          </button>
          <button class="btn-table-action btn-editar" title="Editar" aria-label="Editar">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"></path>
            </svg>
          </button>
        </div>
      </td>
    `;

    vincularAccionesFila(tr);
    tablaCuerpo.appendChild(tr);
    limpiarInputs();
  }
});

// Extraer items de la tabla para generación de PDF o impresión
function obtenerItemsTabla(): ItemCodigo[] {
  const items: ItemCodigo[] = [];
  if (!tablaCuerpo) return items;

  const filas = tablaCuerpo.querySelectorAll("tr");
  filas.forEach((tr) => {
    const celdas = tr.querySelectorAll("td");
    if (celdas.length >= 3) {
      const codigo = celdas[0].textContent?.trim() || "";
      const cantidad = parseInt(celdas[1].textContent?.trim() || "1", 10) || 1;
      const precioConQ = celdas[2].textContent?.trim() || "";
      const precio = precioConQ
        .replace(new RegExp(`^${prefijoMoneda}\\s*`, "i"), "")
        .replace(/^[^\d.]+\s*/, "")
        .trim();

      if (codigo) {
        items.push({ codigo, cantidad, precio });
      }
    }
  });

  return items;
}

// Botón PDF: Guarda archivo mediante diálogo nativo
btnPdf?.addEventListener("click", async () => {
  const items = obtenerItemsTabla();
  if (items.length === 0) {
    alert("Agrega al menos un código a la tabla antes de generar el PDF.");
    return;
  }

  const originalContent = btnPdf.innerHTML;
  btnPdf.disabled = true;
  btnPdf.innerHTML = "Generando...";

  try {
    const mensaje = await invoke<string>("generar_pdf_guardar", { items });
    alert(mensaje);
  } catch (error) {
    const errStr = String(error);
    if (!errStr.includes("cancelada")) {
      alert(`Error al generar el PDF: ${errStr}`);
    }
  } finally {
    btnPdf.disabled = false;
    btnPdf.innerHTML = originalContent;
  }
});

// Botón Imprimir: Genera PDF optimizado y lo envía al visor de impresión del sistema
btnImprimir?.addEventListener("click", async () => {
  const items = obtenerItemsTabla();
  if (items.length === 0) {
    alert("Agrega al menos un código a la tabla antes de imprimir.");
    return;
  }

  const originalContent = btnImprimir.innerHTML;
  btnImprimir.disabled = true;
  btnImprimir.innerHTML = "Preparando...";

  try {
    const mensaje = await invoke<string>("generar_pdf_imprimir", { items });
    console.log(mensaje);
  } catch (error) {
    alert(`Error al enviar a impresión: ${error}`);
  } finally {
    btnImprimir.disabled = false;
    btnImprimir.innerHTML = originalContent;
  }
});

// Funciones del Modal de Configuración
function rellenarFormularioConfig(cfg: AppConfig) {
  if (cfgPagesize) cfgPagesize.value = cfg.pdf.pagesize;
  if (cfgMargenSuperior) cfgMargenSuperior.value = cfg.pdf.margenes.superior_cm.toString();
  if (cfgMargenInferior) cfgMargenInferior.value = cfg.pdf.margenes.inferior_cm.toString();
  if (cfgMargenIzquierdo) cfgMargenIzquierdo.value = cfg.pdf.margenes.izquierdo_cm.toString();
  if (cfgMargenDerecho) cfgMargenDerecho.value = cfg.pdf.margenes.derecho_cm.toString();

  if (cfgBarcodeTipo) cfgBarcodeTipo.value = cfg.codigo_barras.tipo;
  if (cfgBarcodeAncho) cfgBarcodeAncho.value = cfg.codigo_barras.ancho_cm.toString();
  if (cfgBarcodeAlto) cfgBarcodeAlto.value = cfg.codigo_barras.alto_cm.toString();
  if (cfgBarcodeTexto) cfgBarcodeTexto.checked = cfg.codigo_barras.mostrar_texto;

  if (cfgGrillaColumnas) cfgGrillaColumnas.value = cfg.grilla.codigos_por_fila.toString();
  if (cfgGrillaHgap) cfgGrillaHgap.value = cfg.grilla.espacio_horizontal_cm.toString();
  if (cfgGrillaVgap) cfgGrillaVgap.value = cfg.grilla.espacio_vertical_cm.toString();
  if (cfgGrillaPrecioSpace) cfgGrillaPrecioSpace.value = cfg.grilla.espacio_precio_cm.toString();

  if (cfgPrecioFuente) cfgPrecioFuente.value = cfg.precio.fuente;
  if (cfgPrecioTamano) cfgPrecioTamano.value = cfg.precio.tamano.toString();
  if (cfgPrecioPrefijo) cfgPrecioPrefijo.value = cfg.precio.prefijo;
  if (cfgPrecioOffsetY) cfgPrecioOffsetY.value = cfg.precio.offset_y_cm.toString();
}

function leerConfigDeFormulario(): AppConfig {
  return {
    pdf: {
      pagesize: cfgPagesize?.value.trim() || DEFAULT_CONFIG.pdf.pagesize,
      margenes: {
        superior_cm: parseFloat(cfgMargenSuperior?.value || "0") || DEFAULT_CONFIG.pdf.margenes.superior_cm,
        inferior_cm: parseFloat(cfgMargenInferior?.value || "0") || DEFAULT_CONFIG.pdf.margenes.inferior_cm,
        izquierdo_cm: parseFloat(cfgMargenIzquierdo?.value || "0") || DEFAULT_CONFIG.pdf.margenes.izquierdo_cm,
        derecho_cm: parseFloat(cfgMargenDerecho?.value || "0") || DEFAULT_CONFIG.pdf.margenes.derecho_cm,
      },
    },
    codigo_barras: {
      tipo: cfgBarcodeTipo?.value.trim() || DEFAULT_CONFIG.codigo_barras.tipo,
      ancho_cm: parseFloat(cfgBarcodeAncho?.value || "0") || DEFAULT_CONFIG.codigo_barras.ancho_cm,
      alto_cm: parseFloat(cfgBarcodeAlto?.value || "0") || DEFAULT_CONFIG.codigo_barras.alto_cm,
      mostrar_texto: cfgBarcodeTexto ? cfgBarcodeTexto.checked : DEFAULT_CONFIG.codigo_barras.mostrar_texto,
    },
    grilla: {
      codigos_por_fila:
        parseInt(cfgGrillaColumnas?.value || "6", 10) || DEFAULT_CONFIG.grilla.codigos_por_fila,
      espacio_horizontal_cm:
        parseFloat(cfgGrillaHgap?.value || "0") || DEFAULT_CONFIG.grilla.espacio_horizontal_cm,
      espacio_vertical_cm:
        parseFloat(cfgGrillaVgap?.value || "0") || DEFAULT_CONFIG.grilla.espacio_vertical_cm,
      espacio_precio_cm:
        parseFloat(cfgGrillaPrecioSpace?.value || "0") || DEFAULT_CONFIG.grilla.espacio_precio_cm,
    },
    precio: {
      fuente: cfgPrecioFuente?.value.trim() || DEFAULT_CONFIG.precio.fuente,
      tamano: parseFloat(cfgPrecioTamano?.value || "14") || DEFAULT_CONFIG.precio.tamano,
      prefijo: cfgPrecioPrefijo?.value.trim() || DEFAULT_CONFIG.precio.prefijo,
      offset_y_cm: parseFloat(cfgPrecioOffsetY?.value || "0") || DEFAULT_CONFIG.precio.offset_y_cm,
    },
  };
}

async function abrirModalAjustes() {
  if (!modalAjustes) return;
  try {
    const configActual = await cargarConfiguracion();
    prefijoMoneda = configActual.precio.prefijo || "Q";
    rellenarFormularioConfig(configActual);
  } catch (err) {
    console.error("Error al cargar configuración desde el backend:", err);
    rellenarFormularioConfig(DEFAULT_CONFIG);
  }
  modalAjustes.classList.add("is-open");
  modalAjustes.setAttribute("aria-hidden", "false");
}

function cerrarModalAjustes() {
  if (!modalAjustes) return;
  modalAjustes.classList.remove("is-open");
  modalAjustes.setAttribute("aria-hidden", "true");
}

// Event Listeners del Modal de Ajustes
btnAjustes?.addEventListener("click", abrirModalAjustes);
btnCerrarAjustes?.addEventListener("click", cerrarModalAjustes);
btnCancelarAjustes?.addEventListener("click", cerrarModalAjustes);

// Cerrar al hacer clic en el backdrop fuera de la tarjeta
modalAjustes?.addEventListener("click", (e) => {
  if (e.target === modalAjustes) {
    cerrarModalAjustes();
  }
});

// Restablecer valores a los por defecto
btnResetAjustes?.addEventListener("click", () => {
  rellenarFormularioConfig(DEFAULT_CONFIG);
});

// Guardar configuración en el sistema operativo mediante el backend de Tauri
btnGuardarAjustes?.addEventListener("click", async () => {
  if (!btnGuardarAjustes) return;
  const originalHtml = btnGuardarAjustes.innerHTML;
  btnGuardarAjustes.disabled = true;
  btnGuardarAjustes.innerHTML = "Guardando...";

  try {
    const nuevaConfig = leerConfigDeFormulario();
    const respuesta = await guardarConfiguracion(nuevaConfig);
    prefijoMoneda = nuevaConfig.precio.prefijo || "Q";
    cerrarModalAjustes();
    alert(respuesta);
  } catch (error) {
    alert(`Error al guardar la configuración: ${error}`);
  } finally {
    btnGuardarAjustes.disabled = false;
    btnGuardarAjustes.innerHTML = originalHtml;
  }
});

// Permitir presionar Enter en cualquiera de los inputs principales para agregar o actualizar
[inputCodigo, inputCantidad, inputPrecio].forEach((input) => {
  input?.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      btnAgregar?.click();
    }
  });
});

// Cancelar edición o cerrar modal con la tecla Escape
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    if (modalAjustes?.classList.contains("is-open")) {
      cerrarModalAjustes();
      return;
    }
    if (filaEnEdicion) {
      cancelarEdicion();
      limpiarInputs();
    }
  }
});

// Cargar configuración inicial al arrancar la aplicación
cargarConfiguracion()
  .then((cfg) => {
    prefijoMoneda = cfg.precio.prefijo || "Q";
  })
  .catch((err) => {
    console.warn("No se pudo cargar la configuración al inicio, usando por defecto:", err);
  });
