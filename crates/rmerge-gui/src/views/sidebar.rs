use rmerge_application::ports::r#in::RepositoryData;

pub struct SidebarView;

impl SidebarView {
    pub fn render_ascii(repo_data: Option<&RepositoryData>, commit_msg: &str) -> String {
        let mut out = String::new();
        out.push_str("┌─ EXPLORADOR GIT ──────────────────────┐\n");

        if let Some(data) = repo_data {
            // Ramas
            out.push_str("│ ⎇ RAMAS:                               │\n");
            for b in &data.branches {
                let marker = if b.is_head { "* " } else { "  " };
                out.push_str(&format!("│ {:<37} │\n", format!("{marker}{}", b.name)));
            }

            // Staged Files
            out.push_str("├─ ＋ PREPARADOS (STAGED) ────────────────┤\n");
            if data.staging.staged.is_empty() {
                out.push_str("│   (Ningún cambio preparado)           │\n");
            } else {
                for file in &data.staging.staged {
                    out.push_str(&format!("│ [+]        {:<26} │\n", truncate_str(&file.path, 26)));
                }
            }

            // Unstaged Files
            out.push_str("├─ ● MODIFICADOS (UNSTAGED) ─────────────┤\n");
            if data.staging.unstaged.is_empty() {
                out.push_str("│   (Sin modificaciones locales)        │\n");
            } else {
                for file in &data.staging.unstaged {
                    out.push_str(&format!("│ [~]        {:<26} │\n", truncate_str(&file.path, 26)));
                }
            }

            // Untracked Files
            if !data.staging.untracked.is_empty() {
                out.push_str("├─ ? SIN SEGUIMIENTO (UNTRACKED) ────────┤\n");
                for file in &data.staging.untracked {
                    out.push_str(&format!("│ [?]        {:<26} │\n", truncate_str(file, 26)));
                }
            }

            // Conflictos
            if !data.staging.conflicts.is_empty() {
                out.push_str("├─ ▲ CONFLICTOS (3-WAY MERGE) ───────────┤\n");
                for file in &data.staging.conflicts {
                    out.push_str(&format!("│ [▲]        {:<26} │\n", truncate_str(file, 26)));
                }
            }
        } else {
            out.push_str("│ (Cargando datos del repositorio...)   │\n");
        }

        // Commit Box
        out.push_str("├─ MENSAJE DE COMMIT ───────────────────┤\n");
        let display_msg = if commit_msg.is_empty() {
            "Escriba un mensaje para el commit..."
        } else {
            commit_msg
        };
        out.push_str(&format!("│ {:<37} │\n", truncate_str(display_msg, 37)));
        out.push_str("│                                       │\n");
        out.push_str("│ [ COMMIT CAMBIOS (Ctrl+Enter) ]       │\n");
        out.push_str("└───────────────────────────────────────┘\n");

        out
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max {
        let prefix: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{prefix}…")
    } else {
        s.to_string()
    }
}
