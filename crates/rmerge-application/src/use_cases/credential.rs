use std::sync::Arc;
use rmerge_domain::entities::GitCredential;
use rmerge_domain::errors::DomainError;
use crate::ports::r#in::ManageCredentialsUseCase;
use crate::ports::out::CredentialStoragePort;

pub struct CredentialService {
    storage: Arc<dyn CredentialStoragePort>,
}

impl CredentialService {
    pub fn new(storage: Arc<dyn CredentialStoragePort>) -> Self {
        Self { storage }
    }
}

#[async_trait::async_trait]
impl ManageCredentialsUseCase for CredentialService {
    async fn find_credential(&self, host_or_url: &str) -> Result<Option<GitCredential>, DomainError> {
        let host = GitCredential::extract_host(host_or_url);
        self.storage.get_credential(&host).await
    }

    async fn store_credential(&self, host_or_url: &str, username: &str, secret: &str) -> Result<(), DomainError> {
        let host = GitCredential::extract_host(host_or_url);
        let cred = GitCredential::new(host, username.trim(), secret.trim());
        self.storage.save_credential(&cred).await
    }

    async fn remove_credential(&self, host_or_url: &str) -> Result<(), DomainError> {
        let host = GitCredential::extract_host(host_or_url);
        self.storage.delete_credential(&host).await
    }

    async fn get_configured_hosts(&self) -> Result<Vec<String>, DomainError> {
        self.storage.list_hosts().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::collections::HashMap;

    struct InMemoryCredStorage {
        items: Mutex<HashMap<String, GitCredential>>,
    }

    impl InMemoryCredStorage {
        fn new() -> Self {
            Self { items: Mutex::new(HashMap::new()) }
        }
    }

    #[async_trait::async_trait]
    impl CredentialStoragePort for InMemoryCredStorage {
        async fn get_credential(&self, host: &str) -> Result<Option<GitCredential>, DomainError> {
            Ok(self.items.lock().unwrap().get(host).cloned())
        }

        async fn save_credential(&self, credential: &GitCredential) -> Result<(), DomainError> {
            self.items.lock().unwrap().insert(credential.host.clone(), credential.clone());
            Ok(())
        }

        async fn delete_credential(&self, host: &str) -> Result<(), DomainError> {
            self.items.lock().unwrap().remove(host);
            Ok(())
        }

        async fn list_hosts(&self) -> Result<Vec<String>, DomainError> {
            Ok(self.items.lock().unwrap().keys().cloned().collect())
        }
    }

    #[tokio::test]
    async fn test_credential_service_flow() {
        let storage = Arc::new(InMemoryCredStorage::new());
        let service = CredentialService::new(storage);

        // Almacenar credencial usando URL completa
        service.store_credential("https://github.com/my-org/my-repo.git", "octocat", "ghp_secretToken123")
            .await
            .unwrap();

        // Buscar usando solo el host
        let found = service.find_credential("github.com").await.unwrap();
        assert!(found.is_some());
        let c = found.unwrap();
        assert_eq!(c.host, "github.com");
        assert_eq!(c.username, "octocat");
        assert_eq!(c.secret, "ghp_secretToken123");

        // Buscar usando otra URL del mismo host
        let found2 = service.find_credential("https://github.com/another/repo").await.unwrap();
        assert_eq!(found2, Some(c));
    }
}
