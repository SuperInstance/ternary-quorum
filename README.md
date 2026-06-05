# ternary-quorum: Distributed decision making with ternary voting and configurable thresholds

## Why This Exists

A fleet of agents needs to make collective decisions: approve deployments, agree on configurations, authorize resource transfers. Simple majority vote works for casual choices but breaks down when decisions are consequential — you need supermajority, unanimity, or custom thresholds. And with three options (for, against, abstain) instead of two, you can distinguish "I actively oppose this" from "I don't care either way."

Existing voting libraries assume binary choices and simple thresholds. A fleet needs configurable quorum rules, multi-round escalation, and a clear distinction between abstention and opposition.

## Core Concepts

- **Ternary vote**: `Neg` (against), `Zero` (abstain), `Pos` (for). Abstentions don't count toward the decisive vote total.
- **Quorum**: A voting body with members and active proposals. Only members can propose and vote.
- **QuorumProposal**: A motion under consideration, collecting votes with an open/accepted/rejected/expired status.
- **QuorumVote**: A single vote cast by a member in a specific round. Each member can vote once per round.
- **QuorumThreshold**: Configurable majority requirement — stored as parts-per-thousand (500 = 50%, 667 = ~67%). Also supports a minimum vote count.
- **QuorumRound**: Multi-round voting with a configurable maximum. If round 1 doesn't meet threshold, the round advances and members vote again.
- **QuorumConsensus**: Tallies votes and checks whether the threshold is met, producing a final proposal status.

## Quick Start

```toml
[dependencies]
ternary-quorum = "0.1"
```

```rust
use ternary_quorum::*;

let mut quorum = Quorum::new(QuorumThreshold::simple_majority());
quorum.add_member(AgentId(1));
quorum.add_member(AgentId(2));
quorum.add_member(AgentId(3));

let proposal = quorum.propose(AgentId(1), "Deploy v2.3 to production?").unwrap();

QuorumVote::cast(&mut quorum, proposal, AgentId(1), Ternary::Pos, RoundId(0));
QuorumVote::cast(&mut quorum, proposal, AgentId(2), Ternary::Pos, RoundId(0));
QuorumVote::cast(&mut quorum, proposal, AgentId(3), Ternary::Neg, RoundId(0));

let status = QuorumConsensus::is_consensus(quorum.proposal(proposal).unwrap(), &quorum.threshold);
// 2 for, 1 against → 66.7% > 50% → Accepted
```

## API Overview

| Type | Description |
|------|-------------|
| `Quorum` | Voting body with members, proposals, and a threshold |
| `QuorumProposal` | A motion with description, votes, and status |
| `QuorumVote` | A vote cast by a member in a specific round |
| `QuorumThreshold` | Majority requirement (simple, super, unanimous, or custom) |
| `QuorumRound` | Multi-round voting with result tracking per round |
| `QuorumConsensus` | Tallies votes and determines proposal outcome |
| `RoundResult` | Vote counts and outcome for a single round |

## How It Works

Thresholds are stored as parts per thousand to avoid floating point. A simple majority is 500/1000 (50%), supermajority is 667/1000 (~66.7%), and unanimity is 1000/1000. The `is_met` check computes `for_votes / (for_votes + against_votes)` as a ratio — abstentions are excluded from the decisive count but count toward the minimum vote requirement.

Multi-round voting allows escalation: if round 1 doesn't reach threshold but has a plurality of "for" votes, the proposal stays open for another round. This gives members a chance to reconsider or negotiate. The `max_rounds` parameter prevents infinite deliberation.

Consensus checking tallies all votes for a proposal and applies the threshold. If the threshold is met, the proposal is accepted. If the reverse threshold is met (enough against votes), it's rejected. Otherwise, it stays open.

## Known Limitations

- No weighted voting — every member's vote counts equally.
- No proxy or delegation; each member must vote directly.
- No time-based expiry for proposals; they stay open until accepted, rejected, or manually closed.
- Round advancement is manual — the caller must check results and decide whether to advance.
- No mechanism to change votes within a round (once cast, it's final for that round).
- Threshold fractions are limited to 0.1% granularity (parts per thousand).

## Use Cases

- **Deployment approval**: Require supermajority before pushing to production.
- **Configuration changes**: Use unanimous threshold for destructive or irreversible config updates.
- **Resource allocation votes**: Agents vote on whether to grant a room additional compute resources.
- **Multi-round negotiation**: When first-round votes are split, advance to another round for debate and re-voting.

## Ecosystem Context

Part of the SuperInstance ternary crate family. Works with `ternary-oracle` (market-based predictions) for softer consensus and `ternary-quorum` for formal binding decisions. Use the oracle when you want to gauge sentiment; use the quorum when you need a definitive yes/no/abstain outcome.

## License

MIT
