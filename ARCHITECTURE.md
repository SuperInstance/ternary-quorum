# Architecture — ternary-quorum

> *Internal design and data flow.*

## Overview

This crate implements ternary {-1, 0, +1} semantics for the `quorum` domain.
It is one of ~280 ternary crates in the SuperInstance fleet, all sharing Z₃ arithmetic
from [ternary-core](https://github.com/SuperInstance/ternary-core).

## Core Types

- **`AgentId`**
- **`ProposalId`**
- **`RoundId`**
- **`Quorum`**
- **`QuorumProposal`**
- **`QuorumVote`**
- **`QuorumThreshold`**
- **`QuorumRound`**
- **`RoundResult`**
- **`QuorumConsensus`**

## Key Functions

- `from_i8()`
- `to_i8()`
- `new()`
- `add_member()`
- `remove_member()`
- `member_count()`
- `propose()`
- `proposal()`

## Ternary Mapping

| Value | Meaning |
|-------|---------|
| +1 | Active / positive |
| 0  | Neutral |
| -1 | Inactive / negative |

## Source Structure

1 Rust source file(s) in `src/`.
Language: Rust

## Cross-Repo References

- [ternary-core](https://github.com/SuperInstance/ternary-core) — shared Z₃ traits
- [ternary-types](https://github.com/SuperInstance/ternary-types) — type-level encodings
- [Full SuperInstance fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)
