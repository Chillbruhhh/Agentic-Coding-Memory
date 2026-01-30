# AMP CLI Usage Guide

## Installation Options

### Recommended: Install via npm

```bash
# Global installation (recommended)
npm install -g @chillbruhhh/amp-cli

# Verify installation
amp --help
```

### Windows Users (Alternative Methods)

#### Option 1: Install from Source
```powershell
# Run the install script
.\install.ps1

# Or manually:
cd amp\cli
cargo install --path . --force
```

#### Option 2: Build and Run Locally
```powershell
# Build release binary
.\build-cli.ps1

# Run the binary
.\amp-cli.exe --help
```

#### Option 3: Development Mode
```powershell
cd amp\cli
cargo run -- --help
cargo run -- start "kiro-cli"
cargo run -- tui
```

#### Quick Test
```powershell
.\test-cli.ps1
```

### Linux/Mac Users

#### Option 1: Install from Source (Recommended)
```bash
# Run the install script
./install.sh

# Or manually:
cd amp/cli
cargo install --path . --force
```

#### Option 2: Build and Run Locally
```bash
# Build release binary
./build-cli.sh

# Run the binary
./amp-cli --help
```

#### Option 3: Development Mode
```bash
cd amp/cli
cargo run -- --help
cargo run -- start "kiro-cli"
cargo run -- tui
```

## Usage Examples

```bash
# Show help
amp --help

# Index current directory (with better excludes)
amp index

# Index specific directory with custom exclusions
amp index --path /path/to/project --exclude "target,*.log,custom_dir"

# Clear all objects from database (with confirmation)
amp clear

# Clear without interactive confirmation
amp clear --confirm

# Start a session with Kiro CLI
amp start "kiro-cli"

# Check current status
amp status

# View session history
amp history

# Launch interactive TUI
amp tui
```

## Binary Distribution

The CLI compiles to a single binary named `amp` that can be:
- Installed via `cargo install`
- Distributed as standalone binary
- Added to system PATH for global access

## Requirements

- Rust/Cargo (for building from source)
- AMP server running on localhost:8105 (default)
