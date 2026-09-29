# AGENT.md - Contexto y Guía de Arquitectura para Sesiones de IA

Este archivo contiene el contexto integral, las directrices arquitectónicas y las reglas de diseño del proyecto **Git-Client (rmerge)**. Cualquier sesión futura de un agente de IA debe leer y respetar rigurosamente este documento para mantener la coherencia y calidad del código.

---

## 1. Identidad y Propósito del Proyecto

* **Nombre:** Git-Client
* **Binarios:**
  * `rmerge` (CLI y cliente/servidor IPC)
  * `rmerge-gui` (Interfaz Gráfica de Usuario nativa)
* **Versión:** `1.0.0`
* **Licencia:** MIT / Apache-2.0
* **Stack Tecnológico:**
  * **Lenguaje:** Rust (Edición 2021, MSRV 1.80+)
  * **GUI:** [`egui`](https://github.com/emilk/egui) / [`eframe`](https://github.com/emilk/egui/tree/master/crates/eframe) (v0.31)
  * **Motor Git:** [`git2`](https://docs.rs/git2) (bindings nativos de `libgit2`)
  * **Tipografías:** [`font-kit`](https://docs.rs/font-kit) (descubrimiento multiplataforma de fuentes del SO)
  * **CLI:** [`clap`](https://docs.rs/clap) (v4 con derive)
  * **Asincronía e IPC:** [`tokio`](https://tokio.rs/) (runtime multihilo con soporte de sockets locales / Named Pipes)

---

## 2. Arquitectura Limpia y Hexagonal (Reglas Obligatorias)

El proyecto está organizado como un espacio de trabajo (*Cargo Workspace*) dividido estrictamente en **5 capas/crates**. La **Regla de Dependencias** es inviolable: las dependencias del código fuente **SOLO apuntan hacia adentro**.

```
                ┌──────────────────────────────┐
                │          PRESENTACIÓN        │
                │   rmerge-gui  │  rmerge-cli  │
                └──────────────┬───────────────┘
                               │ (usa puertos de entrada)
                               ▼
                ┌──────────────────────────────┐
                │          APLICACIÓN          │
                │      rmerge-application      │
                │  • ports/in (Casos de Uso)   │
                │  • ports/out (Traits/DIP)    │
                │  • use_cases (Orquestación)  │
                └───────┬──────────────┬───────┘
  (implementa   ▲                      │ (depende de)
   puertos out) │                      ▼
 ┌──────────────┴──────────────┐ ┌──────────────────────────────┐
 │       INFRAESTRUCTURA       │ │           DOMINIO            │
 │    rmerge-infrastructure    │ │        rmerge-domain         │
 │ • git (libgit2 adapter)     │ │ • entities (Commit, Diff...) │
 │ • fonts (font-kit adapter)  │ │ • value_objects (Theme...)   │
 │ • settings (json adapter)   │ │ • services (Graph, Merge...) │
 │ • ipc (socket adapter)      │ │ • errors (Cero I/O)          │
 └─────────────────────────────┘ └──────────────────────────────┘
```

### 2.1. Crate: `rmerge-domain` (Núcleo)
* **Responsabilidad:** Entidades de negocio, objetos de valor y algoritmos puros.
* **Prohibido:**
  * NO importar `rmerge-application`, `rmerge-infrastructure`, `rmerge-gui` ni `rmerge-cli`.
  * NO incluir `libgit2`, `egui`, `clap` ni llamadas directas de I/O o red.
* **Componentes clave:**
  * `entities/`: `Commit`, `Branch`, `Tag`, `Diff`, `Author`, `ConflictFile`, `CommitGraph`.
  * `value_objects/`: `Theme`, `TypographyConfig`, `Icons`.
  * `services/`: `author_validator.rs`, `graph_calculator.rs`, `three_way_merge.rs`.
  * `errors.rs`: Errores de dominio tipados.

### 2.2. Crate: `rmerge-application` (Casos de Uso y Puertos)
* **Responsabilidad:** Orquestar la lógica de la aplicación y definir los contratos de comunicación externa.
* **Puertos de Entrada (`ports/in/`):** Métodos que exponen los casos de uso a la GUI y CLI (`RepositoryUseCases`, `StagingUseCases`, `CommitUseCases`, `AuthorUseCases`, `TypographyUseCases`, `MergeUseCases`).
* **Puertos de Salida (`ports/out/`):** Traits que desacoplan los detalles técnicos externos (`GitStoragePort`, `GitConfigPort`, `SettingsStoragePort`, `FontDiscoveryPort`, `ThemeStoragePort`).
* **Prohibido:** No utilizar dependencias concretas como `git2` o llamadas al filesystem; todo se abstrae mediante los traits de salida.

### 2.3. Crate: `rmerge-infrastructure` (Adaptadores de Salida)
* **Responsabilidad:** Implementar los traits de salida de `rmerge-application` conectando con librerías externas.
* **Adaptadores principales:**
  * `git/storage_adapter.rs` -> Implementa `GitStoragePort` con `git2`. Traduce errores externos a `DomainError`.
  * `git/config_adapter.rs` -> Implementa `GitConfigPort`.
  * `fonts/font_kit_adapter.rs` -> Implementa `FontDiscoveryPort` con `font-kit`.
  * `settings/settings_adapter.rs` -> Implementa `SettingsStoragePort` leyendo y escribiendo JSON.
  * `ipc/ipc_adapter.rs` -> Implementa la comunicación IPC cliente/servidor.

### 2.4. Crates: `rmerge-gui` y `rmerge-cli` (Adaptadores de Entrada)
* **Responsabilidad:** Interacción con el usuario y despacho de casos de uso.
* **Regla:** La interfaz de usuario nunca llama directamente a la capa de infraestructura; siempre pasa por los casos de uso de la aplicación.

---

## 3. Convenciones de UI, Tipografía y Diseño

### 3.1. Tipografía Dinámica y Jerarquía de Texto
* La configuración de tipografía del usuario en `self.state.typography` define dos aspectos:
  * **Tipografía de Interfaz (`ui_font`):** Familia proporcional (`FontFamily::Proportional`) y tamaño base (`ui_size`).
  * **Tipografía de Código (`code_font`):** Familia monoespaciada (`FontFamily::Monospace`), tamaño (`code_size`) y ligaduras.
* **Regla para los mensajes:** Todos los mensajes de commit (en el cuadro de redacción, en el detalle del commit, en la tabla de historial y en el grafo lateral), así como mensajes de estado y alertas modales, **DEBEN** utilizar el tamaño de interfaz `ui_size` y la tipografía proporcional (`FontFamily::Proportional`).
* **Visor de Diffs y Monospace:** Los hashes de commit, diffs unificados, hunks y contenido de archivos deben renderizarse exclusivamente con `code_size` y `FontFamily::Monospace`.

### 3.2. Estructura del Encabezado Superior (Top Header)
* El botón para alternar el panel del grafo lateral (`📊 Grafo Rama`) está ubicado **inmediatamente a la derecha del indicador de la rama activa**.
* Todas las acciones globales de navegación están concentradas en el **Menú Hamburguesa (`☰`)** en el extremo derecho:
  * 🏠 Inicio
  * 🔄 Refrescar
  * 📂 Abrir Repo...
  * 📥 Clonar...
  * ⚙ Preferencias
  * ℹ Acerca de

### 3.3. Área de Preparación (Staging) y Commit
* La sección para escribir el mensaje de commit y el botón de confirmación (`✔ Realizar Commit`) solo deben ser visibles y activas cuando existan archivos preparados (`staged`). Si no hay archivos preparados, la sección se oculta automáticamente.
* Botones de control:
  * `+`: Preparar archivo individual (Stage).
  * `-`: Despreparar archivo individual (Unstage).
  * `++`: Preparar todos los archivos modificados.
  * `--`: Despreparar todos los archivos.

### 3.4. Resiliencia de Red y Chequeo de Conexión
* Antes de ejecutar operaciones remotas de Git (`push`, `pull`, `fetch`), el sistema debe realizar una comprobación previa no bloqueante de alcanzabilidad del servidor remoto (verificación de socket TCP hacia host y puerto con timeout corto).
* Si el servidor no responde (ej. red privada o VPN desconectada), se debe desplegar el modal de advertencia (`server_unreachable_modal`) en el idioma activo en lugar de congelar la interfaz.

---

## 4. Internacionalización (i18n)

* Todo texto visible para el usuario en la interfaz debe estar internacionalizado.
* Los recursos de traducción se ubican en `locales/es.json` (Español) y `locales/en.json` (Inglés).
* Se mapean en la estructura fuertemente tipada `I18nStrings` (`crates/rmerge-gui/src/i18n.rs`).
* Al agregar nuevos botones, etiquetas o diálogos, **siempre** añade las claves correspondientes tanto en `es.json` como en `en.json`.

---

## 5. Multiplataforma (Linux, Windows, macOS)

* **Rutas de archivos:** Utilizar siempre `Path` y `PathBuf`. Evitar cadenas concatenadas con barras fijas.
* **Descubrimiento de fuentes:** Usar `FontKitAdapter`, que consulta las APIs nativas del SO (Fontconfig en Linux, DirectWrite en Windows, CoreText en macOS).
* **Sockets IPC:** Utilizar Unix Domain Sockets en Linux/macOS y Named Pipes en Windows (`tokio::net::windows::named_pipe`).

---

## 6. Skills Disponibles para el Agente

En el directorio `.agents/skills/` se encuentran los skills especializados para este repositorio:
* [Clean Architecture Skill](file:///.agents/skills/clean-architecture/SKILL.md): Guía de diseño, reglas de dependencia y checklist de implementación limpia.
* [Hexagonal Architecture Skill](file:///.agents/skills/hexagonal-architecture/SKILL.md): Guía de puertos de entrada/salida, adaptadores e inyección de dependencias.

---

## 7. Comandos de Verificación y Compilación

Antes de dar por concluida cualquier sesión de trabajo, el agente debe ejecutar:

```bash
# 1. Ejecutar toda la suite de pruebas unitarias e integración (18 tests)
cargo test --workspace

# 2. Verificar que no existan errores ni advertencias de compilación
cargo check --workspace

# 3. Compilar los binarios optimizados de producción
cargo build --release --workspace
```
