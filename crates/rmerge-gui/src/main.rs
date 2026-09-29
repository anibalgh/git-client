#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;
mod ipc_server;
mod state;

use std::path::PathBuf;
use std::sync::Arc;
use eframe::egui;

use rmerge_application::ports::out::{AppSettings, SettingsStoragePort, ThemeStoragePort};
use rmerge_application::ports::r#in::{
    AuthorStatus, CloneRepositoryUseCase, CommitChangesUseCase, CommitOutcome, ConfigureAuthorUseCase,
    EnsureAuthorUseCase, InspectCommitUseCase, LoadRepositoryUseCase, ManageBranchesUseCase,
    ManageGitignoreUseCase, ManageStagingUseCase, ManageTagsUseCase, ManageTypographyUseCase, RemoteUrlUseCase, ThreeWayMergeUseCase,
};
use rmerge_application::use_cases::{
    AuthorService, CommitService, CredentialService, MergeService, RepositoryService, StagingService, TypographyService,
};
use rmerge_domain::entities::{
    ConfigScope, ConflictResolution, DeltaKind, FilePatch, GitAuthor,
};
use rmerge_domain::value_objects::{ThemeConfig, ThemeMode};
use rmerge_infrastructure::fonts::FontKitAdapter;
use rmerge_infrastructure::git::{Git2StorageAdapter, GitConfigAdapter};
use rmerge_infrastructure::ipc::IpcMessage;
use rmerge_infrastructure::credentials::EncryptedCredentialAdapter;
use rmerge_infrastructure::settings::JsonSettingsAdapter;
use rmerge_infrastructure::themes::EmbeddedThemeAdapter;

use crate::ipc_server::IpcServer;
use crate::state::{AppState, UnreachableModalData};

#[derive(Debug, Clone)]
pub struct GitCmdResult {
    pub cmd_display: String,
    pub success: bool,
    pub message: String,
    pub unreachable_info: Option<UnreachableModalData>,
}

pub struct RmergeGuiApp {
    state: AppState,
    repo_service: Arc<RepositoryService>,
    commit_service: Arc<CommitService>,
    author_service: Arc<AuthorService>,
    staging_service: Arc<StagingService>,
    merge_service: Arc<MergeService>,
    _typography_service: Arc<TypographyService>,
    credential_service: Arc<CredentialService>,
    theme_storage: Arc<EmbeddedThemeAdapter>,
    settings_storage: Arc<JsonSettingsAdapter>,
    ipc_rx: std::sync::mpsc::Receiver<IpcMessage>,
    selected_file_patch: Option<FilePatch>,
    tokio_handle: tokio::runtime::Handle,
    clone_tx: std::sync::mpsc::Sender<Result<PathBuf, String>>,
    clone_rx: std::sync::mpsc::Receiver<Result<PathBuf, String>>,
    git_cmd_tx: std::sync::mpsc::Sender<GitCmdResult>,
    git_cmd_rx: std::sync::mpsc::Receiver<GitCmdResult>,
}

pub fn extract_remote_host_and_port(raw_url: &str) -> Option<(String, u16)> {
    let url = raw_url.trim();
    if url.is_empty() || url.starts_with('/') || url.starts_with('\\') || url.starts_with("file://") {
        return None;
    }

    // Comprobar si es una ruta local de disco de Windows (e.g. C:\repo o C:/repo)
    let bytes = url.as_bytes();
    if bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
        return None;
    }

    // 1. SSH URL scheme: ssh://[user@]host[:port]/path
    if let Some(rest) = url.strip_prefix("ssh://") {
        let authority = rest.split('/').next().unwrap_or(rest);
        let host_port = authority.split('@').next_back().unwrap_or(authority);
        return parse_host_port(host_port, 22);
    }

    // 2. HTTPS scheme: https://[user:pass@]host[:port]/path
    if let Some(rest) = url.strip_prefix("https://") {
        let authority = rest.split('/').next().unwrap_or(rest);
        let host_port = authority.split('@').next_back().unwrap_or(authority);
        return parse_host_port(host_port, 443);
    }

    // 3. HTTP scheme: http://[user:pass@]host[:port]/path
    if let Some(rest) = url.strip_prefix("http://") {
        let authority = rest.split('/').next().unwrap_or(rest);
        let host_port = authority.split('@').next_back().unwrap_or(authority);
        return parse_host_port(host_port, 80);
    }

    // 4. Git scheme: git://host[:port]/path
    if let Some(rest) = url.strip_prefix("git://") {
        let authority = rest.split('/').next().unwrap_or(rest);
        let host_port = authority.split('@').next_back().unwrap_or(authority);
        return parse_host_port(host_port, 9418);
    }

    // 5. Scp-style SSH syntax: [user@]host:path/to/repo.git
    if !url.contains("://") && url.contains(':') {
        let before_colon = url.split(':').next().unwrap_or("");
        let host = before_colon.split('@').next_back().unwrap_or(before_colon);
        if !host.is_empty() && !host.contains('/') && !host.contains('\\') {
            return Some((host.to_string(), 22));
        }
    }

    None
}

fn parse_host_port(host_port: &str, default_port: u16) -> Option<(String, u16)> {
    let hp = host_port.trim();
    if hp.is_empty() {
        return None;
    }
    // Manejo de IPv6 entre corchetes, ej: [::1]:22
    if hp.starts_with('[') {
        if let Some(end_bracket) = hp.find(']') {
            let host = &hp[1..end_bracket];
            let rest = &hp[end_bracket + 1..];
            if let Some(port_str) = rest.strip_prefix(':') {
                if let Ok(port) = port_str.parse::<u16>() {
                    return Some((host.to_string(), port));
                }
            }
            return Some((host.to_string(), default_port));
        }
    }

    if let Some((host, port_str)) = hp.rsplit_once(':') {
        if let Ok(port) = port_str.parse::<u16>() {
            return Some((host.to_string(), port));
        }
    }

    Some((hp.to_string(), default_port))
}

#[derive(Debug, Clone)]
enum ConnectivityResult {
    Reachable,
    Timeout,
    Error(String),
}

async fn check_remote_connectivity(host: &str, port: u16) -> ConnectivityResult {
    use std::time::Duration;
    use tokio::net::TcpStream;

    let target = format!("{host}:{port}");
    // Timeout breve de 2500ms (2.5 segundos) para no hacer esperar al usuario
    match tokio::time::timeout(Duration::from_millis(2500), TcpStream::connect(&target)).await {
        Ok(Ok(_stream)) => ConnectivityResult::Reachable,
        Ok(Err(e)) => ConnectivityResult::Error(e.to_string()),
        Err(_) => ConnectivityResult::Timeout,
    }
}

impl RmergeGuiApp {
    pub fn new(
        initial_path: PathBuf,
        ipc_rx: std::sync::mpsc::Receiver<IpcMessage>,
    ) -> Self {
        let credential_storage = Arc::new(EncryptedCredentialAdapter::new());
        let credential_service = Arc::new(CredentialService::new(credential_storage.clone()));
        let git_storage = Arc::new(Git2StorageAdapter::with_credentials(credential_storage.clone()));
        let git_config = Arc::new(GitConfigAdapter::new());
        let font_discovery = Arc::new(FontKitAdapter::new());
        let settings_storage = Arc::new(JsonSettingsAdapter::new());
        let theme_storage = Arc::new(EmbeddedThemeAdapter::new());

        let repo_service = Arc::new(RepositoryService::new(git_storage.clone()));
        let commit_service = Arc::new(CommitService::new(git_storage.clone(), git_config.clone()));
        let author_service = Arc::new(AuthorService::new(git_config.clone()));
        let staging_service = Arc::new(StagingService::new(git_storage.clone()));
        let merge_service = Arc::new(MergeService::new(git_storage.clone()));
        let typography_service = Arc::new(TypographyService::new(font_discovery, settings_storage.clone()));

        // 1. Cargar preferencias persistidas desde settings.json
        let loaded_settings = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(settings_storage.load_settings())
        }).unwrap_or_default();

        // 2. Cargar tema persistido o usar oscuro por defecto
        let initial_theme = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(theme_storage.get_theme(&loaded_settings.theme_name))
        }).unwrap_or_else(|_| ThemeConfig::dark());

        let available_themes = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(theme_storage.list_available_themes())
        }).unwrap_or_default();

        // 3. Cargar fuentes disponibles en el SO
        let available_fonts = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(typography_service.get_available_fonts())
        }).unwrap_or_else(|_| rmerge_application::ports::r#in::AvailableFonts {
            system_fonts: vec![
                "DejaVu Sans".into(), "Liberation Sans".into(), "Ubuntu".into(),
                "Segoe UI".into(), "Arial".into(), "SF Pro Text".into(), "Inter".into(),
            ],
            monospace_fonts: vec![
                "JetBrains Mono".into(), "Fira Code".into(), "Cascadia Code".into(), "Consolas".into(),
                "DejaVu Sans Mono".into(), "Liberation Mono".into(), "Ubuntu Mono".into(), "Courier New".into(), "monospace".into(),
            ],
        });

        let mut state = AppState::new(initial_path);
        state.active_theme = initial_theme;
        if !available_themes.is_empty() {
            state.available_themes = available_themes;
        }
        state.available_fonts = available_fonts;
        state.typography = loaded_settings.typography;
        if !state.available_fonts.system_fonts.contains(&state.typography.ui_font.family) {
            state.available_fonts.system_fonts.insert(0, state.typography.ui_font.family.clone());
        }
        if !state.available_fonts.monospace_fonts.contains(&state.typography.code_font.family) {
            state.available_fonts.monospace_fonts.insert(0, state.typography.code_font.family.clone());
        }
        state.recent_repos = loaded_settings.recent_repos;
        state.set_language(&loaded_settings.language);

        let (clone_tx, clone_rx) = std::sync::mpsc::channel();
        let (git_cmd_tx, git_cmd_rx) = std::sync::mpsc::channel();
        let tokio_handle = tokio::runtime::Handle::current();

        let mut app = Self {
            state,
            repo_service,
            commit_service,
            author_service,
            staging_service,
            merge_service,
            _typography_service: typography_service,
            credential_service,
            theme_storage,
            settings_storage,
            ipc_rx,
            selected_file_patch: None,
            tokio_handle,
            clone_tx,
            clone_rx,
            git_cmd_tx,
            git_cmd_rx,
        };

        app.reload_repository();
        app.check_identity();
        app
    }

    fn save_preferences(&self) {
        let settings = AppSettings {
            theme_name: self.state.active_theme.name.clone(),
            typography: self.state.typography.clone(),
            auto_refresh: true,
            recent_repos: self.state.recent_repos.clone(),
            language: self.state.language.clone(),
        };
        let storage = self.settings_storage.clone();
        let _ = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(storage.save_settings(&settings))
        });
    }

    fn apply_custom_fonts(&self, ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        let candidate_paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
            "/System/Library/Fonts/SFPro.ttf",
            "/System/Library/Fonts/SFNS.ttf",
            "/Library/Fonts/Arial.ttf",
        ];

        for path in candidate_paths {
            if let Ok(font_bytes) = std::fs::read(path) {
                fonts.font_data.insert(
                    "system_fallback".to_owned(),
                    Arc::new(egui::FontData::from_owned(font_bytes)),
                );
                if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    prop.push("system_fallback".to_owned());
                }
                if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    mono.push("system_fallback".to_owned());
                }
                break;
            }
        }

        let ui_family = &self.state.typography.ui_font.family;
        if !ui_family.is_empty() {
            if let Some(bytes) = FontKitAdapter::load_font_family_bytes(ui_family) {
                fonts.font_data.insert(
                    "custom_ui".to_owned(),
                    Arc::new(egui::FontData::from_owned(bytes)),
                );
                if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    prop.insert(0, "custom_ui".to_owned());
                }
            }
        }

        let code_family = &self.state.typography.code_font.family;
        if !code_family.is_empty() {
            if let Some(bytes) = FontKitAdapter::load_font_family_bytes(code_family) {
                fonts.font_data.insert(
                    "custom_code".to_owned(),
                    Arc::new(egui::FontData::from_owned(bytes)),
                );
                if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    mono.insert(0, "custom_code".to_owned());
                }
            }
        }

        ctx.set_fonts(fonts);
    }

    fn apply_typography_styles(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        let ui_size = self.state.typography.ui_font.size_pt;
        let code_size = self.state.typography.code_font.size_pt;

        style.text_styles = [
            (egui::TextStyle::Small, egui::FontId::new(ui_size * 0.85, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(ui_size, egui::FontFamily::Proportional)),
            (egui::TextStyle::Button, egui::FontId::new(ui_size, egui::FontFamily::Proportional)),
            (egui::TextStyle::Heading, egui::FontId::new(ui_size * 1.35, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(code_size, egui::FontFamily::Monospace)),
        ].into();

        ctx.set_style(style);
    }

    fn reload_repository(&mut self) {
        let repo_path = self.state.repo_path.clone();
        let svc = self.repo_service.clone();
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(svc.execute(&repo_path))
        });

        match res {
            Ok(data) => {
                // Si había un archivo seleccionado, actualizarlo con los nuevos datos
                if let Some(ref sel) = self.selected_file_patch {
                    let path = sel.path.clone();
                    self.selected_file_patch = data.staging.unstaged.iter()
                        .chain(data.staging.staged.iter())
                        .find(|f| f.path == path)
                        .cloned();
                } else {
                    self.selected_file_patch = data.staging.unstaged.first()
                        .or_else(|| data.staging.staged.first())
                        .cloned();
                }
                self.state.repo_data = Some(data);
                self.state.is_valid_repo = true;
                self.state.set_status("Repositorio actualizado");

                let canonical_str = std::fs::canonicalize(&repo_path)
                    .unwrap_or(repo_path)
                    .to_string_lossy()
                    .to_string();
                self.state.add_recent_repo(canonical_str);
                self.save_preferences();

                // Seleccionar automáticamente el primer commit si no hay uno seleccionado
                if let Some(ref data) = self.state.repo_data {
                    if let Some(first_commit) = data.graph.commits.first() {
                        let cid = self.state.selected_commit_id.clone().unwrap_or_else(|| first_commit.id.clone());
                        self.select_commit(&cid);
                    }
                }
            }
            Err(e) => {
                self.state.repo_data = None;
                self.state.is_valid_repo = false;
                self.selected_file_patch = None;
                self.state.selected_commit_id = None;
                self.state.selected_commit_detail = None;
                self.state.selected_commit_file = None;
                self.state.set_status(format!("No es un repositorio Git válido: {e}"));
            }
        }
    }

    fn select_commit(&mut self, commit_id: &str) {
        self.state.selected_commit_id = Some(commit_id.to_string());
        let repo = self.state.repo_path.clone();
        let svc = self.repo_service.clone();
        let c_id = commit_id.to_string();

        let detail_res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(svc.get_commit_detail(&repo, &c_id))
        });

        match detail_res {
            Ok(detail) => {
                let first_file = detail.files.first().map(|f| f.path.clone());
                self.state.selected_commit_detail = Some(detail);
                if self.state.selected_commit_file.is_none() || !self.state.selected_commit_detail.as_ref().unwrap().files.iter().any(|f| Some(&f.path) == self.state.selected_commit_file.as_ref()) {
                    self.state.selected_commit_file = first_file;
                }
            }
            Err(e) => {
                self.state.set_status(format!("Error al cargar detalle de commit: {e}"));
                self.state.selected_commit_detail = None;
                self.state.selected_commit_file = None;
            }
        }
    }

    fn create_pull_request_in_browser(&mut self) {
        let repo = self.state.repo_path.clone();
        let svc = self.repo_service.clone();
        let branch = self.state.repo_data.as_ref()
            .and_then(|d| d.current_branch.clone())
            .unwrap_or_else(|| "main".to_string());

        let remote_url_opt = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(svc.get_remote_url(&repo))
        }).ok().flatten();

        if let Some(remote_url) = remote_url_opt {
            if let Some(url) = generate_pr_url(&remote_url, &branch) {
                let _ = webbrowser::open(&url);
                self.state.set_status(format!("Abriendo navegador para crear Pull Request / MR en: {url}"));
            } else {
                self.state.set_status(format!("Formato de remote desconocido para crear PR automáticamente: {remote_url}"));
            }
        } else {
            self.state.set_status("No se encontró ningún repositorio remoto configurado (ej: origin)");
        }
    }

    fn open_gitignore_editor(&mut self) {
        let repo = self.state.repo_path.clone();
        let svc = self.repo_service.clone();
        let content_res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(svc.get_gitignore(&repo))
        });

        match content_res {
            Ok(Some(content)) => {
                self.state.gitignore_content = content;
                self.state.gitignore_exists_on_disk = true;
            }
            Ok(None) => {
                self.state.gitignore_content = DEFAULT_GITIGNORE_TEMPLATE.to_string();
                self.state.gitignore_exists_on_disk = false;
            }
            Err(e) => {
                self.state.set_status(format!("Aviso al leer .gitignore: {e}"));
                self.state.gitignore_content = DEFAULT_GITIGNORE_TEMPLATE.to_string();
                self.state.gitignore_exists_on_disk = false;
            }
        }
        self.state.gitignore_save_error = None;
        self.state.show_gitignore_modal = true;
    }

    fn execute_git_cmd(&mut self, args: Vec<&'static str>) {
        if self.state.is_running_git_cmd {
            return;
        }

        let repo_path = self.state.repo_path.clone();
        let cmd_display = format!("git {}", args.join(" "));
        self.state.is_running_git_cmd = true;
        self.state.active_git_cmd_name = Some(cmd_display.clone());
        self.state.set_status(format!("Ejecutando: {cmd_display}..."));

        let tx = self.git_cmd_tx.clone();
        let repo_service = self.repo_service.clone();
        let is_remote_op = args.iter().any(|&a| a == "pull" || a == "push" || a == "fetch");

        self.tokio_handle.spawn(async move {
            // 1. Chequeo de conectividad previa si es un comando remoto (pull, push, fetch)
            if is_remote_op {
                if let Ok(Some(remote_url)) = repo_service.get_remote_url(&repo_path).await {
                    if let Some((host, port)) = extract_remote_host_and_port(&remote_url) {
                        match check_remote_connectivity(&host, port).await {
                            ConnectivityResult::Timeout => {
                                let _ = tx.send(GitCmdResult {
                                    cmd_display: cmd_display.clone(),
                                    success: false,
                                    message: String::new(),
                                    unreachable_info: Some(UnreachableModalData {
                                        cmd: cmd_display,
                                        host,
                                        port,
                                        is_timeout: true,
                                        error_detail: None,
                                    }),
                                });
                                return;
                            }
                            ConnectivityResult::Error(err) => {
                                let _ = tx.send(GitCmdResult {
                                    cmd_display: cmd_display.clone(),
                                    success: false,
                                    message: String::new(),
                                    unreachable_info: Some(UnreachableModalData {
                                        cmd: cmd_display,
                                        host,
                                        port,
                                        is_timeout: false,
                                        error_detail: Some(err),
                                    }),
                                });
                                return;
                            }
                            ConnectivityResult::Reachable => {}
                        }
                    }
                }
            }

            // 2. Ejecutar comando git de forma asíncrona y no interactiva
            let run_fut = async {
                tokio::process::Command::new("git")
                    .args(&args)
                    .current_dir(&repo_path)
                    .env("GIT_TERMINAL_PROMPT", "0")
                    .env("GIT_SSH_COMMAND", "ssh -o ConnectTimeout=5 -o BatchMode=yes")
                    .output()
                    .await
            };

            // Límite de seguridad de 60 segundos
            match tokio::time::timeout(std::time::Duration::from_secs(60), run_fut).await {
                Ok(Ok(out)) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    if out.status.success() {
                        let msg = if !stdout.is_empty() {
                            stdout
                        } else if !stderr.is_empty() {
                            stderr
                        } else {
                            format!("{cmd_display} completado con éxito")
                        };
                        let _ = tx.send(GitCmdResult {
                            cmd_display,
                            success: true,
                            message: msg,
                            unreachable_info: None,
                        });
                    } else {
                        let err = if !stderr.is_empty() { stderr } else { stdout };
                        let _ = tx.send(GitCmdResult {
                            message: format!("Error en {cmd_display}: {err}"),
                            cmd_display,
                            success: false,
                            unreachable_info: None,
                        });
                    }
                }
                Ok(Err(e)) => {
                    let _ = tx.send(GitCmdResult {
                        cmd_display: cmd_display.clone(),
                        success: false,
                        message: format!("Error al ejecutar {cmd_display}: {e}"),
                        unreachable_info: None,
                    });
                }
                Err(_) => {
                    let _ = tx.send(GitCmdResult {
                        cmd_display: cmd_display.clone(),
                        success: false,
                        message: format!("El comando {cmd_display} excedió el tiempo límite (60s) y fue cancelado"),
                        unreachable_info: None,
                    });
                }
            }
        });
    }

    fn check_identity(&mut self) {
        let repo_path = self.state.repo_path.clone();
        let svc = self.author_service.clone();
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(svc.check_author(&repo_path))
        });

        if let Ok(AuthorStatus::Missing { suggested_name, suggested_email }) = res {
            self.state.open_identity_modal(suggested_name, suggested_email);
        }
    }

    fn apply_theme_visuals(&self, ctx: &egui::Context) {
        let theme = &self.state.active_theme;
        let mut visuals = if theme.mode == ThemeMode::Dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        if let Ok(bg_primary) = hex_to_color(&theme.bg_primary) {
            visuals.panel_fill = bg_primary;
            visuals.window_fill = bg_primary;
            visuals.extreme_bg_color = bg_primary;
        }
        if let Ok(bg_sec) = hex_to_color(&theme.bg_secondary) {
            visuals.faint_bg_color = bg_sec;
            visuals.widgets.noninteractive.bg_fill = bg_sec;
        }
        if let Ok(bg_surf) = hex_to_color(&theme.bg_surface) {
            visuals.widgets.inactive.bg_fill = bg_surf;
        }
        if let Ok(border) = hex_to_color(&theme.border_color) {
            visuals.widgets.noninteractive.bg_stroke.color = border;
        }
        if let Ok(accent) = hex_to_color(&theme.accent) {
            visuals.selection.bg_fill = accent;
            visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.25);
            visuals.widgets.active.bg_fill = accent.gamma_multiply(0.4);
        }
        if let Ok(fg_primary) = hex_to_color(&theme.fg_primary) {
            visuals.override_text_color = Some(fg_primary);
            visuals.widgets.noninteractive.fg_stroke.color = fg_primary;
            visuals.widgets.inactive.fg_stroke.color = fg_primary;
        }
        ctx.set_visuals(visuals);
    }
}

fn draw_branch_icon(ui: &mut egui::Ui, color: egui::Color32, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let stroke_w = (size * 0.12).max(1.5);
        let stroke = egui::Stroke::new(stroke_w, color);
        let node_radius = (size * 0.16).max(2.0);

        let p_bottom = egui::pos2(rect.left() + size * 0.30, rect.top() + size * 0.76);
        let p_top = egui::pos2(rect.left() + size * 0.30, rect.top() + size * 0.24);
        let p_branch = egui::pos2(rect.left() + size * 0.74, rect.top() + size * 0.32);

        // Tronco vertical
        painter.line_segment([p_bottom, p_top], stroke);

        // Rama curva hacia el nodo superior derecho
        let p_start_branch = egui::pos2(rect.left() + size * 0.30, rect.top() + size * 0.58);
        let p_ctrl = egui::pos2(rect.left() + size * 0.65, rect.top() + size * 0.58);
        painter.add(egui::epaint::QuadraticBezierShape::from_points_stroke(
            [p_start_branch, p_ctrl, p_branch],
            false,
            egui::Color32::TRANSPARENT,
            stroke,
        ));

        // Nodos circulares
        let fill_bg = ui.visuals().panel_fill;
        painter.circle(p_bottom, node_radius, fill_bg, stroke);
        painter.circle(p_top, node_radius, fill_bg, stroke);
        painter.circle(p_branch, node_radius, fill_bg, stroke);
    }
    response
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let prefix: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        s.to_string()
    }
}

fn draw_copy_button(ui: &mut egui::Ui, tooltip: &str) -> bool {
    let size = egui::vec2(16.0, 16.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let response = response.on_hover_text(tooltip);
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let color = visuals.text_color();
        let stroke = egui::Stroke::new(1.2, color);
        let painter = ui.painter();

        if response.hovered() {
            painter.rect_filled(rect.expand(2.0), 3.0, ui.visuals().widgets.hovered.bg_fill);
        }

        // Rectángulo posterior (arriba y a la derecha, perfectamente vertical)
        let back_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 4.5, rect.top() + 1.5),
            egui::vec2(8.5, 10.5),
        );
        painter.rect_stroke(back_rect, 1.0, stroke, egui::StrokeKind::Inside);

        // Rectángulo frontal (abajo y a la izquierda, perfectamente vertical)
        let front_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 1.5, rect.top() + 4.5),
            egui::vec2(8.5, 10.5),
        );
        painter.rect_filled(front_rect, 1.0, ui.visuals().panel_fill);
        painter.rect_stroke(front_rect, 1.0, stroke, egui::StrokeKind::Inside);
    }
    response.clicked()
}

impl eframe::App for RmergeGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.state.fonts_initialized {
            self.apply_custom_fonts(ctx);
            self.apply_typography_styles(ctx);
            self.state.fonts_initialized = true;
        }
        self.apply_theme_visuals(ctx);
        self.apply_typography_styles(ctx);

        // Actualizar título dinámico de la ventana
        let win_title = if self.state.is_valid_repo {
            if let Some(ref data) = self.state.repo_data {
                let branch_str = data.current_branch.as_deref().unwrap_or("HEAD");
                format!("Git-Client - {} ({})", self.state.repo_path.display(), branch_str)
            } else {
                format!("Git-Client - {}", self.state.repo_path.display())
            }
        } else {
            "Git-Client - Inicio".to_string()
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(win_title));

        // 1. Escuchar eventos IPC provenientes del CLI (`rmerge <path>`)
        while let Ok(msg) = self.ipc_rx.try_recv() {
            match msg {
                IpcMessage::OpenRepository { path } => {
                    self.state.repo_path = PathBuf::from(path);
                    self.reload_repository();
                    self.check_identity();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                IpcMessage::OpenMergeTool { base, local, remote, output } => {
                    let conflict_content = format!(
                        "<<<<<<< LOCAL\n{}\n||||||| BASE\n{}\n=======\n{}\n>>>>>>> REMOTE\n",
                        std::fs::read_to_string(&local).unwrap_or_default(),
                        std::fs::read_to_string(&base).unwrap_or_default(),
                        std::fs::read_to_string(&remote).unwrap_or_default()
                    );
                    let merge_file = rmerge_domain::services::ThreeWayMergeService::parse_conflicted_content(
                        &output,
                        &conflict_content,
                    );
                    self.state.active_merge = Some(merge_file);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                IpcMessage::Blame { file, .. } => {
                    self.state.set_status(format!("Blame para: {file}"));
                }
                IpcMessage::Search { query } => {
                    self.state.set_status(format!("Búsqueda: {query}"));
                }
            }
            ctx.request_repaint();
        }

        // Revisar si terminó alguna tarea de clonación en segundo plano
        if let Ok(res) = self.clone_rx.try_recv() {
            self.state.is_cloning = false;
            match res {
                Ok(dest) => {
                    self.state.show_clone_modal = false;
                    self.state.clone_url.clear();
                    self.state.clone_destination.clear();
                    self.state.clone_error = None;
                    self.state.repo_path = dest;
                    self.reload_repository();
                    self.check_identity();
                    ctx.request_repaint();
                }
                Err(err) => {
                    self.state.clone_error = Some(err);
                    ctx.request_repaint();
                }
            }
        }
        if self.state.is_cloning {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // Revisar si terminó algún comando Git en segundo plano
        while let Ok(res) = self.git_cmd_rx.try_recv() {
            self.state.is_running_git_cmd = false;
            self.state.active_git_cmd_name = None;
            if res.success {
                self.state.set_status(format!("✔ {}", res.message));
                self.reload_repository();
            } else if let Some(unreachable) = res.unreachable_info {
                let failure_detail = if unreachable.is_timeout {
                    self.state.i18n.server_unreachable_timeout
                        .replace("{host}", &unreachable.host)
                        .replace("{port}", &unreachable.port.to_string())
                } else {
                    self.state.i18n.server_unreachable_refused
                        .replace("{host}", &unreachable.host)
                        .replace("{port}", &unreachable.port.to_string())
                        .replace("{detail}", unreachable.error_detail.as_deref().unwrap_or(""))
                };
                self.state.set_status(format!("❌ {failure_detail}"));
                self.state.server_unreachable_modal = Some(unreachable);
            } else {
                self.state.set_status(format!("❌ {}", res.message));
            }
            ctx.request_repaint();
        }
        if self.state.is_running_git_cmd {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // 2. Barra Superior (Header & Breadcrumbs)
        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                draw_branch_icon(ui, egui::Color32::from_rgb(97, 175, 239), 16.0);
                ui.label(egui::RichText::new("Git-Client").strong().color(egui::Color32::from_rgb(97, 175, 239)));
                ui.separator();

                if self.state.is_valid_repo {
                    ui.label(format!("📁 {}", self.state.repo_path.display()));

                    if let Some(ref data) = self.state.repo_data {
                        if let Some(ref branch) = data.current_branch {
                            ui.separator();
                            draw_branch_icon(ui, egui::Color32::from_rgb(152, 195, 121), 14.0);
                            ui.label(egui::RichText::new(branch).strong().color(egui::Color32::from_rgb(152, 195, 121)));

                            // Botón de Branch Graphic sólo con icono y tooltip
                            ui.add_space(4.0);
                            if ui.selectable_label(self.state.show_right_graph_panel, "📊")
                                .on_hover_text(&self.state.i18n.toggle_graph_tooltip)
                                .clicked()
                            {
                                self.state.show_right_graph_panel = !self.state.show_right_graph_panel;
                            }
                        }
                    }
                } else {
                    ui.label(egui::RichText::new("Inicio - Repositorios").italics().weak());
                }

                // Menú único de tres rayas (Hamburger Menu) al final (derecha)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let hamburger_btn = ui.menu_button(egui::RichText::new("☰").size(16.0).strong(), |ui| {
                        ui.set_min_width(170.0);

                        if self.state.is_valid_repo {
                            if ui.button(&self.state.i18n.home).clicked() {
                                self.state.is_valid_repo = false;
                                self.state.repo_data = None;
                                ui.close_menu();
                            }
                            if ui.button(&self.state.i18n.refresh).clicked() {
                                self.reload_repository();
                                ui.close_menu();
                            }
                            ui.separator();
                        }

                        if ui.button(&self.state.i18n.open_repo).clicked() {
                            if let Some(folder) = rfd::FileDialog::new()
                                .set_title(&self.state.i18n.open_repo_dialog_title)
                                .pick_folder()
                            {
                                self.state.repo_path = folder;
                                self.reload_repository();
                                self.check_identity();
                            }
                            ui.close_menu();
                        }

                        if ui.button(if self.state.language == "en" { "📥 Clone..." } else { "📥 Clonar..." }).clicked() {
                            self.state.show_clone_modal = true;
                            self.state.clone_error = None;
                            ui.close_menu();
                        }

                        ui.separator();

                        if ui.button(if self.state.language == "en" { "⚙ Preferences" } else { "⚙ Preferencias" }).clicked() {
                            self.state.show_preferences = !self.state.show_preferences;
                            ui.close_menu();
                        }

                        if ui.button(&self.state.i18n.about_btn).clicked() {
                            self.state.show_about_modal = true;
                            ui.close_menu();
                        }
                    });
                    hamburger_btn.response.on_hover_text(if self.state.language == "en" { "Main Menu" } else { "Menú Principal" });
                });
            });
        });

        // Barra Inferior de Estado (Status Bar)
        egui::TopBottomPanel::bottom("bottom_status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.state.is_running_git_cmd {
                    ui.add(egui::Spinner::new().size(14.0));
                    let cmd_name = self.state.active_git_cmd_name.as_deref().unwrap_or("git...");
                    ui.label(
                        egui::RichText::new(format!("{} {}...", self.state.i18n.running_command, cmd_name))
                            .color(egui::Color32::from_rgb(97, 175, 239))
                            .strong(),
                    );
                } else if !self.state.status_messages.is_empty() {
                    let count = self.state.status_messages.len();
                    let (status_text, color) = if count > 1 {
                        (format!("💬 {}", self.state.i18n.status_messages_count.replace("{count}", &count.to_string())), egui::Color32::from_rgb(97, 175, 239))
                    } else {
                        let msg = &self.state.status_messages[0];
                        let col = if msg.starts_with('✔') {
                            egui::Color32::from_rgb(152, 195, 121)
                        } else if msg.starts_with('❌') {
                            egui::Color32::from_rgb(224, 108, 117)
                        } else if msg.starts_with("⚠️") {
                            egui::Color32::from_rgb(229, 192, 123)
                        } else {
                            ui.visuals().text_color()
                        };
                        (msg.clone(), col)
                    };

                    let resp = ui.add(
                        egui::Label::new(egui::RichText::new(status_text).color(color))
                            .sense(egui::Sense::click())
                    ).on_hover_text(&self.state.i18n.status_messages_tooltip);

                    if resp.double_clicked() {
                        self.state.show_status_messages_modal = true;
                    }

                    if ui.small_button("✖").on_hover_text(&self.state.i18n.status_messages_clear_all).clicked() {
                        self.state.clear_status_messages();
                    }
                } else {
                    ui.weak("Listo");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.state.is_valid_repo {
                        if let Some(ref data) = self.state.repo_data {
                            if let Some(ref branch) = data.current_branch {
                                ui.label(egui::RichText::new(format!("🌿 {branch}")).weak());
                                ui.separator();
                            }
                        }
                    }
                    ui.label(egui::RichText::new("UTF-8").weak());
                });
            });
        });

        if self.state.is_valid_repo && self.state.repo_data.is_some() {
            // 3. Panel Lateral Izquierdo (Ramas, Staging, Archivos Modificados, Commit)
            egui::SidePanel::left("left_sidebar").default_width(320.0).width_range(250.0..=500.0).show(ctx, |ui| {
                ui.heading(&self.state.i18n.git_explorer);
                ui.separator();

                let mut action_select_file: Option<FilePatch> = None;
                let mut action_unstage_path: Option<String> = None;
                let mut action_stage_path: Option<String> = None;
                let mut action_resolve_conflict: Option<String> = None;
                let mut action_stage_all = false;
                let mut action_unstage_all = false;
                let mut action_switch_branch: Option<String> = None;
                let mut action_delete_tag: Option<String> = None;
                let mut action_git_cmd: Option<Vec<&'static str>> = None;

                let mut action_open_gitignore = false;
                let mut action_open_new_branch = false;
                let mut action_open_switch_branch = false;
                let mut action_open_merge = false;
                let mut action_open_new_tag = false;
                let mut action_open_create_pr = false;
                let mut action_reload_repo = false;

                let i18n = self.state.i18n.clone();

                // Submenú de Acciones de la Rama (Menú de la Rama)
                ui.horizontal(|ui| {
                    ui.menu_button(&i18n.branch_menu, |ui| {
                        if ui.button(&i18n.menu_edit_gitignore).clicked() {
                            action_open_gitignore = true;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.menu_new_branch).clicked() {
                            action_open_new_branch = true;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.menu_switch_branch).clicked() {
                            action_open_switch_branch = true;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.menu_merge).clicked() {
                            action_open_merge = true;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.menu_new_tag).clicked() {
                            action_open_new_tag = true;
                            ui.close_menu();
                        }
                        if ui.button(&i18n.menu_create_pr).clicked() {
                            action_open_create_pr = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(&i18n.menu_refresh).clicked() {
                            action_reload_repo = true;
                            ui.close_menu();
                        }
                    });

                    if ui.button("🔄").on_hover_text(&i18n.refresh_tooltip).clicked() {
                        action_reload_repo = true;
                    }
                });

                if action_open_gitignore {
                    self.open_gitignore_editor();
                }
                if action_open_new_branch {
                    self.state.show_create_branch_modal = true;
                    self.state.new_branch_name.clear();
                    self.state.new_branch_start = self.state.repo_data.as_ref().and_then(|d| d.current_branch.clone()).unwrap_or_else(|| "HEAD".to_string());
                    self.state.create_branch_error = None;
                }
                if action_open_switch_branch {
                    self.state.show_switch_branch_modal = true;
                    self.state.switch_branch_error = None;
                }
                if action_open_merge {
                    self.state.show_merge_modal = true;
                    self.state.merge_error = None;
                }
                if action_open_new_tag {
                    self.state.show_create_tag_modal = true;
                    self.state.new_tag_name.clear();
                    self.state.new_tag_message.clear();
                    self.state.new_tag_target = self.state.repo_data.as_ref().and_then(|d| d.current_branch.clone()).unwrap_or_else(|| "HEAD".to_string());
                    self.state.create_tag_error = None;
                }
                if action_open_create_pr {
                    self.create_pull_request_in_browser();
                }
                if action_reload_repo {
                    self.reload_repository();
                }

                ui.add_space(4.0);

                egui::ScrollArea::vertical().id_salt("sidebar_scroll").show(ui, |ui| {
                    if let Some(ref data) = self.state.repo_data {
                        // 1. Ramas
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{} ({})", self.state.i18n.branches_header, data.branches.len())).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("+").on_hover_text(&self.state.i18n.new_branch_tooltip).clicked() {
                                    self.state.show_create_branch_modal = true;
                                    self.state.new_branch_name.clear();
                                    self.state.new_branch_start = data.current_branch.clone().unwrap_or_else(|| "HEAD".to_string());
                                    self.state.create_branch_error = None;
                                }
                            });
                        });

                        egui::CollapsingHeader::new(&self.state.i18n.branch_list)
                            .default_open(true)
                            .id_salt("sidebar_branches_collapse")
                            .show(ui, |ui| {
                                for b in &data.branches {
                                    ui.horizontal(|ui| {
                                        let branch_color = if b.is_head {
                                            egui::Color32::from_rgb(97, 175, 239)
                                        } else {
                                            ui.visuals().text_color()
                                        };
                                        draw_branch_icon(ui, branch_color, 12.0);
                                        let label = if b.is_head { format!("* {}", b.name) } else { format!("  {}", b.name) };
                                        let mut text = egui::RichText::new(label);
                                        if b.is_head {
                                            text = text.strong().color(branch_color);
                                        }
                                        ui.label(text);

                                        if b.is_head {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if self.state.is_running_git_cmd {
                                                    ui.add(egui::Spinner::new().size(12.0));
                                                }
                                                ui.add_enabled_ui(!self.state.is_running_git_cmd, |ui| {
                                                    let push_menu = ui.menu_button("⬆▾", |ui| {
                                                        if ui.button("push").clicked() {
                                                            action_git_cmd = Some(vec!["push"]);
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("push --force").clicked() {
                                                            action_git_cmd = Some(vec!["push", "--force"]);
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("push --force-with-lease").clicked() {
                                                            action_git_cmd = Some(vec!["push", "--force-with-lease"]);
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("push --no-verify").clicked() {
                                                            action_git_cmd = Some(vec!["push", "--no-verify"]);
                                                            ui.close_menu();
                                                        }
                                                    });
                                                    push_menu.response.on_hover_text(&self.state.i18n.push_actions_tooltip);

                                                    let pull_menu = ui.menu_button("⬇▾", |ui| {
                                                        if ui.button("pull").clicked() {
                                                            action_git_cmd = Some(vec!["pull"]);
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("pull --ff-only").clicked() {
                                                            action_git_cmd = Some(vec!["pull", "--ff-only"]);
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("pull --rebase").clicked() {
                                                            action_git_cmd = Some(vec!["pull", "--rebase"]);
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("pull --rebase --autostash").clicked() {
                                                            action_git_cmd = Some(vec!["pull", "--rebase", "--autostash"]);
                                                            ui.close_menu();
                                                        }
                                                        ui.separator();
                                                        if ui.button("fetch").clicked() {
                                                            action_git_cmd = Some(vec!["fetch"]);
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("fetch --tags").clicked() {
                                                            action_git_cmd = Some(vec!["fetch", "--tags"]);
                                                            ui.close_menu();
                                                        }
                                                        if ui.button("fetch --prune").clicked() {
                                                            action_git_cmd = Some(vec!["fetch", "--prune"]);
                                                            ui.close_menu();
                                                        }
                                                    });
                                                    pull_menu.response.on_hover_text(&self.state.i18n.pull_actions_tooltip);
                                                });
                                            });
                                        } else if !b.is_remote {
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button(&self.state.i18n.switch_branch_btn).on_hover_text(format!("{}: '{}'", self.state.i18n.switch_branch_tooltip, b.name)).clicked() {
                                                    action_switch_branch = Some(b.name.clone());
                                                }
                                            });
                                        }
                                    });
                                }
                            });

                        ui.add_space(8.0);

                        // 2. Tags
                        let tags_count = data.tags.len();
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{} ({tags_count})", self.state.i18n.tags_header)).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("+").on_hover_text(&self.state.i18n.new_tag_tooltip).clicked() {
                                    self.state.show_create_tag_modal = true;
                                    self.state.new_tag_name.clear();
                                    self.state.new_tag_message.clear();
                                    self.state.new_tag_target = data.current_branch.clone().unwrap_or_else(|| "HEAD".to_string());
                                    self.state.create_tag_error = None;
                                }
                            });
                        });

                        egui::CollapsingHeader::new(&self.state.i18n.tag_list)
                            .default_open(true)
                            .id_salt("sidebar_tags_collapse")
                            .show(ui, |ui| {
                                if data.tags.is_empty() {
                                    ui.label(egui::RichText::new(&self.state.i18n.no_tags).italics().weak());
                                } else {
                                    for tag in &data.tags {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("🏷").size(12.0));
                                            ui.label(egui::RichText::new(&tag.name).strong());
                                            let short = if tag.target_commit_id.len() >= 7 { &tag.target_commit_id[..7] } else { &tag.target_commit_id };
                                            ui.label(egui::RichText::new(short).monospace().weak().size(11.0));

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("🗑").on_hover_text(&self.state.i18n.delete_tag_tooltip).clicked() {
                                                    action_delete_tag = Some(tag.name.clone());
                                                }
                                            });
                                        });
                                    }
                                }
                            });

                        ui.add_space(8.0);

                        // Conflictos
                        if !data.staging.conflicts.is_empty() {
                            ui.label(egui::RichText::new(format!("{} ({})", self.state.i18n.conflicts_header, data.staging.conflicts.len()))
                                .color(egui::Color32::from_rgb(224, 108, 117)).strong());
                            for c in &data.staging.conflicts {
                                if ui.button(format!("{} {c}", self.state.i18n.resolve_conflict)).clicked() {
                                    action_resolve_conflict = Some(c.clone());
                                }
                            }
                            ui.add_space(8.0);
                        }

                        // 3. Archivos Preparados (Staged)
                        let staged_count = data.staging.staged.len();
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{} ({staged_count})", self.state.i18n.staged_header)).strong());
                            if staged_count > 0 {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("--").on_hover_text(&self.state.i18n.unstage_all_tooltip).clicked() {
                                        action_unstage_all = true;
                                    }
                                });
                            }
                        });

                        egui::CollapsingHeader::new(&self.state.i18n.staged_view)
                            .default_open(true)
                            .id_salt("sidebar_staged_collapse")
                            .show(ui, |ui| {
                                if data.staging.staged.is_empty() {
                                    ui.label(egui::RichText::new(&self.state.i18n.no_staged_changes).italics().weak());
                                } else {
                                    for file in &data.staging.staged {
                                        ui.horizontal(|ui| {
                                            let is_selected = self.selected_file_patch.as_ref().is_some_and(|f| f.path == file.path && f.is_staged);
                                            if ui.selectable_label(is_selected, format!("+ {}", file.path)).clicked() {
                                                action_select_file = Some(file.clone());
                                            }
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("-").on_hover_text(&self.state.i18n.unstage_file_tooltip).clicked() {
                                                    action_unstage_path = Some(file.path.clone());
                                                }
                                            });
                                        });
                                    }
                                }
                            });

                        ui.add_space(8.0);

                        // 4. Archivos Modificados (Unstaged) - Lista con botón '+' por archivo
                        let unstaged_count = data.staging.unstaged.len();
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{} ({unstaged_count})", self.state.i18n.unstaged_header)).strong());
                            if unstaged_count > 0 || !data.staging.untracked.is_empty() {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("++").on_hover_text(&self.state.i18n.stage_all_tooltip).clicked() {
                                        action_stage_all = true;
                                    }
                                });
                            }
                        });

                        egui::CollapsingHeader::new(&self.state.i18n.unstaged_view)
                            .default_open(true)
                            .id_salt("sidebar_unstaged_collapse")
                            .show(ui, |ui| {
                                if data.staging.unstaged.is_empty() {
                                    ui.label(egui::RichText::new(&self.state.i18n.no_unstaged_changes).italics().weak());
                                } else {
                                    for file in &data.staging.unstaged {
                                        ui.horizontal(|ui| {
                                            let is_selected = self.selected_file_patch.as_ref().is_some_and(|f| f.path == file.path && !f.is_staged);
                                            if ui.selectable_label(is_selected, format!("M {}", file.path)).clicked() {
                                                action_select_file = Some(file.clone());
                                            }
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("+").on_hover_text(format!("{}: {}", self.state.i18n.stage_file_tooltip, file.path)).clicked() {
                                                    action_stage_path = Some(file.path.clone());
                                                }
                                            });
                                        });
                                    }
                                }
                            });

                        // Untracked
                        if !data.staging.untracked.is_empty() {
                            ui.add_space(8.0);
                            egui::CollapsingHeader::new(format!("{} ({})", self.state.i18n.untracked_header, data.staging.untracked.len()))
                                .default_open(false)
                                .show(ui, |ui| {
                                    for file in &data.staging.untracked {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("? {file}"));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("+").on_hover_text(&self.state.i18n.stage_untracked_tooltip).clicked() {
                                                    action_stage_path = Some(file.clone());
                                                }
                                            });
                                        });
                                    }
                                });
                        }
                    }
                });

                // Aplicar acciones acumuladas
                if let Some(file) = action_select_file {
                    self.selected_file_patch = Some(file);
                    self.state.selected_commit_id = None;
                    self.state.selected_commit_detail = None;
                }
                if action_stage_all {
                    let repo = self.state.repo_path.clone();
                    let svc = self.staging_service.clone();
                    let _ = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(svc.stage_all(&repo))
                    });
                    self.reload_repository();
                }
                if action_unstage_all {
                    let repo = self.state.repo_path.clone();
                    let svc = self.staging_service.clone();
                    let _ = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(svc.unstage_all(&repo))
                    });
                    self.reload_repository();
                }
                if let Some(target_branch) = action_switch_branch {
                    let repo = self.state.repo_path.clone();
                    let svc = self.repo_service.clone();
                    let res = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(svc.switch_branch(&repo, &target_branch))
                    });
                    match res {
                        Ok(_) => {
                            self.state.set_status(format!("Cambiado a rama: {target_branch}"));
                            self.reload_repository();
                        }
                        Err(e) => {
                            self.state.set_status(format!("Error al cambiar de rama: {e}"));
                        }
                    }
                }
                if let Some(tag_to_delete) = action_delete_tag {
                    let repo = self.state.repo_path.clone();
                    let svc = self.repo_service.clone();
                    let res = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(svc.delete_tag(&repo, &tag_to_delete))
                    });
                    match res {
                        Ok(_) => {
                            self.state.set_status(format!("Tag '{tag_to_delete}' eliminado"));
                            self.reload_repository();
                        }
                        Err(e) => {
                            self.state.set_status(format!("Error al eliminar tag: {e}"));
                        }
                    }
                }
                if let Some(path_str) = action_unstage_path {
                    let repo = self.state.repo_path.clone();
                    let file_path = PathBuf::from(path_str);
                    let svc = self.staging_service.clone();
                    let _ = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(svc.unstage_lines(&repo, &file_path, 0, &[]))
                    });
                    self.reload_repository();
                }
                if let Some(path_str) = action_stage_path {
                    let repo = self.state.repo_path.clone();
                    let file_path = PathBuf::from(path_str);
                    let svc = self.staging_service.clone();
                    let _ = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(svc.stage_hunk(&repo, &file_path, 0))
                    });
                    self.reload_repository();
                }
                if let Some(conflict_path) = action_resolve_conflict {
                    let repo = self.state.repo_path.clone();
                    let svc = self.merge_service.clone();
                    let file_path = PathBuf::from(conflict_path);
                    if let Ok(mf) = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(svc.load_conflicted_file(&repo, &file_path))
                    }) {
                        self.state.active_merge = Some(mf);
                    }
                }
                if let Some(cmd_args) = action_git_cmd {
                    self.execute_git_cmd(cmd_args);
                }

                // Commit Box abajo del panel lateral (solo visible y activo si hay archivos preparados)
                let has_staged = self.state.repo_data.as_ref().is_some_and(|d| !d.staging.staged.is_empty());
                if has_staged {
                    ui.separator();
                    let ui_size = self.state.typography.ui_font.size_pt;
                    ui.label(egui::RichText::new(&self.state.i18n.commit_message_label).strong().size(ui_size).family(egui::FontFamily::Proportional));
                    ui.add(egui::TextEdit::multiline(&mut self.state.commit_message)
                        .font(egui::FontId::new(ui_size, egui::FontFamily::Proportional))
                        .hint_text(&self.state.i18n.commit_message_hint)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY));

                    if ui.button(egui::RichText::new(&self.state.i18n.commit_button).strong()).clicked() {
                        let repo = self.state.repo_path.clone();
                        let msg = self.state.commit_message.clone();
                        let svc = self.commit_service.clone();

                        let res = tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(svc.execute(&repo, &msg))
                        });

                        match res {
                            Ok(CommitOutcome::Success { commit_hash }) => {
                                self.state.commit_message.clear();
                                self.state.set_status(format!("Commit realizado con éxito: {commit_hash}"));
                                self.reload_repository();
                            }
                            Ok(CommitOutcome::AuthorRequired) => {
                                self.check_identity();
                            }
                            Ok(CommitOutcome::EmptyStaging) => {
                                self.state.set_status("Aviso: No hay cambios preparados (staged) para comitear.");
                            }
                            Err(e) => {
                                self.state.set_status(format!("Error en commit: {e}"));
                            }
                        }
                    }
                }
        });

        let mut commit_to_select: Option<String> = None;

        // 4. Panel Lateral Derecho: Grafo de Cambios en la Rama Seleccionada (Puro Gráfico Visual)
        if self.state.show_right_graph_panel {
            egui::SidePanel::right("right_graph_panel")
                .resizable(true)
                .default_width(380.0)
                .min_width(280.0)
                .max_width(650.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&self.state.i18n.branch_graph_title);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("✕").on_hover_text(&self.state.i18n.branch_graph_close).clicked() {
                                self.state.show_right_graph_panel = false;
                            }
                        });
                    });
                    ui.separator();

                    if let Some(ref data) = self.state.repo_data {
                        // Selector de Rama para el Grafo
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&self.state.i18n.branch_graph_selector).strong());
                            let current_selected = self.state.selected_branch_for_graph.clone()
                                .or_else(|| data.current_branch.clone())
                                .unwrap_or_else(|| "HEAD".to_string());

                            egui::ComboBox::from_id_salt("right_graph_branch_combo")
                                .selected_text(&current_selected)
                                .show_ui(ui, |ui| {
                                    for b in &data.branches {
                                        let is_sel = current_selected == b.name;
                                        if ui.selectable_label(is_sel, &b.name).clicked() {
                                            self.state.selected_branch_for_graph = Some(b.name.clone());
                                        }
                                    }
                                });
                        });

                        // Leyenda visual del grafo
                        ui.add_space(2.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(egui::RichText::new(&self.state.i18n.graph_legend_standard).size(10.5).color(egui::Color32::from_rgb(97, 175, 239)));
                            ui.label(egui::RichText::new(&self.state.i18n.graph_legend_merge).size(10.5).color(egui::Color32::from_rgb(224, 108, 117)));
                            ui.label(egui::RichText::new(&self.state.i18n.graph_legend_head).size(10.5).color(egui::Color32::from_rgb(152, 195, 121)));
                        });
                        ui.separator();

                        let commits = &data.graph.commits;
                        if commits.is_empty() {
                            ui.label(egui::RichText::new(&self.state.i18n.no_tags).italics().weak());
                        } else {
                            let row_height = 32.0;
                            let lane_width = 18.0;
                            let x_start = 18.0;
                            let max_lanes = data.graph.max_lanes.max(1);
                            let labels_x_offset = x_start + (max_lanes as f32 + 1.0) * lane_width;
                            let total_height = 16.0 + (commits.len() as f32) * row_height;
                            let total_width = (labels_x_offset + 320.0).max(ui.available_width());

                            egui::ScrollArea::both().id_salt("right_graph_canvas_scroll").show(ui, |ui| {
                                let (response, painter) = ui.allocate_painter(egui::vec2(total_width, total_height), egui::Sense::hover());
                                let min_pos = response.rect.min;

                                // 1. Conexiones (Bifurcaciones y Uniones)
                                for (i, commit) in commits.iter().enumerate() {
                                    let cx = min_pos.x + x_start + (commit.lane as f32) * lane_width;
                                    let cy = min_pos.y + 16.0 + (i as f32) * row_height;

                                    let stroke_color = self.state.active_theme.branch_lanes
                                        .get(commit.lane % self.state.active_theme.branch_lanes.len().max(1))
                                        .and_then(|hex| hex_to_color(hex).ok())
                                        .unwrap_or(egui::Color32::from_rgb(97, 175, 239));

                                    for (parent_idx, parent_hash) in commit.parent_ids.iter().enumerate() {
                                        if let Some(j) = commits.iter().position(|c| c.id == *parent_hash) {
                                            let parent_commit = &commits[j];
                                            let px = min_pos.x + x_start + (parent_commit.lane as f32) * lane_width;
                                            let py = min_pos.y + 16.0 + (j as f32) * row_height;

                                            let is_merge_edge = parent_idx > 0;
                                            let edge_color = if is_merge_edge {
                                                egui::Color32::from_rgb(224, 108, 117)
                                            } else {
                                                stroke_color
                                            };
                                            let stroke = egui::Stroke::new(2.0, edge_color);

                                            if (cx - px).abs() < 1.0 {
                                                painter.line_segment([egui::pos2(cx, cy), egui::pos2(px, py)], stroke);
                                            } else {
                                                let mid_y = cy + (py - cy) * 0.5;
                                                let p0 = egui::pos2(cx, cy);
                                                let p1 = egui::pos2(cx, mid_y);
                                                let p2 = egui::pos2(px, mid_y);
                                                let p3 = egui::pos2(px, py);
                                                let curve = egui::epaint::CubicBezierShape::from_points_stroke([p0, p1, p2, p3], false, egui::Color32::TRANSPARENT, stroke);
                                                painter.add(curve);
                                            }
                                        }
                                    }
                                }

                                // 2. Nodos de commit y badges
                                for (i, commit) in commits.iter().enumerate() {
                                    let cx = min_pos.x + x_start + (commit.lane as f32) * lane_width;
                                    let cy = min_pos.y + 16.0 + (i as f32) * row_height;

                                    let is_merge = commit.parent_ids.len() > 1;
                                    let lane_color = self.state.active_theme.branch_lanes
                                        .get(commit.lane % self.state.active_theme.branch_lanes.len().max(1))
                                        .and_then(|hex| hex_to_color(hex).ok())
                                        .unwrap_or(egui::Color32::from_rgb(97, 175, 239));

                                    if is_merge {
                                        painter.circle_filled(egui::pos2(cx, cy), 6.5, lane_color);
                                        painter.circle_filled(egui::pos2(cx, cy), 3.0, egui::Color32::WHITE);
                                    } else {
                                        painter.circle_filled(egui::pos2(cx, cy), 4.5, lane_color);
                                        painter.circle_stroke(egui::pos2(cx, cy), 4.5, egui::Stroke::new(1.0, egui::Color32::from_gray(200)));
                                    }

                                    let mut curr_x = min_pos.x + labels_x_offset;

                                    for br in &commit.branches {
                                        let badge_w = br.len() as f32 * 7.0 + 14.0;
                                        let badge_rect = egui::Rect::from_min_size(egui::pos2(curr_x, cy - 8.0), egui::vec2(badge_w, 16.0));
                                        painter.rect_filled(badge_rect, 3.0, egui::Color32::from_rgb(38, 70, 83));
                                        painter.text(
                                            egui::pos2(curr_x + 6.0, cy),
                                            egui::Align2::LEFT_CENTER,
                                            format!("🌿 {br}"),
                                            egui::FontId::proportional(10.5),
                                            egui::Color32::from_rgb(152, 195, 121),
                                        );
                                        curr_x += badge_w + 6.0;
                                    }

                                    for tag in &commit.tags {
                                        let tag_w = tag.len() as f32 * 7.0 + 14.0;
                                        let tag_rect = egui::Rect::from_min_size(egui::pos2(curr_x, cy - 8.0), egui::vec2(tag_w, 16.0));
                                        painter.rect_filled(tag_rect, 3.0, egui::Color32::from_rgb(80, 60, 20));
                                        painter.text(
                                            egui::pos2(curr_x + 6.0, cy),
                                            egui::Align2::LEFT_CENTER,
                                            format!("🏷 {tag}"),
                                            egui::FontId::proportional(10.5),
                                            egui::Color32::from_rgb(229, 192, 123),
                                        );
                                        curr_x += tag_w + 6.0;
                                    }

                                    let hash_color = hex_to_color(&self.state.active_theme.accent)
                                        .unwrap_or(egui::Color32::from_rgb(97, 175, 239));
                                    painter.text(
                                        egui::pos2(curr_x, cy),
                                        egui::Align2::LEFT_CENTER,
                                        &commit.short_id,
                                        egui::FontId::monospace(11.0),
                                        hash_color,
                                    );
                                    curr_x += 54.0;

                                    let ui_size = self.state.typography.ui_font.size_pt;
                                    let msg_display = truncate_chars(&commit.message_headline, 28);
                                    let muted_col = hex_to_color(&self.state.active_theme.fg_muted)
                                        .unwrap_or(egui::Color32::from_rgb(171, 178, 191));
                                    painter.text(
                                        egui::pos2(curr_x, cy),
                                        egui::Align2::LEFT_CENTER,
                                        msg_display,
                                        egui::FontId::new(ui_size, egui::FontFamily::Proportional),
                                        muted_col,
                                    );

                                    // Tooltip interactivo informativo (sin botones)
                                    if let Some(hover_pos) = response.hover_pos() {
                                        let row_rect = egui::Rect::from_min_size(
                                            egui::pos2(min_pos.x, cy - row_height * 0.5),
                                            egui::vec2(total_width, row_height),
                                        );
                                        if row_rect.contains(hover_pos) {
                                            painter.rect_filled(row_rect, 4.0, egui::Color32::from_white_alpha(12));
                                            response.clone().on_hover_ui_at_pointer(|ui| {
                                                ui.label(
                                                    egui::RichText::new(&commit.message_headline)
                                                        .strong()
                                                        .size(ui_size)
                                                        .family(egui::FontFamily::Proportional),
                                                );
                                                ui.label(format!("Commit: {}", commit.id));
                                                if is_merge {
                                                    ui.label(egui::RichText::new("◈ Commit de Unión (Merge)").strong().color(egui::Color32::from_rgb(224, 108, 117)));
                                                    ui.label(format!("Padres: {}", commit.parent_ids.join(", ")));
                                                } else if commit.parent_ids.is_empty() {
                                                    ui.label(egui::RichText::new("(Commit Raíz)").italics().weak());
                                                }
                                                if !commit.branches.is_empty() {
                                                    ui.label(format!("Ramas: {}", commit.branches.join(", ")));
                                                }
                                                if !commit.tags.is_empty() {
                                                    ui.label(format!("Tags: {}", commit.tags.join(", ")));
                                                }
                                                ui.label(format!("Autor: {} <{}>", commit.author_name, commit.author_email));
                                                ui.label(format!("Fecha: {}", commit.authored_at.format("%Y-%m-%d %H:%M:%S")));
                                            });
                                        }
                                    }
                                }
                            });
                        }
                    }
                });
        }

        // 5. Panel Central: Historial de Commits (arriba) y Detalle de Commit / Diff Inspector (abajo)
        egui::CentralPanel::default().show(ctx, |ui| {
            // Sección Superior: Historial de Commits
            ui.horizontal(|ui| {
                ui.heading(&self.state.i18n.commit_history_title);
                if let Some(ref data) = self.state.repo_data {
                    ui.label(egui::RichText::new(format!("({} {})", data.graph.commits.len(), self.state.i18n.commits_count)).weak());
                }
            });

            egui::ScrollArea::vertical().max_height(220.0).id_salt("commits_graph_scroll").show(ui, |ui| {
                if let Some(ref data) = self.state.repo_data {
                    egui::Grid::new("commit_table").striped(true).min_col_width(60.0).show(ui, |ui| {
                        ui.label(egui::RichText::new(&self.state.i18n.graph_col).strong());
                        ui.label(egui::RichText::new(&self.state.i18n.hash_col).strong());
                        ui.label(egui::RichText::new(&self.state.i18n.message_col).strong());
                        ui.label(egui::RichText::new(&self.state.i18n.author_col).strong());
                        ui.label(egui::RichText::new(&self.state.i18n.date_col).strong());
                        ui.end_row();

                        for c in &data.graph.commits {
                            let is_selected = self.state.selected_commit_id.as_deref() == Some(&c.id);

                            let mut lane_str = String::new();
                            for l in 0..data.graph.max_lanes.max(1) {
                                if l == c.lane {
                                    lane_str.push('●');
                                } else {
                                    lane_str.push('│');
                                }
                                lane_str.push(' ');
                            }
                            let lane_color = self.state.active_theme.branch_lanes
                                .get(c.lane % self.state.active_theme.branch_lanes.len().max(1))
                                .and_then(|hex| hex_to_color(hex).ok())
                                .unwrap_or(egui::Color32::from_rgb(224, 108, 117));
                            ui.label(egui::RichText::new(lane_str).monospace().color(lane_color));

                            let hash_color = hex_to_color(&self.state.active_theme.accent)
                                .unwrap_or(egui::Color32::from_rgb(97, 175, 239));

                            if ui.selectable_label(is_selected, egui::RichText::new(&c.short_id).monospace().color(hash_color)).clicked() {
                                commit_to_select = Some(c.id.clone());
                            }
                            if ui.selectable_label(
                                is_selected,
                                egui::RichText::new(&c.message_headline)
                                    .size(self.state.typography.ui_font.size_pt)
                                    .family(egui::FontFamily::Proportional),
                            ).clicked() {
                                commit_to_select = Some(c.id.clone());
                            }
                            if ui.selectable_label(is_selected, &c.author_name).clicked() {
                                commit_to_select = Some(c.id.clone());
                            }
                            if ui.selectable_label(is_selected, c.authored_at.format("%Y-%m-%d %H:%M").to_string()).clicked() {
                                commit_to_select = Some(c.id.clone());
                            }
                            ui.end_row();
                        }
                    });
                }
            });

            ui.separator();

            // Sección Inferior: Detalle del Commit Seleccionado y Visor de Diffs
            if let Some(ref detail) = self.state.selected_commit_detail {
                let border_col = hex_to_color(&self.state.active_theme.border_color)
                    .unwrap_or(egui::Color32::from_gray(60));
                let bg_col = hex_to_color(&self.state.active_theme.bg_surface)
                    .unwrap_or(egui::Color32::from_gray(30));
                let add_fg = hex_to_color(&self.state.active_theme.diff_add_fg)
                    .unwrap_or(egui::Color32::from_rgb(152, 195, 121));
                let del_fg = hex_to_color(&self.state.active_theme.diff_del_fg)
                    .unwrap_or(egui::Color32::from_rgb(224, 108, 117));
                let accent_col = hex_to_color(&self.state.active_theme.accent)
                    .unwrap_or(egui::Color32::from_rgb(97, 175, 239));

                // Algoritmo dinámico para determinar el tamaño del texto relativo a la tipografía del GUI
                let ui_size = self.state.typography.ui_font.size_pt.max(12.0);
                let code_size = self.state.typography.code_font.size_pt.max(12.0);

                let font_headline = ui_size;
                let font_body = ui_size;
                let font_label = (ui_size * 0.94).round();
                let font_val = (ui_size * 0.94).round();
                let font_stats = ui_size;
                let font_mono_val = (code_size * 0.94).round();
                let font_diff_file = (ui_size * 1.05).round();
                let font_diff_badge = (ui_size * 0.90).round();

                // 1. Tarjeta de Detalle del Commit
                egui::Frame::group(ui.style())
                    .stroke(egui::Stroke::new(1.0, border_col))
                    .fill(bg_col)
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .show(ui, |ui| {
                        // Título y Mensaje del Commit
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&detail.commit.message_headline)
                                    .strong()
                                    .size(font_headline)
                                    .family(egui::FontFamily::Proportional),
                            );
                        });
                        if let Some(ref body) = detail.commit.message_body {
                            if !body.is_empty() {
                                ui.add_space(2.0);
                                ui.label(
                                    egui::RichText::new(body)
                                        .size(font_body)
                                        .family(egui::FontFamily::Proportional),
                                );
                            }
                        }
                        ui.add_space(6.0);

                        // Metadata Grid: Hash, Tree, Author, Date, Parents, Branches
                        egui::Grid::new("commit_metadata_grid").spacing([20.0, 4.0]).show(ui, |ui| {
                            // Fila 1: Hash y Tree
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&self.state.i18n.commit_label).strong().size(font_label));
                                ui.label(egui::RichText::new(&detail.commit.id).monospace().color(accent_col).size(font_mono_val));
                                if draw_copy_button(ui, &self.state.i18n.copy_hash_tooltip) {
                                    ui.ctx().copy_text(detail.commit.id.clone());
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&self.state.i18n.tree_label).strong().size(font_label));
                                ui.label(egui::RichText::new(&detail.tree_id).monospace().weak().size(font_mono_val));
                            });
                            ui.end_row();

                            // Fila 2: Autor y Fecha
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&self.state.i18n.author_label).strong().size(font_label));
                                ui.label(egui::RichText::new(format!("{} <{}>", detail.commit.author_name, detail.commit.author_email)).size(font_val));
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&self.state.i18n.date_label).strong().size(font_label));
                                ui.label(egui::RichText::new(detail.commit.authored_at.format("%Y-%m-%d %H:%M:%S %z").to_string()).size(font_val));
                            });
                            ui.end_row();

                            // Fila 3: Padres y Ramas
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&self.state.i18n.parents_label).strong().size(font_label));
                                if detail.commit.parent_ids.is_empty() {
                                    ui.label(egui::RichText::new(&self.state.i18n.root_commit).weak().italics().size(font_val));
                                } else {
                                    let parents_str: Vec<String> = detail.commit.parent_ids.iter().map(|p| if p.len() >= 7 { p[..7].to_string() } else { p.clone() }).collect();
                                    ui.label(egui::RichText::new(parents_str.join(", ")).monospace().color(accent_col).size(font_mono_val));
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(&self.state.i18n.branches_label).strong().size(font_label));
                                if detail.commit.branches.is_empty() {
                                    ui.label(egui::RichText::new("-").weak().size(font_val));
                                } else {
                                    ui.label(egui::RichText::new(detail.commit.branches.join(", ")).strong().color(egui::Color32::from_rgb(152, 195, 121)).size(font_val));
                                }
                            });
                            ui.end_row();
                        });

                        ui.add_space(6.0);
                        // Fila de Estadísticas (Stats)
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{} {}", detail.stats.files_changed, self.state.i18n.files_changed_stat)).strong().size(font_stats));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("+{} {}", detail.stats.insertions, self.state.i18n.insertions_stat)).strong().color(add_fg).size(font_stats));
                            ui.separator();
                            ui.label(egui::RichText::new(format!("-{} {}", detail.stats.deletions, self.state.i18n.deletions_stat)).strong().color(del_fg).size(font_stats));

                            // Barra visual de proporción adiciones vs eliminaciones
                            let total_changes = detail.stats.insertions + detail.stats.deletions;
                            if total_changes > 0 {
                                ui.add_space(8.0);
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 10.0), egui::Sense::hover());
                                let add_ratio = detail.stats.insertions as f32 / total_changes as f32;
                                let add_width = rect.width() * add_ratio;
                                let add_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + add_width, rect.max.y));
                                let del_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x + add_width, rect.min.y), rect.max);
                                ui.painter().rect_filled(add_rect, 2.0, add_fg);
                                ui.painter().rect_filled(del_rect, 2.0, del_fg);
                            }
                        });
                    });

                ui.add_space(6.0);

                // 2. ComboBox de Archivos Modificados en este Commit
                let active_file_path = self.state.selected_commit_file.clone()
                    .or_else(|| detail.files.first().map(|f| f.path.clone()));

                let current_file_info = active_file_path.as_ref().and_then(|p| detail.files.iter().find(|f| f.path == *p));
                let display_combo_text = match current_file_info {
                    Some(f) => {
                        let tag = match f.status {
                            DeltaKind::Added => "[A]",
                            DeltaKind::Deleted => "[D]",
                            DeltaKind::Renamed => "[R]",
                            DeltaKind::Conflicted => "[C]",
                            _ => "[M]",
                        };
                        format!("{tag} {} (+{} -{})", f.path, f.additions, f.deletions)
                    }
                    None => self.state.i18n.select_file_placeholder.clone(),
                };

                let combo_width = 480.0_f32.min(ui.available_width() - 30.0).max(260.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&self.state.i18n.modified_files_label).strong().size(font_stats));
                    ui.label(egui::RichText::new(format!("({} {})", detail.files.len(), self.state.i18n.files_changed_stat)).weak().size(font_label));
                    ui.add_space(8.0);

                    egui::ComboBox::from_id_salt("commit_detail_files_combo")
                        .selected_text(egui::RichText::new(&display_combo_text).monospace().size(font_mono_val))
                        .width(combo_width)
                        .show_ui(ui, |ui| {
                            ui.set_width(combo_width);
                            egui::ScrollArea::vertical()
                                .max_height(280.0)
                                .show(ui, |ui| {
                                    for f in &detail.files {
                                        let tag = match f.status {
                                            DeltaKind::Added => "[A]",
                                            DeltaKind::Deleted => "[D]",
                                            DeltaKind::Renamed => "[R]",
                                            DeltaKind::Conflicted => "[C]",
                                            _ => "[M]",
                                        };
                                        let item_label = format!("{tag} {} (+{} -{})", f.path, f.additions, f.deletions);
                                        let is_sel = active_file_path.as_deref() == Some(&f.path);
                                        if ui.selectable_label(is_sel, egui::RichText::new(item_label).monospace().size(font_mono_val)).clicked() {
                                            self.state.selected_commit_file = Some(f.path.clone());
                                        }
                                    }
                                });
                        });
                });

                ui.separator();

                // 3. Visor de Diferencias del Archivo con Fondo Coloreado
                let active_file = self.state.selected_commit_file.clone()
                    .or_else(|| detail.files.first().map(|f| f.path.clone()));

                if let Some(ref file_path) = active_file {
                    if let Some(file_diff) = detail.files.iter().find(|f| f.path == *file_path) {
                        let status_text = match file_diff.status {
                            DeltaKind::Added => &self.state.i18n.diff_status_added,
                            DeltaKind::Deleted => &self.state.i18n.diff_status_deleted,
                            DeltaKind::Renamed => &self.state.i18n.diff_status_renamed,
                            DeltaKind::Untracked => &self.state.i18n.diff_status_untracked,
                            DeltaKind::Conflicted => &self.state.i18n.diff_status_conflicted,
                            DeltaKind::Modified => &self.state.i18n.diff_status_modified,
                        };
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&file_diff.path).strong().color(accent_col).size(font_diff_file));
                            let badge_text = format!("{} {} | +{} {} | -{} {}", self.state.i18n.diff_status_badge, status_text, file_diff.additions, self.state.i18n.insertions_stat, file_diff.deletions, self.state.i18n.deletions_stat);
                            ui.label(egui::RichText::new(badge_text).weak().size(font_diff_badge));
                        });

                        let add_bg = add_fg.gamma_multiply(0.18);
                        let del_bg = del_fg.gamma_multiply(0.18);
                        let muted_col = hex_to_color(&self.state.active_theme.fg_muted)
                            .unwrap_or(egui::Color32::from_rgb(171, 178, 191));

                        egui::ScrollArea::both().id_salt("commit_file_diff_scroll").show(ui, |ui| {
                            for hunk in &file_diff.patch.hunks {
                                ui.label(egui::RichText::new(&hunk.header).monospace().color(muted_col).italics().size(font_mono_val));
                                for line in &hunk.lines {
                                    let old_no = line.old_line_no.map_or("    ".to_string(), |n| format!("{n:<4}"));
                                    let new_no = line.new_line_no.map_or("    ".to_string(), |n| format!("{n:<4}"));
                                    let line_str = format!("{old_no} {new_no} {}", line.content.trim_end());

                                    let (line_bg, line_fg) = match line.kind {
                                        rmerge_domain::entities::LineKind::Addition => (add_bg, add_fg),
                                        rmerge_domain::entities::LineKind::Deletion => (del_bg, del_fg),
                                        _ => (egui::Color32::TRANSPARENT, muted_col),
                                    };

                                    egui::Frame::NONE
                                        .fill(line_bg)
                                        .inner_margin(egui::Margin::symmetric(6, 1))
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            ui.label(egui::RichText::new(line_str).monospace().color(line_fg).size(self.state.typography.code_font.size_pt));
                                        });
                                }
                            }
                        });
                    } else {
                        ui.label("Seleccione un archivo modificado para ver sus cambios.");
                    }
                } else {
                    ui.label("No hay archivos modificados en este commit.");
                }
            } else if let Some(ref patch) = self.selected_file_patch {
                // Fallback: Si no hay commit seleccionado pero hay un archivo de staging/working directory seleccionado
                ui.horizontal(|ui| {
                    let accent_col = hex_to_color(&self.state.active_theme.accent)
                        .unwrap_or(egui::Color32::from_rgb(97, 175, 239));
                    ui.label(egui::RichText::new(&patch.path).strong().color(accent_col));
                    let status_str = if patch.is_staged { "[STAGED]" } else { "[UNSTAGED]" };
                    ui.label(egui::RichText::new(status_str).italics());
                });

                let add_fg = hex_to_color(&self.state.active_theme.diff_add_fg)
                    .unwrap_or(egui::Color32::from_rgb(152, 195, 121));
                let del_fg = hex_to_color(&self.state.active_theme.diff_del_fg)
                    .unwrap_or(egui::Color32::from_rgb(224, 108, 117));
                let add_bg = add_fg.gamma_multiply(0.18);
                let del_bg = del_fg.gamma_multiply(0.18);
                let muted_col = hex_to_color(&self.state.active_theme.fg_muted)
                    .unwrap_or(egui::Color32::from_rgb(171, 178, 191));

                egui::ScrollArea::both().id_salt("diff_content_scroll").show(ui, |ui| {
                    for hunk in &patch.hunks {
                        ui.label(egui::RichText::new(&hunk.header).monospace().color(muted_col).italics());
                        for line in &hunk.lines {
                            let old_no = line.old_line_no.map_or("    ".to_string(), |n| format!("{n:<4}"));
                            let new_no = line.new_line_no.map_or("    ".to_string(), |n| format!("{n:<4}"));
                            let line_str = format!("{old_no} {new_no} {}", line.content.trim_end());

                            let (line_bg, line_fg) = match line.kind {
                                rmerge_domain::entities::LineKind::Addition => (add_bg, add_fg),
                                rmerge_domain::entities::LineKind::Deletion => (del_bg, del_fg),
                                _ => (egui::Color32::TRANSPARENT, muted_col),
                            };

                            egui::Frame::NONE
                                .fill(line_bg)
                                .inner_margin(egui::Margin::symmetric(6, 1))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(egui::RichText::new(line_str).monospace().color(line_fg).size(self.state.typography.code_font.size_pt));
                                });
                        }
                    }
                });
            } else {
                ui.label("Seleccione un commit de la tabla o un archivo de la barra lateral para inspeccionar sus cambios.");
            }
        });

        if let Some(c) = commit_to_select {
            self.select_commit(&c);
        }
        } else {
            // Pantalla de Bienvenida y Repositorios Recientes (cuando no se está en un repositorio Git)
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().id_salt("welcome_scroll").show(ui, |ui| {
                    let available_width = ui.available_width();
                    let content_width = available_width.min(840.0);
                    let left_padding = ((available_width - content_width) / 2.0).max(0.0);

                    ui.horizontal(|ui| {
                        ui.add_space(left_padding);
                        ui.vertical(|ui| {
                            ui.set_width(content_width);

                            ui.add_space(24.0);

                            // Encabezado Hero
                            ui.vertical_centered(|ui| {
                                draw_branch_icon(ui, egui::Color32::from_rgb(97, 175, 239), 56.0);
                                ui.add_space(10.0);
                                ui.heading(
                                    egui::RichText::new("Git-Client")
                                        .size(30.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(97, 175, 239)),
                                );
                                ui.label(
                                    egui::RichText::new("Cliente Git rápido, moderno y multiplataforma")
                                        .size(14.0)
                                        .weak(),
                                );
                                ui.add_space(20.0);

                                // Botones de Acción Primaria
                                ui.horizontal(|ui| {
                                    let btn_w = 200.0;
                                    let spacing = 12.0;
                                    let total_w = btn_w * 3.0 + spacing * 2.0;
                                    if content_width > total_w {
                                        ui.add_space((content_width - total_w) / 2.0);
                                    }

                                    let open_btn = egui::Button::new(
                                        egui::RichText::new("📂 Abrir Repositorio Local...").strong().size(13.0),
                                    ).min_size(egui::vec2(btn_w, 38.0));
                                    if ui.add(open_btn).clicked() {
                                        if let Some(folder) = rfd::FileDialog::new()
                                            .set_title("Seleccionar Repositorio Git Local")
                                            .pick_folder()
                                        {
                                            self.state.repo_path = folder;
                                            self.reload_repository();
                                            self.check_identity();
                                        }
                                    }

                                    let clone_btn = egui::Button::new(
                                        egui::RichText::new("⬇ Clonar Repositorio...").strong().size(13.0),
                                    ).min_size(egui::vec2(btn_w, 38.0));
                                    if ui.add(clone_btn).clicked() {
                                        self.state.show_clone_modal = true;
                                        self.state.clone_error = None;
                                    }

                                    let manual_btn = egui::Button::new(
                                        egui::RichText::new("⌨ Ingresar Ruta...").size(13.0),
                                    ).min_size(egui::vec2(btn_w, 38.0));
                                    if ui.add(manual_btn).clicked() {
                                        self.state.manual_open_path = self.state.repo_path.to_string_lossy().to_string();
                                        self.state.show_manual_open_modal = true;
                                    }
                                });
                            });

                            ui.add_space(24.0);
                            ui.separator();
                            ui.add_space(16.0);

                            // Aviso informativo si el directorio abierto actualmente no es git
                            if !self.state.repo_path.as_os_str().is_empty() && self.state.repo_path != PathBuf::from(".") {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("ℹ ").color(egui::Color32::from_rgb(229, 192, 123)).strong());
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "El directorio actual no es un repositorio Git: {}",
                                            self.state.repo_path.display()
                                        ))
                                        .weak(),
                                    );
                                });
                                ui.add_space(12.0);
                            }

                            // Sección: Repositorios Recientes
                            ui.horizontal(|ui| {
                                let count = self.state.recent_repos.len();
                                ui.label(
                                    egui::RichText::new(format!("Repositorios Recientes ({count})"))
                                        .size(18.0)
                                        .strong(),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if count > 0
                                        && ui
                                            .button(
                                                egui::RichText::new("🗑 Borrar Historial")
                                                    .color(egui::Color32::from_rgb(224, 108, 117)),
                                            )
                                            .on_hover_text("Borrar todos los repositorios del historial reciente")
                                            .clicked()
                                        {
                                            self.state.clear_recent_repos();
                                            self.save_preferences();
                                        }
                                });
                            });

                            ui.add_space(8.0);

                            if self.state.recent_repos.is_empty() {
                                ui.add_space(20.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(
                                        egui::RichText::new("No hay repositorios en el historial.")
                                            .italics()
                                            .weak(),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(
                                            "Abre o clona un repositorio con las opciones superiores para comenzar.",
                                        )
                                        .weak(),
                                    );
                                });
                            } else {
                                let mut repo_to_open: Option<PathBuf> = None;
                                let mut repo_to_remove: Option<String> = None;

                                for repo_path_str in &self.state.recent_repos {
                                    let path = PathBuf::from(repo_path_str);
                                    let exists = path.is_dir();
                                    let dir_name = path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| repo_path_str.clone());

                                    let border_col = hex_to_color(&self.state.active_theme.border_color)
                                        .unwrap_or(egui::Color32::from_gray(60));
                                    let bg_col = hex_to_color(&self.state.active_theme.bg_surface)
                                        .unwrap_or(egui::Color32::from_gray(35));

                                    egui::Frame::group(ui.style())
                                        .stroke(egui::Stroke::new(1.0, border_col))
                                        .fill(bg_col)
                                        .corner_radius(6.0)
                                        .inner_margin(egui::Margin::symmetric(14, 10))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                draw_branch_icon(ui, egui::Color32::from_rgb(97, 175, 239), 18.0);
                                                ui.add_space(8.0);

                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        let mut title_text =
                                                            egui::RichText::new(&dir_name).size(15.0).strong();
                                                        if !exists {
                                                            title_text = title_text.strikethrough();
                                                        }
                                                        ui.label(title_text);

                                                        if !exists {
                                                            ui.label(
                                                                egui::RichText::new("(No encontrado en disco)")
                                                                    .size(11.0)
                                                                    .color(egui::Color32::from_rgb(224, 108, 117)),
                                                            );
                                                        }
                                                    });
                                                    ui.label(
                                                        egui::RichText::new(repo_path_str)
                                                            .monospace()
                                                            .weak()
                                                            .size(12.0),
                                                    );
                                                });

                                                ui.with_layout(
                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                    |ui| {
                                                        if ui
                                                            .button("✕")
                                                            .on_hover_text("Quitar de la lista")
                                                            .clicked()
                                                        {
                                                            repo_to_remove = Some(repo_path_str.clone());
                                                        }
                                                        if exists
                                                            && ui.button("Abrir").clicked() {
                                                                repo_to_open = Some(path.clone());
                                                            }
                                                    },
                                                );
                                            });
                                        });
                                    ui.add_space(6.0);
                                }

                                if let Some(r) = repo_to_remove {
                                    self.state.remove_recent_repo(&r);
                                    self.save_preferences();
                                }
                                if let Some(p) = repo_to_open {
                                    self.state.repo_path = p;
                                    self.reload_repository();
                                    self.check_identity();
                                }
                            }

                            ui.add_space(30.0);
                        });
                        ui.add_space(left_padding);
                    });
                });
            });
        }

        // 5. Modal: Configurar Identidad de Autor Git (cuando user.name o user.email faltan)
        if let Some(ref mut modal) = self.state.identity_modal {
            let mut close_modal = false;
            let mut save_identity = false;

            egui::Window::new("Configurar Identidad de Autor de Git")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("No se detectó un autor configurado para firmar los commits.");
                    ui.label("Por favor ingrese sus datos para continuar:");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Nombre (user.name):  ");
                        ui.text_edit_singleline(&mut modal.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Correo (user.email): ");
                        ui.text_edit_singleline(&mut modal.email);
                    });

                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("¿Dónde desea guardar esta configuración?").strong());
                    ui.radio_value(&mut modal.scope, ConfigScope::Local, "Solo para este repositorio (.git/config)");
                    ui.radio_value(&mut modal.scope, ConfigScope::Global, "Global para todos mis repositorios (~/.gitconfig)");

                    if let Some(ref err) = modal.error_message {
                        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("✔ Guardar y Continuar").clicked() {
                            save_identity = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if save_identity {
                let author = GitAuthor::new(&modal.name, &modal.email);
                let scope = modal.scope;
                let repo = self.state.repo_path.clone();
                let svc = self.author_service.clone();

                let res = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(svc.configure_author(&repo, author, scope))
                });

                match res {
                    Ok(_) => {
                        self.state.close_identity_modal();
                        self.state.set_status("Identidad de autor guardada correctamente.");
                    }
                    Err(e) => {
                        modal.error_message = Some(e.to_string());
                    }
                }
            } else if close_modal {
                self.state.close_identity_modal();
            }
        }

        // 6. Modal: Herramienta 3-Way Merge
        if let Some(ref mut merge_file) = self.state.active_merge {
            let mut close_merge = false;
            let mut save_merge = false;

            egui::Window::new(format!("3-Way Merge: {}", merge_file.file_path))
                .min_width(800.0)
                .min_height(500.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading("Resolución Visual de Conflictos");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for block in &mut merge_file.blocks {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new(format!("Bloque de Conflicto #{}", block.id)).strong());
                                ui.columns(3, |cols| {
                                    cols[0].label(egui::RichText::new("LOCAL / OURS").color(egui::Color32::from_rgb(97, 175, 239)));
                                    cols[0].label(egui::RichText::new(&block.ours_content).monospace());

                                    cols[1].label(egui::RichText::new("ANCESTRO / BASE").color(egui::Color32::GRAY));
                                    cols[1].label(egui::RichText::new(&block.ancestor_content).monospace());

                                    cols[2].label(egui::RichText::new("REMOTO / THEIRS").color(egui::Color32::from_rgb(224, 108, 117)));
                                    cols[2].label(egui::RichText::new(&block.theirs_content).monospace());
                                });

                                ui.horizontal(|ui| {
                                    if ui.button("Usar Ours").clicked() {
                                        block.apply_resolution(ConflictResolution::UseOurs);
                                    }
                                    if ui.button("Usar Theirs").clicked() {
                                        block.apply_resolution(ConflictResolution::UseTheirs);
                                    }
                                    if ui.button("Usar Base").clicked() {
                                        block.apply_resolution(ConflictResolution::UseBase);
                                    }
                                    if ui.button("Combinar Ambos").clicked() {
                                        block.apply_resolution(ConflictResolution::UseBoth { ours_first: true });
                                    }
                                });

                                if let Some(ref res) = block.resolved_content {
                                    ui.label(egui::RichText::new(format!("Resultado: {res}")).color(egui::Color32::from_rgb(152, 195, 121)));
                                }
                            });
                        }
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("💾 Guardar y Marcar Resuelto").clicked() {
                            save_merge = true;
                        }
                        if ui.button("Salir").clicked() {
                            close_merge = true;
                        }
                    });
                });

            if save_merge {
                let repo = self.state.repo_path.clone();
                let svc = self.merge_service.clone();
                let mf = merge_file.clone();
                let _ = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(svc.save_and_resolve(&repo, &mf))
                });
                self.state.active_merge = None;
                self.reload_repository();
            } else if close_merge {
                self.state.active_merge = None;
            }
        }

        // 7. Modal: Preferencias (Fuentes y Temas)
        if self.state.show_preferences {
            egui::Window::new(&self.state.i18n.preferences_title)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(700.0)
                .show(ctx, |ui| {
                    ui.heading(&self.state.i18n.themes_title);
                    ui.label(&self.state.i18n.themes_subtitle);

                    egui::ScrollArea::vertical().max_height(200.0).id_salt("theme_selector_scroll").show(ui, |ui| {
                        egui::Grid::new("themes_grid").spacing([16.0, 6.0]).show(ui, |ui| {
                            let mut col = 0;
                            for t in &self.state.available_themes {
                                let is_selected = self.state.active_theme.name == *t;
                                let storage = self.theme_storage.clone();
                                let marker = if is_selected { "● " } else { "○ " };
                                if ui.selectable_label(is_selected, format!("{marker}{t}")).clicked() {
                                    let theme_name = t.clone();
                                    if let Ok(new_theme) = tokio::task::block_in_place(|| {
                                        tokio::runtime::Handle::current().block_on(storage.get_theme(&theme_name))
                                    }) {
                                        self.state.active_theme = new_theme;
                                        self.save_preferences();
                                    }
                                }
                                col += 1;
                                if col % 3 == 0 {
                                    ui.end_row();
                                }
                            }
                            if col % 3 != 0 {
                                ui.end_row();
                            }
                        });
                    });

                    ui.separator();
                    ui.heading(&self.state.i18n.typography_title);

                    let mut fonts_changed = false;
                    let mut styles_changed = false;

                    // Tipografía de Interfaz (GUI)
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new(&self.state.i18n.ui_font_section).strong());
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.state.i18n.font_family_label));
                        let current_ui_font = self.state.typography.ui_font.family.clone();
                        egui::ComboBox::from_id_salt("modal_ui_font_combo")
                            .width(280.0)
                            .selected_text(&current_ui_font)
                            .show_ui(ui, |ui| {
                                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                                    for font_name in &self.state.available_fonts.system_fonts {
                                        if ui.selectable_label(current_ui_font == *font_name, font_name).clicked() {
                                            self.state.typography.ui_font.family = font_name.clone();
                                            fonts_changed = true;
                                        }
                                    }
                                });
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.state.i18n.ui_font_size_label));
                        if ui.add(egui::Slider::new(&mut self.state.typography.ui_font.size_pt, 10.0..=22.0).suffix(" pt")).changed() {
                            styles_changed = true;
                        }
                    });

                    // Tipografía de Código (Diffs / Monospace)
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(&self.state.i18n.code_font_section).strong());
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.state.i18n.font_family_label));
                        let current_code_font = self.state.typography.code_font.family.clone();
                        egui::ComboBox::from_id_salt("modal_code_font_combo")
                            .width(280.0)
                            .selected_text(&current_code_font)
                            .show_ui(ui, |ui| {
                                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                                    for font_name in &self.state.available_fonts.monospace_fonts {
                                        if ui.selectable_label(current_code_font == *font_name, font_name).clicked() {
                                            self.state.typography.code_font.family = font_name.clone();
                                            fonts_changed = true;
                                        }
                                    }
                                });
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.state.i18n.code_font_size_label));
                        if ui.add(egui::Slider::new(&mut self.state.typography.code_font.size_pt, 10.0..=24.0).suffix(" pt")).changed() {
                            styles_changed = true;
                        }
                    });
                    if ui.checkbox(&mut self.state.typography.code_font.enable_ligatures, &self.state.i18n.enable_ligatures_label).changed() {
                        styles_changed = true;
                    }

                    if fonts_changed {
                        self.apply_custom_fonts(ctx);
                        self.apply_typography_styles(ctx);
                        self.save_preferences();
                    } else if styles_changed {
                        self.apply_typography_styles(ctx);
                        self.save_preferences();
                    }

                    ui.separator();
                    ui.heading(&self.state.i18n.language);
                    let mut lang_changed = false;
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", self.state.i18n.language));
                        let current_lang = self.state.language.clone();
                        let display = match current_lang.as_str() {
                            "en" => "English",
                            _ => "Español",
                        };
                        egui::ComboBox::from_id_salt("modal_pref_language_combo")
                            .selected_text(display)
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(current_lang == "es", "Español").clicked() {
                                    self.state.set_language("es");
                                    lang_changed = true;
                                }
                                if ui.selectable_label(current_lang == "en", "English").clicked() {
                                    self.state.set_language("en");
                                    lang_changed = true;
                                }
                            });
                    });
                    if lang_changed {
                        self.save_preferences();
                    }

                    ui.separator();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(&self.state.i18n.close_btn).clicked() {
                            self.save_preferences();
                            self.state.show_preferences = false;
                        }
                    });
                });
        }

        // Modal: Acerca de (About)
        if self.state.show_about_modal {
            egui::Window::new(&self.state.i18n.about_title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(500.0)
                .show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        draw_branch_icon(ui, egui::Color32::from_rgb(97, 175, 239), 28.0);
                        ui.vertical(|ui| {
                            ui.heading(egui::RichText::new(rmerge_domain::APP_NAME).strong().size(18.0));
                            ui.label(egui::RichText::new(format!("{} {}", self.state.i18n.about_version_label, rmerge_domain::APP_VERSION)).monospace().color(egui::Color32::from_rgb(152, 195, 121)));
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    ui.label(&self.state.i18n.about_description);

                    ui.add_space(10.0);
                    egui::Grid::new("about_grid").spacing([16.0, 5.0]).show(ui, |ui| {
                        ui.label(egui::RichText::new(&self.state.i18n.about_version_label).strong());
                        ui.label(rmerge_domain::APP_VERSION);
                        ui.end_row();

                        ui.label(egui::RichText::new(&self.state.i18n.about_license_label).strong());
                        ui.label(&self.state.i18n.about_license_value);
                        ui.end_row();

                        ui.label(egui::RichText::new(&self.state.i18n.about_author_label).strong());
                        ui.label("AnibalGH");
                        ui.end_row();

                        ui.label(egui::RichText::new(&self.state.i18n.about_platform_label).strong());
                        ui.label(&self.state.i18n.about_platform_value);
                        ui.end_row();

                        ui.label(egui::RichText::new(&self.state.i18n.about_architecture_label).strong());
                        ui.label(&self.state.i18n.about_architecture_value);
                        ui.end_row();
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(&self.state.i18n.close_btn).clicked() {
                            self.state.show_about_modal = false;
                        }
                    });
                });
        }

        // Modal: Servidor Remoto No Alcanzable (Alerta de Red / VPN)
        if let Some(ref data) = self.state.server_unreachable_modal {
            let mut close_modal = false;
            let theme = &self.state.active_theme;
            let bg_surface = hex_to_color(&theme.bg_surface).unwrap_or(egui::Color32::from_rgb(35, 39, 46));
            let border_col = hex_to_color(&theme.border_color).unwrap_or(egui::Color32::from_rgb(60, 65, 75));

            egui::Window::new(egui::RichText::new(format!("⚠️ {}", self.state.i18n.server_unreachable_title)).color(egui::Color32::from_rgb(224, 108, 117)).strong())
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(460.0)
                .max_width(480.0)
                .show(ctx, |ui| {
                    ui.add_space(6.0);

                    let ui_size = self.state.typography.ui_font.size_pt;

                    // 1. Mensaje de fallo con icono destacado
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🚫").size(28.0));
                        ui.vertical(|ui| {
                            let failure_reason = if data.is_timeout {
                                self.state.i18n.server_unreachable_timeout
                                    .replace("{host}", &data.host)
                                    .replace("{port}", &data.port.to_string())
                            } else {
                                self.state.i18n.server_unreachable_refused
                                    .replace("{host}", &data.host)
                                    .replace("{port}", &data.port.to_string())
                                    .replace("{detail}", data.error_detail.as_deref().unwrap_or(""))
                            };
                            ui.add(egui::Label::new(
                                egui::RichText::new(failure_reason)
                                    .strong()
                                    .size(ui_size)
                                    .family(egui::FontFamily::Proportional),
                            ).wrap());
                        });
                    });

                    ui.add_space(10.0);

                    // 2. Tarjeta contenedora con recomendación de VPN y comando afectado
                    egui::Frame::new()
                        .fill(bg_surface)
                        .stroke(egui::Stroke::new(1.0, border_col))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::same(10))
                        .show(ui, |ui| {
                            let cmd_text = self.state.i18n.server_unreachable_cmd_failed
                                .replace("{cmd}", &data.cmd);
                            ui.label(egui::RichText::new(cmd_text).monospace().size(ui_size * 0.9).weak());
                            ui.add_space(6.0);
                            ui.add(egui::Label::new(
                                egui::RichText::new(&self.state.i18n.server_unreachable_hint)
                                    .size(ui_size)
                                    .family(egui::FontFamily::Proportional),
                            ).wrap());
                        });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // 3. Botón de acción con texto internacionalizado
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new(&self.state.i18n.server_unreachable_action_btn).strong()).clicked() {
                            close_modal = true;
                        }
                    });
                });
            if close_modal {
                self.state.server_unreachable_modal = None;
            }
        }

        // 8. Modal: Clonar Repositorio Remoto con soporte de Credenciales HTTPS Seguras
        if self.state.show_clone_modal {
            let mut trigger_clone = false;
            let mut close_clone = false;

            egui::Window::new(&self.state.i18n.clone_modal_title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(540.0)
                .show(ctx, |ui| {
                    ui.label(&self.state.i18n.clone_modal_desc);
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(&self.state.i18n.clone_url_label);
                        let prev_url = self.state.clone_url.clone();
                        let resp = ui.text_edit_singleline(&mut self.state.clone_url);
                        if resp.changed() && self.state.clone_url != prev_url {
                            // Si cambió la URL, verificar si existen credenciales guardadas para el host
                            let trimmed = self.state.clone_url.trim();
                            if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
                                use rmerge_domain::entities::GitCredential;
                                use rmerge_application::ports::r#in::ManageCredentialsUseCase;
                                let host = GitCredential::extract_host(trimmed);
                                let cred_svc = self.credential_service.clone();
                                let found = tokio::task::block_in_place(|| {
                                    tokio::runtime::Handle::current().block_on(cred_svc.find_credential(&host))
                                }).unwrap_or(None);

                                if let Some(cred) = found {
                                    self.state.clone_username = cred.username;
                                    self.state.clone_secret = cred.secret;
                                    self.state.clone_has_saved_credential = true;
                                } else {
                                    self.state.clone_has_saved_credential = false;
                                }
                            } else {
                                self.state.clone_has_saved_credential = false;
                            }
                        }
                    });
                    ui.label(egui::RichText::new(&self.state.i18n.clone_url_hint).weak().size(11.0));

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(&self.state.i18n.clone_dest_label);
                        ui.text_edit_singleline(&mut self.state.clone_destination);
                        if ui.button(&self.state.i18n.clone_browse_btn).clicked() {
                            if let Some(folder) = rfd::FileDialog::new()
                                .set_title("Seleccionar Carpeta de Destino")
                                .pick_folder()
                            {
                                let mut target = folder;
                                let url_trimmed = self.state.clone_url.trim();
                                if !url_trimmed.is_empty() {
                                    if let Some(repo_name) = url_trimmed.split('/').next_back() {
                                        let clean_name = repo_name.trim_end_matches(".git");
                                        if !clean_name.is_empty() && !target.ends_with(clean_name) {
                                            target = target.join(clean_name);
                                        }
                                    }
                                }
                                self.state.clone_destination = target.to_string_lossy().to_string();
                            }
                        }
                    });

                    // Sección de autenticación HTTPS si la URL es HTTPS/HTTP
                    let url_trimmed = self.state.clone_url.trim();
                    let is_https = url_trimmed.starts_with("https://") || url_trimmed.starts_with("http://");
                    if is_https {
                        ui.add_space(10.0);
                        ui.group(|ui| {
                            ui.label(egui::RichText::new(&self.state.i18n.clone_auth_section).strong());
                            use rmerge_domain::entities::GitCredential;
                            let host = GitCredential::extract_host(url_trimmed);

                            if self.state.clone_has_saved_credential && !self.state.clone_override_saved_cred {
                                ui.add_space(4.0);
                                let msg = self.state.i18n.clone_saved_cred_found
                                    .replace("{host}", &host)
                                    .replace("{user}", &self.state.clone_username);
                                ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(152, 195, 121)));
                                if ui.small_button(&self.state.i18n.clone_use_custom_cred).clicked() {
                                    self.state.clone_override_saved_cred = true;
                                }
                            } else {
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.label(&self.state.i18n.clone_username_label);
                                    ui.text_edit_singleline(&mut self.state.clone_username);
                                });
                                ui.horizontal(|ui| {
                                    ui.label(&self.state.i18n.clone_secret_label);
                                    ui.add(egui::TextEdit::singleline(&mut self.state.clone_secret).password(true));
                                });
                                ui.label(egui::RichText::new(&self.state.i18n.clone_secret_hint).weak().size(11.0));
                                ui.checkbox(&mut self.state.clone_save_credentials, &self.state.i18n.clone_save_cred_checkbox);
                            }
                        });
                    }

                    if let Some(ref err) = self.state.clone_error {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("Error: {err}")).color(egui::Color32::from_rgb(224, 108, 117)));
                    }

                    if self.state.is_cloning {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(egui::RichText::new(&self.state.i18n.clone_in_progress).color(egui::Color32::from_rgb(97, 175, 239)));
                        });
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if !self.state.is_cloning {
                            let can_clone = !self.state.clone_url.trim().is_empty() && !self.state.clone_destination.trim().is_empty();
                            if ui.add_enabled(can_clone, egui::Button::new(&self.state.i18n.clone_start_btn)).clicked() {
                                trigger_clone = true;
                            }
                            if ui.button(&self.state.i18n.clone_cancel_btn).clicked() {
                                close_clone = true;
                            }
                        } else {
                            ui.label(egui::RichText::new("Operación en progreso...").italics().weak());
                        }
                    });
                });

            if trigger_clone {
                let url = self.state.clone_url.trim().to_string();
                let dest = PathBuf::from(self.state.clone_destination.trim());
                let tx = self.clone_tx.clone();
                let svc = self.repo_service.clone();
                let cred_svc = self.credential_service.clone();
                let username = self.state.clone_username.trim().to_string();
                let secret = self.state.clone_secret.trim().to_string();
                let save_cred = self.state.clone_save_credentials;

                self.state.is_cloning = true;
                self.state.clone_error = None;

                self.tokio_handle.spawn(async move {
                    use rmerge_application::ports::r#in::ManageCredentialsUseCase;
                    // Si el usuario ingresó credenciales y activó la casilla de guardado seguro, persistirlas en la BD
                    if !username.is_empty() && !secret.is_empty() && save_cred {
                        let _ = cred_svc.store_credential(&url, &username, &secret).await;
                    }

                    let res = svc.clone_repo(&url, &dest).await;
                    let _ = tx.send(res.map(|_| dest).map_err(|e| e.to_string()));
                });
            } else if close_clone {
                self.state.show_clone_modal = false;
                self.state.clone_error = None;
            }
        }

        // 9. Modal: Abrir Repositorio Local Manualmente
        if self.state.show_manual_open_modal {
            let mut do_open = false;
            let mut do_close = false;

            egui::Window::new("📂 Abrir Repositorio Local")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(480.0)
                .show(ctx, |ui| {
                    ui.label("Ingrese o seleccione la ruta del repositorio en el disco:");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Ruta: ");
                        ui.text_edit_singleline(&mut self.state.manual_open_path);
                        if ui.button("Examinar...").clicked() {
                            if let Some(folder) = rfd::FileDialog::new()
                                .set_title("Seleccionar Repositorio Local")
                                .pick_folder()
                            {
                                self.state.manual_open_path = folder.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        let can_open = !self.state.manual_open_path.trim().is_empty();
                        if ui.add_enabled(can_open, egui::Button::new("✔ Abrir")).clicked() {
                            do_open = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            do_close = true;
                        }
                    });
                });

            if do_open {
                let p = PathBuf::from(self.state.manual_open_path.trim());
                self.state.show_manual_open_modal = false;
                self.state.repo_path = p;
                self.reload_repository();
                self.check_identity();
            } else if do_close {
                self.state.show_manual_open_modal = false;
            }
        }

        // 10. Modal: Crear Nuevo Tag
        if self.state.show_create_tag_modal {
            let mut do_create = false;
            let mut do_cancel = false;

            egui::Window::new("🏷 Crear Nuevo Tag (Etiqueta)")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(460.0)
                .show(ctx, |ui| {
                    ui.label("Ingrese los datos para crear la etiqueta en el repositorio:");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Nombre del Tag:    ");
                        ui.text_edit_singleline(&mut self.state.new_tag_name);
                    });
                    ui.label(egui::RichText::new("Ej: v1.0.0, release-2026.09").weak().size(11.0));

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Commit de Destino: ");
                        ui.text_edit_singleline(&mut self.state.new_tag_target);
                    });
                    ui.label(egui::RichText::new("Por defecto: HEAD o hash del commit").weak().size(11.0));

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Mensaje (opcional):");
                        ui.text_edit_singleline(&mut self.state.new_tag_message);
                    });
                    ui.label(egui::RichText::new("Si se indica un mensaje, se crea un tag anotado").weak().size(11.0));

                    if let Some(ref err) = self.state.create_tag_error {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("Error: {err}")).color(egui::Color32::from_rgb(224, 108, 117)));
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        let can_create = !self.state.new_tag_name.trim().is_empty();
                        if ui.add_enabled(can_create, egui::Button::new("✔ Crear Tag")).clicked() {
                            do_create = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            do_cancel = true;
                        }
                    });
                });

            if do_create {
                let tag_name = self.state.new_tag_name.trim().to_string();
                let target = if self.state.new_tag_target.trim().is_empty() {
                    "HEAD".to_string()
                } else {
                    self.state.new_tag_target.trim().to_string()
                };
                let msg = if self.state.new_tag_message.trim().is_empty() {
                    None
                } else {
                    Some(self.state.new_tag_message.trim().to_string())
                };

                let repo = self.state.repo_path.clone();
                let svc = self.repo_service.clone();
                let res = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(svc.create_tag(&repo, &tag_name, Some(&target), msg.as_deref()))
                });

                match res {
                    Ok(_) => {
                        self.state.show_create_tag_modal = false;
                        self.state.new_tag_name.clear();
                        self.state.new_tag_target.clear();
                        self.state.new_tag_message.clear();
                        self.state.create_tag_error = None;
                        self.state.set_status(format!("Tag '{tag_name}' creado con éxito."));
                        self.reload_repository();
                    }
                    Err(e) => {
                        self.state.create_tag_error = Some(e.to_string());
                    }
                }
            } else if do_cancel {
                self.state.show_create_tag_modal = false;
                self.state.create_tag_error = None;
            }
        }

        // 11. Modal: Crear Nueva Rama
        if self.state.show_create_branch_modal {
            let mut do_create = false;
            let mut do_cancel = false;

            egui::Window::new("🌿 Crear Nueva Rama")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(460.0)
                .show(ctx, |ui| {
                    ui.label("Ingrese los datos para crear la nueva rama local:");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Nombre de Rama: ");
                        ui.text_edit_singleline(&mut self.state.new_branch_name);
                    });
                    ui.label(egui::RichText::new("Ej: feature/nueva-pantalla, fix/correccion").weak().size(11.0));

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Punto de Partida:");
                        ui.text_edit_singleline(&mut self.state.new_branch_start);
                    });
                    ui.label(egui::RichText::new("Por defecto: HEAD o rama actual").weak().size(11.0));

                    ui.add_space(6.0);
                    ui.checkbox(&mut self.state.new_branch_checkout, "Cambiar a esta rama tras crear (Checkout)");

                    if let Some(ref err) = self.state.create_branch_error {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("Error: {err}")).color(egui::Color32::from_rgb(224, 108, 117)));
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        let can_create = !self.state.new_branch_name.trim().is_empty();
                        if ui.add_enabled(can_create, egui::Button::new("✔ Crear Rama")).clicked() {
                            do_create = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            do_cancel = true;
                        }
                    });
                });

            if do_create {
                let branch_name = self.state.new_branch_name.trim().to_string();
                let target = if self.state.new_branch_start.trim().is_empty() {
                    "HEAD".to_string()
                } else {
                    self.state.new_branch_start.trim().to_string()
                };
                let checkout = self.state.new_branch_checkout;

                let repo = self.state.repo_path.clone();
                let svc = self.repo_service.clone();
                let res = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(svc.create_branch(&repo, &branch_name, Some(&target), checkout))
                });

                match res {
                    Ok(_) => {
                        self.state.show_create_branch_modal = false;
                        self.state.new_branch_name.clear();
                        self.state.create_branch_error = None;
                        self.state.set_status(format!("Rama '{branch_name}' creada con éxito."));
                        self.reload_repository();
                    }
                    Err(e) => {
                        self.state.create_branch_error = Some(e.to_string());
                    }
                }
            } else if do_cancel {
                self.state.show_create_branch_modal = false;
                self.state.create_branch_error = None;
            }
        }

        // 12. Modal: Cambiar de Rama (Checkout)
        if self.state.show_switch_branch_modal {
            let mut do_switch = false;
            let mut do_cancel = false;

            egui::Window::new("🔀 Cambiar de Rama (Checkout)")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(460.0)
                .show(ctx, |ui| {
                    ui.label("Seleccione la rama a la que desea cambiar:");
                    ui.separator();

                    if let Some(ref data) = self.state.repo_data {
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for b in &data.branches {
                                let is_current = data.current_branch.as_deref() == Some(&b.name);
                                let mut label_text = b.name.clone();
                                if is_current {
                                    label_text.push_str(" (actual)");
                                }
                                let is_selected = self.state.switch_branch_target == b.name;
                                if ui.selectable_label(is_selected, label_text).clicked() {
                                    self.state.switch_branch_target = b.name.clone();
                                }
                            }
                        });
                    }

                    if let Some(ref err) = self.state.switch_branch_error {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("Error: {err}")).color(egui::Color32::from_rgb(224, 108, 117)));
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        let can_switch = !self.state.switch_branch_target.trim().is_empty();
                        if ui.add_enabled(can_switch, egui::Button::new("✔ Cambiar Rama")).clicked() {
                            do_switch = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            do_cancel = true;
                        }
                    });
                });

            if do_switch {
                let target = self.state.switch_branch_target.trim().to_string();
                let repo = self.state.repo_path.clone();
                let svc = self.repo_service.clone();
                let res = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(svc.switch_branch(&repo, &target))
                });

                match res {
                    Ok(_) => {
                        self.state.show_switch_branch_modal = false;
                        self.state.switch_branch_target.clear();
                        self.state.switch_branch_error = None;
                        self.state.set_status(format!("Cambiado exitosamente a la rama '{target}'."));
                        self.reload_repository();
                    }
                    Err(e) => {
                        self.state.switch_branch_error = Some(e.to_string());
                    }
                }
            } else if do_cancel {
                self.state.show_switch_branch_modal = false;
                self.state.switch_branch_error = None;
            }
        }

        // 13. Modal: Merge de Ramas
        if self.state.show_merge_modal {
            let mut do_merge = false;
            let mut do_cancel = false;

            egui::Window::new("🔀 Merge de Ramas")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(480.0)
                .show(ctx, |ui| {
                    let cur_branch = self.state.repo_data.as_ref()
                        .and_then(|d| d.current_branch.clone())
                        .unwrap_or_else(|| "HEAD".to_string());

                    ui.horizontal(|ui| {
                        ui.label("Rama destino (activa): ");
                        ui.label(egui::RichText::new(&cur_branch).strong().color(egui::Color32::from_rgb(152, 195, 121)));
                    });
                    ui.separator();

                    ui.label("Seleccione la rama de origen que desea fusionar:");

                    if let Some(ref data) = self.state.repo_data {
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            for b in &data.branches {
                                if b.name == cur_branch {
                                    continue;
                                }
                                let is_selected = self.state.merge_source_branch == b.name;
                                if ui.selectable_label(is_selected, &b.name).clicked() {
                                    self.state.merge_source_branch = b.name.clone();
                                }
                            }
                        });
                    }

                    if let Some(ref err) = self.state.merge_error {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(format!("Error: {err}")).color(egui::Color32::from_rgb(224, 108, 117)));
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        let can_merge = !self.state.merge_source_branch.trim().is_empty();
                        if ui.add_enabled(can_merge, egui::Button::new("✔ Realizar Merge")).clicked() {
                            do_merge = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            do_cancel = true;
                        }
                    });
                });

            if do_merge {
                let source = self.state.merge_source_branch.trim().to_string();
                let repo = self.state.repo_path.clone();
                let svc = self.repo_service.clone();
                let res = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(svc.merge_branch(&repo, &source))
                });

                match res {
                    Ok(rmerge_domain::entities::MergeOutcome::FastForward { new_commit_id }) => {
                        self.state.show_merge_modal = false;
                        self.state.merge_source_branch.clear();
                        self.state.merge_error = None;
                        self.state.set_status(format!("Merge Fast-Forward completado con rama '{source}' (HEAD en {new_commit_id})."));
                        self.reload_repository();
                    }
                    Ok(rmerge_domain::entities::MergeOutcome::Merged { merge_commit_id }) => {
                        self.state.show_merge_modal = false;
                        self.state.merge_source_branch.clear();
                        self.state.merge_error = None;
                        self.state.set_status(format!("Merge commit creado con éxito: {merge_commit_id}."));
                        self.reload_repository();
                    }
                    Ok(rmerge_domain::entities::MergeOutcome::UpToDate) => {
                        self.state.show_merge_modal = false;
                        self.state.merge_source_branch.clear();
                        self.state.merge_error = None;
                        self.state.set_status("La rama ya está completamente al día (Already up to date).");
                    }
                    Ok(rmerge_domain::entities::MergeOutcome::Conflicts { conflict_files }) => {
                        self.state.show_merge_modal = false;
                        self.state.merge_source_branch.clear();
                        self.state.merge_error = None;
                        self.state.set_status(format!("Conflictos detectados en {} archivos. Utilice la herramienta de resolución 3-way merge.", conflict_files.len()));
                        self.reload_repository();
                    }
                    Err(e) => {
                        self.state.merge_error = Some(e.to_string());
                    }
                }
            } else if do_cancel {
                self.state.show_merge_modal = false;
                self.state.merge_error = None;
            }
        }

        // 14. Modal: Editor de .gitignore
        if self.state.show_gitignore_modal {
            let mut do_save = false;
            let mut do_cancel = false;

            egui::Window::new("📄 Editor de .gitignore")
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .min_width(680.0)
                .min_height(520.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if self.state.gitignore_exists_on_disk {
                            ui.label(egui::RichText::new("📁 Archivo existente: .gitignore").strong().color(egui::Color32::from_rgb(152, 195, 121)));
                        } else {
                            ui.label(egui::RichText::new("⚠️ No existe .gitignore — Se ha precargado una plantilla recomendada").strong().color(egui::Color32::from_rgb(229, 192, 123)));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let path_str = self.state.repo_path.join(".gitignore").display().to_string();
                            ui.label(egui::RichText::new(path_str).weak().monospace().size(11.0));
                        });
                    });
                    ui.separator();

                    // Barra de plantillas rápidas
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Insertar patrones comunes:").size(12.0).weak());
                        if ui.small_button("＋ Rust").on_hover_text("Añadir reglas para Rust y Cargo").clicked() {
                            self.state.gitignore_content.push_str("\n# Rust / Cargo\ntarget/\n**/*.rs.bk\nCargo.lock\n");
                        }
                        if ui.small_button("＋ Node / JS").on_hover_text("Añadir reglas para Node.js y npm").clicked() {
                            self.state.gitignore_content.push_str("\n# Node.js\nnode_modules/\ndist/\n.npm\n*.tsbuildinfo\n");
                        }
                        if ui.small_button("＋ Python").on_hover_text("Añadir reglas para Python y venv").clicked() {
                            self.state.gitignore_content.push_str("\n# Python\n__pycache__/\n*.py[cod]\n*$py.class\n.venv/\nenv/\n");
                        }
                        if ui.small_button("＋ Java / JVM").on_hover_text("Añadir reglas para Java, Maven y Gradle").clicked() {
                            self.state.gitignore_content.push_str("\n# Java / Gradle / Maven\n*.class\n.gradle/\nbuild/\ntarget/\n");
                        }
                        if ui.small_button("＋ IDEs").on_hover_text("Añadir reglas para VS Code, JetBrains y editores").clicked() {
                            self.state.gitignore_content.push_str("\n# IDEs & Editores\n.vscode/\n.idea/\n*.swp\n*.swo\n*~\n.DS_Store\nThumbs.db\n");
                        }
                    });
                    ui.add_space(6.0);

                    // Área de edición de texto
                    let editor_height = (ui.available_height() - 55.0).max(280.0);
                    egui::ScrollArea::vertical().id_salt("gitignore_editor_scroll").max_height(editor_height).show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.state.gitignore_content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_rows(18)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                        );
                    });

                    if let Some(ref err) = self.state.gitignore_save_error {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(format!("Error: {err}")).color(egui::Color32::from_rgb(224, 108, 117)));
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("💾 Guardar .gitignore").clicked() {
                            do_save = true;
                        }
                        if ui.button("Cancelar").clicked() {
                            do_cancel = true;
                        }
                    });
                });

            if do_save {
                let repo = self.state.repo_path.clone();
                let svc = self.repo_service.clone();
                let content = self.state.gitignore_content.clone();

                let res = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(svc.save_gitignore(&repo, &content))
                });

                match res {
                    Ok(_) => {
                        self.state.show_gitignore_modal = false;
                        self.state.gitignore_save_error = None;
                        self.state.set_status("Archivo .gitignore guardado correctamente. Reglas aplicadas al repositorio.");
                        self.reload_repository();
                    }
                    Err(e) => {
                        self.state.gitignore_save_error = Some(e.to_string());
                    }
                }
            } else if do_cancel {
                self.state.show_gitignore_modal = false;
                self.state.gitignore_save_error = None;
            }
        }

        // 11. Modal: Historial y Detalle de Mensajes de Estado
        if self.state.show_status_messages_modal {
            let mut close_modal = false;
            let mut msg_to_remove = None;
            let mut clear_all = false;

            egui::Window::new(&self.state.i18n.status_messages_title)
                .collapsible(false)
                .resizable(true)
                .default_size([550.0, 360.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let count = self.state.status_messages.len();
                        ui.label(egui::RichText::new(format!("{}: {count}", self.state.i18n.status_messages_title)).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if count > 0 && ui.button(egui::RichText::new(&self.state.i18n.status_messages_clear_all).color(egui::Color32::from_rgb(224, 108, 117))).clicked() {
                                clear_all = true;
                            }
                        });
                    });
                    ui.separator();

                    if self.state.status_messages.is_empty() {
                        ui.add_space(20.0);
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(&self.state.i18n.status_messages_empty).italics().weak());
                        });
                        ui.add_space(20.0);
                    } else {
                        egui::ScrollArea::vertical()
                            .id_salt("status_messages_modal_scroll")
                            .max_height(260.0)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for (idx, msg) in self.state.status_messages.iter().enumerate() {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(format!("#{}.", idx + 1)).weak());
                                            let color = if msg.starts_with('✔') {
                                                egui::Color32::from_rgb(152, 195, 121)
                                            } else if msg.starts_with('❌') {
                                                egui::Color32::from_rgb(224, 108, 117)
                                            } else if msg.starts_with("⚠️") {
                                                egui::Color32::from_rgb(229, 192, 123)
                                            } else {
                                                ui.visuals().text_color()
                                            };
                                            ui.label(egui::RichText::new(msg).color(color));

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("✕").on_hover_text("Cerrar este mensaje").clicked() {
                                                    msg_to_remove = Some(idx);
                                                }
                                            });
                                        });
                                    });
                                    ui.add_space(3.0);
                                }
                            });
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(&self.state.i18n.status_messages_close).clicked() {
                            close_modal = true;
                        }
                    });
                });

            if clear_all {
                self.state.clear_status_messages();
            } else if let Some(idx) = msg_to_remove {
                self.state.remove_status_message(idx);
            }

            if close_modal {
                self.state.show_status_messages_modal = false;
            }
        }
    }
}

fn hex_to_color(hex: &str) -> Result<egui::Color32, ()> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
        Ok(egui::Color32::from_rgb(r, g, b))
    } else {
        Err(())
    }
}

fn load_app_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../../../assets/icons/rmerge_64x64.png");
    if let Ok(img) = image::load_from_memory(bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidate_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "/System/Library/Fonts/SFPro.ttf",
    ];

    for path in candidate_paths {
        if let Ok(font_bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "system_fallback".to_owned(),
                Arc::new(egui::FontData::from_owned(font_bytes)),
            );
            if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                prop.push("system_fallback".to_owned());
            }
            if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                mono.push("system_fallback".to_owned());
            }
            break;
        }
    }

    ctx.set_fonts(fonts);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Iniciar runtime de Tokio para IPC en segundo plano
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    let initial_path = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let canonical_path = std::fs::canonicalize(&initial_path).unwrap_or(initial_path);

    // 2. Canal de comunicación entre el servidor IPC y la GUI
    let (ipc_tx, ipc_rx_async) = tokio::sync::mpsc::unbounded_channel::<IpcMessage>();
    IpcServer::start(ipc_tx);

    let (sync_tx, sync_rx) = std::sync::mpsc::channel::<IpcMessage>();
    rt.spawn(async move {
        let mut rx = ipc_rx_async;
        while let Some(msg) = rx.recv().await {
            let _ = sync_tx.send(msg);
        }
    });

    // 3. Configuración de la Ventana Nativa Desktop
    let icon_data = load_app_icon();
    let mut viewport = egui::ViewportBuilder::default()
        .with_app_id("rmerge-gui")
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([800.0, 500.0])
        .with_title("Git-Client");

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    println!("Abriendo ventana gráfica nativa de Git-Client en DISPLAY: {}...", std::env::var("DISPLAY").unwrap_or_default());

    // 4. Lanzar la aplicación en el loop nativo de eventos de la ventana
    eframe::run_native(
        "rmerge-gui",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            Ok(Box::new(RmergeGuiApp::new(canonical_path, sync_rx)))
        }),
    ).map_err(|e| format!("Error en interfaz gráfica: {e}"))?;

    Ok(())
}

fn generate_pr_url(remote: &str, branch: &str) -> Option<String> {
    let trimmed = remote.trim();
    if trimmed.contains("github.com") {
        let path = if let Some(idx) = trimmed.find("github.com:") {
            &trimmed[idx + "github.com:".len()..]
        } else if let Some(idx) = trimmed.find("github.com/") {
            &trimmed[idx + "github.com/".len()..]
        } else {
            return None;
        };
        let clean_path = path.trim_end_matches(".git").trim_start_matches('/');
        Some(format!("https://github.com/{clean_path}/compare/{branch}?expand=1"))
    } else if trimmed.contains("gitlab.com") {
        let path = if let Some(idx) = trimmed.find("gitlab.com:") {
            &trimmed[idx + "gitlab.com:".len()..]
        } else if let Some(idx) = trimmed.find("gitlab.com/") {
            &trimmed[idx + "gitlab.com/".len()..]
        } else {
            return None;
        };
        let clean_path = path.trim_end_matches(".git").trim_start_matches('/');
        Some(format!("https://gitlab.com/{clean_path}/-/merge_requests/new?merge_request%5Bsource_branch%5D={branch}"))
    } else if trimmed.contains("bitbucket.org") {
        let path = if let Some(idx) = trimmed.find("bitbucket.org:") {
            &trimmed[idx + "bitbucket.org:".len()..]
        } else if let Some(idx) = trimmed.find("bitbucket.org/") {
            &trimmed[idx + "bitbucket.org/".len()..]
        } else {
            return None;
        };
        let clean_path = path.trim_end_matches(".git").trim_start_matches('/');
        Some(format!("https://bitbucket.org/{clean_path}/pull-requests/new?source={branch}"))
    } else if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Some(trimmed.trim_end_matches(".git").to_string())
    } else {
        None
    }
}

const DEFAULT_GITIGNORE_TEMPLATE: &str = r#"# ============================================
# Reglas .gitignore generadas por Git-Client
# ============================================

# Compilación y artefactos de build
target/
dist/
build/
bin/
obj/
*.o
*.a
*.so
*.dylib
*.dll
*.exe

# Dependencias de paquetes
node_modules/
vendor/
.pnp
.pnp.js

# Entorno y credenciales confidenciales
.env
.env.local
*.env
*.local

# IDEs y Editores de código
.idea/
.vscode/
*.swp
*.swo
*~
.DS_Store
Thumbs.db

# Logs y volcados de depuración
*.log
logs/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_remote_host_and_port() {
        // SSH scp-like
        assert_eq!(
            extract_remote_host_and_port("git@git.ice.go.cr:Desarrollo/Recursos/ci-templates.git"),
            Some(("git.ice.go.cr".to_string(), 22))
        );
        assert_eq!(
            extract_remote_host_and_port("git@github.com:user/repo.git"),
            Some(("github.com".to_string(), 22))
        );

        // SSH url
        assert_eq!(
            extract_remote_host_and_port("ssh://git@git.ice.go.cr:2222/Desarrollo/ci-templates.git"),
            Some(("git.ice.go.cr".to_string(), 2222))
        );
        assert_eq!(
            extract_remote_host_and_port("ssh://git@github.com/user/repo.git"),
            Some(("github.com".to_string(), 22))
        );

        // HTTPS
        assert_eq!(
            extract_remote_host_and_port("https://github.com/rust-lang/rust.git"),
            Some(("github.com".to_string(), 443))
        );
        assert_eq!(
            extract_remote_host_and_port("https://gitlab.corp.net:8443/project/repo.git"),
            Some(("gitlab.corp.net".to_string(), 8443))
        );

        // HTTP
        assert_eq!(
            extract_remote_host_and_port("http://internal.git:8080/repo.git"),
            Some(("internal.git".to_string(), 8080))
        );

        // Git protocol
        assert_eq!(
            extract_remote_host_and_port("git://git.kernel.org/repo.git"),
            Some(("git.kernel.org".to_string(), 9418))
        );

        // Local paths (should NOT trigger remote connectivity check)
        assert_eq!(extract_remote_host_and_port("/home/user/my-repo"), None);
        assert_eq!(extract_remote_host_and_port("file:///home/user/my-repo"), None);
        assert_eq!(extract_remote_host_and_port(r"C:\Users\User\repo"), None);
        assert_eq!(extract_remote_host_and_port("C:/Users/User/repo"), None);
        assert_eq!(extract_remote_host_and_port(""), None);
    }
}
