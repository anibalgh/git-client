# Git-Client (rmerge)

[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](Cargo.toml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)](#multiplataforma)

**Git-Client** (distribuido bajo el binario CLI `rmerge` y GUI `rmerge-gui`) es un cliente Git y herramienta gráfica de resolución **3-Way Merge** de alto rendimiento, nativo y multiplataforma, construido íntegramente en **Rust** sobre [`egui`](https://github.com/emilk/egui) y [`libgit2`](https://libgit2.org/).

Diseñado para reemplazar clientes pesados basados en Electron o navegadores web, ofreciendo un consumo de memoria mínimo (< 35 MB), arranque instantáneo, soporte integral de atajos de teclado y una arquitectura limpia modular.

---

## 🚀 Características Principales

### ⚡ Rendimiento y Arquitectura Nativa
* **Arquitectura Limpia / Hexagonal:** Separación estricta en 5 crates (`rmerge-domain`, `rmerge-application`, `rmerge-infrastructure`, `rmerge-gui` y `rmerge-cli`).
* **Instancia Única con IPC:** Lanzar `rmerge .` desde cualquier terminal enfoca o actualiza la ventana gráfica existente al instante mediante sockets locales (Unix Domain Sockets en Linux/macOS y Named Pipes en Windows).
* **100% Multiplataforma:** Compatibilidad garantizada en Linux, Windows y macOS.

### 🌿 Explorador de Git y Ramas
* **Lista de Ramas:** Visualización clara de ramas locales, cabecera activa (`HEAD`), creación rápida de ramas (`+`) y cambio de rama (*checkout*).
* **Menú de la Rama:** Menú contextual para merge de ramas, cambio de rama, checkout de commits y creación de Pull Requests / Merge Requests en el navegador con un clic (GitHub, GitLab, Bitbucket, Gitea).
* **Gestión de Tags:** Listado de etiquetas, creación de tags ligeros o anotados (`+`) y eliminación de tags.
* **Operaciones Remotas Avanzadas:** Menús desplegables dedicados para operaciones de:
  * **Push:** `push`, `push --force`, `push --force-with-lease`, `push --no-verify`.
  * **Pull & Fetch:** `pull`, `pull --ff-only`, `pull --rebase`, `pull --rebase --autostash`, `fetch`, `fetch --tags`, `fetch --prune`.

### 📦 Preparación (Staging) y Realización de Commits
* **Staging Selectivo:** Botones dedicados `+` (Stage) y `-` (Unstage) por cada archivo.
* **Acciones Masivas:** Botón `++` para preparar todos los modificados y `--` para retirar todos del stage.
* **Gestión Inteligente del Commit:** La sección de mensaje y confirmación de commit solo se habilita dinámicamente cuando existen cambios preparados. Atajo `Ctrl+Enter` para confirmar.
* **Validación de Identidad:** Verificación proactiva de `user.name` y `user.email`. Si no están configurados, despliega un diálogo intuitivo para asignarlos a nivel local o global.
* **Editor de `.gitignore` Integrado:** Edición directa del archivo `.gitignore` con recarga inmediata respetando los patrones de exclusión de Git.

### 📊 Historial y Grafo Visual de Cambios
* **Grafo de Bifurcaciones y Uniones:** Panel lateral derecho con renderizado continuo mediante curvas de Bézier cúbicas, diferenciando carriles de ramas, bifurcaciones (*forks*) y uniones (*merges* con nodos romboidales).
* **Insignias y Tooltips:** Etiquetas de ramas y tags en cada punto del historial con información detallada de autor y mensaje al posar el cursor.

### 🔍 Inspector Detallado de Commits
* **Metadatos Completos:** Hash completo con botón de copia vertical (`📋`), Tree ID, Autor con correo, Fecha formateada, Commits padres y Ramas asociadas.
* **Estadísticas de Cambios:** Contador de archivos modificados, adiciones (`+`) y eliminaciones (`-`) con barra de proporción visual según el tema activo.
* **Menú Desplegable (ComboBox) de Archivos:** Selección rápida de cualquier archivo modificado en el commit.
* **Visor de Diferencias (Diff):** Resaltado sintáctico con colores de fondo para adiciones y eliminaciones.

### 🔀 Herramienta 3-Way Merge Nativa
* Integración transparente con `git mergetool`.
* Resolución de conflictos de tres vías visualizando versión Local (*Ours*), Base y Remota (*Theirs*), con previsualización del archivo resultante y guardado automático.

### 🎨 Personalización y Apariencia
* **17 Temas Visuales:** Sublime Dark/Light, Monokai Pro, Kanagawa (Wave, Dragon, Lotus), Gruvbox (Dark, Light), Dracula, Nord, Tokyo Night, Catppuccin (Mocha, Latte), One Dark Pro, Rose Pine y Solarized (Dark, Light).
* **Distribución en 3 Columnas:** Selector de temas organizado y ergonómico.
* **Tipografías del Sistema Operativo:** Detección automática de fuentes instaladas en el sistema (Windows, Linux, macOS):
  * Selección independiente de **Tipografía de Interfaz (GUI)** con control de tamaño.
  * Selección independiente de **Tipografía de Código (Diffs / Monospace)** con control de tamaño y ligaduras tipográficas (`->`, `!=`, `===`).
* **Internacionalización (i18n):** Soporte completo para **Español** e **Inglés**, configurable desde la ventana de Preferencias con persistencia automática en `settings.json`.

---

## 💻 Uso desde la Línea de Comandos (CLI)

El ejecutable `rmerge` ofrece una interfaz de terminal completa y veloz:

```bash
# Abrir el repositorio actual en Git-Client
rmerge .

# Abrir una ruta específica
rmerge /ruta/al/proyecto

# Configurar identidad de Git (Local o Global)
rmerge config-identity --name "Tu Nombre" --email "tu@correo.com"
rmerge config-identity --name "Tu Nombre" --email "tu@correo.com" --global

# Inspeccionar autor activo configurado
rmerge config-identity

# Ver historial / culpa (blame) de un archivo
rmerge blame src/main.rs --line 42

# Búsqueda en el historial de commits
rmerge search "refactor login"

# Ver versión del aplicativo
rmerge --version
```

### Configurar como herramienta oficial de Merge en Git

Puedes configurar `rmerge` como tu herramienta estándar de resolución de conflictos para `git mergetool`:

```bash
git config --global merge.tool rmerge
git config --global mergetool.rmerge.cmd 'rmerge mergetool "$BASE" "$LOCAL" "$REMOTE" -o "$MERGED"'
git config --global mergetool.rmerge.trustExitCode true
```

Cuando ocurra un conflicto durante un `git merge`, simplemente ejecuta:
```bash
git mergetool
```
Y se abrirá la vista gráfica de 3-Way Merge de Git-Client para resolverlo de forma visual.

---

## 🖥️ Uso de la Interfaz Gráfica (GUI)

* **Inicio sin repositorio:** Al abrir la aplicación en un directorio que no sea un repositorio Git, se muestra la pantalla de bienvenida con la lista de **Repositorios Recientemente Abiertos**, con botones para abrir una carpeta local, clonar un repositorio remoto (`git clone`) o limpiar el historial.
* **Barra Superior (Header):**
  * `📁 Ruta del Repositorio | 🌿 Rama Activa`: Indicador en tiempo real de la ruta y cabecera actual.
  * `📊 Grafo Rama`: Botón ubicado inmediatamente a la derecha del nombre de la rama para alternar el panel del grafo lateral.
  * `☰ Menú Hamburguesa`: Menú unificado en el extremo derecho que agrupa:
    * `🏠 Inicio`: Regresa a la vista de bienvenida y repositorios recientes.
    * `🔄 Refrescar`: Recarga el estado completo de Git en memoria.
    * `📂 Abrir Repo...`: Diálogo nativo para seleccionar cualquier repositorio del disco.
    * `📥 Clonar...`: Diálogo modal para clonar repositorios por URL (HTTPS/SSH).
    * `⚙ Preferencias`: Configuración de temas en 3 columnas, fuentes de interfaz y código con sliders y ligaduras, e idioma.
    * `ℹ Acerca de`: Resumen del aplicativo, versión activa (1.0.0), licencia y arquitectura.
* **Panel Izquierdo (Explorador Git):**
  * `⚡ Menú de la Rama ▾`: Acceso a creación de ramas, cambio de rama, merge, tags y editor `.gitignore`.
  * `🌿 Ramas`: Botón `+` para crear una nueva rama. Botones `⬆▾` (Push) y `⬇▾` (Pull/Fetch) en la rama activa.
  * `🏷 Tags`: Botón `+` para crear un nuevo tag anotado o ligero.
  * `✔ Preparados`: Archivos listos para el commit con botón `-` individual y `--` para todos.
  * `● Modificados`: Archivos modificados sin preparar con botón `+` individual y `++` para todos.
  * **Sección de Commit Inteligente:** El cuadro de texto multilínea (con tipografía adaptada a la del GUI) y el botón `✔ Realizar Commit` solo se muestran y habilitan cuando existen archivos preparados en el stage.
* **Panel Central:**
  * **Historial de Commits:** Tabla con grafo lineal, hash con color acento, mensaje con tipografía y tamaño de la interfaz, autor y fecha.
  * **Detalle del Commit:** Metadatos completos (Hash con botón vertical `📋`, Tree ID, Autor, Fecha, Padres, Ramas), estadísticas de inserciones y eliminaciones con barra gráfica proporcional, selector (ComboBox) de archivos modificados y visor de diffs sintáctico con tipografía de código.
* **Seguridad de Red (Pre-flight Check):** Comprobación no bloqueante de conectividad hacia el host remoto antes de comandos de red (`push`, `pull`, `fetch`), desplegando una alerta visual en caso de servidor no alcanzable (ej. requerimiento de VPN) evitando congelamientos.

---

## 🛠️ Compilación e Instalación

### Prerrequisitos
* **Rust & Cargo** (versión 1.80 o superior): [https://rustup.rs/](https://rustup.rs/)
* Librerías del sistema para compilar dependencias C (libgit2, fontconfig en Linux):
  ```bash
  # En Ubuntu / Debian:
  sudo apt-get install build-essential cmake pkg-config libssl-dev libfontconfig1-dev

  # En Fedora / RHEL:
  sudo dnf install gcc cmake pkg-config openssl-devel fontconfig-devel

  # En Arch Linux:
  sudo pacman -S base-devel cmake fontconfig
  ```

### Compilar desde el código fuente

```bash
# Clonar el proyecto
git clone https://github.com/AnibalGH/git-client.git
cd git-client

# Compilar en modo Release (optimizado con LTO)
cargo build --release --workspace

# Ejecutar las pruebas unitarias y de integración
cargo test --workspace
```

Los binarios optimizados se generarán en `target/release/`:
* `target/release/rmerge`: Ejecutable CLI principal y despachador IPC.
* `target/release/rmerge-gui`: Interfaz gráfica nativa.

### Instalar en el sistema (Linux)

```bash
# Copiar binarios al PATH
sudo cp target/release/rmerge /usr/local/bin/
sudo cp target/release/rmerge-gui /usr/local/bin/

# Instalar lanzador de escritorio
mkdir -p ~/.local/share/applications
cp assets/rmerge.desktop ~/.local/share/applications/
```

---

## 🏛️ Estructura del Proyecto y Arquitectura

El proyecto sigue rigurosamente los principios de **Clean Architecture** y **Arquitectura Hexagonal (Puertos y Adaptadores)**:

```text
git-client/
├── Cargo.toml                    # Configuración del espacio de trabajo (Workspace)
├── AGENT.md                      # Contexto integral y reglas de arquitectura para agentes de IA
├── README.md                     # Documentación general para usuarios y desarrolladores
├── LICENSE                       # Licencia de código abierto MIT / Apache-2.0
├── assets/                       # Iconos y archivo .desktop
├── locales/                      # Archivos de idioma (es.json, en.json)
├── .agents/skills/               # Skills de desarrollo para agentes IA
│   ├── clean-architecture/       # Directrices de Clean Architecture y regla de dependencias
│   └── hexagonal-architecture/   # Patrón de Puertos y Adaptadores e inversión de control
└── crates/
    ├── rmerge-domain/            # Entidades puras, reglas de negocio y algoritmos (3-Way Merge, Graph)
    ├── rmerge-application/       # Casos de uso, servicios y puertos de entrada/salida
    ├── rmerge-infrastructure/    # Adaptadores para Git2, FontKit, Temas, Ajustes IPC y Disco
    ├── rmerge-gui/               # Interfaz gráfica de usuario con egui / eframe
    └── rmerge-cli/               # CLI con clap, comandos de terminal y cliente IPC
```

Para una descripción detallada de las reglas de diseño, separación de responsabilidades y convenciones para futuras sesiones o contribuciones, consulta el archivo [AGENT.md](AGENT.md).

---

## 📄 Licencia

Este proyecto es software de código abierto bajo la licencia **MIT** (o **Apache 2.0** a elección del usuario).
Eres libre de utilizarlo, modificarlo, estudiarlo y distribuirlo libremente tanto para fines personales como comerciales.

Para más detalles, consulta el archivo [LICENSE](LICENSE).

---

## 👤 Autor

* **AnibalGH** - Creador y mantenedor principal.
