const inputCodigo = document.querySelector<HTMLInputElement>("#input-codigo");
const inputCantidad = document.querySelector<HTMLInputElement>("#input-cantidad");
const inputPrecio = document.querySelector<HTMLInputElement>("#input-precio");
const btnAgregar = document.querySelector<HTMLButtonElement>("#btn-agregar");
const tablaCuerpo = document.querySelector<HTMLTableSectionElement>("#tabla-cuerpo");

btnAgregar?.addEventListener("click", () => {
  if (!inputCodigo || !inputCodigo.value.trim()) return;

  const codigo = inputCodigo.value.trim();
  const cantidad = inputCantidad?.value.trim() || "1";
  const precio = inputPrecio?.value.trim() || "0.00";

  if (tablaCuerpo) {
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td>${codigo}</td>
      <td>${cantidad}</td>
      <td>Q ${precio}</td>
      <td><button class="btn-eliminar" style="background:transparent; border:none; color:#f85149; cursor:pointer; font-weight:600;">Eliminar</button></td>
    `;

    // Botón para eliminar la fila al hacer clic
    tr.querySelector(".btn-eliminar")?.addEventListener("click", () => {
      tr.remove();
    });

    tablaCuerpo.appendChild(tr);

    // Limpiar inputs
    inputCodigo.value = "";
    if (inputCantidad) inputCantidad.value = "";
    if (inputPrecio) inputPrecio.value = "";
    inputCodigo.focus();
  }
});