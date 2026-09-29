use crate::entities::conflict::{ConflictBlock, ConflictResolution, ThreeWayMergeFile};

pub struct ThreeWayMergeService;

impl ThreeWayMergeService {
    /// Parsea un archivo con marcadores de conflicto de Git (formato diff3 o standard)
    pub fn parse_conflicted_content(file_path: &str, content: &str) -> ThreeWayMergeFile {
        let mut blocks = Vec::new();
        let mut block_id = 0;

        let mut current_clean = Vec::new();
        let mut in_conflict = false;
        let mut in_base = false;
        let mut in_theirs = false;

        let mut ours_lines = Vec::new();
        let mut base_lines = Vec::new();
        let mut theirs_lines = Vec::new();

        for line in content.lines() {
            if line.starts_with("<<<<<<<") {
                in_conflict = true;
                in_base = false;
                in_theirs = false;

                // Si había contenido limpio previo, guárdalo como bloque pre-resuelto
                if !current_clean.is_empty() {
                    let clean_text = current_clean.join("\n");
                    let mut b = ConflictBlock::new(block_id, &clean_text, &clean_text, &clean_text);
                    b.apply_resolution(ConflictResolution::Custom(clean_text));
                    blocks.push(b);
                    block_id += 1;
                    current_clean.clear();
                }

                ours_lines.clear();
                base_lines.clear();
                theirs_lines.clear();
            } else if in_conflict && line.starts_with("|||||||") {
                // Comienzo de la sección Base en modo diff3
                in_base = true;
            } else if in_conflict && line.starts_with("=======") {
                // Comienzo de la sección Theirs
                in_base = false;
                in_theirs = true;
            } else if in_conflict && line.starts_with(">>>>>>>") {
                // Fin del conflicto
                let ours_text = ours_lines.join("\n");
                let base_text = base_lines.join("\n");
                let theirs_text = theirs_lines.join("\n");

                let conflict_b = ConflictBlock::new(block_id, base_text, ours_text, theirs_text);
                blocks.push(conflict_b);
                block_id += 1;

                in_conflict = false;
                in_base = false;
                in_theirs = false;
                ours_lines.clear();
                base_lines.clear();
                theirs_lines.clear();
            } else if in_conflict {
                if in_theirs {
                    theirs_lines.push(line);
                } else if in_base {
                    base_lines.push(line);
                } else {
                    ours_lines.push(line);
                }
            } else {
                current_clean.push(line);
            }
        }

        // Agregar el resto del archivo limpio si quedó algo
        if !current_clean.is_empty() {
            let clean_text = current_clean.join("\n");
            let mut b = ConflictBlock::new(block_id, &clean_text, &clean_text, &clean_text);
            b.apply_resolution(ConflictResolution::Custom(clean_text));
            blocks.push(b);
        }

        ThreeWayMergeFile {
            file_path: file_path.to_string(),
            blocks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conflict() {
        let content = r#"fn main() {
<<<<<<< HEAD
    println!("Hola desde local");
=======
    println!("Hola desde remoto");
>>>>>>> branch-remota
}
"#;
        let mut merge_file = ThreeWayMergeService::parse_conflicted_content("main.rs", content);
        assert_eq!(merge_file.blocks.len(), 3); // Limpio inicio, conflicto, limpio fin

        // Resolver el bloque de conflicto tomando Ours
        assert!(!merge_file.is_fully_resolved());
        merge_file.blocks[1].apply_resolution(ConflictResolution::UseOurs);
        assert!(merge_file.is_fully_resolved());

        let assembled = merge_file.assemble_merged_file().unwrap();
        assert!(assembled.contains("Hola desde local"));
        assert!(!assembled.contains("Hola desde remoto"));
    }
}
