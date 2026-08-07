# Project Structure

```
aegis/
├── .github/
│   └── workflows/
│       ├── rust.yml          # Rust CI (build, test, fmt, clippy)
│       ├── go.yml            # Go CI (build, test, lint)
│       └── web.yml           # Web UI CI (build, test, check)
│
├── aegis-core/               # Core types and traits (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # Input, ExecutionResult, CoverageSnapshot, etc.
│
├── aegis-mutate/             # Mutation engine (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # 6 mutation strategies + pipeline
│
├── aegis-sancov/             # Coverage collection (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # SanCov backend, bitmap, C FFI
│
├── aegis-executor/           # Process execution (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # ProcessExecutor, sandboxing, crash detection
│
├── aegis-corpus/             # Corpus management (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # Power scheduling, minimization
│
├── aegis-engine/             # Core fuzzing loop (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # Engine, ParallelEngine, campaign runner
│
├── aegis-target/             # Target abstraction (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # TargetFactory, validation, registry
│
├── aegis-cli/                # Command-line interface (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs           # init, run, validate, replay, minimize
│
├── aegis-advanced/           # Advanced targets (Rust)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs            # HTTP, gRPC, EVM, eBPF, protocol fuzzers
│
├── aegis-control/            # Distributed control plane (Go)
│   ├── cmd/
│   │   └── server/
│   │       └── main.go       # Server + worker entry points
│   ├── internal/
│   │   ├── api/
│   │   │   └── handlers.go   # REST API handlers (Gin)
│   │   ├── auth/
│   │   │   └── auth.go       # JWT authentication
│   │   ├── scheduler/
│   │   │   └── scheduler.go  # Campaign scheduling
│   │   ├── store/
│   │   │   └── postgres.go   # PostgreSQL repository
│   │   ├── messaging/
│   │   │   └── nats.go       # NATS JetStream integration
│   │   └── worker/
│   │       └── client.go     # Worker node client
│   ├── pkg/
│   │   └── models/
│   │       └── models.go     # Data models
│   ├── migrations/
│   │   └── 001_initial_schema.sql
│   ├── go.mod
│   ├── Dockerfile.server
│   ├── Dockerfile.worker
│   └── docker-compose.yml
│
├── aegis-web/                # Web UI (SvelteKit)
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css
│   │   ├── lib/
│   │   │   ├── api/
│   │   │   │   └── client.ts
│   │   │   └── stores/
│   │   │       └── index.ts
│   │   └── routes/
│   │       ├── +layout.svelte
│   │       ├── login/
│   │       │   └── +page.svelte
│   │       ├── dashboard/
│   │       │   └── +page.svelte
│   │       ├── campaigns/
│   │       │   ├── +page.svelte
│   │       │   └── [id]/
│   │       │       └── +page.svelte
│   │       ├── crashes/
│   │       │   └── +page.svelte
│   │       ├── targets/
│   │       │   └── +page.svelte
│   │       └── workers/
│   │           └── +page.svelte
│   ├── package.json
│   ├── svelte.config.js
│   ├── vite.config.ts
│   └── tailwind.config.js
│
├── examples/                 # Example harnesses
│   └── harness.c             # Vulnerable C program for testing
│
├── Cargo.toml                # Rust workspace configuration
├── Makefile                  # Build automation
├── docker-compose.yml        # Full stack deployment
├── README.md                 # Main project documentation
├── GETTING_STARTED.md        # Step-by-step tutorial
├── ARCHITECTURE.md           # Detailed architecture docs
├── API.md                    # API reference
├── CONTRIBUTING.md           # Contribution guidelines
├── CITATION.cff              # Academic citation format
├── LICENSE                   # MIT License
└── .gitignore                # Git ignore rules
```
