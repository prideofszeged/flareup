# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Flare** is an open-source, Raycast-compatible launcher for Linux. It's a sophisticated desktop application that combines:
- A modern UI (Svelte 5 + SvelteKit 2 + Tailwind CSS)
- System integration backend (Rust + Tauri 2)
- JavaScript plugin host for Raycast extensions
- Advanced features: clipboard history, snippet expansion, AI integration, advanced calculator

**Repository:** https://github.com/ByteAtATime/flare
**Key Blog Post:** https://byteatatime.dev/posts/recreating-raycast

## Quick Start Commands

All builds use `just` command runner. Run `just --list` to see all available recipes.

### Development
```bash
pnpm install                    # Install all workspace dependencies
just dev                        # Run in dev mode with hot-reload
just dev-frontend               # Frontend-only dev (no Tauri backend)
pnpm check                      # Type check (svelte-check)
pnpm lint                       # Lint with prettier & eslint
pnpm test                       # Run tests (vitest)
```

### Building
```bash
just build                      # Full production build (AppImage + DEB + RPM)
just build-appimage             # AppImage only
just build-deb                  # DEB package only
just build-appimage-fast        # AppImage with fast compile profile
just install                    # Build and install to ~/.local/bin
just run                        # Run installed AppImage
just build-and-run              # Full pipeline: build → install → run
```

### Utilities
```bash
just check-deps                 # Verify all dependencies installed
just setup-tools                # Download AppImage tools to ~/.local/bin
just clean                      # Remove all build artifacts
just info                       # Show build configuration
```

## Architecture Overview

### Multi-Process Architecture

**Three main processes:**
1. **Tauri Backend (Rust)** — System integration, database, file I/O, extension loading
2. **Sidecar (Node.js)** — JavaScript runtime for Raycast extensions, compiled to binary
3. **UI (WebView)** — Svelte frontend rendered by Tauri's WebView

**Communication:** MessagePack-based IPC protocol via `@flare/protocol` package

### Directory Structure

```
flareup/
├── src/                        # Frontend (Svelte 5 + TypeScript)
│   ├── lib/
│   │   ├── components/         # UI components (command-palette, extensions, etc.)
│   │   ├── views/              # Page views (Settings, Clipboard, etc.)
│   │   ├── *.svelte.ts         # State stores (command-palette, apps, quicklinks, etc.)
│   │   └── types.ts            # Shared TypeScript types
│   └── routes/                 # SvelteKit routes (+page.svelte, hud/, etc.)
│
├── src-tauri/                  # Tauri backend (Rust)
│   ├── src/
│   │   ├── lib.rs              # Main Tauri setup & commands
│   │   ├── ai.rs               # OpenRouter AI integration
│   │   ├── clipboard_history/  # Clipboard monitor & history DB
│   │   ├── file_search/        # File indexing & search
│   │   ├── snippets/           # Text snippet expansion engine
│   │   ├── extensions.rs       # Raycast extension loading
│   │   ├── extension_shims.rs  # API compatibility layer
│   │   ├── soulver.rs          # Calculator (SoulverCore wrapper)
│   │   ├── browser_extension.rs # WebSocket for browser integration
│   │   ├── frecency.rs         # Search result ranking algorithm
│   │   ├── app.rs              # Application discovery
│   │   ├── system_monitors.rs  # Real-time CPU/RAM/disk info
│   │   └── (20+ modules)       # OAuth, cache, store, desktop detection, etc.
│   └── SoulverWrapper/         # Swift wrapper for calculator binary
│
├── sidecar/                    # JavaScript extension runtime
│   ├── src/
│   │   ├── index.ts            # Plugin loader & executor
│   │   ├── api/                # Raycast API compatibility
│   │   └── io.ts               # MessagePack I/O
│   └── dist/                   # Compiled binary (via pkg)
│
├── packages/
│   └── protocol/               # Shared IPC message types (TypeScript)
│
├── justfile                    # Build recipes (use: just <recipe>)
├── pnpm-workspace.yaml         # pnpm workspace configuration
├── vite.config.js              # Vite bundler config
├── svelte.config.js            # SvelteKit config
└── [README.md, BUILDING.md]    # Documentation
```

### Key Modules by Responsibility

**Frontend (`src/`):**
- State management via Svelte stores (command-palette.svelte.ts, apps.svelte.ts, etc.)
- Component hierarchy: CommandPalette → NodeRenderer → Views
- Settings UI, Clipboard History view, AI Chat, File Search, System Monitors
- Communication with backend via Tauri `invoke()` and event listeners

**Backend (`src-tauri/src/`):**

| Module | Purpose |
|--------|---------|
| `app.rs` | Discover & cache installed applications |
| `clipboard_history/` | Monitor clipboard, store history with AES-GCM encryption |
| `file_search/` | Index filesystem recursively, watch for changes |
| `snippets/` | Manage & expand text snippets with hotkey triggers |
| `extensions.rs` | Parse `.raycast` archives, load via sidecar |
| `extension_shims.rs` | Raycast API compatibility layer |
| `soulver.rs` | FFI to Swift calculator wrapper |
| `ai.rs` | OpenRouter API, token tracking, streaming |
| `system_monitors.rs` | CPU, RAM, disk, temperature |
| `browser_extension.rs` | WebSocket server for browser integration |
| `frecency.rs` | Ranking algorithm for search results |
| `store.rs` | User preferences (JSON) & keyring secrets |
| `cache.rs` | App metadata caching |
| `oauth.rs` | Token storage & refresh |
| `desktop.rs` | Desktop environment detection (X11/Wayland) |

**Sidecar (`sidecar/`):**
- Node.js runtime compiled to binary via `pkg`
- Executes Raycast extensions in separate process
- React Reconciler for component rendering patterns
- MessagePack protocol for backend communication

**Protocol (`packages/protocol/`):**
- Zod-validated TypeScript schemas for all IPC messages
- Types: `api.ts`, `command.ts`, `plugin.ts`, `ai.ts`, `preferences.ts`, etc.

## Data Storage

- **SQLite** (via rusqlite): Clipboard history, snippets, AI usage
- **Keyring** (native): OAuth tokens, sensitive credentials
- **JSON files** (`.local/share/flare/`): User preferences, quicklinks

Database schemas managed by individual modules. Indices added for frequently queried columns to prevent N+1 queries.

## Build Process & Profiles

### Build Steps
1. Build Node.js sidecar → standalone binary (`pnpm --filter sidecar build`)
2. Compile Swift SoulverCore wrapper (`swift build -c release`)
3. Build Vite frontend (optimized JS/CSS)
4. Bundle with Tauri (creates AppImage/DEB/RPM)

### Compile Profiles
- **Release (default):** LTO enabled, single-threaded codegen (slower build, smaller binary)
- **Release-Fast:** No LTO, parallel codegen (faster build, slightly larger binary)

Use `just build-appimage-fast` or `just build-deb-fast` for development builds.

## Environment Variables & Paths

**For development with SoulverCore:**
```bash
export LD_LIBRARY_PATH="$(pwd)/src-tauri/SoulverWrapper/.build/release:$(pwd)/src-tauri/SoulverWrapper/Vendor/SoulverCore-linux"
```

This is automatically set by `just dev`.

**AppImage tools location:** `~/.local/bin/` (linuxdeploy, appimagetool)

## Dependencies to Know

### Frontend Critical
- **Svelte 5** — Reactive UI framework
- **SvelteKit 2** — Routing, SSR (static adapter)
- **Tauri Plugins** — Clipboard, fs, dialog, shell, http, opener, os, etc.
- **Zod** — Schema validation for IPC messages
- **fuse.js** — Fuzzy search for command palette
- **Tailwind CSS 4** — Utility styling

### Backend Critical
- **Tauri 2** — Desktop app framework, IPC
- **Tokio** — Async runtime
- **SQLite (rusqlite)** — Database with encryption support
- **Keyring** — Secure credential storage
- **notify** — File system watching
- **rdev/evdev** — Input event capture
- **enigo** — Keyboard/mouse simulation
- **arboard** — Clipboard access

### Shared
- **MessagePack (msgpackr)** — Binary serialization for protocol
- **TypeScript** — Type safety across all layers

## Key Design Patterns

### Event-Driven Updates
- Tauri events push real-time updates from backend to frontend
- Svelte stores subscribe to events for reactive UI
- File watchers (clipboard, filesystem) emit events on changes

### Extensibility via Shims
- Raycast API mapped to Linux equivalents
- Extensions run in isolated sidecar process
- Browser API emulation for file access
- CLI substitutes for macOS-specific commands

### Platform Abstraction
- Input managers pluggable per desktop environment (Evdev for Wayland, Rdev for X11)
- Desktop environment detected at startup
- Feature detection for conditional capabilities

### Database Indices
- Prevents N+1 query problems in file search and clipboard history
- Structured queries with proper schema design
- Consider indices when adding new searchable fields

## Testing

```bash
pnpm test                       # Run unit tests (Vitest)
pnpm check                      # Type check + svelte-check
```

Tests use Vitest + Testing Library. No extensive test suite currently; focus on critical logic.

## Packaging & Distribution

- **AppImage** — Portable, single-file executable (recommended)
- **DEB** — Debian/Ubuntu packages
- **RPM** — Fedora/Red Hat packages

All built to `src-tauri/target/release/bundle/` subdirectories.

## Recent Work & Branch Info

**Active branch:** `fixes/claudit`

**Recent changes:**
- Database index optimization (clipboard, AI, snippets)
- File indexing performance improvements
- Snippet editing UI enhancements
- Terminal detection for paste behavior
- AI chat conversation saving
- Debug log removal

## Common Development Tasks

### Adding a New Tauri Command
1. Add `#[tauri::command]` function in `src-tauri/src/lib.rs` or module
2. Define Zod schema in `packages/protocol/src/` if complex
3. Call via `invoke('command_name')` in Svelte frontend
4. Rebuild: `just dev` (or `pnpm tauri dev`)

### Creating a New Component
1. Create `.svelte` file in `src/lib/components/`
2. Use Tailwind + bits-ui for styling/interactivity
3. Accept props, emit events
4. Consider extracting reusable logic to Svelte stores

### Working with Snippets or Clipboard History
- Modify schema in respective module (e.g., `clipboard_history/types.rs`)
- Update database queries in manager
- Ensure database migration handled (delete `.db` for dev)
- Test UI changes in settings/clipboard views

### Testing Frontend Logic
- Use Vitest for unit tests (`src/**/*.test.ts`)
- Testing Library for component tests
- Playwright for E2E (rarely used currently)

## Troubleshooting

### "linuxdeploy not found"
```bash
just setup-tools
just install-tools-system    # Optional: install system-wide
```

### Missing Swift/SoulverCore errors
```bash
swift build -c release --package-path src-tauri/SoulverWrapper
export LD_LIBRARY_PATH="$(pwd)/src-tauri/SoulverWrapper/.build/release:$(pwd)/src-tauri/SoulverWrapper/Vendor/SoulverCore-linux"
```

### Type errors after code changes
```bash
pnpm check                   # Full type check
svelte-kit sync              # Sync SvelteKit generated types
```

### Database corruption or schema mismatch
```bash
just clean                   # Removes all build artifacts including .db files
```

## Important Files to Know

| File | Purpose |
|------|---------|
| `justfile` | All build recipes |
| `pnpm-workspace.yaml` | Workspace configuration |
| `packages/protocol/src/index.ts` | IPC message schemas |
| `src-tauri/tauri.conf.json` | Tauri app configuration (window, capabilities) |
| `src-tauri/Cargo.toml` | Rust dependencies |
| `vite.config.js` | Frontend bundler config |
| `svelte.config.js` | SvelteKit adapter & hooks |
| `.env.example` (if present) | Environment variable template |

## Performance Considerations

- **Indexing:** File search uses database indices; linear scanning for large directories avoided
- **Caching:** App metadata cached to avoid repeated desktop file parsing
- **Frecency:** Search results ranked by frequency + recency
- **Streaming:** AI responses streamed to UI in real-time
- **Input capture:** Snippet expansion uses native input events (not polling)
- **Compilation:** LTO disabled in fast profile for development

## Security Notes

- Clipboard history encrypted with AES-GCM
- OAuth tokens stored in system keyring
- Zod validation on all IPC inputs
- No unsanitized user input passed to system commands
- File operations restricted to user-owned directories (mostly)

## Related Resources

- [Official Raycast API Docs](https://developers.raycast.com/)
- [Tauri 2 Docs](https://v2.tauri.app/)
- [SvelteKit Docs](https://kit.svelte.dev/)
- [Blog Post on Recreating Raycast](https://byteatatime.dev/posts/recreating-raycast)
