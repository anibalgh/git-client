use std::path::Path;
use git2::build::RepoBuilder;
use git2::{Config, Cred, CredentialType, DiffOptions, RemoteCallbacks, Repository, Signature, StatusOptions};
use chrono::{DateTime, TimeZone, Utc};
use rmerge_application::ports::out::GitStoragePort;
use rmerge_domain::entities::{
    Branch, Commit, CommitDetail, CommitFileDiff, CommitStats, DeltaKind, DiffLine, FilePatch,
    GitAuthor, GitCredential, Hunk, MergeOutcome, StagingArea, Tag,
};
use rmerge_application::ports::out::CredentialStoragePort;
use std::sync::Arc;
use rmerge_domain::errors::DomainError;

fn create_remote_callbacks(saved_credential: Option<GitCredential>) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, allowed_types| {
        let user = username_from_url.unwrap_or("git");

        // 1. Si el servidor remoto soporta autenticación SSH
        if allowed_types.contains(CredentialType::SSH_KEY) {
            // Intentar primero con el agente SSH activo en el sistema (ssh-agent / GNOME Keyring / 1Password)
            if let Ok(cred) = Cred::ssh_key_from_agent(user) {
                return Ok(cred);
            }

            // Si el agente no tiene la clave cargada, buscar claves SSH estándar en ~/.ssh/
            if let Some(home) = dirs::home_dir() {
                let ssh_dir = home.join(".ssh");
                let key_names = ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"];
                for name in &key_names {
                    let priv_key = ssh_dir.join(name);
                    let pub_key = ssh_dir.join(format!("{name}.pub"));
                    if priv_key.exists() {
                        let pub_path = if pub_key.exists() { Some(pub_key.as_path()) } else { None };
                        if let Ok(cred) = Cred::ssh_key(user, pub_path, &priv_key, None) {
                            return Ok(cred);
                        }
                    }
                }
            }
        }

        // 2. Si es HTTPS con credenciales de usuario/contraseña o token
        if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
            // A. Primero intentar con credencial segura guardada en nuestro almacén centralizado
            if let Some(ref saved) = saved_credential {
                if let Ok(cred) = Cred::userpass_plaintext(&saved.username, &saved.secret) {
                    return Ok(cred);
                }
            }

            // B. Intentar con el credential helper de git estándar
            if let Ok(config) = Config::open_default() {
                if let Ok(cred) = Cred::credential_helper(&config, _url, username_from_url) {
                    return Ok(cred);
                }
            }
        }

        // 3. Credenciales por defecto del sistema
        if allowed_types.contains(CredentialType::DEFAULT) {
            return Cred::default();
        }

        Cred::default()
    });

    callbacks
}

pub struct Git2StorageAdapter {
    credential_storage: Option<Arc<dyn CredentialStoragePort>>,
}

impl Git2StorageAdapter {
    pub fn new() -> Self {
        Self { credential_storage: None }
    }

    pub fn with_credentials(credential_storage: Arc<dyn CredentialStoragePort>) -> Self {
        Self { credential_storage: Some(credential_storage) }
    }
}

impl Default for Git2StorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GitStoragePort for Git2StorageAdapter {
    async fn get_status(&self, repo_path: &Path) -> Result<StagingArea, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let mut status_opts = StatusOptions::new();
        status_opts
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let statuses = repo.statuses(Some(&mut status_opts))
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        let mut staging = StagingArea::default();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or_default().to_string();
            let status = entry.status();

            // Respetar estrictamente .gitignore: omitir si está marcado como ignorado o coincide con las reglas
            if status.is_ignored() || repo.is_path_ignored(Path::new(&path)).unwrap_or(false) {
                continue;
            }

            if status.is_conflicted() {
                staging.conflicts.push(path.clone());
            }

            if status.is_wt_new() {
                staging.untracked.push(path.clone());
            }

            // Unstaged changes (working directory vs index)
            if status.is_wt_modified() || status.is_wt_deleted() || status.is_wt_renamed() {
                staging.unstaged.push(FilePatch {
                    path: path.clone(),
                    old_path: None,
                    status: if status.is_wt_deleted() {
                        DeltaKind::Deleted
                    } else if status.is_wt_renamed() {
                        DeltaKind::Renamed
                    } else {
                        DeltaKind::Modified
                    },
                    is_staged: false,
                    hunks: parse_file_hunks(&repo, &path, false)?,
                    additions: 0,
                    deletions: 0,
                });
            }

            // Staged changes (index vs HEAD)
            if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() || status.is_index_renamed() {
                staging.staged.push(FilePatch {
                    path: path.clone(),
                    old_path: None,
                    status: if status.is_index_new() {
                        DeltaKind::Added
                    } else if status.is_index_deleted() {
                        DeltaKind::Deleted
                    } else if status.is_index_renamed() {
                        DeltaKind::Renamed
                    } else {
                        DeltaKind::Modified
                    },
                    is_staged: true,
                    hunks: parse_file_hunks(&repo, &path, true)?,
                    additions: 0,
                    deletions: 0,
                });
            }
        }

        Ok(staging)
    }

    async fn get_commit_history(&self, repo_path: &Path, limit: usize) -> Result<Vec<Commit>, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let mut revwalk = match repo.revwalk() {
            Ok(rw) => rw,
            Err(_) => return Ok(Vec::new()),
        };

        // Si HEAD no existe (ej. repo recién inicializado), retornar lista vacía
        if revwalk.push_head().is_err() {
            return Ok(Vec::new());
        }

        let mut commits = Vec::new();
        for (idx, oid_res) in revwalk.enumerate() {
            if idx >= limit {
                break;
            }
            let oid = oid_res.map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
            let git_commit = repo.find_commit(oid)
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

            let id_str = git_commit.id().to_string();
            let short_id = if id_str.len() >= 7 { id_str[..7].to_string() } else { id_str.clone() };

            let author = git_commit.author();
            let time_secs = author.when().seconds();
            let authored_at: DateTime<Utc> = Utc.timestamp_opt(time_secs, 0).single().unwrap_or_else(Utc::now);

            let message = git_commit.message().unwrap_or_default();
            let mut lines = message.lines();
            let headline = lines.next().unwrap_or_default().to_string();
            let body = lines.collect::<Vec<_>>().join("\n");
            let message_body = if body.trim().is_empty() { None } else { Some(body) };

            let parent_ids = git_commit.parent_ids().map(|p| p.to_string()).collect();

            commits.push(Commit {
                id: id_str,
                short_id,
                message_headline: headline,
                message_body,
                author_name: author.name().unwrap_or_default().to_string(),
                author_email: author.email().unwrap_or_default().to_string(),
                authored_at,
                parent_ids,
                lane: 0,
                branches: Vec::new(),
                tags: Vec::new(),
            });
        }

        Ok(commits)
    }

    async fn get_branches(&self, repo_path: &Path) -> Result<Vec<Branch>, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let _head_target = repo.head().ok().and_then(|h| h.target()).map(|o| o.to_string());
        let head_shorthand = repo.head().ok().and_then(|h| h.shorthand().map(|s| s.to_string()));

        let mut branches = Vec::new();
        let git_branches = repo.branches(None)
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        for item in git_branches {
            let (b, branch_type) = item.map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
            let name = b.name().ok().flatten().unwrap_or_default().to_string();
            let target_commit_id = b.get().target().map(|o| o.to_string()).unwrap_or_default();
            let is_remote = branch_type == git2::BranchType::Remote;
            let is_head = !is_remote && head_shorthand.as_deref() == Some(&name);

            branches.push(Branch {
                name,
                is_head,
                is_remote,
                target_commit_id,
                upstream: None,
            });
        }

        Ok(branches)
    }

    async fn stage_hunk(&self, repo_path: &Path, file_path: &Path, _hunk_index: usize) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        if repo.is_path_ignored(file_path).unwrap_or(false) {
            return Err(DomainError::GitOperationFailed(format!(
                "El archivo '{}' está ignorado por .gitignore y no puede prepararse.",
                file_path.display()
            )));
        }

        let mut index = repo.index()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        index.add_path(file_path)
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        index.write()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn stage_lines(&self, repo_path: &Path, file_path: &Path, _hunk_index: usize, _line_indices: &[usize]) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        if repo.is_path_ignored(file_path).unwrap_or(false) {
            return Err(DomainError::GitOperationFailed(format!(
                "El archivo '{}' está ignorado por .gitignore y no puede prepararse.",
                file_path.display()
            )));
        }

        let mut index = repo.index()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        index.add_path(file_path)
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        index.write()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn unstage_lines(&self, repo_path: &Path, file_path: &Path, _hunk_index: usize, _line_indices: &[usize]) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;
        
        let head = repo.head().map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        let head_commit = head.peel_to_commit().map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        repo.reset_default(Some(head_commit.as_object()), [file_path])
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn discard_changes(&self, repo_path: &Path, file_path: &Path) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.path(file_path).force();

        repo.checkout_head(Some(&mut checkout_opts))
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn create_commit(&self, repo_path: &Path, message: &str, author: &GitAuthor) -> Result<String, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let mut index = repo.index()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        let tree_id = index.write_tree()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        let tree = repo.find_tree(tree_id)
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        let sig = Signature::now(&author.name, &author.email)
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        let mut parents = Vec::new();
        if let Ok(head) = repo.head() {
            if let Ok(commit) = head.peel_to_commit() {
                parents.push(commit);
            }
        }
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

        let commit_oid = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &parent_refs,
        ).map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        Ok(commit_oid.to_string())
    }

    async fn write_resolved_conflict(&self, repo_path: &Path, file_path: &Path, content: &str) -> Result<(), DomainError> {
        let full_path = repo_path.join(file_path);
        std::fs::write(&full_path, content)
            .map_err(|e| DomainError::Io(e.to_string()))?;

        // Añadir al index para marcar como resuelto
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;
        let mut index = repo.index()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        index.add_path(file_path)
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        index.write()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn clone_repository(&self, url: &str, destination: &Path) -> Result<(), DomainError> {
        let dest = destination.to_path_buf();
        let url_str = url.to_string();

        let host = GitCredential::extract_host(url);
        let saved_cred = if let Some(ref cred_store) = self.credential_storage {
            cred_store.get_credential(&host).await.ok().flatten()
        } else {
            None
        };

        let cred_for_git2 = saved_cred.clone();
        let cred_for_cli = saved_cred.clone();

        tokio::task::spawn_blocking(move || {
            let callbacks = create_remote_callbacks(cred_for_git2);
            let mut fetch_opts = git2::FetchOptions::new();
            fetch_opts.remote_callbacks(callbacks);

            let mut builder = RepoBuilder::new();
            builder.fetch_options(fetch_opts);

            match builder.clone(&url_str, &dest) {
                Ok(_) => Ok(()),
                Err(libgit2_err) => {
                    // Si libgit2 falla (por ejemplo, agentes SSH propietarios como GNOME Keyring o 1Password,
                    // llaves con passphrase protegidas por pinentry/GUI, o configuración personalizada en ~/.ssh/config),
                    // limpiamos el directorio de destino si quedó parcial y recurrimos al comando `git clone` del sistema.
                    if dest.exists() {
                        let _ = std::fs::remove_dir_all(&dest);
                    }

                    let clone_url_final = if let Some(ref cred) = cred_for_cli {
                        if (url_str.starts_with("https://") || url_str.starts_with("http://")) && !url_str.contains("@") {
                            let scheme = if url_str.starts_with("https://") { "https://" } else { "http://" };
                            let remainder = url_str.trim_start_matches(scheme);
                            let encoded_user = percent_encoding_light(&cred.username);
                            let encoded_secret = percent_encoding_light(&cred.secret);
                            format!("{scheme}{encoded_user}:{encoded_secret}@{remainder}")
                        } else {
                            url_str.clone()
                        }
                    } else {
                        url_str.clone()
                    };

                    let output = std::process::Command::new("git")
                        .arg("clone")
                        .arg("--")
                        .arg(&clone_url_final)
                        .arg(&dest)
                        .output();

                    match output {
                        Ok(out) if out.status.success() => Ok(()),
                        Ok(out) => {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            let git_err = if stderr.trim().is_empty() {
                                String::from_utf8_lossy(&out.stdout).to_string()
                            } else {
                                stderr.to_string()
                            };
                            Err(DomainError::GitOperationFailed(format!(
                                "Error al clonar repositorio con libgit2: {libgit2_err}. Fallback git CLI: {}",
                                git_err.trim()
                            )))
                        }
                        Err(spawn_err) => {
                            Err(DomainError::GitOperationFailed(format!(
                                "Error al clonar repositorio con libgit2: {libgit2_err} (No se pudo ejecutar git CLI: {spawn_err})"
                            )))
                        }
                    }
                }
            }
        })
        .await
        .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?
    }

    async fn stage_all(&self, repo_path: &Path) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;
        let mut index = repo.index()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        // add_all con IndexAddOption::DEFAULT respeta .gitignore.
        // Adicionalmente filtramos con el callback para garantizar que ningún archivo ignorado sea indexado.
        index.add_all(
            ["*"].iter(),
            git2::IndexAddOption::DEFAULT,
            Some(&mut |path, _matched_pathspec| {
                if repo.is_path_ignored(path).unwrap_or(false) {
                    1 // Omitir archivo ignorado
                } else {
                    0 // Añadir archivo permitido
                }
            }),
        ).map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        index.update_all(["*"].iter(), None)
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        index.write()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn unstage_all(&self, repo_path: &Path) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        if let Ok(head) = repo.head() {
            if let Ok(commit) = head.peel_to_commit() {
                repo.reset_default(Some(commit.as_object()), ["*"])
                    .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
                return Ok(());
            }
        }

        let mut index = repo.index()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        index.clear()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        index.write()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        Ok(())
    }

    async fn get_tags(&self, repo_path: &Path) -> Result<Vec<Tag>, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let tag_names = repo.tag_names(None)
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        let mut tags = Vec::new();
        for name in tag_names.iter().flatten() {
            let target_id = if let Ok(obj) = repo.revparse_single(name) {
                obj.id().to_string()
            } else {
                String::new()
            };
            tags.push(Tag {
                name: name.to_string(),
                target_commit_id: target_id,
            });
        }
        tags.sort_by(|a, b| b.name.cmp(&a.name));
        Ok(tags)
    }

    async fn create_tag(&self, repo_path: &Path, name: &str, target_commit: Option<&str>, message: Option<&str>) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let target_obj = if let Some(target_id) = target_commit.filter(|t| !t.trim().is_empty()) {
            repo.revparse_single(target_id)
                .map_err(|e| DomainError::GitOperationFailed(format!("Commit de destino no válido: {e}")))?
        } else {
            let head = repo.head()
                .map_err(|e| DomainError::GitOperationFailed(format!("No hay HEAD para crear el tag: {e}")))?;
            head.peel(git2::ObjectType::Commit)
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?
        };

        if let Some(msg) = message.filter(|m| !m.trim().is_empty()) {
            let sig = repo.signature()
                .unwrap_or_else(|_| Signature::now("Git-Client", "user@git-client.local").unwrap());
            repo.tag(name, &target_obj, &sig, msg, false)
                .map_err(|e| DomainError::GitOperationFailed(format!("Error al crear tag anotado: {e}")))?;
        } else {
            repo.tag_lightweight(name, &target_obj, false)
                .map_err(|e| DomainError::GitOperationFailed(format!("Error al crear tag ligero: {e}")))?;
        }

        Ok(())
    }

    async fn delete_tag(&self, repo_path: &Path, name: &str) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;
        repo.tag_delete(name)
            .map_err(|e| DomainError::GitOperationFailed(format!("Error al eliminar tag: {e}")))?;
        Ok(())
    }

    async fn create_branch(&self, repo_path: &Path, name: &str, start_point: Option<&str>, checkout: bool) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let target_commit = if let Some(sp) = start_point.filter(|s| !s.trim().is_empty()) {
            let obj = repo.revparse_single(sp)
                .map_err(|e| DomainError::GitOperationFailed(format!("Punto de inicio no válido: {e}")))?;
            obj.peel_to_commit()
                .map_err(|e| DomainError::GitOperationFailed(format!("No se pudo obtener commit de {sp}: {e}")))?
        } else {
            let head = repo.head()
                .map_err(|e| DomainError::GitOperationFailed(format!("No hay HEAD disponible: {e}")))?;
            head.peel_to_commit()
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?
        };

        repo.branch(name, &target_commit, false)
            .map_err(|e| DomainError::GitOperationFailed(format!("Error al crear rama '{name}': {e}")))?;

        if checkout {
            let refname = format!("refs/heads/{name}");
            repo.set_head(&refname)
                .map_err(|e| DomainError::GitOperationFailed(format!("Error al establecer HEAD: {e}")))?;
            let mut opts = git2::build::CheckoutBuilder::new();
            opts.safe();
            repo.checkout_head(Some(&mut opts))
                .map_err(|e| DomainError::GitOperationFailed(format!("Error al realizar checkout de rama: {e}")))?;
        }

        Ok(())
    }

    async fn switch_branch(&self, repo_path: &Path, name: &str) -> Result<(), DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let refname = if name.starts_with("refs/") {
            name.to_string()
        } else {
            format!("refs/heads/{name}")
        };

        let mut opts = git2::build::CheckoutBuilder::new();
        opts.safe();

        repo.set_head(&refname)
            .map_err(|e| DomainError::GitOperationFailed(format!("Error al cambiar a rama '{name}': {e}")))?;
        repo.checkout_head(Some(&mut opts))
            .map_err(|e| DomainError::GitOperationFailed(format!("Error al actualizar archivos de rama: {e}")))?;

        Ok(())
    }

    async fn merge_branch(&self, repo_path: &Path, source_branch: &str) -> Result<MergeOutcome, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        // 1. Obtener el commit de la rama fuente
        let source_branch_obj = repo.revparse_single(source_branch)
            .map_err(|e| DomainError::GitOperationFailed(format!("Rama no encontrada: {e}")))?;
        let source_commit = source_branch_obj.peel_to_commit()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        let annotated_commit = repo.find_annotated_commit(source_commit.id())
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        // 2. Analizar el merge
        let (analysis, _) = repo.merge_analysis(&[&annotated_commit][..])
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        if analysis.is_up_to_date() {
            return Ok(MergeOutcome::UpToDate);
        }

        if analysis.is_fast_forward() {
            let mut head_ref = repo.head()
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
            let new_commit_id = source_commit.id().to_string();

            let mut checkout_opts = git2::build::CheckoutBuilder::new();
            checkout_opts.safe();
            repo.checkout_tree(&source_commit.clone().into_object(), Some(&mut checkout_opts))
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

            head_ref.set_target(source_commit.id(), &format!("Fast-forward merge to {source_branch}"))
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

            return Ok(MergeOutcome::FastForward { new_commit_id });
        }

        if analysis.is_normal() {
            let mut merge_opts = git2::MergeOptions::new();
            let mut checkout_opts = git2::build::CheckoutBuilder::new();
            checkout_opts.safe();

            repo.merge(&[&annotated_commit][..], Some(&mut merge_opts), Some(&mut checkout_opts))
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

            let mut index = repo.index()
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

            if index.has_conflicts() {
                let conflicts = repo.statuses(None)
                    .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?
                    .iter()
                    .filter(|s| s.status().is_conflicted())
                    .map(|s| s.path().unwrap_or_default().to_string())
                    .collect();
                return Ok(MergeOutcome::Conflicts { conflict_files: conflicts });
            }

            // Si no hay conflictos, crear commit de merge automáticamente
            let tree_id = index.write_tree()
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
            let tree = repo.find_tree(tree_id)
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
            let sig = repo.signature()
                .unwrap_or_else(|_| Signature::now("Git-Client", "user@git-client.local").unwrap());

            let head_commit = repo.head()
                .and_then(|h| h.peel_to_commit())
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

            let commit_id = repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("Merge branch '{source_branch}'"),
                &tree,
                &[&head_commit, &source_commit][..],
            ).map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

            repo.cleanup_state()
                .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

            return Ok(MergeOutcome::Merged { merge_commit_id: commit_id.to_string() });
        }

        Ok(MergeOutcome::UpToDate)
    }

    async fn get_commit_detail(&self, repo_path: &Path, commit_id: &str) -> Result<CommitDetail, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let oid = git2::Oid::from_str(commit_id)
            .map_err(|e| DomainError::GitOperationFailed(format!("ID de commit inválido: {e}")))?;
        let git_commit = repo.find_commit(oid)
            .map_err(|e| DomainError::GitOperationFailed(format!("Commit no encontrado: {e}")))?;

        let id_str = git_commit.id().to_string();
        let short_id = if id_str.len() >= 7 { id_str[..7].to_string() } else { id_str.clone() };

        let author = git_commit.author();
        let time_secs = author.when().seconds();
        let authored_at: DateTime<Utc> = Utc.timestamp_opt(time_secs, 0).single().unwrap_or_else(Utc::now);

        let message = git_commit.message().unwrap_or_default();
        let mut lines = message.lines();
        let headline = lines.next().unwrap_or_default().to_string();
        let body = lines.collect::<Vec<_>>().join("\n");
        let message_body = if body.trim().is_empty() { None } else { Some(body) };

        let parent_ids: Vec<String> = git_commit.parent_ids().map(|p| p.to_string()).collect();

        // Ramas que apuntan a este commit
        let mut branches = Vec::new();
        if let Ok(branch_iter) = repo.branches(None) {
            for b in branch_iter.flatten() {
                let (branch, _) = b;
                if let Ok(Some(name)) = branch.name() {
                    if branch.get().target() == Some(oid) {
                        branches.push(name.to_string());
                    }
                }
            }
        }

        // Tags en este commit
        let mut tags = Vec::new();
        if let Ok(tag_names) = repo.tag_names(None) {
            for t in tag_names.iter().flatten() {
                if let Ok(obj) = repo.revparse_single(t) {
                    if obj.id() == oid || obj.peel_to_commit().map(|c| c.id() == oid).unwrap_or(false) {
                        tags.push(t.to_string());
                    }
                }
            }
        }

        let commit_obj = Commit {
            id: id_str,
            short_id,
            message_headline: headline,
            message_body,
            author_name: author.name().unwrap_or_default().to_string(),
            author_email: author.email().unwrap_or_default().to_string(),
            authored_at,
            parent_ids,
            lane: 0,
            branches,
            tags,
        };

        let commit_tree = git_commit.tree()
            .map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        let tree_id = commit_tree.id().to_string();

        let parent_tree = if git_commit.parent_count() > 0 {
            git_commit.parent(0).ok().and_then(|p| p.tree().ok())
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        let diff = repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&commit_tree),
            Some(&mut diff_opts),
        ).map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

        let stats_raw = diff.stats().map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;
        let stats = CommitStats {
            files_changed: stats_raw.files_changed(),
            insertions: stats_raw.insertions(),
            deletions: stats_raw.deletions(),
        };

        // Extraer los parches y hunks de cada archivo modificado en este commit
        let mut files = Vec::new();
        for (delta_idx, delta) in diff.deltas().enumerate() {
            let new_file = delta.new_file();
            let old_file = delta.old_file();
            let path = new_file.path().or_else(|| old_file.path()).unwrap_or(Path::new("")).to_string_lossy().to_string();
            let old_path = if delta.status() == git2::Delta::Renamed {
                old_file.path().map(|p| p.to_string_lossy().to_string())
            } else {
                None
            };

            let delta_kind = match delta.status() {
                git2::Delta::Added => DeltaKind::Added,
                git2::Delta::Deleted => DeltaKind::Deleted,
                git2::Delta::Renamed => DeltaKind::Renamed,
                git2::Delta::Conflicted => DeltaKind::Conflicted,
                _ => DeltaKind::Modified,
            };

            let mut file_hunks = Vec::new();
            if let Ok(Some(patch)) = git2::Patch::from_diff(&diff, delta_idx) {
                for h_idx in 0..patch.num_hunks() {
                    if let Ok((hunk, _)) = patch.hunk(h_idx) {
                        let mut lines = Vec::new();
                        let num_lines = patch.num_lines_in_hunk(h_idx).unwrap_or(0);
                        for l_idx in 0..num_lines {
                            if let Ok(line) = patch.line_in_hunk(h_idx, l_idx) {
                                let content = String::from_utf8_lossy(line.content()).to_string();
                                let diff_line = match line.origin() {
                                    '+' => DiffLine::addition(line.new_lineno().unwrap_or(0), content),
                                    '-' => DiffLine::deletion(line.old_lineno().unwrap_or(0), content),
                                    _ => DiffLine::context(
                                        line.old_lineno().unwrap_or(0),
                                        line.new_lineno().unwrap_or(0),
                                        content,
                                    ),
                                };
                                lines.push(diff_line);
                            }
                        }
                        file_hunks.push(Hunk {
                            id: h_idx,
                            header: String::from_utf8_lossy(hunk.header()).trim().to_string(),
                            old_start: hunk.old_start(),
                            old_lines: hunk.old_lines(),
                            new_start: hunk.new_start(),
                            new_lines: hunk.new_lines(),
                            lines,
                        });
                    }
                }
            }

            let file_adds = file_hunks.iter().flat_map(|h| &h.lines).filter(|l| l.kind == rmerge_domain::entities::LineKind::Addition).count();
            let file_dels = file_hunks.iter().flat_map(|h| &h.lines).filter(|l| l.kind == rmerge_domain::entities::LineKind::Deletion).count();

            files.push(CommitFileDiff {
                path: path.clone(),
                old_path: old_path.clone(),
                status: delta_kind,
                additions: file_adds,
                deletions: file_dels,
                patch: FilePatch {
                    path,
                    old_path,
                    status: delta_kind,
                    is_staged: false,
                    hunks: file_hunks,
                    additions: file_adds,
                    deletions: file_dels,
                },
            });
        }

        Ok(CommitDetail {
            commit: commit_obj,
            tree_id,
            stats,
            files,
        })
    }

    async fn get_remote_url(&self, repo_path: &Path, remote_name: Option<&str>) -> Result<Option<String>, DomainError> {
        let repo = Repository::open(repo_path)
            .map_err(|e| DomainError::RepositoryNotFound(e.to_string()))?;

        let r_name = remote_name.unwrap_or("origin");
        if let Ok(remote) = repo.find_remote(r_name) {
            if let Some(url) = remote.url() {
                return Ok(Some(url.to_string()));
            }
        }

        if let Ok(remotes) = repo.remotes() {
            if let Some(first) = remotes.get(0) {
                if let Ok(remote) = repo.find_remote(first) {
                    if let Some(url) = remote.url() {
                        return Ok(Some(url.to_string()));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_gitignore(&self, repo_path: &Path) -> Result<Option<String>, DomainError> {
        let gitignore_path = repo_path.join(".gitignore");
        if gitignore_path.exists() {
            let content = std::fs::read_to_string(&gitignore_path)
                .map_err(|e| DomainError::Io(format!("Error al leer .gitignore: {e}")))?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    async fn save_gitignore(&self, repo_path: &Path, content: &str) -> Result<(), DomainError> {
        let gitignore_path = repo_path.join(".gitignore");
        std::fs::write(&gitignore_path, content)
            .map_err(|e| DomainError::Io(format!("Error al guardar .gitignore: {e}")))?;
        Ok(())
    }
}

/// Extrae los hunks y líneas de diferencia para un archivo específico
fn parse_file_hunks(repo: &Repository, file_path: &str, staged: bool) -> Result<Vec<Hunk>, DomainError> {
    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(file_path);

    let diff = if staged {
        let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
        repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut diff_opts))
    } else {
        repo.diff_index_to_workdir(None, Some(&mut diff_opts))
    }.map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

    let mut hunks = Vec::new();
    let mut current_hunk: Option<Hunk> = None;

    diff.print(git2::DiffFormat::Patch, |_delta, git_hunk, line| {
        if let Some(h) = git_hunk {
            if current_hunk.as_ref().is_none_or(|curr| curr.header != std::str::from_utf8(h.header()).unwrap_or_default()) {
                if let Some(finished) = current_hunk.take() {
                    hunks.push(finished);
                }
                current_hunk = Some(Hunk {
                    id: hunks.len(),
                    header: String::from_utf8_lossy(h.header()).trim().to_string(),
                    old_start: h.old_start(),
                    old_lines: h.old_lines(),
                    new_start: h.new_start(),
                    new_lines: h.new_lines(),
                    lines: Vec::new(),
                });
            }
        }

        if let Some(ref mut hunk) = current_hunk {
            let origin = line.origin();
            let content = String::from_utf8_lossy(line.content()).to_string();
            let diff_line = match origin {
                '+' => DiffLine::addition(line.new_lineno().unwrap_or(0), content),
                '-' => DiffLine::deletion(line.old_lineno().unwrap_or(0), content),
                _ => DiffLine::context(
                    line.old_lineno().unwrap_or(0),
                    line.new_lineno().unwrap_or(0),
                    content,
                ),
            };
            hunk.lines.push(diff_line);
        }
        true
    }).map_err(|e| DomainError::GitOperationFailed(e.to_string()))?;

    if let Some(finished) = current_hunk {
        hunks.push(finished);
    }

    Ok(hunks)
}

fn percent_encoding_light(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                use std::fmt::Write;
                let _ = write!(encoded, "%{b:02X}");
            }
        }
    }
    encoded
}
