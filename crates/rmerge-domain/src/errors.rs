use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("Error de validación: {0}")]
    ValidationError(String),

    #[error("Identidad de autor no configurada. Se requiere user.name y user.email")]
    AuthorNotConfigured,

    #[error("Repositorio no encontrado o inválido en: {0}")]
    RepositoryNotFound(String),

    #[error("Error en operación de Git: {0}")]
    GitOperationFailed(String),

    #[error("Conflicto no resuelto en el archivo: {0}")]
    UnresolvedConflict(String),

    #[error("Hunk no encontrado o índice fuera de rango")]
    HunkNotFound,

    #[error("Error en descubrimiento de fuentes: {0}")]
    FontDiscovery(String),

    #[error("Error de configuración o persistencia: {0}")]
    ConfigurationError(String),

    #[error("Error de entrada/salida (I/O): {0}")]
    Io(String),

    #[error("Error interno del sistema: {0}")]
    Internal(String),

    #[error("Error de Red o Conectividad: {0}")]
    NetworkError(String),

    #[error("Error en proceso externo: {0}")]
    ProcessError(String),
}
