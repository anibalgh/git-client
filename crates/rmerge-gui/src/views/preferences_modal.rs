use rmerge_domain::value_objects::TypographyConfig;

pub struct PreferencesModalView;

impl PreferencesModalView {
    pub fn render_ascii(typo: &TypographyConfig, active_theme: &str, available_themes: &[String]) -> String {
        let mut out = String::new();
        out.push_str("╔════════════════════════════════════════════════════════════════════════════════╗\n");
        out.push_str("║                     PREFERENCIAS DE APARIENCIA Y FUENTES                       ║\n");
        out.push_str("╟────────────────────────────────────────────────────────────────────────────────╢\n");
        out.push_str(&format!("║  Tema Activo:                 [ {:<46} ] ║\n", active_theme));
        out.push_str(&format!("║  Temas Disponibles:           {:<48} ║\n", available_themes.join(", ")));
        out.push_str("║                                                                                ║\n");
        out.push_str("║  TIPOGRAFÍA DE INTERFAZ (UI):                                                  ║\n");
        out.push_str(&format!("║    Familia:                   [ {:<46} ] ║\n", typo.ui_font.family));
        out.push_str(&format!("║    Tamaño:                    [ {:<5.1} pt ]                                     ║\n", typo.ui_font.size_pt));
        out.push_str("║                                                                                ║\n");
        out.push_str("║  TIPOGRAFÍA DE CÓDIGO Y DIFFS:                                                 ║\n");
        out.push_str(&format!("║    Familia Monoespaciada:     [ {:<46} ] ║\n", typo.code_font.family));
        out.push_str(&format!("║    Tamaño:                    [ {:<5.1} pt ]                                     ║\n", typo.code_font.size_pt));
        out.push_str(&format!("║    Altura de Línea:           [ {:<5.2} x  ]                                     ║\n", typo.code_font.line_height));
        out.push_str(&format!("║    Ligaduras Tipográficas:    [{}] Habilitado (ej. ->, !=, ===)                ║\n", if typo.code_font.enable_ligatures { "X" } else { " " }));
        out.push_str("║                                                                                ║\n");
        out.push_str("║  VISTA PREVIA DE DIFF:                                                         ║\n");
        out.push_str("║    + fn calculate_merge_resolution(ours: &Hunk, theirs: &Hunk) -> Result<()>  ║\n");
        out.push_str("║    - fn calculate_merge(ours: &Hunk) -> Result<()>                            ║\n");
        out.push_str("║                                                                                ║\n");
        out.push_str("║    [ APLICAR Y GUARDAR ]                           [ CANCELAR ]                ║\n");
        out.push_str("╚════════════════════════════════════════════════════════════════════════════════╝\n");
        out
    }
}
