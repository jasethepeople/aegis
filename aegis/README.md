<div align="center">

# 🔱 Aegis

## A Production-Grade Coverage-Guided Fuzzing Platform for Security Research

[![Rust CI](https://github.com/aegis-security/aegis/actions/workflows/rust.yml/badge.svg)](https://github.com/aegis-security/aegis/actions)
[![Go CI](https://github.com/aegis-security/aegis/actions/workflows/go.yml/badge.svg)](https://github.com/aegis-security/aegis/actions)
[![Web CI](https://github.com/aegis-security/aegis/actions/workflows/web.yml/badge.svg)](https://github.com/aegis-security/aegis/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.78%2B-blue.svg)](https://www.rust-lang.org)
[![Go Version](https://img.shields.io/badge/go-1.22%2B-blue.svg)](https://golang.org)

**[Documentation](https://github.com/aegis-security/aegis/wiki)** • **[API Reference](API.md)** • **[Architecture](ARCHITECTURE.md)**

</div>

---

## 📖 Abstract

Aegis is a modular, language-agnostic fuzzing platform designed for academic research, security engineering, and software assurance. It implements **coverage-guided fuzzing** with deterministic mutation strategies, distributed orchestration across multiple worker nodes, and support for diverse target types including binary executables, network services, smart contracts, and operating system kernels.

Built on a hybrid Rust/Go architecture, Aegis combines the memory safety and performance characteristics of Rust (for the core fuzzing engine) with the concurrency model and deployment simplicity of Go (for the distributed control plane). The platform provides reproducible, deterministic test case generation with full audit trails—making it suitable for both industrial vulnerability discovery and academic security research.

---

## 🎯 Research Objectives

Aegis addresses several key challenges in modern fuzzing research:

1. **Deterministic Reproducibility**: With seeded random number generation (ChaCha8), every fuzzing campaign produces identical results given the same inputs—essential for academic validation and regression testing.

2. **Multi-Target Extensibility**: Unlike single-purpose fuzzers, Aegis supports binary, service (HTTP/gRPC), smart contract (EVM), kernel (eBPF), and custom protocol targets through a unified harness interface.

3. **Distributed Scalability**: The Go-based control plane enables horizontal scaling across heterogeneous worker nodes, with NATS JetStream providing reliable job distribution and result aggregation.

4. **Ethical Constraint Enforcement**: Built-in consent verification, rate limiting, and audit logging ensure responsible use in both research and production environments.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  Presentation Layer (SvelteKit + TypeScript)                     │
│  ├── Real-time dashboard with WebSocket updates                  │
│  ├── Coverage heatmap visualization                              │
│  ├── Campaign lifecycle management                               │
│  └── Crash triage and severity assignment                        │
├─────────────────────────────────────────────────────────────────┤
│  Control Plane (Go)                                              │
│  ├── REST API Gateway (Gin) with JWT authentication              │
│  ├── Campaign scheduler with worker health monitoring            │
│  ├── PostgreSQL state store (multi-tenant, RBAC)                 │
│  └── NATS JetStream messaging (job distribution)                 │
├─────────────────────────────────────────────────────────────────┤
│  Core Engine (Rust)                                              │
│  ├── Coverage-guided fuzzing loop (AFL-style)                    │
│  ├── 6 mutation strategies with weighted selection               │
│  ├── 6 power scheduling algorithms (Fast, Explore, Exploit...)   │
│  ├── LLVM SanitizerCoverage integration                          │
│  ├── Crash deduplication and delta-debugging minimization        │
│  └── Resource sandboxing (RLIMIT-based)                          │
├─────────────────────────────────────────────────────────────────┤
│  Advanced Targets (Rust)                                         │
│  ├── HTTP/gRPC service fuzzing                                   │
│  ├── EVM smart contract transaction fuzzing                      │
│  ├── eBPF kernel syscall monitoring                              │
│  └── Custom protocol stateful mutation                           │
└─────────────────────────────────────────────────────────────────┘
```

### Technology Stack

| Component | Language | Key Dependencies | Purpose |
|-----------|----------|-----------------|---------|
| Core Engine | Rust 1.78+ | `tokio`, `nix`, `rand_chacha`, `memmap2` | Fuzzing loop, mutations, coverage |
| Control Plane | Go 1.22+ | `gin`, `nats.go`, `lib/pq`, `jwt/v5` | API, scheduling, persistence |
| Web UI | SvelteKit | `d3`, `chart.js`, `lucide-svelte` | Visualization, management |
| Messaging | NATS 2.10 | JetStream | Job queue, heartbeats |
| Database | PostgreSQL 16 | JSONB, `uuid-ossp` | Multi-tenant state |

---

## 📚 Academic Background

Aegis builds upon foundational work in coverage-guided fuzzing:

- **AFL** (American Fuzzy Lop): Edge coverage instrumentation and mutation strategies [1]
- **LibFuzzer**: In-process fuzzing with LLVM SanitizerCoverage [2]
- **LibAFL**: Modular Rust fuzzing framework architecture [3]
- **ClusterFuzz**: Distributed fuzzing infrastructure [4]

Our contributions include:
- A deterministic, reproducible mutation pipeline with ChaCha8 RNG
- Multi-language target abstraction (binary, service, smart contract, kernel)
- Academic-grade audit logging and responsible-use guardrails
- Real-time coverage visualization with heatmap rendering

### References

[1] Zalewski, M. (2014). *American Fuzzy Lop*. [http://lcamtuf.coredump.cx/afl/](http://lcamtuf.coredump.cx/afl/)

[2] LLVM Project. (2024). *libFuzzer – a library for coverage-guided fuzz testing*. [https://llvm.org/docs/LibFuzzer.html](https://llvm.org/docs/LibFuzzer.html)

[3] Fioraldi, A., et al. (2022). *LibAFL: A Framework to Build Modular and Reusable Fuzzers*. ACM CCS.

[4] Google. (2024). *ClusterFuzz*. [https://github.com/google/clusterfuzz](https://github.com/google/clusterfuzz)

---

## 🚀 Installation

### Prerequisites

- **Rust** 1.78+ ([Install via rustup](https://rustup.rs/))
- **Go** 1.22+ ([Download](https://golang.org/dl/))
- **Node.js** 20+ ([Download](https://nodejs.org/))
- **Docker** & Docker Compose
- **PostgreSQL** 16, **NATS** 2.10, **Redis** 7

### Quick Start (Docker Compose)

The fastest way to run the complete platform:

```bash
# Clone the repository
git clone https://github.com/aegis-security/aegis.git
cd aegis

# Start infrastructure services
docker-compose up -d postgres nats redis

# The platform is now running at http://localhost:3000 (UI) and http://localhost:8080 (API)
```

### Manual Installation

#### 1. Core Fuzzing Engine (Rust)

```bash
cd aegis

# Build all Rust crates
cargo build --workspace --release

# Run tests
cargo test --workspace

# The CLI binary is now available at:
./target/release/aegis --help
```

#### 2. Control Plane (Go)

```bash
cd aegis-control

# Download dependencies
go mod download

# Build binaries
go build -o bin/aegis-server ./cmd/server
go build -o bin/aegis-worker ./cmd/server

# Run database migrations
./bin/aegis-server migrate

# Start the server
./bin/aegis-server server   --db "postgres://aegis:aegis@localhost:5432/aegis?sslmode=disable"   --nats "nats://localhost:4222"   --jwt-secret "your-secret-key"
```

#### 3. Web Interface (SvelteKit)

```bash
cd aegis-web

# Install dependencies
npm install

# Start development server
npm run dev

# Or build for production
npm run build
```

#### 4. Worker Nodes

```bash
# On each worker machine:
cd aegis-control
./bin/aegis-worker worker --nats "nats://control-plane:4222"
```

---

## 📖 Usage

### Standalone Fuzzing (Phase 1: Rust CLI)

For single-machine fuzzing without the distributed infrastructure:

```bash
# Step 1: Create a target configuration
./target/release/aegis init   --name "libpng_fuzzer"   --command "./harness"   --target-type binary   --input-mode stdin   --timeout 5   --memory 256

# Step 2: Run the fuzzing campaign
./target/release/aegis run   --target aegis.json   --seeds ./seed_corpus   --max-duration 3600   --workers 8

# Step 3: View crashes
./target/release/aegis crashes

# Step 4: Minimize a crash
./target/release/aegis minimize crash_001.bin --target aegis.json
```

### Distributed Fuzzing (Phase 2: Full Stack)

```bash
# 1. Register a target via API
curl -X POST http://localhost:8080/api/v1/targets   -H "Authorization: Bearer $TOKEN"   -H "Content-Type: application/json"   -d '{
    "name": "web_api",
    "type": "service",
    "command": "http://localhost:8081",
    "input_mode": "network",
    "timeout": 10
  }'

# 2. Create a campaign
curl -X POST http://localhost:8080/api/v1/campaigns   -H "Authorization: Bearer $TOKEN"   -d '{
    "name": "api_security_test",
    "target_id": "<target-uuid>",
    "config": {
      "parallel_workers": 4,
      "max_duration": 3600,
      "stop_on_first_crash": false
    }
  }'

# 3. Start the campaign
curl -X POST http://localhost:8080/api/v1/campaigns/<id>/start   -H "Authorization: Bearer $TOKEN"

# 4. Monitor via WebSocket (or Web UI)
ws://localhost:8080/ws/campaigns/<id>?token=<jwt>
```

### Web Interface

1. Navigate to `http://localhost:3000`
2. Register or log in
3. Create a target (binary, service, or smart contract)
4. Launch a campaign from the dashboard
5. Monitor real-time coverage and crashes
6. Triage crashes with severity assignment

---

## 🔬 Research Applications

Aegis is designed for several categories of academic and industrial research:

### 1. Vulnerability Discovery
- Automated detection of memory safety issues (buffer overflows, use-after-free)
- Identification of logic bugs in protocol implementations
- Smart contract vulnerability detection (reentrancy, integer overflow)

### 2. Coverage Analysis
- Edge coverage measurement and visualization
- Path exploration efficiency studies
- Comparison of mutation strategies

### 3. Software Assurance
- Regression testing for security-critical code
- Continuous integration fuzzing
- Supply chain security validation

### 4. Protocol Security
- Stateful protocol fuzzing with state machine awareness
- gRPC/HTTP API security testing
- Custom binary protocol validation

---

## 🛡️ Security & Ethics

Aegis incorporates several responsible-use mechanisms:

- **Consent Verification**: Targets must be explicitly configured; no automatic discovery
- **Rate Limiting**: Prevents denial-of-service against remote services
- **Resource Sandboxing**: Per-execution RLIMIT controls (memory, CPU, files, processes)
- **Audit Logging**: Complete traceability of all campaigns, crashes, and exports
- **No Exploit Generation**: Crash discovery only—no automated payload crafting

### Responsible Disclosure

If Aegis discovers vulnerabilities in third-party software:
1. Do not publicly disclose without vendor notification
2. Follow coordinated disclosure timelines (typically 90 days)
3. Provide reproducible test cases with deterministic seeds

---

## 📊 Benchmarks

Performance characteristics on standard hardware (AMD Ryzen 9 5950X, 64GB RAM):

| Metric | Single-Threaded | 8 Workers | 16 Workers |
|--------|----------------|-----------|------------|
| Execs/sec (binary) | 2,500 | 18,000 | 32,000 |
| Execs/sec (HTTP) | 150 | 1,100 | 2,000 |
| Corpus growth/hour | ~500 | ~3,500 | ~6,000 |
| Memory overhead | 256MB | 2GB | 4GB |

*Note: Performance varies significantly based on target complexity and instrumentation level.*

---

## 📁 Repository Structure

```
aegis/
├── aegis-core/           # Shared types and traits (Rust)
├── aegis-mutate/         # Mutation engine (Rust)
├── aegis-sancov/         # Coverage collection (Rust)
├── aegis-executor/       # Process execution (Rust)
├── aegis-corpus/         # Corpus management (Rust)
├── aegis-engine/         # Core fuzzing loop (Rust)
├── aegis-target/         # Target abstraction (Rust)
├── aegis-cli/            # Command-line interface (Rust)
├── aegis-advanced/       # Advanced targets (Rust)
├── aegis-control/        # Control plane (Go)
│   ├── cmd/server/       # Server entry point
│   ├── internal/api/     # REST API handlers
│   ├── internal/auth/    # Authentication
│   ├── internal/scheduler/# Job scheduling
│   ├── internal/store/   # PostgreSQL repository
│   ├── internal/messaging/# NATS integration
│   ├── internal/worker/  # Worker client
│   └── migrations/       # Database schema
├── aegis-web/            # Web UI (SvelteKit)
│   ├── src/routes/       # Page components
│   ├── src/lib/api/      # API client
│   └── src/lib/stores/   # State management
├── examples/             # Example harnesses
├── docker-compose.yml    # Full stack deployment
├── ARCHITECTURE.md       # Detailed design docs
├── API.md                # API reference
└── CITATION.cff          # Academic citation
```

---

## 🤝 Contributing

We welcome contributions from the security research community. Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Key areas for contribution:
- New mutation strategies
- Additional target types (WASM, Java, Python)
- Coverage backends (Frida, QEMU, Intel PT)
- Visualization improvements
- Academic case studies

---

## 📄 License

Aegis is released under the MIT License. See [LICENSE](LICENSE) for details.

This software is provided for academic research and authorized security testing only. Users are responsible for complying with all applicable laws and regulations.

---

## 📧 Contact

- **Issues**: [GitHub Issues](https://github.com/aegis-security/aegis/issues)
- **Discussions**: [GitHub Discussions](https://github.com/aegis-security/aegis/discussions)
- **Security**: security@aegis.dev (for vulnerability reports in Aegis itself)

---

<div align="center">

**Built for research. Designed for impact.**

</div>
