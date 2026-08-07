# Contributing to Aegis

Thank you for your interest in contributing to Aegis! This document provides guidelines for contributing to the project.

## Code of Conduct

This project adheres to a strict code of conduct focused on responsible security research:

- **Consent-First**: Only fuzz targets you own or have explicit authorization to test
- **No Exploit Generation**: Aegis is for vulnerability discovery, not exploit development
- **Responsible Disclosure**: Report findings through proper channels
- **Academic Integrity**: Cite sources and maintain reproducibility

## Development Setup

### Prerequisites
- Rust 1.78+ (install via [rustup](https://rustup.rs/))
- Go 1.22+ (install via [golang.org](https://golang.org/dl/))
- Node.js 20+ (install via [nodejs.org](https://nodejs.org/))
- Docker & Docker Compose
- PostgreSQL 16, NATS 2.10, Redis 7

### Building

```bash
# Clone the repository
git clone https://github.com/aegis-security/aegis.git
cd aegis

# Build Rust core engine
cargo build --workspace --release

# Build Go control plane
cd aegis-control
go mod download
go build -o bin/aegis-server ./cmd/server
go build -o bin/aegis-worker ./cmd/server

# Build Web UI
cd ../aegis-web
npm install
npm run build
```

### Running Tests

```bash
# Rust tests
cargo test --workspace

# Go tests
cd aegis-control
go test ./...

# Web UI tests
cd ../aegis-web
npm run test
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Commit Message Format

We follow conventional commits:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Test additions/modifications
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `security:` Security-related changes

## Research Contributions

If you use Aegis in academic research, please:
1. Cite the project using the CITATION.cff file
2. Share your findings (with appropriate authorization)
3. Consider contributing improvements back to the project
