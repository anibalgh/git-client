use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use rmerge_application::ports::out::CredentialStoragePort;
use rmerge_domain::entities::GitCredential;
use rmerge_domain::errors::DomainError;

const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 12;
const PBKDF2_ROUNDS: u32 = 100_000;

#[derive(Serialize, Deserialize)]
struct EncryptedStore {
    /// Salt de 16 bytes codificado en hexadecimal para derivación de llave
    salt_hex: String,
    /// Nonce de 12 bytes codificado en hexadecimal
    nonce_hex: String,
    /// Payload cifrado con AES-256-GCM codificado en hexadecimal
    ciphertext_hex: String,
}

pub struct EncryptedCredentialAdapter {
    custom_path: Option<PathBuf>,
}

impl EncryptedCredentialAdapter {
    pub fn new() -> Self {
        Self { custom_path: None }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { custom_path: Some(path) }
    }

    fn resolve_path(&self) -> Result<PathBuf, DomainError> {
        if let Some(ref path) = self.custom_path {
            return Ok(path.clone());
        }

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rmerge");

        fs::create_dir_all(&config_dir)
            .map_err(|e| DomainError::ConfigurationError(format!("Error creando directorio de configuración: {e}")))?;

        Ok(config_dir.join("credentials.enc"))
    }

    /// Obtiene una semilla de máquina consistente y segura basada en el usuario y el equipo
    fn get_machine_seed() -> Vec<u8> {
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "git-client-user".to_string());

        let home = dirs::home_dir()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|| "/default/home".to_string());

        // Intentar leer machine-id en Linux si está disponible
        let machine_id = fs::read_to_string("/etc/machine-id")
            .or_else(|_| fs::read_to_string("/var/lib/dbus/machine-id"))
            .unwrap_or_else(|_| "machine-default-id-seed".to_string());

        let combined = format!("{user}:{home}:{machine_id}:rmerge-vault-kdf-v1");
        combined.into_bytes()
    }

    /// Deriva una llave AES-256 de 32 bytes usando PBKDF2-HMAC-SHA256
    fn derive_key(salt: &[u8]) -> [u8; 32] {
        let seed = Self::get_machine_seed();
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(&seed, salt, PBKDF2_ROUNDS, &mut key);
        key
    }

    fn read_map(&self) -> Result<HashMap<String, GitCredential>, DomainError> {
        let path = self.resolve_path()?;
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| DomainError::Io(format!("Error leyendo almacén de credenciales: {e}")))?;

        let store: EncryptedStore = match serde_json::from_str(&raw) {
            Ok(s) => s,
            Err(_) => return Ok(HashMap::new()),
        };

        let salt = hex_decode(&store.salt_hex)
            .map_err(|e| DomainError::ConfigurationError(format!("Salt corrupto: {e}")))?;
        let nonce_bytes = hex_decode(&store.nonce_hex)
            .map_err(|e| DomainError::ConfigurationError(format!("Nonce corrupto: {e}")))?;
        let ciphertext = hex_decode(&store.ciphertext_hex)
            .map_err(|e| DomainError::ConfigurationError(format!("Cifrado corrupto: {e}")))?;

        if nonce_bytes.len() != NONCE_SIZE {
            return Err(DomainError::ConfigurationError("Tamaño de nonce inválido".into()));
        }

        let key = Self::derive_key(&salt);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| DomainError::Internal(format!("Error inicializando cifrador: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let decrypted = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| DomainError::ConfigurationError(format!("Error al descifrar credenciales: {e}")))?;

        let map: HashMap<String, GitCredential> = serde_json::from_slice(&decrypted)
            .map_err(|e| DomainError::ConfigurationError(format!("Error deserializando credenciales: {e}")))?;

        Ok(map)
    }

    fn write_map(&self, map: &HashMap<String, GitCredential>) -> Result<(), DomainError> {
        let path = self.resolve_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| DomainError::Io(format!("Error creando directorio de credenciales: {e}")))?;
        }

        let json_bytes = serde_json::to_vec(map)
            .map_err(|e| DomainError::ConfigurationError(format!("Error serializando credenciales: {e}")))?;

        let mut salt = [0u8; SALT_SIZE];
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let key = Self::derive_key(&salt);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| DomainError::Internal(format!("Error inicializando cifrador: {e}")))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, json_bytes.as_ref())
            .map_err(|e| DomainError::Internal(format!("Error al cifrar credenciales: {e}")))?;

        let store = EncryptedStore {
            salt_hex: hex_encode(&salt),
            nonce_hex: hex_encode(&nonce_bytes),
            ciphertext_hex: hex_encode(&ciphertext),
        };

        let store_json = serde_json::to_string_pretty(&store)
            .map_err(|e| DomainError::ConfigurationError(format!("Error formateando JSON cifrado: {e}")))?;

        fs::write(&path, store_json)
            .map_err(|e| DomainError::Io(format!("Error guardando almacén de credenciales: {e}")))?;

        // Proteger permisos de lectura/escritura solo para el usuario en Unix (0600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }
}

impl Default for EncryptedCredentialAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CredentialStoragePort for EncryptedCredentialAdapter {
    async fn get_credential(&self, host: &str) -> Result<Option<GitCredential>, DomainError> {
        let map = self.read_map()?;
        let normalized = host.trim().to_lowercase();
        Ok(map.get(&normalized).cloned())
    }

    async fn save_credential(&self, credential: &GitCredential) -> Result<(), DomainError> {
        let mut map = self.read_map()?;
        let normalized = credential.host.trim().to_lowercase();
        let mut updated = credential.clone();
        updated.host = normalized.clone();
        map.insert(normalized, updated);
        self.write_map(&map)
    }

    async fn delete_credential(&self, host: &str) -> Result<(), DomainError> {
        let mut map = self.read_map()?;
        let normalized = host.trim().to_lowercase();
        if map.remove(&normalized).is_some() {
            self.write_map(&map)?;
        }
        Ok(())
    }

    async fn list_hosts(&self) -> Result<Vec<String>, DomainError> {
        let map = self.read_map()?;
        let mut hosts: Vec<String> = map.keys().cloned().collect();
        hosts.sort();
        Ok(hosts)
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("Longitud hexadecimal impar".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encrypted_credential_adapter_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("rmerge_test_vault_{}", rand::random::<u64>()));
        let vault_file = temp_dir.join("credentials.enc");

        let adapter = EncryptedCredentialAdapter::with_path(vault_file.clone());

        // Inicialmente vacío
        let hosts = adapter.list_hosts().await.unwrap();
        assert!(hosts.is_empty());

        // Guardar credencial
        let cred = GitCredential::new("github.com", "octocat", "ghp_123456789Secret");
        adapter.save_credential(&cred).await.unwrap();

        // Archivo en disco fue creado y está cifrado (no contiene la clave en texto plano)
        let raw_disk = fs::read_to_string(&vault_file).unwrap();
        assert!(!raw_disk.contains("ghp_123456789Secret"));
        assert!(!raw_disk.contains("octocat"));
        assert!(raw_disk.contains("salt_hex"));

        // Recuperar credencial
        let retrieved = adapter.get_credential("github.com").await.unwrap();
        assert_eq!(retrieved, Some(cred.clone()));

        // Sensibilidad a mayúsculas/minúsculas normalizada
        let retrieved_upper = adapter.get_credential("GITHUB.COM").await.unwrap();
        assert_eq!(retrieved_upper, Some(cred));

        // Borrar credencial
        adapter.delete_credential("github.com").await.unwrap();
        let retrieved_deleted = adapter.get_credential("github.com").await.unwrap();
        assert_eq!(retrieved_deleted, None);

        // Limpiar
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
