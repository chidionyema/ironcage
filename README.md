# Ironcage: Research Kernel for Oracle Free Tier

Production-grade provable research engine running on 2 OCPU, 12 GB RAM. MCTS + formal verification + immutable ledger. Zero GPU required.

## Status: Scaffold Complete ✅

- [x] Workspace: 5 crates, fully modular
- [x] Kernel: MCTS with UCB1 strategy (ironcage-kernel, 2.1 MB)
- [x] Inference: llama.cpp integration stubs (ironcage-inference)
- [x] Verifier: Z3 MCP client (ironcage-verifier)
- [x] Ledger: BABYLON-60 SHA3-256 hash-chained append-only (ironcage-ledger)
- [x] API: Axum WebSocket server for graph deltas (ironcage-api, 1.9 MB)
- [x] Tests: 19 unit tests passing
- [x] Release build: 4.0 MB total binaries

## Build

```bash
cd /Users/chidionyema/dev/code/ironcage
cargo build --release
cargo test --lib
```

## Architecture

```
crates/
├── kernel      MCTS engine, hypothesis state machine, UCB1 scoring
├── inference   llama.cpp integration (7B Q4_K_M model)
├── verifier    Z3 formal verification via MCP
├── ledger      SQLite WAL, SHA3-256 hash chaining, immutable
└── api         Axum WebSocket server, /ws + /health + /metrics
```

## Key Design Decisions

1. **Rust kernel**: 2.1 MB binary, no GC, safe concurrency via tokio
2. **Modular crates**: Each has independent tests, swap backends at compile time
3. **BABYLON-60 ledger**: SHA3-256 hash chaining + SQLite WAL makes tamper detection instant
4. **Stateless API**: Node graph deltas pushed over WebSocket; UI renders in browser (Sigma.js)
5. **MCP verifier**: Z3 lives in separate binary, called via MCP protocol (domain-agnostic)

## Integration Points

### llama.cpp (Inference)
- Path: `crates/inference/src/lib.rs::HeuristicGenerator`
- Integration: Load 7B Q4_K_M model, generate N candidates per prompt
- Stub: Currently returns placeholder responses; swap with actual llama-cpp-rs when available

### Z3 (Verification)
- Path: `crates/verifier/src/lib.rs::FormalVerifier`
- Integration: Connect to z39 MCP server (runs separately, e.g., `z39 mcp`)
- Protocol: MCP tool call with claim → Z3 returns sat/unsat/unknown
- Timeout: 5000 ms default (configurable per claim)

### Ledger (Evidence)
- Path: `crates/ledger/src/lib.rs::Ledger`
- Storage: SQLite WAL at `/data/research.db`
- Verification: Hash chain is cryptographically tamper-evident
- Checkpoint: Every 1000 writes, archive old entries

### MCTS (Planning)
- Path: `crates/kernel/src/lib.rs::ResearchMCTS`
- Node structure: Hypothesis { claim, evidence, verified, visits, reward, parent_id }
- UCB1 formula: exploitation + exploration·√(ln(visits))
- Backprop: Recursively update ancestors with new reward

### API (Exposure)
- Endpoint: `ws://127.0.0.1:3000/ws` (WebSocket graph deltas)
- REST: `/health`, `/metrics`
- Broadcast: NodeDelta { node_id, visits, reward, verified }

## Deployment

### Local Development

```bash
# Terminal 1: Start kernel
cargo run --release --bin ironcage-kernel

# Terminal 2: Start API
cargo run --release --bin ironcage-api

# Terminal 3: Start Z3 MCP server (placeholder)
z39 mcp

# Browser: Open http://localhost:3001/ui (React + Sigma.js + @graphrs)
```

### Kubernetes (K3s on Oracle Free Tier)

See `k8s/` manifests (pending).

### Docker

```bash
docker build -t ironcage:latest .
docker run -p 3000:3000 -v /data:/data ironcage:latest
```

## Memory Budget (12 GB)

| Component | RAM | Headroom |
|-----------|-----|----------|
| K3s + SQLite | 400 MB | 1.0 GB |
| ironcage-kernel | 150 MB | 0.3 GB |
| ironcage-api | 100 MB | 0.2 GB |
| Z3 MCP server | 150 MB | 0.3 GB |
| gVisor sandboxes (4×) | 400 MB | 0.8 GB |
| llama.cpp (7B Q4_K_M) | 4.5 GB | — |
| **Subtotal** | **5.7 GB** | **6.3 GB** |

KV cache capacity: ~4K context × 4-8 branches = 16K–32K token buffer.

## Open Items

1. **llama.cpp bindings** (ironcage-inference)
   - Replace `generate_one()` stub with actual llama-cpp-rs calls
   - Integrate liblloyal for KV cache branching
   - Benchmark: expect 10–20 tokens/sec on 2 OCPU CPU inference

2. **Z3 MCP server** (ironcage-verifier)
   - Implement z39 binary or use z3-rs bindings
   - Expose via MCP protocol: tool `z39_safety` taking (claim, timeout_ms)
   - Benchmark: expect <500ms for SAT/UNSAT on 5K-char claims

3. **UI Layer**
   - React component + Sigma.js renderer
   - @graphrs WASM for PageRank/betweenness layout
   - WebSocket subscription to /ws for real-time node updates

4. **Durable Execution** (Restate)
   - Wrap research workflows in Restate state machine
   - Recovery from crashes via journal replay
   - Budget accounting and rate limits

5. **gVisor Sandboxing**
   - RuntimeClass integration with K3s
   - Agent execution in isolated user-space kernels
   - Syscall filter + audit logging to ledger

## Testing

Unit tests:
```bash
cargo test --lib
```

Integration tests (pending):
```bash
cargo test --test integration
```

End-to-end (pending):
- MCTS expands 100 nodes
- Z3 verifies first 10 claims
- Ledger records all actions
- WebSocket receives 50+ delta events

## Reference

- BABYLON-60 ledger: Hash-chained append-only with immutability triggers
- UCB1 formula: Balances exploration vs. exploitation in tree search
- MCP: Model Context Protocol (Anthropic) for tool composition
- gVisor: User-space Linux kernel, intercepts syscalls without KVM

---

**Status:** Scaffold ready for integration.  
**Next:** Wire llama.cpp + Z3 + Restate, then benchmark on Oracle Free Tier.
