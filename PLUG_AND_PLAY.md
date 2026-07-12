# PLUG_AND_PLAY — ternary-quorum

> *Integration guide.*

## Dependency

```toml
[dependencies]
ternary_quorum = "0.1.0"
```

## Feature Flags

This crate has no optional feature flags. It uses the Rust standard library.

## Integration

Import `ternary_quorum` in your project to access the functionality.

```rust
use ternary_quorum::{Quorum, QuorumThreshold, AgentId, Ternary};
```

## Compatibility

- **Rust edition**: 2021+
- **Targets**: All tier-1 Rust targets
