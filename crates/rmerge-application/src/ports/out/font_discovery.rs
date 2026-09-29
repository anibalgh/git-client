use rmerge_domain::errors::DomainError;

#[async_trait::async_trait]
pub trait FontDiscoveryPort: Send + Sync {
    /// Lista los nombres de las familias tipográficas instaladas en el sistema operativo
    async fn list_system_fonts(&self) -> Result<Vec<String>, DomainError>;

    /// Lista exclusivamente las fuentes monoespaciadas aptas para la vista de código/diffs
    async fn list_monospace_fonts(&self) -> Result<Vec<String>, DomainError>;

    /// Carga los bytes binarios de la fuente para incrustar en la interfaz gráfica
    async fn load_font_bytes(&self, family_name: &str) -> Result<Option<Vec<u8>>, DomainError>;
}
