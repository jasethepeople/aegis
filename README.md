# Aegis

A modular coverage-guided fuzzing platform: a Rust fuzzing engine, a Go distributed control plane, and a SvelteKit web UI for running and monitoring fuzzing campaigns.

## Features

- **Rust fuzzing workspace** — crates for the core engine (`aegis-core`), mutation (`aegis-mutate`), sanitizer coverage (`aegis-sancov`), target execution (`aegis-executor`), corpus management (`aegis-corpus`), campaign orchestration (`aegis-engine`), target abstractions (`aegis-target`), advanced targets (`aegis-advanced`), and a CLI (`aegis-cli`).
- **`aegis` CLI** — subcommands to initialize, run, and manage fuzzing campaigns against binary targets (default type `binary`, stdin input mode), per `aegis-cli/src/main.rs`.
- **Go control plane** (`aegis-control`) — campaign scheduler and REST gateway built on Gin with NATS for job distribution and PostgreSQL state storage, plus server/worker Dockerfiles.
- **Web UI** (`aegis-web`) — SvelteKit + Tailwind dashboard for campaign management and result visualization.
- **Docs** — `API.md`, `ARCHITECTURE.md`, `GETTING_STARTED.md`, `STRUCTURE.md`, `CONTRIBUTING.md`, MIT license, and an examples harness (`examples/harness.c`).

## Tech stack

- Rust 1.78+ (workspace: tokio, rand_chacha, nix, serde, tracing)
- Go 1.22+ (Gin, nats.go, lib/pq, cobra, viper, zap)
- SvelteKit + TypeScript + Tailwind (web UI)
- PostgreSQL 16, NATS 2.10, Redis 7 (via `docker-compose.yml`)

## Getting started

The repo ships `GETTING_STARTED.md` with the full walkthrough. The fastest path is Docker Compose:

```bash
git clone https://github.com/aegis-security/aegis.git
cd aegis
docker-compose up -d
```

This starts PostgreSQL (5432), NATS (4222), Redis (6379), the control plane (8080), and the web UI (3000). For manual builds: Rust 1.78+ (`cargo build --workspace`), Go 1.22+ (`cd aegis-control && go build ./...`), Node.js for `aegis-web` (`npm install && npm run dev`).

## Project structure

```
aegis/
├── aegis-core, aegis-engine, aegis-mutate, aegis-sancov,
│   aegis-executor, aegis-corpus, aegis-target,
│   aegis-advanced, aegis-cli   # Rust fuzzing workspace (v0.4.0)
├── aegis-control/               # Go control plane (Gin + NATS + Postgres)
├── aegis-web/                   # SvelteKit dashboard
├── examples/                    # C harness example
└── API.md, ARCHITECTURE.md, GETTING_STARTED.md, STRUCTURE.md
```

## Status

Active multi-component codebase. Note: the repo's badge links point at `github.com/aegis-security/aegis`, which is not the `jasethepeople` account — the origin of the code in this repo is not documented here.
