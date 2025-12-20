# Building Flare

## Prerequisites

Install [Just](https://github.com/casey/just) - a command runner:

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/.local/bin

# Or via package manager
cargo install just        # Cargo
brew install just         # macOS
sudo apt install just     # Debian/Ubuntu (if available)
```

## Quick Start

```bash
# 1. Set up AppImage tools (one-time)
just setup-tools
just install-tools-system  # Requires sudo (optional but recommended)

# 2. Build and install
just install

# 3. Run
just run
```

## Available Commands

Run `just --list` to see all available commands. Key recipes:

| Recipe | Description |
|--------|-------------|
| `just dev` | Development mode with hot-reload |
| `just build` | Full production build (AppImage) |
| `just build-deb` | Build DEB package only |
| `just build-rpm` | Build RPM package only |
| `just install` | Build and install to ~/.local/bin |
| `just run` | Run the installed app |
| `just check-deps` | Verify all dependencies are installed |
| `just clean` | Remove all build artifacts |
| `just info` | Show build configuration |

## Troubleshooting

### "linuxdeploy not found"

AppImage tools aren't set up:

```bash
just setup-tools           # Download to ~/.local/bin
just install-tools-system  # Symlink to /usr/local/bin (recommended)
```

### Missing FUSE

For AppImage support:

```bash
sudo apt install libfuse2   # Ubuntu/Debian
sudo dnf install fuse-libs  # Fedora
```

### Build Without AppImage Tools

Use DEB/RPM packages instead:

```bash
just build-deb  # Creates .deb in src-tauri/target/release/bundle/deb/
just build-rpm  # Creates .rpm in src-tauri/target/release/bundle/rpm/
```

## Development

For development with hot-reload:

```bash
just dev  # Runs Tauri in dev mode
```

Or frontend-only:

```bash
just dev-frontend  # Vite dev server only
```
