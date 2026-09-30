# Git-Client (rmerge)

[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](Cargo.toml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)](#-características-principales)

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

### 🔐 Clonación y Almacén Seguro Centralizado de Credenciales

* **Detección Dinámica de URLs HTTPS:** Al ingresar URLs remotas HTTPS (GitHub, GitLab, Bitbucket, Gitea u organizaciones privadas), el asistente despliega los campos de autenticación (Usuario y Personal Access Token / PAT) con campo de contraseña enmascarada.
* **Bóveda Cifrada Centralizada de Credenciales:**
  * Almacenamiento seguro en disco persistente (`~/.config/rmerge/credentials.enc` en Linux o la ruta nativa de configuración del SO).
  * Cifrado de grado bancario **AES-256-GCM** autenticado.
  * Derivación de clave mediante **PBKDF2-HMAC-SHA256** con 100,000 rondas, utilizando sal aleatoria criptográfica por guardado y semilla ligada a la máquina y sesión del usuario.
  * Restricción estricta de permisos de archivo en Unix a nivel de usuario (`chmod 600`).
* **Autocompletado y Reutilización Transparente:** Las credenciales almacenadas para un host (ej. `github.com`) son reconocidas inmediatamente en futuras clonaciones u operaciones de red, evitando el reingreso continuo con cada interacción con el repositorio.

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
  * `📊`: Botón con tooltip interactivo (*"Ocultar Grafo"* / *"Mostrar Grafo"* según estado), ubicado inmediatamente a la derecha del nombre de la rama para alternar el panel del grafo lateral manteniendo la barra despejada.
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
* **Barra de Estado y Gestor Interactivo de Mensajes:**
  * Visualización en tiempo real del último mensaje de estado, advertencia o error.
  * Cuando hay múltiples mensajes en cola, se contrae a un indicador sintético `Mensajes (#)`.
  * **Interacción por Doble Clic:** Al hacer doble clic sobre la barra de estado se despliega un diálogo modal enriquecido con el historial cronológico completo de mensajes. Cada mensaje cuenta con su propio botón de cierre individual (`✕`) para depurar notificaciones resueltas.
* **Seguridad de Red (Pre-flight Check):** Comprobación no bloqueante de conectividad hacia el host remoto antes de comandos de red (`push`, `pull`, `fetch`), desplegando una alerta visual en caso de servidor no alcanzable (ej. requerimiento de VPN) evitando congelamientos.

---

## 🛠️ Instalación y Compilación según el Sistema Operativo

Puedes instalar **Git-Client** descargando los paquetes precompilados oficiales desde la sección de [Releases de GitHub](https://github.com/AnibalGH/git-client/releases) o compilando el código fuente localmente.

---

### 📦 1. Instalación mediante Paquetes Precompilados (Releases)

#### 🐧 Linux

Git-Client ofrece paquetes listos para las principales distribuciones:

* **Debian / Ubuntu / Linux Mint (`.deb`):**

  ```bash
  # Descargar la versión deseada e instalar con dpkg o apt:
  sudo dpkg -i git-client_1.0.0_amd64.deb
  # Si faltan dependencias del sistema:
  sudo apt-get install -f
  ```

  *Incluye lanzador de menú, iconos en todas las resoluciones y comandos de terminal.*

* **Fedora / RHEL / CentOS / Rocky Linux / openSUSE (`.rpm`):**

  ```bash
  sudo dnf install ./git-client-1.0.0-1.x86_64.rpm
  # o con rpm:
  sudo rpm -ivh git-client-1.0.0-1.x86_64.rpm
  ```

* **AppImage (Portable para cualquier distribución Linux):**

  ```bash
  # 1. Dar permisos de ejecución:
  chmod +x git-client-1.0.0-x86_64.AppImage

  # 2. Ejecutar directamente:
  ./git-client-1.0.0-x86_64.AppImage
  ```

* **Tarball Genérico (`.tar.gz`):**

  ```bash
  tar -xzvf git-client-1.0.0-linux-x86_64.tar.gz
  cd git-client-1.0.0-linux-x86_64
  # Mover los binarios a un directorio en el PATH del sistema:
  sudo cp rmerge rmerge-gui /usr/local/bin/
  ```

#### 🪟 Windows

1. Descarga el archivo comprimido **`git-client-1.0.0-windows-x86_64.zip`** desde GitHub Releases.
2. Descomprime la carpeta en una ubicación permanente (por ejemplo: `C:\Program Files\Git-Client` o `%USERPROFILE%\bin`).
3. Agrega la carpeta a la variable de entorno `PATH` del sistema si deseas utilizar el comando `rmerge` desde la terminal PowerShell o CMD.
4. Puedes enviar un acceso directo de `rmerge-gui.exe` al Escritorio o al menú Inicio. El ejecutable tiene el ícono nativo multi-resolución incrustado y se ejecuta sin abrir ventanas de consola secundarias.

#### 🍏 macOS

1. Descarga el paquete correspondiente a la arquitectura de tu Mac:
   * **Apple Silicon (M1/M2/M3/M4):** `git-client-1.0.0-aarch64-apple-darwin.tar.gz`
   * **Intel:** `git-client-1.0.0-x86_64-apple-darwin.tar.gz`
2. Descomprime el archivo `.tar.gz`:

   ```bash
   tar -xzvf git-client-1.0.0-aarch64-apple-darwin.tar.gz
   ```

3. Mueve la aplicación **`Git-Client.app`** a tu carpeta `/Applications`:

   ```bash
   mv Git-Client.app /Applications/
   ```

4. Opcionalmente, para acceder al binario CLI `rmerge` en tu terminal Zsh / Bash:

   ```bash
   sudo ln -sf /Applications/Git-Client.app/Contents/MacOS/rmerge /usr/local/bin/rmerge
   ```

---

### ⚙️ 2. Compilar desde el Código Fuente

Si prefieres construir la aplicación desde las fuentes:

#### Prerrequisitos de Compilación

* **Rust & Cargo** (versión 1.80 o superior): [https://rustup.rs/](https://rustup.rs/)
* Librerías del sistema:

  ```bash
  # En Ubuntu / Debian:
  sudo apt-get install build-essential cmake pkg-config libssl-dev libfontconfig1-dev

  # En Fedora / RHEL:
  sudo dnf install gcc cmake pkg-config openssl-devel fontconfig-devel

  # En Arch Linux:
  sudo pacman -S base-devel cmake fontconfig

  # En macOS (con Homebrew):
  brew install cmake pkg-config
  ```

#### Pasos de Construcción

```bash
# 1. Clonar el repositorio
git clone https://github.com/AnibalGH/git-client.git
cd git-client

# 2. Compilar en modo Release optimizado
cargo build --release --workspace

# 3. Ejecutar las pruebas unitarias y de integración
cargo test --workspace
```

Los binarios optimizados se generan en `target/release/`:

* `target/release/rmerge`: Binario CLI y cliente IPC.
* `target/release/rmerge-gui`: Interfaz gráfica nativa.

#### Instalación Local Rápida en Linux (Script Automatizado)

El repositorio incluye un script instalador para Linux que compila en Release, copia los binarios a `~/.local/bin`, registra los iconos del sistema en resoluciones de 16x16 a 512x512 y configura el archivo `.desktop`:

```bash
./install.sh
```

---

## 🏷️ Generación de Versiones, Publicación de Tags y Notas de Release

El proyecto cuenta con un flujo de integración continua en [`.github/workflows/release.yml`](.github/workflows/release.yml) que automatiza la compilación, empaquetado y publicación de releases al detectar nuevos tags.

### 1. Instrucciones para Crear y Publicar un Tag

Para publicar una nueva versión oficial del aplicativo:

1. **Asegúrate de tener la rama principal actualizada y limpia:**

   ```bash
   git checkout main
   git pull origin main
   cargo test --workspace
   ```

2. **Crear una etiqueta anotada con la versión (formato `vX.Y.Z`):**

   ```bash
   # Crear el tag con un mensaje resumen:
   git tag -a v1.0.0 -m "Release v1.0.0: Soporte HTTPS, Bóveda Segura de Credenciales y Grafo de Ramas"
   ```

3. **Publicar el tag hacia el repositorio remoto:**

   ```bash
   git push origin v1.0.0
   ```

En ese momento, GitHub Actions iniciará automáticamente los jobs paralelos para compilar en Linux, Windows y macOS, empaquetará los instaladores (`.deb`, `.rpm`, `.AppImage`, `.zip`, `.tar.gz`) y creará el Release oficial adjuntando los archivos.

---

### 2. Cómo Generar el Detalle y Notas del Release (Changelog)

> Para consultar el historial completo de cambios y artefactos publicados de cada versión, revisa el archivo [CHANGELOG.md](CHANGELOG.md).

Para acompañar el Release con un detalle profesional de los cambios realizados entre versiones:

#### Opción A: Mediante la Interfaz Web de GitHub (Automático)

1. En GitHub, navega a la sección **Releases** (`https://github.com/AnibalGH/git-client/releases`).
2. Una vez que el workflow del tag haya concluido, haz clic en **Edit release** (icono de lápiz) en el release creado.
3. Haz clic en el botón **"Generate release notes"**. GitHub generará automáticamente la lista de Pull Requests, commits y autores que contribuyeron desde el último tag.
4. Revisa, organiza las secciones (Novedades, Correcciones, Mejoras de Seguridad) y presiona **Update release**.

#### Opción B: Generar el Detalle desde la Terminal con Git

Puedes generar el registro de cambios detallado entre dos versiones antes o después de crear el tag:

```bash
# Ver lista concisa de commits entre el tag anterior (ej: v0.9.0) y el actual (v1.0.0):
git log v0.9.0..v1.0.0 --oneline --no-merges

# Generar un resumen agrupado por autor:
git shortlog v0.9.0..v1.0.0 -sne

# Ver los archivos y módulos modificados:
git diff --stat v0.9.0..v1.0.0
```

#### Opción C: Mediante GitHub CLI (`gh`)

Si tienes instalada la herramienta oficial [GitHub CLI (`gh`)](https://cli.github.com/):

```bash
# Crear el release y generar automáticamente las notas a partir de los commits:
gh release create v1.0.0 --generate-notes --title "Git-Client v1.0.0"
```

#### Plantilla Estándar Recomendada para el Detalle del Release

```markdown
# 🚀 Git-Client v1.0.0

## ✨ Novedades
* **Bóveda Centralizada de Credenciales:** Soporte para HTTPS (GitHub, GitLab, Bitbucket) con cifrado AES-256-GCM y PBKDF2.
* **Historial y Diálogo de Mensajes:** Doble-clic en la barra de estado para ver la cola de eventos y cerrarlos de manera individual.
* **Visualizador de Grafo Simplificado:** Botón del panel lateral optimizado con icono representativo y tooltips.

## 🛠️ Mejoras y Correcciones
* Optimización de rendimiento en el analizador de ramas y diferencias de diffs.
* Corrección de advertencias del linter Clippy y limpieza de dependencias.
* Suite completa de pruebas unitarias y de integración al 100%.

## 📦 Paquetes y Binarios Disponibles
* **Linux:** `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), `.AppImage` (Universal), `.tar.gz`.
* **Windows:** `.zip` portátil con ejecutables para x86_64.
* **macOS:** `.tar.gz` para Apple Silicon (`aarch64`) e Intel (`x86_64`) con el bundle `Git-Client.app`.
```

---

## 🏛️ Estructura del Proyecto y Arquitectura

El proyecto sigue rigurosamente los principios de **Clean Architecture** y **Arquitectura Hexagonal (Puertos y Adaptadores)**:

```text
git-client/
├── Cargo.toml                    # Configuración del espacio de trabajo (Workspace)
├── AGENT.md                      # Contexto integral y reglas de arquitectura para agentes de IA
├── README.md                     # Documentación general para usuarios y desarrolladores
├── CHANGELOG.md                  # Registro histórico de versiones y notas de release
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
