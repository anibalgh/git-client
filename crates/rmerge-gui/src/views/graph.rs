use rmerge_domain::entities::CommitGraph;

pub struct CommitGraphView;

impl CommitGraphView {
    pub fn render_ascii(graph: &CommitGraph) -> String {
        let mut out = String::new();
        out.push_str("┌─ HISTORIAL DE COMMITS (GRAFO) ────────────────────────────────────────────────────────┐\n");
        out.push_str("│ GRAFO │ HASH    │ MENSAJE                             │ AUTOR          │ FECHA        │\n");
        out.push_str("├───────┼─────────┼─────────────────────────────────────┼────────────────┼──────────────┤\n");

        if graph.commits.is_empty() {
            out.push_str("│       │         │ (Repositorio sin commits registrados) │                │              │\n");
        } else {
            for commit in &graph.commits {
                let mut lane_str = String::new();
                for l in 0..graph.max_lanes.max(1) {
                    if l == commit.lane {
                        lane_str.push('*');
                    } else {
                        lane_str.push('|');
                    }
                    lane_str.push(' ');
                }

                let branches_tag = if !commit.branches.is_empty() {
                    format!(" [{}]", commit.branches.join(", "))
                } else {
                    String::new()
                };

                let full_msg = format!("{}{branches_tag}", commit.message_headline);
                let truncated_msg = truncate(&full_msg, 35);
                let truncated_author = truncate(&commit.author_name, 14);
                let date_str = commit.authored_at.format("%Y-%m-%d").to_string();

                out.push_str(&format!(
                    "│ {:<5} │ {:<7} │ {:<35} │ {:<14} │ {:<12} │\n",
                    truncate(&lane_str, 5),
                    commit.short_id,
                    truncated_msg,
                    truncated_author,
                    date_str
                ));
            }
        }

        out.push_str("└───────┴─────────┴─────────────────────────────────────┴────────────────┴──────────────┘\n");
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
