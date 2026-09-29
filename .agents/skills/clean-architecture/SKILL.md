---
name: clean-architecture
description: >-
  Guía y directrices obligatorias para estructurar, extender y mantener el código siguiendo
  los principios de Arquitectura Limpia (Clean Architecture) en el proyecto Git-Client (rmerge).
  Activar este skill al crear nuevas características, diseñar entidades de negocio,
  organizar casos de uso o definir límites de capas entre crates.
---

# Clean Architecture en Git-Client (rmerge)

Este documento establece las directrices y estándares para preservar la **Arquitectura Limpia** en todo el espacio de trabajo de `git-client`. Cualquier cambio o nueva funcionalidad debe respetar estrictamente las reglas de dependencia y la separación de responsabilidades aquí descritas.

---

## 1. La Regla de Oro: Regla de Dependencias (Dependency Rule)

> **Las dependencias en el código fuente solo deben apuntar HACIA ADENTRO, en dirección a las políticas de mayor nivel (el Dominio).**

```
 ┌────────────────────────────────────────────────────────┐
 │  Drivers / Presentación (rmerge-gui, rmerge-cli)       │
 │   ┌────────────────────────────────────────────────────┤
 │   │  Adaptadores / Infraestructura (rmerge-infrastructure)
 │   │   ┌────────────────────────────────────────────────┤
 │   │   │  Casos de Uso / Aplicación (rmerge-application)│
 │   │   │   ┌────────────────────────────────────────────┤
 │   │   │   │  Entidades / Dominio (rmerge-domain)       │
 │   │   │   │   • Cero dependencias externas             │
 │   │   │   │   • Lógica pura de negocio y Git           │
```

* **Ningún elemento de un círculo interior puede conocer nada sobre un círculo exterior:**
  * `rmerge-domain` **NUNCA** debe importar `rmerge-application`, `rmerge-infrastructure`, `rmerge-gui` ni `rmerge-cli`.
  * `rmerge-domain` **NUNCA** debe importar `git2`, `egui`, `clap` ni librerías de infraestructura.
  * `rmerge-application` solo depende de `rmerge-domain`. **NUNCA** de `rmerge-infrastructure` ni de la UI.
  * `rmerge-infrastructure` implementa los puertos de salida definidos en `rmerge-application`.
  * `rmerge-gui` y `rmerge-cli` son clientes de los casos de uso de `rmerge-application`.

---

## 2. Definición y Responsabilidades de las Capas

### Capa 1: Dominio (`crates/rmerge-domain`)
* **Propósito:** Alojar las entidades de negocio críticas, objetos de valor y servicios de dominio.
* **Componentes:**
  * `entities/`: Representaciones de conceptos fundamentales (`Commit`, `Branch`, `Tag`, `Diff`, `Author`, `ConflictFile`, `CommitGraph`).
  * `value_objects/`: Estructuras inmutables o de configuración (`Theme`, `TypographyConfig`, `Icons`).
  * `services/`: Algoritmos puros y reglas de validación (`author_validator.rs`, `graph_calculator.rs`, `three_way_merge.rs`).
  * `errors.rs`: Errores de dominio que no dependen de I/O o APIs de terceros.
* **Regla estricta:** Cero I/O, cero llamadas a red o sistema de archivos, 100% determinista y fácilmente testeable con pruebas unitarias puras.

### Capa 2: Aplicación (`crates/rmerge-application`)
* **Propósito:** Orquestar el flujo de datos hacia y desde las entidades, y dirigir a las entidades para que usen sus reglas de negocio a fin de cumplir los casos de uso.
* **Componentes:**
  * `ports/in/`: Contratos de casos de uso ejecutados por los actores externos (GUI, CLI).
  * `ports/out/`: Contratos (traits) que representan las capacidades que la aplicación necesita del exterior (`GitStoragePort`, `SettingsStoragePort`, `FontDiscoveryPort`, `ThemeStoragePort`).
  * `use_cases/`: Implementaciones de la lógica de aplicación (`repository.rs`, `staging.rs`, `commit.rs`, `author.rs`, `merge.rs`, `typography.rs`).
* **Regla estricta:** No debe contener código de `libgit2`, `egui` ni I/O directo. Todo acceso externo ocurre a través de los traits en `ports/out/`.

### Capa 3: Infraestructura (`crates/rmerge-infrastructure`)
* **Propósito:** Proveer las implementaciones concretas (adaptadores) para los puertos de salida de la aplicación.
* **Componentes:**
  * `git/`: Implementación de operaciones Git mediante `libgit2` (`storage_adapter.rs`, `config_adapter.rs`).
  * `fonts/`: Búsqueda de fuentes del sistema con `font-kit` (`font_kit_adapter.rs`).
  * `settings/`: Persistencia de configuración en disco JSON (`settings_adapter.rs`).
  * `themes/`: Carga de temas visuales integrados (`theme_adapter.rs`).
  * `ipc/`: Comunicación interproceso mediante sockets locales (`ipc_adapter.rs`).

### Capa 4: Presentación (`crates/rmerge-gui` y `crates/rmerge-cli`)
* **Propósito:** Adaptadores primarios que interactúan con el usuario humano o con el entorno.
* **Componentes:**
  * `rmerge-gui`: Aplicación gráfica basada en `egui` y `eframe`. Mantiene el estado de la UI (`state.rs`), renderiza las vistas (`views/`), gestiona temas, tipografías y eventos.
  * `rmerge-cli`: Interfaz de línea de comandos basada en `clap` y cliente IPC para invocar o controlar la GUI.
* **Regla estricta:** La UI nunca debe interactuar con `libgit2` directamente; siempre debe invocar los casos de uso de `rmerge-application`.

---

## 3. Flujo para Implementar una Nueva Funcionalidad

Al agregar una nueva característica a Git-Client, sigue rigurosamente este orden:

1. **Modelar el Dominio (`rmerge-domain`):**
   * ¿Hay nuevas estructuras de datos o reglas de negocio puras? Define entidades, value objects o servicios de dominio con sus tests unitarios.
2. **Definir los Puertos (`rmerge-application/src/ports`):**
   * Si se requiere interactuar con el sistema operativo o Git, agrega un método al puerto de salida correspondiente en `ports/out/` (o crea un nuevo trait).
   * Define el caso de uso en `ports/in/`.
3. **Implementar el Caso de Uso (`rmerge-application/src/use_cases`):**
   * Escribe la lógica que coordina las entidades y los puertos de salida.
   * Añade tests unitarios con mocks/fakes de los puertos en `use_cases/tests.rs`.
4. **Implementar el Adaptador (`rmerge-infrastructure`):**
   * Implementa el trait del puerto de salida utilizando la librería externa correspondiente (`git2`, filesystem, etc.).
5. **Consumir desde la Presentación (`rmerge-gui` / `rmerge-cli`):**
   * Conecta la acción del usuario en la UI con la ejecución del caso de uso.
   * Actualiza el estado reactivo (`state.rs`) y los textos en los archivos de idioma (`locales/es.json` y `locales/en.json`).

---

## 4. Antipatrones Prohibidos

* ❌ **Acoplamiento de I/O en el Dominio:** Usar `std::fs`, llamadas a procesos externos o dependencias como `git2` dentro de `rmerge-domain`.
* ❌ **Salteo de Capas:** Invocar adaptadores de infraestructura directamente desde la GUI sin pasar por un caso de uso de `rmerge-application`.
* ❌ **Fugas de Tipos Externos hacia el Dominio:** Exponer tipos de `git2` (como `git2::Oid`, `git2::Repository`) en las firmas de `rmerge-domain` o `rmerge-application`. Deben convertirse siempre a tipos de dominio puros (ej. `String`, `CommitId`).
* ❌ **Lógica de Negocio en la UI:** Calcular grafos de bifurcación, parsear diffs de tres vías o validar autores dentro de callbacks de `egui`.
