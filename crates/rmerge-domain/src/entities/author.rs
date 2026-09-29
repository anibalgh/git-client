use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitAuthor {
    pub name: String,
    pub email: String,
}

impl GitAuthor {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigScope {
    /// Ámbito de repositorio local (.git/config)
    Local,
    /// Ámbito global de usuario (~/.gitconfig o %USERPROFILE%\.gitconfig)
    Global,
}
