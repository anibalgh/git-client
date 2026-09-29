use rmerge_domain::entities::{FilePatch, LineKind};

pub struct DiffInspectorView;

impl DiffInspectorView {
    pub fn render_ascii(patch: Option<&FilePatch>) -> String {
        let mut out = String::new();
        out.push_str("┌─ VISOR DE DIFERENCIAS (DIFF INSPECTOR) ───────────────────────────────────────────────┐\n");

        if let Some(file_patch) = patch {
            out.push_str(&format!("│ ARCHIVO: {:<77} │\n", file_patch.path));
            out.push_str("├──────┬──────┬───┬─────────────────────────────────────────────────────────────────────┤\n");
            out.push_str("│ VIEJO│ NUEVO│ T │ CONTENIDO                                                           │\n");
            out.push_str("├──────┼──────┼───┼─────────────────────────────────────────────────────────────────────┤\n");

            for hunk in &file_patch.hunks {
                out.push_str(&format!("│ @@@  │ @@@  │ H │ {:<67} │\n", truncate(&hunk.header, 67)));
                for line in &hunk.lines {
                    let old_str = line.old_line_no.map_or("".into(), |n| format!("{n:<4}"));
                    let new_str = line.new_line_no.map_or("".into(), |n| format!("{n:<4}"));
                    let type_char = match line.kind {
                        LineKind::Addition => '+',
                        LineKind::Deletion => '-',
                        LineKind::Context => ' ',
                        LineKind::Header => '@',
                    };
                    let content_preview = truncate(line.content.trim_end(), 67);
                    out.push_str(&format!(
                        "│ {:<4} │ {:<4} │ {} │ {:<67} │\n",
                        old_str, new_str, type_char, content_preview
                    ));
                }
            }

            out.push_str("├──────┴──────┴───┴─────────────────────────────────────────────────────────────────────┤\n");
            out.push_str("│ ACCIONES DE HUNK: [ PREPARAR HUNK ]    [ PREPARAR LÍNEAS ]    [ DESCARTAR CAMBIOS ]   │\n");
        } else {
            out.push_str("│ Seleccione un archivo modificado en la barra lateral para inspeccionar sus cambios.   │\n");
        }

        out.push_str("└───────────────────────────────────────────────────────────────────────────────────────┘\n");
        out
    }
}

fn truncate(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count > max {
        let prefix: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{prefix}…")
    } else {
        s.to_string()
    }
}
