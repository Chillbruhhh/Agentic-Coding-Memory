# AMP CLI - The Bridge

Terminal-based interface for integrating coding agents with AMP (Agentic Memory Protocol).

## Installation

```bash
cd amp/cli
cargo build --release
```

## Usage

### Start a session with an agent
```bash
amp start "your-agent-command"
```

### Check status
```bash
amp status
```

### View session history
```bash
amp history
```

### Launch interactive TUI
```bash
amp tui
```

## Configuration

Set the AMP server URL:
```bash
export AMP_SERVER_URL=http://localhost:8105
```

## Features

- ✅ Session lifecycle management
- ✅ Agent process spawning and monitoring
- ✅ Automatic git diff capture
- ✅ Lease-based heartbeat system
- ✅ Interactive TUI interface
- ✅ Session persistence and history
