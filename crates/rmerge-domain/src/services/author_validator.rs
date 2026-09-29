use crate::entities::author::GitAuthor;
use crate::errors::DomainError;

pub struct AuthorValidator;

impl AuthorValidator {
    pub fn validate(author: &GitAuthor) -> Result<(), DomainError> {
        let trimmed_name = author.name.trim();
        if trimmed_name.is_empty() {
            return Err(DomainError::ValidationError(
                "El nombre de autor no puede estar vacío.".into(),
            ));
        }

        let trimmed_email = author.email.trim();
        if trimmed_email.is_empty() {
            return Err(DomainError::ValidationError(
                "El correo electrónico no puede estar vacío.".into(),
            ));
        }

        if !trimmed_email.contains('@') || !trimmed_email.contains('.') {
            return Err(DomainError::ValidationError(
                "El formato del correo electrónico es inválido.".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_author() {
        let author = GitAuthor::new("Anibal Garcia", "anibal@example.com");
        assert!(AuthorValidator::validate(&author).is_ok());
    }

    #[test]
    fn test_invalid_author_empty_name() {
        let author = GitAuthor::new("   ", "anibal@example.com");
        assert!(AuthorValidator::validate(&author).is_err());
    }

    #[test]
    fn test_invalid_author_bad_email() {
        let author = GitAuthor::new("Anibal", "invalid-email");
        assert!(AuthorValidator::validate(&author).is_err());
    }
}
