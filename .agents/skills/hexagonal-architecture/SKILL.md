---
name: hexagonal-architecture
description: >-
  Directrices y patrones de Arquitectura Hexagonal (Puertos y Adaptadores / Ports & Adapters)
  en el proyecto Git-Client (rmerge). Activar este skill al diseñar o modificar puertos
  (inbound/outbound), escribir casos de uso, crear adaptadores (primarios o secundarios)
  y gestionar la inyección de dependencias.
---

# Arquitectura Hexagonal (Puertos y Adaptadores) en Git-Client

La Arquitectura Hexagonal (propuesta por Alistair Cockburn) aísla el núcleo de negocio de la aplicación de sus tecnologías externas (interfaz de usuario, sistemas de archivos, bibliotecas de Git, APIs de fuentes).

---

## 1. El Concepto del Hexágono

```text
           [ Adaptadores Primarios / Driving ]
            (Interacción iniciada desde afuera)
             ┌─────────────┐       ┌─────────────┐
             │  rmerge-gui │       │  rmerge-cli │
             └──────┬──────┘       └──────┬──────┘
                    │                     │
                    ▼                     ▼
          ═════════════════════════════════════════
          ║    PUERTOS DE ENTRADA (Inbound)       ║
          ║    (Casos de uso / use_cases)         ║
          ║                                       ║
          ║            NÚCLEO (Core)              ║
          ║         rmerge-application            ║
          ║            rmerge-domain              ║
          ║                                       ║
          ║    PUERTOS DE SALIDA (Outbound)       ║
          ║    (Traits en ports/out/)             ║
          ═════════════════════════════════════════
                    │            │           │
                    ▼            ▼           ▼
             ┌─────────────┬───────────┬─────────────┐
             │ GitStorage  │ Settings  │ FontKit     │
             │ Adapter     │ Adapter   │ Adapter     │
             └─────────────┴───────────┴─────────────┘
          [ Adaptadores Secundarios / Driven / Infra ]
            (Invocados por el núcleo hacia afuera)
```text

---

## 2. Puertos de Entrada vs Puertos de Salida

### Puertos de Entrada (Inbound / Driving Ports)

* **Definición:** Contratos que exponen lo que la aplicación puede hacer. Son el punto de entrada para los adaptadores primarios (GUI y CLI).
* **Ubicación:** `crates/rmerge-application/src/ports/in/` y `crates/rmerge-application/src/use_cases/`.
* **Ejemplos en el proyecto:**
  * `RepositoryUseCases`: Carga de repositorios, refresco de estado y grafo.
  * `StagingUseCases`: Preparar (`stage_file`), retirar (`unstage_file`), preparar todo y limpiar.
  * `CommitUseCases`: Ejecutar un commit, validar requerimiento de autor.
  * `AuthorUseCases`: Configuración y lectura de la identidad del autor Git.
  * `TypographyUseCases`: Descubrimiento de fuentes del sistema y aplicación de estilos.

### Puertos de Salida (Outbound / Driven Ports)

* **Definición:** Interfaces (`traits`) que definen lo que el núcleo necesita que el exterior haga por él. Permiten la **Inversión de Dependencias (DIP)**.
* **Ubicación:** `crates/rmerge-application/src/ports/out/`.
* **Ejemplos en el proyecto:**
  * `GitStoragePort`: Abstracción de operaciones sobre el repositorio Git local (historial, ramas, tags, diffs, commits).
  * `GitConfigPort`: Lectura y escritura de valores de configuración de Git (`user.name`, `user.email`).
  * `SettingsStoragePort`: Guardado y recuperación de preferencias del usuario (`settings.json`).
  * `FontDiscoveryPort`: Consulta de tipografías instaladas en el sistema operativo.
  * `ThemeStoragePort`: Carga y gestión de temas visuales.

---

## 3. Implementación de Adaptadores

### Adaptadores Secundarios (Outbound Adapters)

Residen en `crates/rmerge-infrastructure` e implementan los traits de `ports/out/`:

1. **`GitStorageAdapter` (`crates/rmerge-infrastructure/src/git/storage_adapter.rs`):**
   * Implementa `GitStoragePort` utilizando `git2::Repository`.
   * **Regla crucial:** Traduce todos los errores de `git2::Error` y estructuras nativas a tipos de dominio de `rmerge-domain` antes de devolverlos al caso de uso.
2. **`FontKitAdapter` (`crates/rmerge-infrastructure/src/fonts/font_kit_adapter.rs`):**
   * Implementa `FontDiscoveryPort` consultando `font-kit` de manera multiplataforma.
3. **`SettingsAdapter` (`crates/rmerge-infrastructure/src/settings/settings_adapter.rs`):**
   * Implementa `SettingsStoragePort` leyendo y escribiendo JSON en la ruta estándar de configuración del sistema (`~/.config/rmerge/` o `%APPDATA%`).

### Adaptadores Primarios (Driving Adapters)

Residen en `crates/rmerge-gui` y `crates/rmerge-cli`:

* Crean las instancias de los adaptadores de infraestructura (`GitStorageAdapter`, etc.).
* Inyectan los adaptadores en los casos de uso (`Arc::new(...)`).
* Invocan los casos de uso en respuesta a eventos del usuario (clics en botones, atajos de teclado, comandos de consola).

---

## 4. Guía Paso a Paso para Agregar un Nuevo Puerto y Adaptador

Cuando una funcionalidad requiera comunicarse con un sistema externo o un nuevo subsistema:

1. **Declarar el Puerto de Salida:**
   * En `crates/rmerge-application/src/ports/out/<nombre>.rs`:

   ```rust
   #[async_trait::async_trait]
   pub trait RemoteProviderPort: Send + Sync {
       async fn check_connection(&self, url: &str) -> Result<bool, DomainError>;
   }
   ```

2. **Implementar el Caso de Uso:**
   * En `crates/rmerge-application/src/use_cases/<nombre>.rs`:

   ```rust
   pub struct VerifyRemoteUseCase {
       provider: Arc<dyn RemoteProviderPort>,
   }
   impl VerifyRemoteUseCase {
       pub fn new(provider: Arc<dyn RemoteProviderPort>) -> Self { Self { provider } }
       pub async fn execute(&self, url: &str) -> Result<bool, DomainError> {
           self.provider.check_connection(url).await
       }
   }
   ```

3. **Implementar el Adaptador de Infraestructura:**
   * En `crates/rmerge-infrastructure/src/<modulo>/adapter.rs`:

   ```rust
   pub struct NetworkRemoteAdapter;
   #[async_trait::async_trait]
   impl RemoteProviderPort for NetworkRemoteAdapter {
       async fn check_connection(&self, url: &str) -> Result<bool, DomainError> {
           // Lógica concreta de red
           Ok(true)
       }
   }
   ```

4. **Conectar en la Capa de Presentación:**
   * En `crates/rmerge-gui/src/main.rs`:

   ```rust
   let remote_adapter = Arc::new(NetworkRemoteAdapter);
   let verify_use_case = VerifyRemoteUseCase::new(remote_adapter);
   ```

---

## 5. Pruebas Unitarias Aisladas (Testability)

Gracias a los puertos de salida, los casos de uso pueden probarse exhaustivamente en `rmerge-application/src/use_cases/tests.rs` sin tocar Git real ni el disco, creando implementaciones falsas (Fakes/Mocks) de los traits:

```rust
struct MockGitStorage;
#[async_trait::async_trait]
impl GitStoragePort for MockGitStorage {
    // Retorna datos de prueba en memoria sin tocar git2
}
```text

Esto garantiza pruebas ultrarrápidas, deterministas y libres de efectos secundarios.
