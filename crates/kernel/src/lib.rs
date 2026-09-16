use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub claim: String,
    pub evidence: Vec<String>,
    pub verified: bool,
    pub visits: usize,
    pub reward: f64,
    pub parent_id: Option<String>,
}

impl Hypothesis {
    pub fn new(claim: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            claim,
            evidence: vec![],
            verified: false,
            visits: 0,
            reward: 0.0,
            parent_id: None,
        }
    }

    pub fn ucb1(&self, exploration: f64) -> f64 {
        if self.visits == 0 {
            f64::INFINITY
        } else {
            let exploitation = self.reward / (self.visits as f64);
            let exploration_term = exploration * ((self.visits as f64).ln().sqrt());
            exploitation + exploration_term
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResearchMove {
    ProposeSubClaim(String),
    QueryTool(String),
    RequestProof,
}

pub struct ResearchMCTS {
    nodes: HashMap<String, Hypothesis>,
    root_id: String,
}

impl ResearchMCTS {
    pub fn new(initial_claim: String) -> Self {
        let root = Hypothesis::new(initial_claim);
        let root_id = root.id.clone();
        let mut nodes = HashMap::new();
        nodes.insert(root_id.clone(), root);

        Self { nodes, root_id }
    }

    pub fn expand(&mut self, node_id: &str, moves: Vec<ResearchMove>) {
        for mov in moves {
            let new_claim = match mov {
                ResearchMove::ProposeSubClaim(c) => c,
                ResearchMove::QueryTool(t) => format!("Query: {}", t),
                ResearchMove::RequestProof => "Proof requested".to_string(),
            };

            let mut new_node = Hypothesis::new(new_claim);
            new_node.parent_id = Some(node_id.to_string());
            self.nodes.insert(new_node.id.clone(), new_node);
        }
    }

    pub fn backprop(&mut self, node_id: &str, reward: f64) {
        let mut current_id = Some(node_id.to_string());

        while let Some(id) = current_id {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.visits += 1;
                node.reward += reward;
                current_id = node.parent_id.clone();
            } else {
                break;
            }
        }
    }

    pub fn best_child(&self, node_id: &str, exploration: f64) -> Option<String> {
        let children: Vec<_> = self
            .nodes
            .values()
            .filter(|n| n.parent_id.as_deref() == Some(node_id))
            .collect();

        children.into_iter().max_by(|a, b| {
            a.ucb1(exploration)
                .partial_cmp(&b.ucb1(exploration))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        None // TODO: return the best child ID
    }

    pub fn root(&self) -> &Hypothesis {
        &self.nodes[&self.root_id]
    }

    pub fn get_node(&self, id: &str) -> Option<&Hypothesis> {
        self.nodes.get(id)
    }

    pub fn all_nodes(&self) -> Vec<&Hypothesis> {
        self.nodes.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypothesis_creation() {
        let h = Hypothesis::new("P != NP".to_string());
        assert!(!h.claim.is_empty());
        assert!(!h.verified);
    }

    #[test]
    fn test_mcts_initialization() {
        let mcts = ResearchMCTS::new("Fermat's Last Theorem".to_string());
        assert_eq!(mcts.root().claim, "Fermat's Last Theorem");
        assert_eq!(mcts.root().visits, 0);
    }

    #[test]
    fn test_ucb1_unvisited_node() {
        let h = Hypothesis::new("test".to_string());
        assert_eq!(h.ucb1(1.0), f64::INFINITY);
    }

    #[test]
    fn test_backprop() {
        let mut mcts = ResearchMCTS::new("root".to_string());
        let root_id = mcts.root_id.clone();

        mcts.backprop(&root_id, 0.8);
        assert_eq!(mcts.root().visits, 1);
        assert_eq!(mcts.root().reward, 0.8);
    }
}
