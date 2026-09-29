#!/usr/bin/env bash
set -e

# ==============================================================================
# Script de Instalación y Registro de Git-Client (rmerge / rmerge-gui) en Linux
# ==============================================================================

INSTALL_PREFIX="${HOME}/.local"
BIN_DIR="${INSTALL_PREFIX}/bin"
APP_DIR="${INSTALL_PREFIX}/share/applications"
ICON_DIR="${INSTALL_PREFIX}/share/icons/hicolor"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> Compilando Git-Client en modo release..."
cargo build --release --workspace

echo "==> Creando directorios de destino..."
mkdir -p "${BIN_DIR}"
mkdir -p "${APP_DIR}"

echo "==> Instalando binarios en ${BIN_DIR}..."
cp -f "${PROJECT_DIR}/target/release/rmerge" "${BIN_DIR}/rmerge"
cp -f "${PROJECT_DIR}/target/release/rmerge-gui" "${BIN_DIR}/rmerge-gui"
chmod +x "${BIN_DIR}/rmerge" "${BIN_DIR}/rmerge-gui"

echo "==> Instalando iconos del sistema en ${ICON_DIR}..."
for size in 16 24 32 48 64 128 256 512; do
  target_dir="${ICON_DIR}/${size}x${size}/apps"
  mkdir -p "${target_dir}"
  cp -f "${PROJECT_DIR}/assets/icons/rmerge_${size}x${size}.png" "${target_dir}/rmerge.png"
done

# Copiar el SVG escalable si existe soporte
mkdir -p "${ICON_DIR}/scalable/apps"
cp -f "${PROJECT_DIR}/assets/icons/rmerge-app-icon.svg" "${ICON_DIR}/scalable/apps/rmerge.svg"

echo "==> Actualizando caché de iconos de GTK/escritorio..."
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${ICON_DIR}" 2>/dev/null || true
fi

echo "==> Registrando lanzador de escritorio (.desktop)..."
cat << DESKTOP_EOF > "${APP_DIR}/rmerge.desktop"
[Desktop Entry]
Name=Git-Client
GenericName=Git Client
Comment=Git-Client: Cliente Git de alto rendimiento desarrollado en Rust
Exec=${BIN_DIR}/rmerge %F
Icon=rmerge
Terminal=false
Type=Application
Categories=Development;RevisionControl;Git;
MimeType=inode/directory;
Keywords=git;merge;diff;commit;client;rmerge;
StartupWMClass=rmerge-gui
Actions=OpenCurrent;

[Desktop Action OpenCurrent]
Name=Abrir Directorio Actual
Exec=${BIN_DIR}/rmerge .
DESKTOP_EOF

chmod 644 "${APP_DIR}/rmerge.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${APP_DIR}" 2>/dev/null || true
fi

echo "=============================================================================="
echo " ✅ Instalación completada con éxito."
echo " - Binarios: ${BIN_DIR}/rmerge y ${BIN_DIR}/rmerge-gui"
echo " - Entrada de escritorio: ${APP_DIR}/rmerge.desktop"
echo " - Iconos registrados en tema hicolor (16px a 512px + SVG escalable)"
echo " Asegúrate de que ${BIN_DIR} esté en tu PATH para ejecutar 'rmerge' directamente."
echo "=============================================================================="
