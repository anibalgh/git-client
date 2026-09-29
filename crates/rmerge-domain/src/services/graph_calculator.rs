use crate::entities::commit::{Commit, CommitGraph};

pub struct CommitGraphCalculator;

impl CommitGraphCalculator {
    /// Asigna carriles (lanes) a los commits para permitir el renderizado de árboles visuales fluidos
    pub fn compute_lanes(mut commits: Vec<Commit>) -> CommitGraph {
        if commits.is_empty() {
            return CommitGraph::default();
        }

        let mut active_lanes: Vec<Option<String>> = Vec::new();
        let mut max_lanes = 0;

        for commit in &mut commits {
            // 1. Buscar si el commit actual ya tiene un carril reservado por uno de sus hijos
            let assigned_lane = match active_lanes.iter().position(|slot| slot.as_deref() == Some(&commit.id)) {
                Some(index) => {
                    // Liberar o marcar temporalmente el slot
                    active_lanes[index] = None;
                    index
                }
                None => {
                    // Buscar el primer carril libre o añadir uno nuevo
                    match active_lanes.iter().position(|slot| slot.is_none()) {
                        Some(free_index) => free_index,
                        None => {
                            active_lanes.push(None);
                            active_lanes.len() - 1
                        }
                    }
                }
            };

            commit.lane = assigned_lane;

            // 2. Asignar los padres del commit a carriles para los siguientes pasos
            if let Some(first_parent) = commit.parent_ids.first() {
                // El primer padre continúa en el mismo carril
                active_lanes[assigned_lane] = Some(first_parent.clone());

                // Si hay padres adicionales (merges), se asignan a otros carriles libres
                for additional_parent in commit.parent_ids.iter().skip(1) {
                    if !active_lanes.iter().any(|slot| slot.as_deref() == Some(additional_parent)) {
                        match active_lanes.iter().position(|slot| slot.is_none()) {
                            Some(free_idx) => {
                                active_lanes[free_idx] = Some(additional_parent.clone());
                            }
                            None => {
                                active_lanes.push(Some(additional_parent.clone()));
                            }
                        }
                    }
                }
            } else {
                // Commit raíz (sin padres): liberar el carril
                active_lanes[assigned_lane] = None;
            }

            // Podar carriles vacíos al final del vector
            while active_lanes.last() == Some(&None) {
                active_lanes.pop();
            }

            if active_lanes.len() > max_lanes {
                max_lanes = active_lanes.len();
            }
        }

        CommitGraph {
            commits,
            max_lanes: max_lanes.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_linear_history_lanes() {
        let commits = vec![
            Commit {
                id: "c2".into(),
                short_id: "c2".into(),
                message_headline: "Second".into(),
                message_body: None,
                author_name: "Test".into(),
                author_email: "test@test.com".into(),
                authored_at: Utc::now(),
                parent_ids: vec!["c1".into()],
                lane: 0,
                branches: vec!["main".into()],
                tags: vec![],
            },
            Commit {
                id: "c1".into(),
                short_id: "c1".into(),
                message_headline: "First".into(),
                message_body: None,
                author_name: "Test".into(),
                author_email: "test@test.com".into(),
                authored_at: Utc::now(),
                parent_ids: vec![],
                lane: 0,
                branches: vec![],
                tags: vec![],
            },
        ];

        let graph = CommitGraphCalculator::compute_lanes(commits);
        assert_eq!(graph.commits[0].lane, 0);
        assert_eq!(graph.commits[1].lane, 0);
        assert_eq!(graph.max_lanes, 1);
    }
}
