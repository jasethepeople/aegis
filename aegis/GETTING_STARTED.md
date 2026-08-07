# Getting Started with Aegis

This guide will walk you through your first fuzzing campaign with Aegis, from installation to crash discovery.

## Table of Contents
1. [Installation](#installation)
2. [Your First Binary Fuzzing Campaign](#your-first-binary-fuzzing-campaign)
3. [Distributed Fuzzing](#distributed-fuzzing)
4. [Web Interface Tour](#web-interface-tour)
5. [Understanding Results](#understanding-results)
6. [Next Steps](#next-steps)

---

## Installation

### Option A: Docker Compose (Recommended)

The fastest way to get started:

```bash
git clone https://github.com/aegis-security/aegis.git
cd aegis
docker-compose up -d
```

This starts:
- PostgreSQL (port 5432)
- NATS (port 4222)
- Redis (port 6379)
- Aegis Control Plane (port 8080)
- Aegis Web UI (port 3000)

### Option B: Manual Installation

#### Step 1: Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Should show 1.78+
```

#### Step 2: Install Go
```bash
# Download from https://golang.org/dl/
# Verify:
go version  # Should show 1.22+
```

#### Step 3: Install Node.js
```bash
# Download from https://nodejs.org/
# Verify:
node --version  # Should show 20+
npm --version
```

#### Step 4: Start Infrastructure
```bash
# PostgreSQL
docker run -d --name aegis-postgres   -e POSTGRES_USER=aegis   -e POSTGRES_PASSWORD=aegis   -e POSTGRES_DB=aegis   -p 5432:5432 postgres:16-alpine

# NATS
docker run -d --name aegis-nats   -p 4222:4222 -p 8222:8222   nats:2.10-alpine --js

# Redis
docker run -d --name aegis-redis   -p 6379:6379 redis:7-alpine
```

#### Step 5: Build Aegis
```bash
cd aegis

# Build Rust core
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

---

## Your First Binary Fuzzing Campaign

### Step 1: Create a Test Target

We'll use a simple vulnerable C program:

```c
// examples/harness.c
#include <stdio.h>
#include <string.h>

void process(const char *data, size_t len) {
    char buf[64];
    if (len > 0) {
        memcpy(buf, data, len);  // Buffer overflow if len > 64
    }
}

int main() {
    char buf[4096];
    size_t n = fread(buf, 1, sizeof(buf), stdin);
    process(buf, n);
    return 0;
}
```

Compile with sanitizers:
```bash
clang -fsanitize=address,undefined       -fsanitize-coverage=trace-pc-guard       -g -O1       examples/harness.c -o harness
```

### Step 2: Initialize Target

```bash
./target/release/aegis init   --name "buffer_overflow_test"   --command "./harness"   --target-type binary   --input-mode stdin   --timeout 5   --memory 256   --output buffer_overflow.json
```

### Step 3: Run Fuzzing

```bash
# Create a seed directory
mkdir -p seeds
echo -n "AAAA" > seeds/seed1

# Run for 60 seconds with 4 workers
./target/release/aegis run   --target buffer_overflow.json   --seeds ./seeds   --max-duration 60   --workers 4
```

### Step 4: View Results

```bash
# List crashes
./target/release/aegis crashes

# Replay a specific crash
./target/release/aegis replay crash_001.bin --target buffer_overflow.json

# Minimize the crash input
./target/release/aegis minimize crash_001.bin   --target buffer_overflow.json   --output minimized.bin
```

---

## Distributed Fuzzing

For larger campaigns across multiple machines:

### 1. Start Control Plane

```bash
cd aegis-control

# Run migrations
./bin/aegis-server migrate

# Start server
./bin/aegis-server server   --db "postgres://aegis:aegis@localhost:5432/aegis?sslmode=disable"   --nats "nats://localhost:4222"   --jwt-secret "your-secure-secret"
```

### 2. Register via Web UI

1. Open `http://localhost:3000`
2. Register a new account
3. Create an organization

### 3. Create a Target

Via the web interface:
1. Navigate to **Targets** → **New Target**
2. Fill in:
   - Name: `api_security_test`
   - Type: `Binary`
   - Command: `./harness`
   - Input Mode: `Stdin`
   - Timeout: 5 seconds
   - Memory Limit: 256 MB
3. Enable ASan and SanCov instrumentation

### 4. Launch Campaign

1. Go to **Campaigns** → **New Campaign**
2. Select your target
3. Configure:
   - Workers: 4
   - Max Duration: 1 hour
   - Stop on First Crash: No
4. Click **Start Campaign**

### 5. Add Worker Nodes

On each worker machine:
```bash
./bin/aegis-worker worker --nats "nats://control-plane-ip:4222"
```

---

## Web Interface Tour

### Dashboard
- Real-time execs/sec across all campaigns
- Active campaign count and worker status
- Recent crash feed with severity indicators

### Campaign Detail Page
- **Overview Tab**: Configuration and live activity
- **Coverage Tab**: Heatmap visualization of edge coverage
- **Crashes Tab**: List of discovered crashes with triage controls
- **Metrics Tab**: Time-series performance data

### Crash Triage
1. Click on a crash to view details
2. Review sanitizer output and stack trace
3. Assign severity: Critical / High / Medium / Low
4. Download the crash input for reproduction
5. Mark as triaged when analyzed

---

## Understanding Results

### Coverage Metrics
- **Edge Count**: Number of unique control-flow edges hit
- **Coverage Percent**: (Edges Found / Total Edges) × 100
- **Path Hash**: Unique identifier for the execution path

### Crash Classification
- **Stack Hash**: Deduplication key based on top stack frames
- **Crash Type**: Categorized as SEGFAULT, Heap Overflow, UAF, etc.
- **Sanitizer Output**: ASan/UBSan diagnostic messages

### Performance Metrics
- **Execs/sec**: Executions per second (throughput indicator)
- **Corpus Size**: Number of inputs in the corpus
- **Corpus Growth Rate**: New inputs found per unit time

---

## Next Steps

- Read [ARCHITECTURE.md](ARCHITECTURE.md) for design details
- Explore [API.md](API.md) for programmatic access
- Check `examples/` for sample harnesses
- Review [CONTRIBUTING.md](CONTRIBUTING.md) to contribute

---

## Troubleshooting

### "Failed to spawn target"
- Ensure the target binary exists and is executable
- Check file permissions: `chmod +x ./harness`

### "No coverage found"
- Verify the target is compiled with `-fsanitize-coverage=trace-pc-guard`
- Check that ASan is linked: `nm harness | grep __asan`

### "Worker not connecting"
- Verify NATS is running: `docker logs aegis-nats`
- Check firewall rules for port 4222
- Ensure worker can reach control plane IP

### "Database connection failed"
- Verify PostgreSQL is running: `docker ps | grep postgres`
- Check connection string format
- Ensure database `aegis` exists
