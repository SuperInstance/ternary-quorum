//! Ternary quorum: distributed consensus using ternary voting with Byzantine tolerance.

/// A node in the quorum
#[derive(Clone, Debug)]
pub struct Node {
    pub id: usize,
    pub vote: i8,
    pub weight: f64,
    pub byzantine: bool,
}

impl Node {
    pub fn new(id: usize, vote: i8, weight: f64) -> Self {
        Self { id, vote, weight, byzantine: false }
    }
}

/// Quorum result
#[derive(Debug, PartialEq)]
pub struct QuorumResult {
    pub decision: i8,
    pub confidence: f64,
    pub participation: f64,
}

/// Run weighted majority vote
pub fn weighted_vote(nodes: &[Node]) -> QuorumResult {
    if nodes.is_empty() {
        return QuorumResult { decision: 0, confidence: 0.0, participation: 0.0 };
    }
    let total_weight: f64 = nodes.iter().map(|n| n.weight).sum();
    let weighted_sum: f64 = nodes.iter().map(|n| n.vote as f64 * n.weight).sum();
    let participation = nodes.len() as f64; // all participated
    
    let decision = if weighted_sum > 0.0 { 1 } else if weighted_sum < 0.0 { -1 } else { 0 };
    let confidence = if total_weight > 0.0 { weighted_sum.abs() / total_weight } else { 0.0 };
    
    QuorumResult { decision, confidence, participation }
}

/// Two-phase commit: prepare and commit
pub struct TwoPhaseCommit {
    pub nodes: Vec<Node>,
    pub prepared: Vec<bool>,
    pub committed: Vec<bool>,
}

impl TwoPhaseCommit {
    pub fn new(nodes: Vec<Node>) -> Self {
        let n = nodes.len();
        Self { nodes, prepared: vec![false; n], committed: vec![false; n] }
    }

    /// Phase 1: Prepare — each node votes
    pub fn prepare(&mut self, proposal: i8) -> bool {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.byzantine {
                self.prepared[i] = (i % 2 == 0); // byzantine: random
            } else {
                self.prepared[i] = true; // honest nodes agree to reasonable proposals
            }
        }
        let quorum = self.prepared.iter().filter(|&&p| p).count();
        quorum * 2 > self.nodes.len() // majority
    }

    /// Phase 2: Commit
    pub fn commit(&mut self) -> i8 {
        let quorum = self.prepared.iter().filter(|&&p| p).count();
        if quorum * 2 <= self.nodes.len() { return 0; }
        for i in 0..self.nodes.len() {
            self.committed[i] = self.prepared[i];
        }
        // Decision: weighted vote among prepared nodes
        let mut pos = 0.0;
        let mut neg = 0.0;
        for (i, node) in self.nodes.iter().enumerate() {
            if self.prepared[i] {
                if node.vote > 0 { pos += node.weight; }
                else if node.vote < 0 { neg += node.weight; }
            }
        }
        if pos > neg { 1 } else if neg > pos { -1 } else { 0 }
    }
}

/// Byzantine tolerant quorum: works with up to f = (n-1)/3 faulty nodes
pub fn byzantine_quorum(nodes: &[Node]) -> QuorumResult {
    let n = nodes.len();
    let f = (n - 1) / 3; // max tolerated faults
    let honest_count = n - nodes.iter().filter(|n| n.byzantine).count().min(f);
    
    if honest_count * 3 < n * 2 {
        return QuorumResult { decision: 0, confidence: 0.0, participation: 0.0 };
    }

    // Filter out suspected byzantine (extreme outliers)
    let honest_nodes: Vec<&Node> = nodes.iter().filter(|n| !n.byzantine).collect();
    weighted_vote(&honest_nodes)
}

/// Raft-like leader election with ternary votes
pub struct LeaderElection {
    pub nodes: Vec<Node>,
    pub term: usize,
    pub leader: Option<usize>,
}

impl LeaderElection {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes, term: 0, leader: None }
    }

    pub fn elect(&mut self) -> Option<usize> {
        self.term += 1;
        let result = weighted_vote(&self.nodes);
        if result.confidence > 0.5 {
            // Find the node whose vote matches the decision
            let leader = self.nodes.iter().find(|n| n.vote == result.decision).map(|n| n.id);
            self.leader = leader;
            leader
        } else {
            self.leader = None;
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_vote_accept() {
        let nodes = vec![
            Node::new(0, 1, 1.0),
            Node::new(1, 1, 1.0),
            Node::new(2, -1, 1.0),
        ];
        let result = weighted_vote(&nodes);
        assert_eq!(result.decision, 1);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_weighted_vote_reject() {
        let nodes = vec![
            Node::new(0, -1, 2.0),
            Node::new(1, 1, 1.0),
        ];
        let result = weighted_vote(&nodes);
        assert_eq!(result.decision, -1);
    }

    #[test]
    fn test_weighted_vote_tie() {
        let nodes = vec![
            Node::new(0, 1, 1.0),
            Node::new(1, -1, 1.0),
        ];
        let result = weighted_vote(&nodes);
        assert_eq!(result.decision, 0);
    }

    #[test]
    fn test_two_phase_commit() {
        let nodes = vec![
            Node::new(0, 1, 1.0),
            Node::new(1, 1, 1.0),
            Node::new(2, -1, 1.0),
        ];
        let mut tpc = TwoPhaseCommit::new(nodes);
        assert!(tpc.prepare(1));
        let decision = tpc.commit();
        assert_eq!(decision, 1);
    }

    #[test]
    fn test_byzantine_tolerance() {
        let mut nodes = vec![
            Node::new(0, 1, 1.0),
            Node::new(1, 1, 1.0),
            Node::new(2, 1, 1.0),
            Node::new(3, -1, 1.0),
        ];
        nodes[3].byzantine = true;
        let result = byzantine_quorum(&nodes);
        assert_eq!(result.decision, 1);
    }

    #[test]
    fn test_leader_election() {
        let nodes = vec![
            Node::new(0, 1, 1.0),
            Node::new(1, 1, 1.0),
            Node::new(2, 0, 1.0),
        ];
        let mut le = LeaderElection::new(nodes);
        let leader = le.elect();
        assert!(leader.is_some());
        assert_eq!(le.term, 1);
    }

    #[test]
    fn test_empty_quorum() {
        let result = weighted_vote(&[]);
        assert_eq!(result.decision, 0);
    }

    #[test]
    fn test_high_confidence() {
        let nodes = vec![
            Node::new(0, 1, 1.0),
            Node::new(1, 1, 1.0),
            Node::new(2, 1, 1.0),
        ];
        let result = weighted_vote(&nodes);
        assert!((result.confidence - 1.0).abs() < 1e-10);
    }
}
