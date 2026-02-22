# Flare Build System
# Run `just --list` to see available recipes

# ============================================================================
# Variables
# ============================================================================

# Read version from package.json
version := `jq -r .version package.json`

# Get the Rust target triple for sidecar naming
arch := `rustc -vV | awk '/host:/ {print $2}'`

# Install directories
local_bin := env_var('HOME') / ".local/bin"
appimage_dir := "src-tauri/target/release/bundle/appimage"
deb_dir := "src-tauri/target/release/bundle/deb"

# Swift library paths for runtime
swift_lib_path := justfile_directory() / "src-tauri/SoulverWrapper/.build/release:" + justfile_directory() / "src-tauri/SoulverWrapper/Vendor/SoulverCore-linux"

# ============================================================================
# Dependency Checks
# ============================================================================

# Check all required build dependencies are installed
[group('setup')]
check-deps:
    #!/usr/bin/env bash
    set -e
    echo "🔍 Checking dependencies..."
    
    missing=()
    
    command -v pnpm &>/dev/null || missing+=("pnpm")
    command -v swift &>/dev/null || missing+=("swift")
    command -v cargo &>/dev/null || missing+=("cargo (rustup)")
    command -v jq &>/dev/null || missing+=("jq")
    
    if [ ${#missing[@]} -ne 0 ]; then
        echo "❌ Missing dependencies:"
        for dep in "${missing[@]}"; do
            echo "   - $dep"
        done
        exit 1
    fi
    
    echo "✅ All dependencies found"
    echo "   pnpm: $(pnpm --version)"
    echo "   swift: $(swift --version 2>&1 | head -1)"
    echo "   cargo: $(cargo --version)"

# Check AppImage tools are installed (for full build)
[group('setup')]
check-appimage-tools:
    #!/usr/bin/env bash
    set -e
    
    if ! command -v linuxdeploy &>/dev/null; then
        echo "❌ linuxdeploy not found"
        echo "Run: just setup-tools"
        exit 1
    fi
    
    if ! command -v appimagetool &>/dev/null; then
        echo "❌ appimagetool not found"
        echo "Run: just setup-tools"
        exit 1
    fi
    
    echo "✅ AppImage tools found"

# ============================================================================
# Setup
# ============================================================================

# Download and install AppImage build tools to ~/.local/bin
[group('setup')]
setup-tools:
    #!/usr/bin/env bash
    set -e
    
    mkdir -p "{{local_bin}}"
    
    echo "📦 Installing AppImage build tools..."
    
    if [ ! -f "{{local_bin}}/linuxdeploy-x86_64.AppImage" ]; then
        echo "⬇️  Downloading linuxdeploy..."
        curl -L -o "{{local_bin}}/linuxdeploy-x86_64.AppImage" \
            "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
        chmod +x "{{local_bin}}/linuxdeploy-x86_64.AppImage"
        ln -sf "{{local_bin}}/linuxdeploy-x86_64.AppImage" "{{local_bin}}/linuxdeploy"
        echo "✅ linuxdeploy installed"
    else
        echo "✅ linuxdeploy already installed"
    fi
    
    if [ ! -f "{{local_bin}}/appimagetool-x86_64.AppImage" ]; then
        echo "⬇️  Downloading appimagetool..."
        curl -L -o "{{local_bin}}/appimagetool-x86_64.AppImage" \
            "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
        chmod +x "{{local_bin}}/appimagetool-x86_64.AppImage"
        ln -sf "{{local_bin}}/appimagetool-x86_64.AppImage" "{{local_bin}}/appimagetool"
        echo "✅ appimagetool installed"
    else
        echo "✅ appimagetool already installed"
    fi
    
    echo ""
    echo "✅ All AppImage tools installed!"
    echo "Make sure {{local_bin}} is in your PATH"

# Install AppImage tools system-wide (requires sudo)
[group('setup')]
[confirm("This will create symlinks in /usr/local/bin. Continue?")]
install-tools-system:
    #!/usr/bin/env bash
    set -e
    
    echo "📦 Installing AppImage tools system-wide..."
    
    if [ -L "/usr/local/bin/linuxdeploy" ] && [ -L "/usr/local/bin/appimagetool" ]; then
        echo "✅ Tools already installed system-wide"
        exit 0
    fi
    
    if [ -f "{{local_bin}}/linuxdeploy-x86_64.AppImage" ]; then
        sudo ln -sf "{{local_bin}}/linuxdeploy-x86_64.AppImage" /usr/local/bin/linuxdeploy
        echo "✅ linuxdeploy symlinked"
    else
        echo "❌ linuxdeploy not found. Run: just setup-tools"
        exit 1
    fi
    
    if [ -f "{{local_bin}}/appimagetool-x86_64.AppImage" ]; then
        sudo ln -sf "{{local_bin}}/appimagetool-x86_64.AppImage" /usr/local/bin/appimagetool
        echo "✅ appimagetool symlinked"
    else
        echo "❌ appimagetool not found. Run: just setup-tools"
        exit 1
    fi
    
    echo "✅ All tools installed to /usr/local/bin"

# ============================================================================
# Build Components
# ============================================================================

# Build the sidecar (Node.js extension runtime)
[group('build')]
sidecar: check-deps
    #!/usr/bin/env bash
    set -e
    echo "📦 Building sidecar..."
    pnpm --filter sidecar build
    echo "✅ Sidecar built"

# Build the SoulverCore Swift wrapper
[group('build')]
swift: check-deps
    #!/usr/bin/env bash
    set -e
    echo "🐦 Building SoulverCore wrapper..."
    swift build -c release --package-path src-tauri/SoulverWrapper
    echo "✅ Swift wrapper built"

# Build the Svelte frontend
[group('build')]
frontend: check-deps
    #!/usr/bin/env bash
    set -e
    echo "🎨 Building frontend..."
    pnpm build
    echo "✅ Frontend built"

# ============================================================================
# Full Builds
# ============================================================================

# Build everything (AppImage)
[group('build')]
build: check-deps check-appimage-tools sidecar swift
    #!/usr/bin/env bash
    set -e
    
    export PATH="{{local_bin}}:$PATH"
    export LD_LIBRARY_PATH="/opt/swift/usr/lib/swift/linux:${LD_LIBRARY_PATH:-}"
    
    echo "🚀 Building Tauri app..."
    pnpm tauri build
    
    echo ""
    echo "✅ Build complete!"
    echo "AppImage location: {{appimage_dir}}/"
    ls -lh {{appimage_dir}}/*.AppImage 2>/dev/null || echo "No AppImage found"

# Build DEB package only (no AppImage tools required)
[group('build')]
build-deb: check-deps sidecar swift
    #!/usr/bin/env bash
    set -e
    
    echo "🚀 Building Tauri app (DEB only)..."
    pnpm tauri build --bundles deb
    
    echo ""
    echo "✅ Build complete!"
    echo "DEB package: {{deb_dir}}/flare_{{version}}_amd64.deb"
    echo ""
    echo "To install:"
    echo "  sudo dpkg -i {{deb_dir}}/flare_{{version}}_amd64.deb"

# Build RPM package only
[group('build')]
build-rpm: check-deps sidecar swift
    #!/usr/bin/env bash
    set -e
    
    echo "🚀 Building Tauri app (RPM only)..."
    pnpm tauri build --bundles rpm
    
    echo ""
    echo "✅ Build complete!"
    ls -lh src-tauri/target/release/bundle/rpm/*.rpm 2>/dev/null || echo "No RPM found"

# Build AppImage only (requires AppImage tools)
[group('build')]
build-appimage: check-deps check-appimage-tools sidecar swift
    #!/usr/bin/env bash
    set -e
    
    export PATH="{{local_bin}}:$PATH"
    export LD_LIBRARY_PATH="/opt/swift/usr/lib/swift/linux:${LD_LIBRARY_PATH:-}"
    
    echo "🚀 Building Tauri app (AppImage only)..."
    pnpm tauri build --bundles appimage
    
    echo ""
    echo "✅ Build complete!"
    echo "AppImage location: {{appimage_dir}}/"
    ls -lh {{appimage_dir}}/*.AppImage 2>/dev/null || echo "No AppImage found"

# Build AppImage with faster profile (no LTO, parallel codegen)
[group('build')]
build-appimage-fast: check-deps check-appimage-tools sidecar swift
    #!/usr/bin/env bash
    set -e
    
    export PATH="{{local_bin}}:$PATH"
    export LD_LIBRARY_PATH="/opt/swift/usr/lib/swift/linux:${LD_LIBRARY_PATH:-}"
    
    echo "🚀 Building Tauri app (AppImage, fast profile)..."
    pnpm tauri build --bundles appimage -- --profile release-fast
    
    echo ""
    echo "✅ Build complete!"
    echo "AppImage location: {{appimage_dir}}/"
    ls -lh {{appimage_dir}}/*.AppImage 2>/dev/null || echo "No AppImage found"

# Build DEB with faster profile (no LTO, parallel codegen)
[group('build')]
build-deb-fast: check-deps sidecar swift
    #!/usr/bin/env bash
    set -e
    
    echo "🚀 Building Tauri app (DEB, fast profile)..."
    pnpm tauri build --bundles deb -- --profile release-fast
    
    echo ""
    echo "✅ Build complete!"
    echo "DEB package: {{deb_dir}}/flare_{{version}}_amd64.deb"

# ============================================================================
# Install & Run
# ============================================================================

# Install built AppImage to ~/.local/bin
[group('run')]
install: build
    #!/usr/bin/env bash
    set -e
    
    echo "📥 Installing Flare..."
    
    APPIMAGE=$(find {{appimage_dir}} -name "*.AppImage" -type f 2>/dev/null | head -1)
    
    if [ -z "$APPIMAGE" ]; then
        echo "❌ No AppImage found. Build may have failed."
        exit 1
    fi
    
    mkdir -p "{{local_bin}}"
    
    # Kill any running instances
    echo "🛑 Stopping any running instances..."
    pkill -f "flare.AppImage" || true
    sleep 1
    
    # Copy and make executable
    echo "📋 Copying to {{local_bin}}/flare.AppImage..."
    cp "$APPIMAGE" "{{local_bin}}/flare.AppImage"
    chmod +x "{{local_bin}}/flare.AppImage"
    
    echo ""
    echo "✅ Installation complete!"
    echo "Installed to: {{local_bin}}/flare.AppImage"

# Build DEB and install via dpkg
[group('run')]
install-deb: build-deb
    #!/usr/bin/env bash
    set -e
    DEB=$(find {{deb_dir}} -name "*.deb" -type f 2>/dev/null | head -1)
    if [ -z "$DEB" ]; then
        echo "❌ No .deb found. Build may have failed."
        exit 1
    fi
    echo "📦 Installing $DEB..."
    sudo dpkg -i "$DEB"
    echo "✅ Installed via dpkg"

# Run the installed AppImage
[group('run')]
run:
    #!/usr/bin/env bash
    set -e
    
    if [ ! -f "{{local_bin}}/flare.AppImage" ]; then
        echo "❌ Flare not installed at {{local_bin}}/flare.AppImage"
        echo "Run: just install"
        exit 1
    fi
    
    echo "🚀 Starting Flare..."
    exec "{{local_bin}}/flare.AppImage"

# Build, install, and run (full pipeline)
[group('run')]
build-and-run: install run

# ============================================================================
# Development
# ============================================================================

# Run in development mode with hot-reload
[group('dev')]
dev: check-deps
    #!/usr/bin/env bash
    set -e
    export LD_LIBRARY_PATH="{{swift_lib_path}}"
    exec pnpm tauri dev

# Run frontend only (no Tauri)
[group('dev')]
dev-frontend:
    pnpm dev

# Type check the codebase
[group('dev')]
check:
    pnpm check

# Run linting
[group('dev')]
lint:
    pnpm lint

# Run tests
[group('dev')]
test:
    pnpm test

# ============================================================================
# MCP Development (Meta AF)
# ============================================================================

# Test MCP server health and all endpoints
[group('mcp')]
mcp-health:
    #!/usr/bin/env bash
    set -e
    echo "🔍 Testing Flareup MCP server..."
    ./test-mcp-server.sh

# Scaffold a new MCP-enabled feature
[group('mcp')]
mcp-new-feature FEATURE_NAME:
    #!/usr/bin/env bash
    set -e
    ./scripts/new-mcp-feature.sh {{FEATURE_NAME}}

# Start MCP server for testing
[group('mcp')]
mcp-start:
    #!/usr/bin/env bash
    set -e
    echo "🚀 Starting Flareup MCP server..."
    cd packages/mcp-server && pnpm dev

# Check if debug API is running
[group('mcp')]
mcp-check-api:
    #!/usr/bin/env bash
    echo "Checking Flareup debug API..."
    curl -s http://127.0.0.1:7266/health | jq .

# Query specific debug endpoint
[group('mcp')]
mcp-query ENDPOINT:
    #!/usr/bin/env bash
    curl -s http://127.0.0.1:7266{{ENDPOINT}} | jq .

# Tail recent logs via MCP
[group('mcp')]
mcp-logs LIMIT="10":
    #!/usr/bin/env bash
    curl -s "http://127.0.0.1:7266/logs?limit={{LIMIT}}" | jq -r '.data[] | "[\\(.timestamp)] \\(.level | ascii_upcase) \\(.target): \\(.message)"'

# Set log level via MCP
[group('mcp')]
mcp-set-log-level LEVEL:
    #!/usr/bin/env bash
    curl -s -X POST http://127.0.0.1:7266/logs/config \
        -H "Content-Type: application/json" \
        -d '{"level":"{{LEVEL}}"}' | jq .

# Show MCP development workflow
[group('mcp')]
mcp-workflow:
    @echo "📚 MCP Development Workflow"
    @echo "==========================="
    @echo ""
    @echo "See: .agent/workflows/mcp-driven-development.md"
    @echo ""
    @echo "Quick commands:"
    @echo "  just mcp-new-feature <name>  - Scaffold new MCP-enabled feature"
    @echo "  just mcp-health              - Test all MCP endpoints"
    @echo "  just mcp-logs                - View recent logs"
    @echo "  just mcp-check-api           - Check if API is running"

# ============================================================================
# Utilities
# ============================================================================

# Clean all build artifacts
[group('util')]
[confirm("This will delete all build artifacts. Continue?")]
clean:
    #!/usr/bin/env bash
    set -e
    echo "🧹 Cleaning build artifacts..."
    
    rm -rf src-tauri/target
    rm -rf build
    rm -rf .svelte-kit
    rm -rf sidecar/dist
    rm -rf src-tauri/binaries/app-*
    rm -rf src-tauri/SoulverWrapper/.build
    
    echo "✅ Clean complete"

# Bump patch version across package.json, tauri.conf.json, and Cargo.toml
[group('util')]
bump-patch:
    #!/usr/bin/env bash
    set -e
    OLD=$(jq -r .version package.json)
    IFS='.' read -r MAJOR MINOR PATCH <<< "$OLD"
    NEW="$MAJOR.$MINOR.$((PATCH + 1))"
    echo "Bumping $OLD → $NEW"
    # package.json
    jq --arg v "$NEW" '.version = $v' package.json > package.json.tmp && mv package.json.tmp package.json
    # tauri.conf.json
    jq --arg v "$NEW" '.version = $v' src-tauri/tauri.conf.json > src-tauri/tauri.conf.json.tmp && mv src-tauri/tauri.conf.json.tmp src-tauri/tauri.conf.json
    # Cargo.toml (first [package] version field only)
    sed -i "0,/^version = \"$OLD\"/s/^version = \"$OLD\"/version = \"$NEW\"/" src-tauri/Cargo.toml
    echo "✅ Version bumped to $NEW"

# Show build configuration
[group('util')]
info:
    @echo "Flare Build Info"
    @echo "================"
    @echo "Version: {{version}}"
    @echo "Target:  {{arch}}"
    @echo "Local bin: {{local_bin}}"

# List all recipes
[group('util')]
@help:
    just --list --unsorted

# Enable Flare to start on login
[group('util')]
autostart:
    #!/usr/bin/env bash
    set -e
    
    LOCAL_BIN="${HOME}/.local/bin"
    AUTOSTART_DIR="${HOME}/.config/autostart"
    DESKTOP_FILE="${AUTOSTART_DIR}/flare.desktop"
    
    mkdir -p "$AUTOSTART_DIR"
    
    echo "[Desktop Entry]" > "$DESKTOP_FILE"
    echo "Type=Application" >> "$DESKTOP_FILE"
    echo "Name=Flare" >> "$DESKTOP_FILE"
    echo "Comment=Spotlight-like launcher for Linux" >> "$DESKTOP_FILE"
    echo "Exec=${LOCAL_BIN}/flare.AppImage" >> "$DESKTOP_FILE"
    echo "Icon=flare" >> "$DESKTOP_FILE"
    echo "Terminal=false" >> "$DESKTOP_FILE"
    echo "Categories=Utility;" >> "$DESKTOP_FILE"
    echo "X-GNOME-Autostart-enabled=true" >> "$DESKTOP_FILE"
    echo "StartupNotify=false" >> "$DESKTOP_FILE"
    
    echo "✅ Autostart enabled"
    echo "Flare will start automatically on login"
    echo "Desktop file: $DESKTOP_FILE"

# Disable Flare autostart
[group('util')]
remove-autostart:
    #!/usr/bin/env bash
    set -e
    
    DESKTOP_FILE="${HOME}/.config/autostart/flare.desktop"
    
    if [ -f "$DESKTOP_FILE" ]; then
        rm "$DESKTOP_FILE"
        echo "✅ Autostart disabled"
    else
        echo "ℹ️  Autostart was not enabled"
    fi
