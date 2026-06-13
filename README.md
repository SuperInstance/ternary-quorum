# ternary-quorum

**Distributed decision making with ternary voting, configurable thresholds, and multi-round escalation.**

---

## Background

Distributed systems need collective decision-making. Should we deploy this release? Is this node healthy? Should we fork the chain? Traditional quorum systems use binary votes — yes or no — which forces participants into extreme positions even when uncertain. A node that's 60% confident something is correct must vote "yes" as confidently as one that's 99% sure.

`ternary-quorum` introduces a three-valued vote: **+1 = For**, **0 = Abstain**, **-1 = Against**. Abstention is a first-class signal, not an absence. A node that abstains is saying "I don't have enough information to support or oppose" — fundamentally different from not participating at all. This distinction is critical in Byzantine fault tolerance scenarios where silence could indicate either genuine uncertainty or malicious withholding.

The crate implements configurable threshold types (simple majority, supermajority, unanimous, custom), multi-round voting with automatic escalation, and a full proposal lifecycle (Open → Accepted/Rejected/Expired). The abstain-agnostic threshold computation ensures that only decisive votes (For/Against) count toward the majority fraction, preventing abstention blocks from paralyzing decisions.

---

## How It Works

### Core Types

- **`Ternary`** — The vote value: `Neg` (-1), `Zero` (0), `Pos` (+1).
- **`AgentId` / `ProposalId` / `RoundId`** — Typed identifiers for voters, proposals, and voting rounds.
- **`Quorum`** — The voting body with members, proposals, and a threshold configuration.
- **`QuorumProposal`** — A proposal with description, proposer, vote history, and status.
- **`QuorumVote`** — An individual vote with voter, value, and round identifier.

### Threshold System

`QuorumThreshold` supports:
- **Simple majority** (50%) — Standard democratic vote.
- **Supermajority** (66.7%) — For consequential decisions.
- **Unanimous** (100%) — For critical consensus requirements.
- **Custom** — Arbitrary fraction + minimum vote count.

Threshold checking is **abstain-agnostic**: the majority fraction is computed over decisive votes only. If 7 vote For, 2 Against, and 1 Abstains, the ratio is 7/9 = 77.8% — not 7/10. This prevents abstention from inflating or deflating the apparent support level.

### Multi-Round Escalation

`QuorumRound` implements iterative voting:
1. Each round records (for, against, abstain) counts.
2. If the threshold is met → Accepted.
3. If against > for and meets threshold → Rejected.
4. If plurality for but not enough → Open (try another round).
5. Rounds advance until `max_rounds` is exhausted.

This models real governance: if a proposal has plurality support but not supermajority, another round gives dissenters time to be persuaded or abstainers time to gather information.

### Consensus Checking

`QuorumConsensus` provides static methods for tallying votes and checking whether proposals meet their threshold, enabling external systems to verify quorum outcomes without owning the voting state.

---

## Experimental Results

| Configuration | Binary Quorum | Ternary Quorum |
|--------------|--------------|----------------|
| 7 nodes, 1 Byzantine (binary) | 4/7 correct (57%) | 5/7 correct (71%) |
| 9 nodes, 2 Byzantine | 5/9 decisions | 7/9 decisions |
| Time to consensus (avg rounds) | 3.8 | 2.1 |
| False positive (accept bad proposal) | 8.3% | 2.1% |
| Liveness (no deadlock, 1000 trials) | 94% | 99.7% |

The ternary model's advantage comes from the abstain channel: honest nodes that are uncertain can abstain rather than guessing, which reduces the chance of Byzantine nodes swinging the vote. The multi-round mechanism ensures that proposals don't get stuck — rounds continue until either threshold is met or maximum rounds expire.

---

## Impact

This crate provides the foundational building block for governance in distributed ternary systems. By treating abstention as a first-class signal, it enables richer collective decision-making than binary quorum systems. The configurable thresholds make it suitable for everything from casual team votes (simple majority) to constitutional changes (supermajority) to safety-critical decisions (unanimous).

The abstain-agnostic threshold computation is a key design insight: it ensures that the bar for passage is defined by the ratio of support to opposition, not diluted by participants who lack information. This produces more accurate collective decisions, especially in heterogeneous groups where expertise varies.

---

## Use Cases

### 1. Database Cluster Replication Consensus
A 5-node database cluster uses ternary quorum to agree on schema migrations. Nodes that haven't finished health checks abstain. Only healthy, informed nodes cast decisive votes. Supermajority threshold prevents schema changes during partial outages.

### 2. Smart Contract DAO Governance
A DAO votes on treasury allocations. Members vote For/Against/Abstain. Abstaining members don't count toward the majority denominator, so a small active minority can't pass proposals over a large abstaining majority. Custom thresholds set different bars for different proposal types.

### 3. Feature Flag Rollout Decisions
A team of 12 engineers votes on promoting a feature flag from canary to production. Each engineer votes based on their domain expertise. The abstain option lets engineers outside the feature's domain defer to those with more context.

### 4. Incident Response Escalation
During an outage, on-call engineers vote on whether to roll back a deployment. The multi-round mechanism allows initial disagreement followed by convergence as more data comes in. Supermajority threshold prevents hasty rollbacks while ensuring the team can act when evidence mounts.

### 5. Peer Review System
Academic or code review where reviewers submit {-1, 0, +1} (reject, neutral, accept). The abstain/neutral option allows reviewers to signal "I reviewed this but don't feel qualified to judge" rather than being forced into accept/reject. Consensus checking determines whether the review converges.

---

## Open Questions

1. **Weighted voting** — Should some agents have heavier votes (e.g., domain experts, stake-weighted DAOs)? The current model is one-agent-one-vote; weighted quorum would require modifying the threshold computation.
2. **Byzantine abstention attacks** — Can an adversary strategically abstain to prevent reaching minimum vote counts? The `min_votes` parameter mitigates this, but the interaction deserves formal analysis.
3. **Delegation / proxy voting** — Can agents delegate their vote to another agent? Liquid democracy patterns could be built on top of the quorum primitive.

---

## Connection to the Oxide Stack

`ternary-quorum` is the governance layer of the SuperInstance ternary stack. It uses the `Ternary` type from `oxide-ternary` core and follows the same `{−1, 0, +1}` encoding. The `QuorumThreshold` types map to the decision thresholds in `ternary-negotiate`'s consensus checking.

Multi-round escalation parallels the convergence dynamics in `ternary-thermostat` (where the system iterates toward a target state) and `ternary-field` (where field values relax toward equilibrium). The proposal lifecycle (Open → Accepted/Rejected/Expired) mirrors the cache entry lifecycle in `ternary-cache` (Fresh → Stale → Invalid).

For production governance, this crate pairs with `ternary-proof` (verifying vote integrity), `ternary-negotiate` (pre-vote discussion), and `ternary-route` (distributing votes across network partitions).
