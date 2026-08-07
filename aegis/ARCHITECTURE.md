# Aegis Architecture

## System Overview

Aegis is a four-phase fuzzing platform built with a hybrid Rust/Go/TypeScript stack.

## Phase 1: Core Engine (Rust)

The Rust engine performs the actual fuzzing work:

- **aegis-core**: Defines `Input`, `ExecutionResult`, `CoverageSnapshot`, `TargetConfig`, `CampaignConfig`, and core traits (`Executor`, `MutationStrategy`, `Corpus`, `FuzzingEngine`)
- **aegis-mutate**: Implements 6 mutation strategies with deterministic ChaCha8 RNG
- **aegis-sancov**: Integrates LLVM SanitizerCoverage via C FFI callbacks and a 64KB atomic bitmap
- **aegis-executor**: Spawns sandboxed processes with RLIMIT controls, timeout handling, and crash detection
- **aegis-corpus**: Manages inputs with 6 power scheduling algorithms and delta-debugging minimization
- **aegis-engine**: Orchestrates the fuzzing loop with single-threaded and parallel modes
- **aegis-target**: Factory and validation for binary, service, and custom targets
- **aegis-cli**: Command-line interface for standalone fuzzing
- **aegis-advanced**: HTTP/gRPC, EVM smart contract, eBPF kernel, and protocol fuzzers

## Phase 2: Control Plane (Go)

The Go control plane handles distributed orchestration:

- **API (Gin)**: REST endpoints for campaigns, targets, workers, crashes with JWT auth
- **Scheduler**: Assigns workers to campaigns, dispatches jobs, monitors health
- **Store (PostgreSQL)**: Multi-tenant persistence with JSONB for flexible config
- **Messaging (NATS)**: JetStream for job distribution, heartbeats, and commands
- **Worker Client**: Node registration, job consumption, heartbeat loops

## Phase 3: Web UI (SvelteKit)

The SvelteKit dashboard provides real-time visualization:

- **Dashboard**: Overview stats, active campaigns, recent crashes
- **Campaigns**: List, start, pause, stop campaigns
- **Campaign Detail**: Overview, coverage heatmap, crashes, metrics tabs
- **Crashes**: Triage workflow with severity assignment
- **Targets & Workers**: Management and monitoring

## Phase 4: Advanced Targets

Specialized fuzzing harnesses for modern systems:

- **HTTP Service**: Sends mutated requests to REST endpoints
- **gRPC Service**: Protobuf-aware mutation
- **EVM Contracts**: Transaction generation with function selector preservation
- **eBPF Kernel**: Syscall monitoring via kernel probes
- **Custom Protocols**: Stateful mutation with message type awareness

## Data Flow

```
Web UI → API → Scheduler → NATS → Worker → Rust Engine → Target
                ↓              ↓
           PostgreSQL     Heartbeats/Results
```

## Security Model

- Consent-based targeting (no automatic discovery)
- RLIMIT sandboxing per execution
- Rate limiting for remote services
- Audit logging of all operations
- No exploit generation (crash discovery only)
