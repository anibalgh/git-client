use serde::{Deserialize, Serialize};

/// Representa las credenciales de autenticación para un host o repositorio Git remoto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCredential {
    /// Host o identificador del repositorio (ej. "github.com", "gitlab.com", o URL canónica)
    pub host: String,
    /// Nombre de usuario, email o cuenta de servicio
    pub username: String,
    /// Token de acceso personal (PAT), token OAuth o contraseña (almacenado de forma segura)
    pub secret: String,
}

impl GitCredential {
    pub fn new(host: impl Into<String>, username: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            username: username.into(),
            secret: secret.into(),
        }
    }

    /// Normaliza una URL o dirección remota extrayendo su dominio base (ej. "github.com" de "https://github.com/user/repo.git")
    pub fn extract_host(url: &str) -> String {
        let trimmed = url.trim();
        if let Some(rest) = trimmed.strip_prefix("https://").or_else(|| trimmed.strip_prefix("http://")) {
            let host_part = rest.split('/').next().unwrap_or(rest);
            let host_only = host_part.split('@').next_back().unwrap_or(host_part);
            let domain = host_only.split(':').next().unwrap_or(host_only);
            return domain.to_lowercase();
        }
        trimmed.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host_from_https_urls() {
        assert_eq!(
            GitCredential::extract_host("https://github.com/user/repo.git"),
            "github.com"
        );
        assert_eq!(
            GitCredential::extract_host("https://gitlab.com/group/subgroup/project"),
            "gitlab.com"
        );
        assert_eq!(
            GitCredential::extract_host("https://user:token@github.com/user/repo.git"),
            "github.com"
        );
        assert_eq!(
            GitCredential::extract_host("https://internal.gitlab.corp.net:8443/project/repo.git"),
            "internal.gitlab.corp.net"
        );
    }
}
