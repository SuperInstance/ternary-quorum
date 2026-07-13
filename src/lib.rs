#![forbid(unsafe_code)]

//! Distributed decision making with ternary voting and configurable thresholds.

/// A ternary vote: For, Against, or Abstain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ternary {
    Neg = -1, // Against
    Zero = 0, // Abstain
    Pos = 1,  // For
}

impl Ternary {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// Unique identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AgentId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProposalId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoundId(pub u64);

/// A voting body with members and active proposals.
#[derive(Clone, Debug)]
pub struct Quorum {
    pub members: Vec<AgentId>,
    pub proposals: Vec<QuorumProposal>,
    pub threshold: QuorumThreshold,
    next_proposal_id: u64,
}

impl Quorum {
    pub fn new(threshold: QuorumThreshold) -> Self {
        Quorum {
            members: Vec::new(),
            proposals: Vec::new(),
            threshold,
            next_proposal_id: 0,
        }
    }

    pub fn add_member(&mut self, agent: AgentId) -> bool {
        if self.members.contains(&agent) {
            return false;
        }
        self.members.push(agent);
        true
    }

    pub fn remove_member(&mut self, agent: AgentId) -> bool {
        let before = self.members.len();
        self.members.retain(|m| *m != agent);
        self.members.len() < before
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Submit a new proposal.
    pub fn propose(&mut self, proposer: AgentId, description: &str) -> Option<ProposalId> {
        if !self.members.contains(&proposer) {
            return None;
        }
        let id = ProposalId(self.next_proposal_id);
        self.next_proposal_id += 1;
        self.proposals.push(QuorumProposal {
            id,
            proposer,
            description: description.to_string(),
            votes: Vec::new(),
            status: ProposalStatus::Open,
        });
        Some(id)
    }

    pub fn proposal(&self, id: ProposalId) -> Option<&QuorumProposal> {
        self.proposals.iter().find(|p| p.id == id)
    }

    pub fn proposal_mut(&mut self, id: ProposalId) -> Option<&mut QuorumProposal> {
        self.proposals.iter_mut().find(|p| p.id == id)
    }
}

/// A proposal with ternary voting options.
#[derive(Clone, Debug)]
pub struct QuorumProposal {
    pub id: ProposalId,
    pub proposer: AgentId,
    pub description: String,
    pub votes: Vec<QuorumVote>,
    pub status: ProposalStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalStatus {
    Open,
    Accepted,
    Rejected,
    Expired,
}

/// A vote cast by an agent.
#[derive(Clone, Debug)]
pub struct QuorumVote {
    pub voter: AgentId,
    pub vote: Ternary,
    pub round: RoundId,
}

impl QuorumVote {
    pub fn new(voter: AgentId, vote: Ternary, round: RoundId) -> Self {
        QuorumVote { voter, vote, round }
    }

    /// Cast a vote on a proposal. Returns false if voter isn't a member or already voted this round.
    pub fn cast(
        quorum: &mut Quorum,
        proposal_id: ProposalId,
        voter: AgentId,
        vote: Ternary,
        round: RoundId,
    ) -> bool {
        if !quorum.members.contains(&voter) {
            return false;
        }
        if let Some(proposal) = quorum.proposal_mut(proposal_id) {
            if proposal.status != ProposalStatus::Open {
                return false;
            }
            // Check if already voted in this round
            if proposal
                .votes
                .iter()
                .any(|v| v.voter == voter && v.round == round)
            {
                return false;
            }
            proposal.votes.push(QuorumVote::new(voter, vote, round));
            return true;
        }
        false
    }
}

/// Configurable majority threshold.
#[derive(Clone, Copy, Debug)]
pub struct QuorumThreshold {
    /// Fraction of non-abstain votes needed to pass (0.0 to 1.0).
    pub majority_fraction: u32, // Stored as parts per thousand
    /// Minimum number of votes required (including abstains).
    pub min_votes: usize,
}

impl QuorumThreshold {
    pub fn simple_majority() -> Self {
        QuorumThreshold {
            majority_fraction: 500, // 50%
            min_votes: 1,
        }
    }

    pub fn super_majority() -> Self {
        QuorumThreshold {
            majority_fraction: 667, // ~66.7%
            min_votes: 1,
        }
    }

    pub fn unanimous() -> Self {
        QuorumThreshold {
            majority_fraction: 1000, // 100%
            min_votes: 1,
        }
    }

    pub fn custom(majority_fraction: u32, min_votes: usize) -> Self {
        QuorumThreshold {
            majority_fraction: majority_fraction.min(1000),
            min_votes,
        }
    }

    /// Check if a vote count meets the threshold.
    pub fn is_met(&self, for_votes: usize, against_votes: usize, abstain_votes: usize) -> bool {
        let total_votes = for_votes + against_votes + abstain_votes;
        if total_votes < self.min_votes {
            return false;
        }
        let decisive = for_votes + against_votes;
        if decisive == 0 {
            return false;
        }
        let ratio = (for_votes as u32 * 1000) / decisive as u32;
        ratio >= self.majority_fraction
    }
}

/// Multi-round voting with escalation.
#[derive(Clone, Debug)]
pub struct QuorumRound {
    pub current_round: RoundId,
    pub max_rounds: u32,
    pub results: Vec<RoundResult>,
}

#[derive(Clone, Debug)]
pub struct RoundResult {
    pub round: RoundId,
    pub for_votes: usize,
    pub against_votes: usize,
    pub abstain_votes: usize,
    pub outcome: Option<ProposalStatus>,
}

impl QuorumRound {
    pub fn new(max_rounds: u32) -> Self {
        QuorumRound {
            current_round: RoundId(0),
            max_rounds,
            results: Vec::new(),
        }
    }

    /// Advance to the next round.
    pub fn advance(&mut self) -> bool {
        if (self.current_round.0 as u32) < self.max_rounds {
            self.current_round = RoundId(self.current_round.0 + 1);
            true
        } else {
            false
        }
    }

    /// Record the result of a round.
    pub fn record_result(
        &mut self,
        for_votes: usize,
        against_votes: usize,
        abstain_votes: usize,
        threshold: &QuorumThreshold,
    ) -> ProposalStatus {
        let outcome = if threshold.is_met(for_votes, against_votes, abstain_votes) {
            ProposalStatus::Accepted
        } else if for_votes > against_votes {
            // Not enough for threshold but plurality for — try another round
            ProposalStatus::Open
        } else if against_votes > for_votes {
            // Reject only if the against-side also meets the threshold fraction,
            // consistent with QuorumConsensus::is_consensus and the README spec:
            // "If against > for and meets threshold → Rejected."
            let decisive = for_votes + against_votes;
            let ratio = (against_votes as u32 * 1000) / decisive as u32;
            if ratio >= threshold.majority_fraction {
                ProposalStatus::Rejected
            } else {
                ProposalStatus::Open
            }
        } else {
            ProposalStatus::Open
        };
        self.results.push(RoundResult {
            round: self.current_round,
            for_votes,
            against_votes,
            abstain_votes,
            outcome: Some(outcome),
        });
        outcome
    }

    pub fn round_count(&self) -> usize {
        self.results.len()
    }
}

/// Reach consensus through the quorum process.
#[derive(Clone, Debug)]
pub struct QuorumConsensus;

impl QuorumConsensus {
    /// Tally votes for a proposal, returning (for, against, abstain) counts.
    pub fn tally(proposal: &QuorumProposal) -> (usize, usize, usize) {
        let for_votes = proposal
            .votes
            .iter()
            .filter(|v| v.vote == Ternary::Pos)
            .count();
        let against = proposal
            .votes
            .iter()
            .filter(|v| v.vote == Ternary::Neg)
            .count();
        let abstain = proposal
            .votes
            .iter()
            .filter(|v| v.vote == Ternary::Zero)
            .count();
        (for_votes, against, abstain)
    }

    /// Check if consensus has been reached.
    pub fn is_consensus(proposal: &QuorumProposal, threshold: &QuorumThreshold) -> ProposalStatus {
        let (for_votes, against, abstain) = Self::tally(proposal);
        if threshold.is_met(for_votes, against, abstain) {
            ProposalStatus::Accepted
        } else if against > for_votes && (for_votes + against) > 0 {
            let ratio = (against as u32 * 1000) / (for_votes + against) as u32;
            if ratio >= threshold.majority_fraction {
                ProposalStatus::Rejected
            } else {
                ProposalStatus::Open
            }
        } else {
            ProposalStatus::Open
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_values() {
        assert_eq!(Ternary::from_i8(-1), Some(Ternary::Neg));
        assert_eq!(Ternary::from_i8(0), Some(Ternary::Zero));
        assert_eq!(Ternary::from_i8(1), Some(Ternary::Pos));
    }

    #[test]
    fn test_quorum_add_member() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        assert!(q.add_member(AgentId(1)));
        assert_eq!(q.member_count(), 1);
    }

    #[test]
    fn test_quorum_no_duplicate_member() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        assert!(q.add_member(AgentId(1)));
        assert!(!q.add_member(AgentId(1)));
    }

    #[test]
    fn test_quorum_remove_member() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        assert!(q.remove_member(AgentId(1)));
        assert_eq!(q.member_count(), 0);
    }

    #[test]
    fn test_quorum_propose() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        let pid = q.propose(AgentId(1), "Deploy to prod?");
        assert!(pid.is_some());
        assert!(q.proposal(pid.unwrap()).is_some());
    }

    #[test]
    fn test_quorum_propose_non_member_fails() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        let pid = q.propose(AgentId(99), "Test?");
        assert!(pid.is_none());
    }

    #[test]
    fn test_vote_cast() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        q.add_member(AgentId(2));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        assert!(QuorumVote::cast(
            &mut q,
            pid,
            AgentId(1),
            Ternary::Pos,
            RoundId(0)
        ));
        assert!(QuorumVote::cast(
            &mut q,
            pid,
            AgentId(2),
            Ternary::Neg,
            RoundId(0)
        ));
    }

    #[test]
    fn test_vote_cast_twice_same_round_fails() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        assert!(QuorumVote::cast(
            &mut q,
            pid,
            AgentId(1),
            Ternary::Pos,
            RoundId(0)
        ));
        assert!(!QuorumVote::cast(
            &mut q,
            pid,
            AgentId(1),
            Ternary::Neg,
            RoundId(0)
        ));
    }

    #[test]
    fn test_vote_non_member_fails() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        assert!(!QuorumVote::cast(
            &mut q,
            pid,
            AgentId(99),
            Ternary::Pos,
            RoundId(0)
        ));
    }

    #[test]
    fn test_threshold_simple_majority() {
        let t = QuorumThreshold::simple_majority();
        assert!(t.is_met(3, 2, 1)); // 3/5 = 60% > 50%
        assert!(!t.is_met(2, 3, 1)); // 2/5 = 40% < 50%
    }

    #[test]
    fn test_threshold_super_majority() {
        let t = QuorumThreshold::super_majority();
        assert!(t.is_met(7, 2, 1)); // 7/9 = 77.7% > 66.7%
        assert!(!t.is_met(5, 3, 2)); // 5/8 = 62.5% < 66.7%
    }

    #[test]
    fn test_threshold_unanimous() {
        let t = QuorumThreshold::unanimous();
        assert!(t.is_met(5, 0, 0)); // 100%
        assert!(!t.is_met(4, 1, 0)); // 80%
    }

    #[test]
    fn test_threshold_min_votes() {
        let t = QuorumThreshold::custom(500, 5);
        assert!(!t.is_met(3, 0, 0)); // Only 3 votes, need 5
    }

    #[test]
    fn test_threshold_no_decisive_votes() {
        let t = QuorumThreshold::simple_majority();
        assert!(!t.is_met(0, 0, 5)); // All abstain, no decisive votes
    }

    #[test]
    fn test_round_advance() {
        let mut round = QuorumRound::new(3);
        assert!(round.advance());
        assert_eq!(round.current_round, RoundId(1));
        assert!(round.advance());
        assert!(round.advance());
        assert!(!round.advance()); // maxed out
    }

    #[test]
    fn test_round_record_result() {
        let mut round = QuorumRound::new(3);
        let threshold = QuorumThreshold::simple_majority();
        let result = round.record_result(3, 1, 1, &threshold);
        assert_eq!(result, ProposalStatus::Accepted);
    }

    #[test]
    fn test_round_record_rejection() {
        let mut round = QuorumRound::new(3);
        let threshold = QuorumThreshold::simple_majority();
        let result = round.record_result(1, 3, 1, &threshold);
        assert_eq!(result, ProposalStatus::Rejected);
    }

    #[test]
    fn test_consensus_tally() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        q.add_member(AgentId(2));
        q.add_member(AgentId(3));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        QuorumVote::cast(&mut q, pid, AgentId(1), Ternary::Pos, RoundId(0));
        QuorumVote::cast(&mut q, pid, AgentId(2), Ternary::Pos, RoundId(0));
        QuorumVote::cast(&mut q, pid, AgentId(3), Ternary::Neg, RoundId(0));
        let proposal = q.proposal(pid).unwrap();
        let (f, a, ab) = QuorumConsensus::tally(proposal);
        assert_eq!(f, 2);
        assert_eq!(a, 1);
        assert_eq!(ab, 0);
    }

    #[test]
    fn test_consensus_is_reached() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        q.add_member(AgentId(2));
        q.add_member(AgentId(3));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        QuorumVote::cast(&mut q, pid, AgentId(1), Ternary::Pos, RoundId(0));
        QuorumVote::cast(&mut q, pid, AgentId(2), Ternary::Pos, RoundId(0));
        QuorumVote::cast(&mut q, pid, AgentId(3), Ternary::Neg, RoundId(0));
        let proposal = q.proposal(pid).unwrap();
        let status = QuorumConsensus::is_consensus(proposal, &q.threshold);
        assert_eq!(status, ProposalStatus::Accepted);
    }

    #[test]
    fn test_consensus_not_reached() {
        let mut q = Quorum::new(QuorumThreshold::super_majority());
        q.add_member(AgentId(1));
        q.add_member(AgentId(2));
        q.add_member(AgentId(3));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        QuorumVote::cast(&mut q, pid, AgentId(1), Ternary::Pos, RoundId(0));
        QuorumVote::cast(&mut q, pid, AgentId(2), Ternary::Neg, RoundId(0));
        QuorumVote::cast(&mut q, pid, AgentId(3), Ternary::Zero, RoundId(0));
        let proposal = q.proposal(pid).unwrap();
        let status = QuorumConsensus::is_consensus(proposal, &q.threshold);
        assert_eq!(status, ProposalStatus::Open);
    }

    // --- Ternary round-trip and edge cases ---

    #[test]
    fn test_ternary_to_i8() {
        assert_eq!(Ternary::Neg.to_i8(), -1);
        assert_eq!(Ternary::Zero.to_i8(), 0);
        assert_eq!(Ternary::Pos.to_i8(), 1);
    }

    #[test]
    fn test_ternary_from_i8_roundtrip() {
        for v in [-1i8, 0, 1] {
            let t = Ternary::from_i8(v).unwrap();
            assert_eq!(t.to_i8(), v);
        }
    }

    #[test]
    fn test_ternary_from_i8_invalid() {
        assert_eq!(Ternary::from_i8(2), None);
        assert_eq!(Ternary::from_i8(-2), None);
        assert_eq!(Ternary::from_i8(127), None);
        assert_eq!(Ternary::from_i8(-128), None);
    }

    // --- Threshold boundary precision ---

    #[test]
    fn test_threshold_boundary_simple_majority_exact() {
        let t = QuorumThreshold::simple_majority();
        // Exactly 50% (a tie) — ratio = 500 >= 500 → passes with >= semantics
        assert!(t.is_met(1, 1, 0));
        assert!(t.is_met(5, 5, 10));
        // Just below: 1/3 = 333 < 500
        assert!(!t.is_met(1, 2, 0));
    }

    #[test]
    fn test_threshold_boundary_super_majority() {
        let t = QuorumThreshold::super_majority();
        // 2/3 decisive: ratio = 2000/3 = 666 < 667 → fails (integer truncation)
        assert!(!t.is_met(2, 1, 0));
        // 3/4 decisive: ratio = 3000/4 = 750 >= 667 → passes
        assert!(t.is_met(3, 1, 0));
    }

    #[test]
    fn test_threshold_custom_clamping() {
        // majority_fraction > 1000 should be clamped to 1000
        let t = QuorumThreshold::custom(1500, 1);
        assert!(t.is_met(1, 0, 0)); // 100% passes
        assert!(!t.is_met(1, 1, 0)); // 50% < 100% fails
    }

    #[test]
    fn test_threshold_unanimous_with_abstains() {
        let t = QuorumThreshold::unanimous();
        // All decisive votes For → 100% → passes even with abstains
        assert!(t.is_met(3, 0, 2));
        // One Against → 75% < 100% → fails
        assert!(!t.is_met(3, 1, 0));
    }

    // --- Vote edge cases ---

    #[test]
    fn test_vote_cast_on_nonexistent_proposal() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        assert!(!QuorumVote::cast(
            &mut q,
            ProposalId(999),
            AgentId(1),
            Ternary::Pos,
            RoundId(0)
        ));
    }

    #[test]
    fn test_vote_cast_on_closed_proposal() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        q.add_member(AgentId(2));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        q.proposal_mut(pid).unwrap().status = ProposalStatus::Accepted;
        assert!(!QuorumVote::cast(
            &mut q,
            pid,
            AgentId(2),
            Ternary::Pos,
            RoundId(0)
        ));
    }

    #[test]
    fn test_vote_cast_different_rounds_allowed() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        assert!(QuorumVote::cast(
            &mut q,
            pid,
            AgentId(1),
            Ternary::Pos,
            RoundId(0)
        ));
        // Same voter can cast again in a new round
        assert!(QuorumVote::cast(
            &mut q,
            pid,
            AgentId(1),
            Ternary::Neg,
            RoundId(1)
        ));
    }

    #[test]
    fn test_remove_nonexistent_member() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        assert!(!q.remove_member(AgentId(99)));
        assert_eq!(q.member_count(), 1);
    }

    #[test]
    fn test_proposal_mut_updates_status() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        q.proposal_mut(pid).unwrap().status = ProposalStatus::Expired;
        assert_eq!(q.proposal(pid).unwrap().status, ProposalStatus::Expired);
    }

    // --- Consensus rejection path ---

    #[test]
    fn test_consensus_rejected() {
        let mut q = Quorum::new(QuorumThreshold::simple_majority());
        q.add_member(AgentId(1));
        q.add_member(AgentId(2));
        q.add_member(AgentId(3));
        let pid = q.propose(AgentId(1), "Test?").unwrap();
        QuorumVote::cast(&mut q, pid, AgentId(1), Ternary::Neg, RoundId(0));
        QuorumVote::cast(&mut q, pid, AgentId(2), Ternary::Neg, RoundId(0));
        QuorumVote::cast(&mut q, pid, AgentId(3), Ternary::Pos, RoundId(0));
        let proposal = q.proposal(pid).unwrap();
        let status = QuorumConsensus::is_consensus(proposal, &q.threshold);
        assert_eq!(status, ProposalStatus::Rejected);
    }

    // --- record_result threshold-weighted rejection (regression tests) ---

    #[test]
    fn test_record_result_super_majority_below_threshold_stays_open() {
        // Regression: record_result previously rejected on simple plurality
        // (against > for) without checking the threshold fraction.
        // With super_majority(667): 3 for / 4 against = 57.1% < 66.7% → Open
        let mut round = QuorumRound::new(3);
        let threshold = QuorumThreshold::super_majority();
        let result = round.record_result(3, 4, 0, &threshold);
        assert_eq!(result, ProposalStatus::Open);
    }

    #[test]
    fn test_record_result_super_majority_above_threshold_rejects() {
        // With super_majority(667): 2 for / 5 against = 71.4% >= 66.7% → Rejected
        let mut round = QuorumRound::new(3);
        let threshold = QuorumThreshold::super_majority();
        let result = round.record_result(2, 5, 0, &threshold);
        assert_eq!(result, ProposalStatus::Rejected);
    }

    #[test]
    fn test_record_result_tie_stays_open() {
        // for == against with super_majority: 1/2 = 50% < 66.7% → Open
        let mut round = QuorumRound::new(3);
        let threshold = QuorumThreshold::super_majority();
        let result = round.record_result(1, 1, 0, &threshold);
        assert_eq!(result, ProposalStatus::Open);
    }
}
