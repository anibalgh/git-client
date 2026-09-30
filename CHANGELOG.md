# Changelog & Notas de Release

Todos los cambios notables, lanzamientos y correcciones del proyecto **Git-Client (rmerge)** están documentados en este archivo. El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y este proyecto se adhiere a [Semantic Versioning](https://semver.org/lang/es/).

---

## [v1.0.0] - 2026-09-29

### 🚀 Novedades y Características Principales

* **Bóveda Centralizada y Cifrada de Credenciales HTTPS:**
  * Soporte completo para repositorios remotos sobre HTTPS (GitHub, GitLab, Bitbucket, Gitea y servidores empresariales).
  * Almacenamiento seguro y persistente de tokens de acceso personal (PAT) y contraseñas en `~/.config/rmerge/credentials.enc`.
  * Cifrado de grado militar autenticado **AES-256-GCM**.
  * Derivación de clave mediante **PBKDF2-HMAC-SHA256** con 100,000 iteraciones y sal criptográfica aleatoria, enlazada a la sesión y máquina del usuario.
  * Restricción estricta de permisos de lectura y escritura en sistemas Unix (`chmod 600`).
  * Autocompletado transparente de credenciales por nombre de host en operaciones subsiguientes de red (`clone`, `fetch`, `pull`, `push`).

* **Gestor Interactivo de Mensajes y Notificaciones:**
  * Barra de estado inferior responsiva con visualización del último evento y colapso inteligente a `Mensajes (#)` cuando existen múltiples notificaciones acumuladas.
  * Diálogo modal enriquecido activable mediante **doble clic** sobre la barra de estado, con visualización cronológica en formato de lista.
  * Capacidad de descarte individual de mensajes mediante botón `✕`.

* **Visualizador y Grafo de Ramas Optimizado:**
  * Botón en el encabezado superior simplificado a icono puro `📊` con tooltips contextuales dinámicos (*"Ocultar Grafo"* / *"Mostrar Grafo"*), ahorrando espacio horizontal.
  * Renderizado mediante curvas de Bézier cúbicas con detección visual de carriles paralelos, bifurcaciones (*forks*) y uniones (*merges* con nodos romboidales).

* **Resiliencia de Conexión de Red (Pre-flight Check):**
  * Verificación preliminar no bloqueante de disponibilidad del host remoto vía TCP antes de emitir operaciones de red (`push`, `pull`, `fetch`).
  * Modal visual informativo en el idioma activo cuando el servidor no es alcanzable (ej. requerimiento de VPN corporativa o corte de red), evitando congelamientos en la interfaz.

* **Soporte Multiplataforma y Distribución Automatizada:**
  * Pipeline completo de CI/CD en GitHub Actions para compilación nativa en Linux, Windows y macOS.
  * Inclusión de características `vendored-openssl` y `vendored-libgit2` para garantizar portabilidad y compilación cruzada limpia tanto en arquitecturas Intel (`x86_64`) como Apple Silicon (`aarch64`).

---

### 📦 Paquetes y Artefactos Publicados en el Release

| Plataforma | Arquitectura | Formato de Paquete | Nombre del Archivo |
| :--- | :--- | :--- | :--- |
| **Linux (Debian / Ubuntu / Mint)** | `x86_64` | `.deb` | `git-client_1.0.0_amd64.deb` |
| **Linux (Fedora / RHEL / CentOS)** | `x86_64` | `.rpm` | `git-client-1.0.0-1.x86_64.rpm` |
| **Linux (Universal Portable)** | `x86_64` | `.AppImage` | `git-client-1.0.0-x86_64.AppImage` |
| **Linux (Genérico)** | `x86_64` | `.tar.gz` | `git-client-1.0.0-linux-x86_64.tar.gz` |
| **Windows** | `x86_64` | `.zip` | `git-client-1.0.0-windows-x86_64.zip` |
| **macOS (Apple Silicon M1/M2/M3/M4)** | `aarch64` | `.tar.gz` (App Bundle) | `git-client-1.0.0-aarch64-apple-darwin.tar.gz` |
| **macOS (Intel Core)** | `x86_64` | `.tar.gz` (App Bundle) | `git-client-1.0.0-x86_64-apple-darwin.tar.gz` |

---

### 🛠️ Mejoras Internas y Correcciones

* Eliminación del 100% de advertencias de Clippy y optimización de advertencias de claves duplicadas en componentes de UI.
* Refactorización y cumplimiento riguroso de principios de Clean Architecture y Arquitectura Hexagonal en los 5 crates modulares.
* Suite completa de pruebas unitarias y de integración en verde (`cargo test --workspace`).
