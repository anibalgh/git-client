use rmerge_domain::entities::{Branch, Commit, CommitDetail, GitAuthor, MergeOutcome, StagingArea, Tag};
use rmerge_domain::errors::DomainError;
use std::path::Path;

#[async_trait::async_trait]
pub trait GitStoragePort: Send + Sync {
    /// Obtiene el estado actual del repositorio (archivos staged, unstaged, sin seguimiento)
    async fn get_status(&self, repo_path: &Path) -> Result<StagingArea, DomainError>;

    /// Carga el historial de commits recientes
    async fn get_commit_history(&self, repo_path: &Path, limit: usize) -> Result<Vec<Commit>, DomainError>;

    /// Obtiene la lista de ramas locales y remotas
    async fn get_branches(&self, repo_path: &Path) -> Result<Vec<Branch>, DomainError>;

    /// Agrega un hunk completo al área de preparación (stage)
    async fn stage_hunk(&self, repo_path: &Path, file_path: &Path, hunk_index: usize) -> Result<(), DomainError>;

    /// Agrega líneas específicas de un archivo al staging
    async fn stage_lines(&self, repo_path: &Path, file_path: &Path, hunk_index: usize, line_indices: &[usize]) -> Result<(), DomainError>;

    /// Quita líneas específicas del staging (unstage)
    async fn unstage_lines(&self, repo_path: &Path, file_path: &Path, hunk_index: usize, line_indices: &[usize]) -> Result<(), DomainError>;

    /// Agrega todos los archivos modificados y sin seguimiento al stage (Stage All)
    async fn stage_all(&self, repo_path: &Path) -> Result<(), DomainError>;

    /// Quita todos los archivos preparados del stage (Unstage All)
    async fn unstage_all(&self, repo_path: &Path) -> Result<(), DomainError>;

    /// Obtiene la lista de tags del repositorio
    async fn get_tags(&self, repo_path: &Path) -> Result<Vec<Tag>, DomainError>;

    /// Crea un nuevo tag (ligero o anotado si se provee mensaje)
    async fn create_tag(&self, repo_path: &Path, name: &str, target_commit: Option<&str>, message: Option<&str>) -> Result<(), DomainError>;

    /// Elimina un tag existente
    async fn delete_tag(&self, repo_path: &Path, name: &str) -> Result<(), DomainError>;

    /// Crea una nueva rama a partir de un punto de inicio
    async fn create_branch(&self, repo_path: &Path, name: &str, start_point: Option<&str>, checkout: bool) -> Result<(), DomainError>;

    /// Cambia a la rama especificada (checkout)
    async fn switch_branch(&self, repo_path: &Path, name: &str) -> Result<(), DomainError>;

    /// Combina (merge) una rama en la rama activa actual
    async fn merge_branch(&self, repo_path: &Path, source_branch: &str) -> Result<MergeOutcome, DomainError>;

    /// Obtiene el detalle completo de un commit específico incluyendo estadísticas y diffs de archivos
    async fn get_commit_detail(&self, repo_path: &Path, commit_id: &str) -> Result<CommitDetail, DomainError>;

    /// Obtiene la URL del remote (ej. origin) si existe
    async fn get_remote_url(&self, repo_path: &Path, remote_name: Option<&str>) -> Result<Option<String>, DomainError>;

    /// Descarta cambios de un archivo en el directorio de trabajo
    async fn discard_changes(&self, repo_path: &Path, file_path: &Path) -> Result<(), DomainError>;

    /// Crea un commit con el mensaje y autor especificados
    async fn create_commit(&self, repo_path: &Path, message: &str, author: &GitAuthor) -> Result<String, DomainError>;

    /// Guarda la resolución final de un archivo en conflicto y lo marca como resuelto
    async fn write_resolved_conflict(&self, repo_path: &Path, file_path: &Path, content: &str) -> Result<(), DomainError>;

    /// Clona un repositorio remoto en el directorio de destino especificado
    async fn clone_repository(&self, url: &str, destination: &Path) -> Result<(), DomainError>;

    /// Lee el contenido del archivo .gitignore del repositorio si existe
    async fn get_gitignore(&self, repo_path: &Path) -> Result<Option<String>, DomainError>;

    /// Guarda el contenido en el archivo .gitignore del repositorio
    async fn save_gitignore(&self, repo_path: &Path, content: &str) -> Result<(), DomainError>;
}
