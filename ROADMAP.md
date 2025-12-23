# Flare Development Roadmap
**Last Updated:** 2025-12-22
**Current Version:** 0.1.0
**Raycast Parity:** ~70%

---

## 🎯 Project Vision

Build a Raycast-quality launcher for Linux with native system integration and extension compatibility.

**Core Goals:**
- 90%+ Raycast feature parity
- Better Linux-native integration than Raycast
- Maintain extension compatibility where possible
- Superior performance and stability

---

## ✅ Recent Wins (Last 7 Days)

### Extension Compatibility Fixed
- ✅ `usePersistentState` now actually persists (was just `useState`)
- ✅ React Reconciler stubs return safe values instead of throwing errors
- ✅ TcpListener gracefully handles port conflicts (no more crashes)

### Performance & Stability
- ✅ Database indices added (clipboard, AI, snippets) - major query speedup
- ✅ N+1 query eliminated in file indexer - 10x faster indexing
- ✅ CPU monitoring moved to background thread - non-blocking UI
- ✅ Structured logging via `tracing` crate - production-ready

### Code Quality
- ✅ Debug `console.log` statements removed
- ✅ `println!`/`eprintln!` replaced with proper logging

**Result:** 60% → **70% Raycast parity**

---

## 🚀 Current Status

### What Works Well

| Feature | Status | Quality | Notes |
|---------|--------|---------|-------|
| Command Palette | ✅ Complete | Excellent | Fuzzy search, frecency ranking |
| Calculator | ✅ Complete | Excellent | SoulverCore integration |
| Clipboard History | ✅ Complete | Excellent | Text, images, colors, AES-GCM encryption |
| Snippets | ✅ Complete | Good | Rich placeholders, terminal detection |
| AI Chat | ✅ Complete | Excellent | Multi-provider, conversation history |
| File Search | ✅ Complete | Good | Fast indexing, watch for changes |
| Extensions | 🟡 Partial | Good | Basic compatibility, some limitations |
| System Monitors | ✅ Complete | Excellent | CPU, RAM, disk, battery, background updates |
| Quick Toggles | 🟡 Partial | Good | WiFi, Bluetooth, Dark Mode (DE-specific) |
| GitHub OAuth | ✅ Complete | Good | Token management via keyring |

### Critical Gaps

| Feature | Status | Impact | Blocking |
|---------|--------|--------|----------|
| **Window Management** | ❌ Missing | Critical | Move/resize/snap windows |
| **System Commands** | ❌ Missing | Critical | Shutdown, sleep, lock, volume |
| **Per-Command Hotkeys** | ❌ Missing | Critical | Only app toggle exists |
| Downloads Manager | ❌ Missing | Medium | Track/manage downloads |
| Menu Bar / System Tray | ❌ Missing | Medium | Background indicator |

---

## 📋 Remaining Work

### Phase 1: Extension Robustness (2-3 days) 🔴

**Goal:** 70% → 75% parity

| Task | Effort | Files | Priority |
|------|--------|-------|----------|
| Expand AppleScript shims | 4 hours | `extension_shims.rs` | High |
| Replace unsafe `.unwrap()` calls | 1 day | 17 Rust files | High |

**AppleScript Shims to Add:**
- `do shell script "cmd"` → Execute shell command
- `open location "url"` → `xdg-open`
- `delay N` → Sleep/no-op
- `beep` → System sound
- `the clipboard` / `set the clipboard` → Clipboard API

---

### Phase 2: System Integration (2 weeks) 🔴

**Goal:** 75% → 85% parity - **THIS IS THE BIG ONE**

#### 2.1 Window Management (1 week)

**Priority:** CRITICAL - This is Raycast's killer feature

**X11 Implementation:**
- [ ] Create `src-tauri/src/window_manager.rs`
- [ ] Add `x11rb` dependency
- [ ] Detect active window
- [ ] Commands:
  - `move_window_to_left_half()`
  - `move_window_to_right_half()`
  - `center_window()`
  - `maximize_window()`
  - `move_to_next_desktop()`
- [ ] Add UI to command palette
- [ ] Test on GNOME, KDE, XFCE

**Wayland (future):**
- Sway: IPC socket integration
- GNOME: D-Bus extensions
- KDE: KWin scripts

#### 2.2 System Commands (2 days)

**Priority:** CRITICAL - Expected baseline functionality

- [ ] Create `src-tauri/src/system_commands.rs`
- [ ] Commands:
  - `shutdown()` - `systemctl poweroff`
  - `restart()` - `systemctl reboot`
  - `sleep()` - `systemctl suspend`
  - `lock_screen()` - `loginctl lock-session`
  - `set_volume(level)` - `pactl set-sink-volume`
  - `volume_up()` / `volume_down()` / `volume_mute()`
  - `empty_trash()` - Clear `~/.local/share/Trash`
  - `eject_drive(device)` - `udisksctl unmount`
- [ ] Add confirmation dialogs for destructive operations
- [ ] Test on multiple desktop environments

#### 2.3 Per-Command Hotkeys (1 week)

**Priority:** CRITICAL - Major usability feature

- [ ] Create `src-tauri/src/hotkey_manager.rs`
- [ ] Store keybindings in SQLite
- [ ] Settings UI for hotkey configuration
- [ ] Conflict detection (warn on duplicate bindings)
- [ ] Default hotkeys:
  - Clipboard History (Cmd+Shift+C)
  - Snippets (Cmd+Shift+S)
  - File Search (Cmd+Shift+F)
  - System Monitors (Cmd+Shift+M)
  - AI Chat (Cmd+Shift+A)

---

### Phase 3: Polish & Features (1 week) 🟡

**Goal:** 85% → 90% parity

#### 3.1 Downloads Manager (2 days)

- [ ] Create `src-tauri/src/downloads/` module
- [ ] Watch `~/Downloads` for new files
- [ ] SQLite storage for download history
- [ ] UI view in command palette
- [ ] Commands: `list_downloads`, `open_download`, `clear_history`

#### 3.2 Extension Compatibility Improvements (1 day)

- [ ] Add compatibility scoring system
- [ ] Detect macOS-only code patterns
- [ ] Show warnings in Extensions UI
- [ ] Create "verified for Linux" badge

#### 3.3 Performance Profiling (1 day)

- [ ] Profile startup time (target: <500ms)
- [ ] Profile search latency (target: <50ms)
- [ ] Memory usage audit
- [ ] Bundle size optimization

---

### Phase 4: Nice-to-Haves 🟢

**Timeline:** After 90% parity achieved

| Feature | Effort | Priority | Notes |
|---------|--------|----------|-------|
| Menu Bar / System Tray | 3 days | Medium | Background indicator |
| Wayland Window Mgmt | 2 weeks | Medium | Compositor-specific |
| Settings Sync | 1 week | Medium | Cross-device sync |
| Extension Hot Reload | 2 days | Low | Dev experience |
| Trash Management | 1 day | Low | Restore from trash |

---

## 🎨 Future Enhancements

### Linux-Exclusive Features

Features that go beyond Raycast:

#### 1. Keyboard Maestro-Style Macros ⭐
**Priority:** HIGH - Major differentiator

- Record keyboard sequences
- Multiple trigger types (hotkey, typed string, schedule)
- Variable substitution (`{clipboard}`, `{date}`, `{shell:cmd}`)
- Conditional branching and loops

**MVP Scope:**
- Keyboard recording only (no mouse)
- Hotkey triggers
- Basic actions: type text, key combo, delay, shell command
- Simple variables: `{clipboard}`, `{date}`, `{input}`

#### 2. Scheduled Actions
- Run extensions on timers
- Cron-like scheduling
- Daily digest commands

#### 3. Webhooks / Remote Triggers
- HTTP endpoints trigger commands
- Integration with n8n, Zapier, Home Assistant
- Authentication for security

#### 4. Chained Commands / Pipes
- Command output → next command input
- Visual workflow builder
- Save pipelines as reusable workflows

#### 5. Linux System Integration
- Systemd service control
- DBus-native toggles (faster than shell commands)
- Docker/Podman container management
- Flatpak/Snap integration

---

## 📊 Strategic Priorities

From most to least critical for Raycast replacement:

| Rank | Initiative | Impact | Effort | Timeline |
|------|-----------|--------|--------|----------|
| 1 | **Window Management** | Critical | High | Week 1-2 |
| 2 | **System Commands** | Critical | Medium | Week 2 |
| 3 | **Per-Command Hotkeys** | Critical | Medium | Week 3 |
| 4 | Extension Compatibility | High | Medium | Week 4 |
| 5 | Downloads Manager | Medium | Low | Week 5 |
| 6 | Performance Tuning | Medium | Medium | Week 6 |
| 7 | Settings Sync | Medium | High | Future |
| 8 | Macro System | High | High | Future |

---

## 🐛 Known Issues & Limitations

### Extension Compatibility

**What Works:**
- Pure UI extensions (lists, forms, detail views) - 90%
- Clipboard operations - 80% (HTML not supported)
- HTTP/API calls - 95%
- Local storage & preferences - 100%

**What Doesn't:**
- AppleScript (only 4 basic patterns) - 10%
- Native macOS binaries - 0%
- macOS-specific system APIs - 5%
- Browser JS evaluation - 0%

**AppleScript Coverage:**

| Pattern | Status |
|---------|--------|
| `tell app "X" to activate` | ✅ Supported |
| `tell app "X" to quit` | ✅ Supported |
| `display notification` | ✅ Supported |
| `set volume` | ✅ Supported |
| `do shell script` | ❌ Not yet |
| `open location` | ❌ Not yet |
| `tell app "System Events"` | ❌ Complex |
| `tell app "Finder"` | ❌ Complex |

### Platform Limitations

| Feature | X11 | Wayland | Notes |
|---------|-----|---------|-------|
| Window Management | ✅ Planned | 🟡 Partial | Compositor-specific |
| Global Hotkeys | ✅ Works | ✅ Works | Via Tauri plugin |
| Clipboard | ✅ Works | ✅ Works | Via Tauri plugin |
| Selected Text | ✅ Works | ⚠️ Limited | Wayland security model |
| Snippet Expansion | ✅ Works | ⚠️ Requires udev | Need keyboard access |

---

## 📈 Progress Tracking

### Milestones

- [x] **v0.1.0** - Core features (command palette, clipboard, snippets, AI)
- [ ] **v0.2.0** - Extension compatibility & performance (ETA: 2 weeks)
- [ ] **v0.3.0** - System integration (window mgmt, system commands) (ETA: 4 weeks)
- [ ] **v0.4.0** - Polish & optimization (ETA: 6 weeks)
- [ ] **v0.5.0** - Linux-exclusive features (macros, webhooks) (ETA: 3 months)
- [ ] **v1.0.0** - 90%+ Raycast parity + stable API (ETA: 6 months)

### Raycast Feature Parity

```
[████████████████░░░░] 70%
```

**Breakdown:**
- Core UI/UX: 95%
- Built-in Commands: 60%
- Extension System: 65%
- System Integration: 40%
- Performance: 80%

---

## 🏗️ Architecture Notes

### Multi-Process Design

1. **Tauri Backend (Rust)** - System integration, database, file I/O
2. **Sidecar (Node.js)** - JavaScript runtime for extensions
3. **UI (WebView)** - Svelte 5 frontend

**Communication:** MessagePack IPC via `@flare/protocol` package

### Key Technologies

- **Frontend:** Svelte 5, SvelteKit 2, Tailwind CSS 4
- **Backend:** Rust, Tauri 2, Tokio async runtime
- **Database:** SQLite (rusqlite) with AES-GCM encryption
- **Credentials:** System keyring (Linux native)
- **Calculator:** SoulverCore (Swift wrapper)

---

## 📝 Changelog

### 2025-12-22
- Fixed `usePersistentState` to actually persist
- Fixed React Reconciler stubs (no-op instead of throw)
- Fixed TcpListener port conflict crash
- Added database indices for performance
- Eliminated N+1 query in file indexer
- Moved CPU monitoring to background thread
- Replaced println!/eprintln! with structured logging
- **Parity:** 60% → 70%

### 2025-12-21
- Created comprehensive audit and TODO
- Identified critical gaps
- Prioritized roadmap

### Earlier Work
- ✅ AI chat with multi-provider support (OpenRouter, Ollama)
- ✅ Snippet editing UI with terminal detection
- ✅ File search indexing and watching
- ✅ OAuth integration (GitHub)
- ✅ System monitors with real-time updates

---

## 🎯 Next Actions (This Week)

1. **Update this roadmap** as work progresses
2. **Implement AppleScript shims** (Tier 1: 4 hours)
3. **Start window management research** (X11 APIs)
4. **Replace unsafe `.unwrap()` calls** (ongoing)

---

**Legend:**
- 🔴 Critical priority (needed for Raycast replacement)
- 🟡 High priority (important but not blocking)
- 🟢 Medium/Low priority (nice to have)
- ✅ Complete
- 🟡 Partial
- ❌ Not started
