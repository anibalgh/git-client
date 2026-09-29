#![allow(clippy::module_inception)]
#[cfg(test)]
mod tests {
    use std::fs;
    use git2::Repository;
    use rmerge_application::ports::out::{FontDiscoveryPort, GitConfigPort, GitStoragePort, ThemeStoragePort};
    use rmerge_domain::entities::{ConfigScope, GitAuthor};
    use crate::fonts::FontKitAdapter;
    use crate::git::{Git2StorageAdapter, GitConfigAdapter};
    use crate::themes::EmbeddedThemeAdapter;

    #[tokio::test]
    async fn test_git_storage_lifecycle_in_temp_repo() {
        let temp_dir = std::env::temp_dir().join(format!("rmerge_test_repo_{}", uuid_v4()));
        let _ = fs::create_dir_all(&temp_dir);

        let repo = Repository::init(&temp_dir).expect("Failed to init git repo");
        let storage = Git2StorageAdapter::new();

        // 1. Repo vacío: sin commits
        let history = storage.get_commit_history(&temp_dir, 10).await.unwrap();
        assert!(history.is_empty());

        // 2. Crear un archivo de prueba
        let file_path = temp_dir.join("test.txt");
        fs::write(&file_path, "Hola Mundo\nSegunda linea\n").unwrap();

        // 3. Revisar status: debe aparecer en untracked
        let status = storage.get_status(&temp_dir).await.unwrap();
        assert!(status.untracked.iter().any(|p| p == "test.txt"));

        // 4. Stage y Commit
        let author = GitAuthor::new("Test User", "test@example.com");
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let commit_hash = storage.create_commit(&temp_dir, "Commit inicial de prueba", &author).await.unwrap();
        assert!(!commit_hash.is_empty());

        // 5. Historial: ahora debe contener el commit
        let history = storage.get_commit_history(&temp_dir, 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].message_headline, "Commit inicial de prueba");
        assert_eq!(history[0].author_name, "Test User");

        // Limpieza
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_git_config_local_author_write_and_read() {
        let temp_dir = std::env::temp_dir().join(format!("rmerge_test_cfg_{}", uuid_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let _ = Repository::init(&temp_dir).expect("Failed to init repo");

        let config_adapter = GitConfigAdapter::new();
        let author = GitAuthor::new("Anibal Developer", "anibal.dev@company.com");

        // Inicialmente no hay autor local
        let initial = config_adapter.get_local_author(&temp_dir).await.unwrap();
        assert!(initial.is_none());

        // Guardar a nivel local
        config_adapter.set_author(&temp_dir, &author, ConfigScope::Local).await.unwrap();

        // Leer autor local
        let retrieved = config_adapter.get_local_author(&temp_dir).await.unwrap();
        assert_eq!(retrieved, Some(author.clone()));

        // Limpieza
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_embedded_themes() {
        let theme_adapter = EmbeddedThemeAdapter::new();
        let themes = theme_adapter.list_available_themes().await.unwrap();
        assert!(themes.contains(&"Sublime Dark".to_string()));
        assert!(themes.contains(&"Sublime Light".to_string()));
        assert!(themes.contains(&"Kanagawa Wave".to_string()));
        assert!(themes.contains(&"Kanagawa Dragon".to_string()));
        assert!(themes.contains(&"Kanagawa Lotus".to_string()));
        assert!(themes.contains(&"Gruvbox Dark".to_string()));
        assert!(themes.contains(&"Gruvbox Light".to_string()));
        assert!(themes.contains(&"Dracula".to_string()));
        assert!(themes.contains(&"Nord".to_string()));
        assert!(themes.contains(&"Tokyo Night".to_string()));
        assert!(themes.contains(&"Catppuccin Mocha".to_string()));
        assert!(themes.contains(&"One Dark Pro".to_string()));
        assert!(themes.contains(&"Rose Pine".to_string()));
        assert!(themes.len() >= 15);

        let kanagawa = theme_adapter.get_theme("Kanagawa Wave").await.unwrap();
        assert_eq!(kanagawa.name, "Kanagawa Wave");
        assert!(!kanagawa.branch_lanes.is_empty());
        assert_eq!(kanagawa.accent, "#7e9cd8");

        let gruvbox = theme_adapter.get_theme("Gruvbox Dark").await.unwrap();
        assert_eq!(gruvbox.name, "Gruvbox Dark");
        assert_eq!(gruvbox.accent, "#fe8019");

        let dark = theme_adapter.get_theme("Sublime Dark").await.unwrap();
        assert_eq!(dark.name, "Sublime Dark");
        assert!(!dark.branch_lanes.is_empty());
    }

    #[tokio::test]
    async fn test_font_discovery() {
        let font_adapter = FontKitAdapter::new();
        let mono_fonts = font_adapter.list_monospace_fonts().await.unwrap();
        assert!(!mono_fonts.is_empty(), "Monospace fonts list should not be empty");
    }

    #[tokio::test]
    async fn test_theme_icon_service_colors_match_selected_theme() {
        use crate::icons::ThemeIconService;
        use rmerge_domain::value_objects::{IconId, ThemeConfig};

        let icon_service = ThemeIconService::new();
        let dark_theme = ThemeConfig::dark();
        let light_theme = ThemeConfig::light();

        // 1. Verificar que DiffAdded adopta el verde del tema oscuro
        let dark_svg = icon_service.render_svg(IconId::DiffAdded, &dark_theme);
        assert!(dark_svg.contains(&dark_theme.diff_add_fg));

        // 2. Verificar que DiffAdded adopta el verde del tema claro
        let light_svg = icon_service.render_svg(IconId::DiffAdded, &light_theme);
        assert!(light_svg.contains(&light_theme.diff_add_fg));
        assert_ne!(dark_theme.diff_add_fg, light_theme.diff_add_fg);

        // 3. Exportar iconos temáticos a carpeta de assets
        let export_dir = std::env::temp_dir().join("rmerge_themed_icons_test");
        icon_service.export_themed_icons(&export_dir, &dark_theme).unwrap();
        assert!(export_dir.join("gitcommit.svg").exists());
        let _ = fs::remove_dir_all(&export_dir);
    }

    #[tokio::test]
    async fn test_gitignore_respected_and_editable() {
        let temp_dir = std::env::temp_dir().join(format!("rmerge_test_gitignore_{}", uuid_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let _ = Repository::init(&temp_dir).expect("Failed to init repo");

        let storage = Git2StorageAdapter::new();

        // 1. Inicialmente no hay .gitignore
        let initial_gi = storage.get_gitignore(&temp_dir).await.unwrap();
        assert_eq!(initial_gi, None);

        // 2. Guardar un nuevo .gitignore
        let rules = "*.log\nsecret_dir/\nbuild/\n";
        storage.save_gitignore(&temp_dir, rules).await.unwrap();

        let read_gi = storage.get_gitignore(&temp_dir).await.unwrap();
        assert_eq!(read_gi, Some(rules.to_string()));

        // 3. Crear archivos: uno permitido y dos ignorados
        fs::write(temp_dir.join("allowed.txt"), "codigo fuente valido").unwrap();
        fs::write(temp_dir.join("debug.log"), "archivo de log que debe ignorarse").unwrap();
        let secret_dir = temp_dir.join("secret_dir");
        let _ = fs::create_dir_all(&secret_dir);
        fs::write(secret_dir.join("keys.env"), "clave_secreta").unwrap();

        // 4. Verificar get_status: solo allowed.txt y .gitignore deben aparecer
        let status = storage.get_status(&temp_dir).await.unwrap();
        assert!(status.untracked.iter().any(|p| p == "allowed.txt"));
        assert!(status.untracked.iter().any(|p| p == ".gitignore"));
        assert!(!status.untracked.iter().any(|p| p.ends_with(".log")));
        assert!(!status.untracked.iter().any(|p| p.contains("secret_dir")));

        // 5. Stage All: debe indexar allowed.txt y .gitignore, pero NINGÚN archivo ignorado
        storage.stage_all(&temp_dir).await.unwrap();
        let status_after_stage = storage.get_status(&temp_dir).await.unwrap();
        assert!(status_after_stage.staged.iter().any(|p| p.path == "allowed.txt"));
        assert!(status_after_stage.staged.iter().any(|p| p.path == ".gitignore"));
        assert!(!status_after_stage.staged.iter().any(|p| p.path.ends_with(".log")));
        assert!(!status_after_stage.staged.iter().any(|p| p.path.contains("secret_dir")));

        // 6. Intentar preparar directamente un archivo ignorado debe ser rechazado
        let reject_res = storage.stage_hunk(&temp_dir, std::path::Path::new("debug.log"), 0).await;
        assert!(reject_res.is_err(), "Preparar un archivo ignorado debe fallar");

        // Limpieza
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_clone_repository_local_and_callbacks() {
        let src_dir = std::env::temp_dir().join(format!("rmerge_test_clone_src_{}", uuid_v4()));
        let dst_dir = std::env::temp_dir().join(format!("rmerge_test_clone_dst_{}", uuid_v4()));
        let _ = fs::create_dir_all(&src_dir);

        let repo = Repository::init(&src_dir).expect("Failed to init src repo");
        fs::write(src_dir.join("README.md"), "# Hello Clone").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("README.md")).unwrap();
        index.write().unwrap();

        let storage = Git2StorageAdapter::new();
        let author = GitAuthor::new("Clone Tester", "clone@test.com");
        storage.create_commit(&src_dir, "Commit for clone", &author).await.unwrap();

        // Clonar repo
        let res = storage.clone_repository(src_dir.to_str().unwrap(), &dst_dir).await;
        assert!(res.is_ok(), "El clonado debe completarse con éxito: {:?}", res.err());
        assert!(dst_dir.join(".git").exists(), "El directorio .git debe existir en el destino");
        assert!(dst_dir.join("README.md").exists(), "El archivo clonado debe existir");

        let _ = fs::remove_dir_all(&src_dir);
        let _ = fs::remove_dir_all(&dst_dir);
    }

    #[tokio::test]
    #[ignore]
    async fn test_clone_ssh_real_repo() {
        let dst_dir = std::env::temp_dir().join(format!("rmerge_test_ssh_clone_{}", uuid_v4()));
        let storage = Git2StorageAdapter::new();
        let res = storage.clone_repository("git@github.com:anibalgh/git-client.git", &dst_dir).await;
        println!("SSH Clone Result: {res:?}");
        assert!(res.is_ok(), "Clone failed: {res:?}");
        assert!(dst_dir.join(".git").exists());
        let _ = fs::remove_dir_all(&dst_dir);
    }

    fn uuid_v4() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        format!("{d}")
    }
}
