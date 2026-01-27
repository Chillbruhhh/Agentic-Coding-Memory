# Installation Guide

Complete installation instructions for all AMP components.

## System Requirements

### Minimum Requirements
- **OS**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 20.04+)
- **RAM**: 4GB minimum, 8GB recommended
- **Disk**: 2GB free space
- **Network**: Internet connection for initial setup

### Software Dependencies

**For Docker Installation:**
- Docker 20.10+
- Docker Compose 2.0+

**For Manual Installation:**
- Rust 1.70+ with Cargo
- Node.js 16+ with npm
- Git

## Installation Methods

### Method 1: Docker (Recommended)

Docker provides the easiest setup with all components pre-configured.

#### Step 1: Install Docker

**Windows:**
1. Download [Docker Desktop for Windows](https://www.docker.com/products/docker-desktop)
2. Run installer and follow prompts
3. Restart computer if prompted

**macOS:**
```bash
brew install --cask docker
```

**Linux (Ubuntu):**
```bash
sudo apt-get update
sudo apt-get install docker.io docker-compose
sudo usermod -aG docker $USER
# Log out and back in
```

#### Step 2: Clone Repository

```bash
git clone <repo-url>
cd amp
```

#### Step 3: Start Services

```bash
docker compose up -d
```

This starts:
- AMP Server (port 8105)
- MCP Server (port 8106)
- Desktop UI (port 8109)
- SurrealDB (port 7505)

#### Step 4: Verify Installation

```bash
# Check all services are running
docker compose ps

# Test server health
curl http://localhost:8105/health
```

### Method 2: CLI Tool Only

Install just the command-line tool without Docker.

#### Windows Installation

```powershell
# Run installation script
.\scripts\install.ps1

# Verify installation
amp --version
```

The script:
1. Checks for Rust installation
2. Builds the CLI tool
3. Adds to system PATH
4. Creates config directory

#### Linux/macOS Installation

```bash
# Run installation script
chmod +x scripts/install.sh
./scripts/install.sh

# Verify installation
amp --version
```

#### Manual CLI Build

If scripts fail, build manually:

```bash
cd amp/cli
cargo build --release

# Copy binary to PATH
# Linux/macOS:
sudo cp target/release/amp /usr/local/bin/

# Windows:
# Copy target\release\amp.exe to C:\Program Files\amp\
```

### Method 3: Full Manual Installation

Build all components from source.

#### Step 1: Install Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Download from https://rustup.rs/
```

#### Step 2: Install Node.js

```bash
# Linux/macOS (using nvm)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Windows
# Download from https://nodejs.org/
```

#### Step 3: Build Server

```bash
cd amp/server
cargo build --release

# Server binary at: target/release/amp-server
```

#### Step 4: Build CLI

```bash
cd amp/cli
cargo build --release

# CLI binary at: target/release/amp
```

#### Step 5: Build MCP Server

```bash
cd amp/mcp-server
cargo build --release

# MCP binary at: target/release/amp-mcp-server
```

#### Step 6: Build Desktop UI (Optional)

```bash
cd amp/ui
npm install
npm run tauri build

# Application at: src-tauri/target/release/
```

## Post-Installation Setup

### Configure Environment

Settings can be configured via the **UI Settings tab** (http://localhost:8109) or environment variables.

Create `.env` file in `amp/server/` for custom configuration:

```bash
# Server
PORT=8105
BIND_ADDRESS=0.0.0.0

# SurrealDB (configured via docker-compose by default)
SURREALDB_URL=ws://localhost:7505
SURREALDB_NS=amp
SURREALDB_DB=amp

# Embeddings (choose one)
EMBEDDING_PROVIDER=ollama
OLLAMA_URL=http://localhost:11434

# Or use OpenAI
# EMBEDDING_PROVIDER=openai
# OPENAI_API_KEY=sk-...
```

> **Note:** When using Docker, SurrealDB is automatically configured. The `.env` file is only needed for manual installations or custom configurations.

### Start Services

**Docker:**
```bash
docker compose up -d
```

**Manual:**
```bash
# Terminal 1: Start server
cd amp/server
cargo run --release

# Terminal 2: Start MCP server
cd amp/mcp-server
cargo run --release

# Terminal 3: Start UI (optional)
cd amp/ui
npm run tauri dev
```

### Verify Installation

```bash
# Check server
curl http://localhost:8105/health

# Check MCP server
curl http://localhost:8106/health

# Test CLI
amp status
```

## Installing Embedding Providers

### Ollama (Local, Recommended)

**Linux/macOS:**
```bash
curl https://ollama.ai/install.sh | sh
ollama pull nomic-embed-text
```

**Windows:**
Download from https://ollama.ai/download

### OpenAI (Cloud)

No installation needed. Just set API key:

```bash
export OPENAI_API_KEY=sk-...
```

### OpenRouter (Cloud)

No installation needed. Set API key:

```bash
export OPENROUTER_API_KEY=sk-...
```

## Updating AMP

### Docker

```bash
git pull
docker compose down
docker compose up --build -d
```

### Manual

```bash
git pull
cd amp/server && cargo build --release
cd amp/cli && cargo build --release
cd amp/mcp-server && cargo build --release
```

## Uninstalling

### Docker

```bash
docker compose down -v  # Remove containers and volumes
```

### Manual

```bash
# Remove binaries
rm /usr/local/bin/amp
rm /usr/local/bin/amp-server
rm /usr/local/bin/amp-mcp-server

# Remove data
rm -rf ~/.amp
```

## Troubleshooting Installation

### Rust Installation Issues

```bash
# Update Rust
rustup update

# Check version
rustc --version  # Should be 1.70+
```

### Build Failures

```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build --release
```

### Docker Issues

```bash
# Check Docker is running
docker ps

# Restart Docker service
sudo systemctl restart docker  # Linux
# Or restart Docker Desktop (Windows/macOS)

# Check logs
docker compose logs
```

### Permission Issues (Linux)

```bash
# Add user to docker group
sudo usermod -aG docker $USER

# Log out and back in
```

### Port Conflicts

If ports are already in use, change them in `docker-compose.yml` or `.env`:

```yaml
# docker-compose.yml
ports:
  - "8106:8105"  # Change 8105 to 8106
```

## Next Steps

- [Configuration Guide](configuration.md) - Configure AMP for your needs
- [First Steps](first-steps.md) - Start using AMP
- [MCP Integration](../guides/agents/mcp-integration.md) - Connect AI agents
