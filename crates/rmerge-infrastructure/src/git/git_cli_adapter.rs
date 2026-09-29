use rmerge_application::ports::out::GitCliPort;
use rmerge_domain::errors::DomainError;
use std::path::Path;

pub struct TokioGitCliAdapter;

impl TokioGitCliAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TokioGitCliAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GitCliPort for TokioGitCliAdapter {
    async fn execute(&self, repo_path: &Path, args: &[&str]) -> Result<String, DomainError> {
        let run_fut = async {
            tokio::process::Command::new("git")
                .args(args)
                .current_dir(repo_path)
                .env("GIT_TERMINAL_PROMPT", "0")
                .env(
                    "GIT_SSH_COMMAND",
                    "ssh -o ConnectTimeout=5 -o BatchMode=yes",
                )
                .output()
                .await
        };

        match tokio::time::timeout(std::time::Duration::from_secs(60), run_fut).await {
            Ok(Ok(out)) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                if out.status.success() {
                    if !stdout.is_empty() {
                        Ok(stdout)
                    } else if !stderr.is_empty() {
                        Ok(stderr)
                    } else {
                        Ok("comando completado con éxito".to_string())
                    }
                } else {
                    let err = if !stderr.is_empty() { stderr } else { stdout };
                    Err(DomainError::ProcessError(err))
                }
            }
            Ok(Err(e)) => Err(DomainError::ProcessError(format!(
                "Error de I/O al lanzar proceso: {e}"
            ))),
            Err(_) => Err(DomainError::ProcessError(
                "El comando excedió el tiempo límite (60s)".into(),
            )),
        }
    }
}
