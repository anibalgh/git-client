use rmerge_domain::entities::ThreeWayMergeFile;

pub struct MergeToolView;

impl MergeToolView {
    pub fn render_ascii(merge_file: &ThreeWayMergeFile) -> String {
        let mut out = String::new();
        out.push_str("╔══════════════════════════════════════════════════════════════════════════════════════════════╗\n");
        out.push_str(&format!("║ HERRAMIENTA 3-WAY MERGE: {:<67} ║\n", merge_file.file_path));
        out.push_str("╠══════════════════════════════╦══════════════════════════════╦════════════════════════════════╣\n");
        out.push_str("║ LOCAL / OURS                 ║ ANCESTRO / BASE              ║ REMOTO / THEIRS                ║\n");
        out.push_str("╠══════════════════════════════╬══════════════════════════════╬════════════════════════════════╣\n");

        for block in &merge_file.blocks {
            let ours_line = truncate(block.ours_content.lines().next().unwrap_or(""), 28);
            let base_line = truncate(block.ancestor_content.lines().next().unwrap_or(""), 28);
            let theirs_line = truncate(block.theirs_content.lines().next().unwrap_or(""), 30);

            out.push_str(&format!("║ {:<28} ║ {:<28} ║ {:<30} ║\n", ours_line, base_line, theirs_line));
            out.push_str("╠──────────────────────────────╨──────────────────────────────╨────────────────────────────────╣\n");
            let res_status = if let Some(ref res) = block.resolved_content {
                format!("RESUELTO: {}", truncate(res.lines().next().unwrap_or(""), 50))
            } else {
                "CONFLICTO PENDIENTE".to_string()
            };
            out.push_str(&format!("║ BLOQUE #{} -> {:<78} ║\n", block.id, res_status));
            out.push_str("║ ACCIONES: [ USAR OURS ]   [ USAR THEIRS ]   [ USAR BASE ]   [ COMBINAR AMBOS ]               ║\n");
            out.push_str("╠══════════════════════════════════════════════════════════════════════════════════════════════╣\n");
        }

        out.push_str("║                                                                                              ║\n");
        out.push_str("║  [ GUARDAR Y MARCAR CONFLICTO COMO RESUELTO ]                         [ SALIR ]              ║\n");
        out.push_str("╚══════════════════════════════════════════════════════════════════════════════════════════════╝\n");
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
