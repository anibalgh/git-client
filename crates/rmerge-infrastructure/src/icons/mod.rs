use std::collections::HashMap;
use rmerge_domain::value_objects::{IconId, ThemeConfig};

pub struct ThemeIconService {
    templates: HashMap<IconId, &'static str>,
}

impl ThemeIconService {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // 1. Git Commit (Lucide based, theme colored)
        templates.insert(
            IconId::GitCommit,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{COLOR}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><line x1="3" y1="12" x2="9" y2="12"/><line x1="15" y1="12" x2="21" y2="12"/></svg>"#,
        );

        // 2. Git Branch
        templates.insert(
            IconId::GitBranch,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{COLOR}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="6" y1="3" x2="6" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg>"#,
        );

        // 3. Git Merge
        templates.insert(
            IconId::GitMerge,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{COLOR}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M6 21V9a9 9 0 0 0 9 9"/></svg>"#,
        );

        // 4. Git Pull Request
        templates.insert(
            IconId::GitPullRequest,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{COLOR}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M13 6h3a2 2 0 0 1 2 2v7"/><line x1="6" y1="9" x2="6" y2="21"/></svg>"#,
        );

        // 5. Git Compare
        templates.insert(
            IconId::GitCompare,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{COLOR}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M13 6h3a2 2 0 0 1 2 2v7"/><path d="M11 18H8a2 2 0 0 1-2-2V9"/></svg>"#,
        );

        // 6. Diff Added (+)
        templates.insert(
            IconId::DiffAdded,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{DIFF_ADD}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/></svg>"#,
        );

        // 7. Diff Removed (-)
        templates.insert(
            IconId::DiffRemoved,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{DIFF_DEL}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="8" y1="12" x2="16" y2="12"/></svg>"#,
        );

        // 8. Diff Modified (~)
        templates.insert(
            IconId::DiffModified,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{DIFF_MOD}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="7" x2="12" y2="13"/><circle cx="12" cy="17" r="1" fill="{{DIFF_MOD}}"/></svg>"#,
        );

        // 9. Stage Changes (Plus Box)
        templates.insert(
            IconId::Stage,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{DIFF_ADD}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/></svg>"#,
        );

        // 10. Unstage Changes (Minus Box)
        templates.insert(
            IconId::Unstage,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{DIFF_DEL}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="8" y1="12" x2="16" y2="12"/></svg>"#,
        );

        // 11. Discard Changes (Revert / Undo)
        templates.insert(
            IconId::Discard,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{DIFF_DEL}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v6h6"/><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"/></svg>"#,
        );

        // 12. Tag
        templates.insert(
            IconId::Tag,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{ACCENT}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>"#,
        );

        // 13. Search
        templates.insert(
            IconId::Search,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{FG_PRIMARY}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>"#,
        );

        // 14. Settings (Gear)
        templates.insert(
            IconId::Settings,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{FG_MUTED}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>"#,
        );

        // 15. User (Identity / Author)
        templates.insert(
            IconId::User,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{ACCENT}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>"#,
        );

        // 16. Check (Success)
        templates.insert(
            IconId::Check,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{DIFF_ADD}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>"#,
        );

        // 17. Alert / Conflict Triangle
        templates.insert(
            IconId::AlertTriangle,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{CONFLICT}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>"#,
        );

        // 18. Folder Git
        templates.insert(
            IconId::FolderGit,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{ACCENT}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="2"/></svg>"#,
        );

        // 19. Terminal
        templates.insert(
            IconId::Terminal,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="{{FG_PRIMARY}}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>"#,
        );

        Self { templates }
    }

    /// Renderiza un icono SVG aplicando la paleta de colores del tema seleccionado
    pub fn render_svg(&self, icon_id: IconId, theme: &ThemeConfig) -> String {
        let template = self.templates.get(&icon_id).copied().unwrap_or(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" stroke="#888888"/></svg>"##,
        );

        let default_color = &theme.fg_primary;
        template
            .replace("{{COLOR}}", default_color)
            .replace("{{ACCENT}}", &theme.accent)
            .replace("{{FG_PRIMARY}}", &theme.fg_primary)
            .replace("{{FG_MUTED}}", &theme.fg_muted)
            .replace("{{DIFF_ADD}}", &theme.diff_add_fg)
            .replace("{{DIFF_DEL}}", &theme.diff_del_fg)
            .replace("{{DIFF_MOD}}", &theme.diff_mod_fg)
            .replace("{{CONFLICT}}", &theme.conflict_border)
    }

    /// Guarda la colección completa de iconos SVG con los colores del tema indicado en una carpeta
    pub fn export_themed_icons(&self, dest_dir: &std::path::Path, theme: &ThemeConfig) -> std::io::Result<()> {
        std::fs::create_dir_all(dest_dir)?;
        for id in self.templates.keys() {
            let svg_content = self.render_svg(*id, theme);
            let filename = format!("{id:?}.svg").to_lowercase();
            let path = dest_dir.join(filename);
            std::fs::write(path, svg_content)?;
        }
        Ok(())
    }
}

impl Default for ThemeIconService {
    fn default() -> Self {
        Self::new()
    }
}
