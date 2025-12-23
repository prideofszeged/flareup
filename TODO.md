# Flareup Development TODO
**Last Updated:** 2025-12-21
**Version:** 0.1.0

This file tracks planned and completed work based on the comprehensive audit. Items are organized by priority and estimated effort.

---

## 🔴 Critical Priority (Do Immediately)

### Code Quality & Bug Fixes
- [ ] **Remove debug console.log statements** (30 min)
  - [ ] CommandPalette.svelte:95 - `console.log('null haha');`
  - [ ] AiSettingsView.svelte:55
  - [ ] sidecar.svelte.ts:46, 79, 147, 277
  - [ ] +page.svelte:160

- [ ] **Replace println!/eprintln! with structured logging** (3 hours)
  - [ ] Add `tracing` crate to dependencies
  - [ ] Replace ~15 println! calls (info/status messages)
  - [ ] Replace ~45 eprintln! calls (error logging)
  - [ ] Key files: lib.rs, browser_extension.rs, file_search/*.rs, snippets/*.rs, extensions.rs
  - [ ] Configure log levels for dev vs release builds

- [ ] **Fix .unwrap() panic risks** (1 day)
  - [ ] snippets/input_manager.rs
  - [ ] snippets/manager.rs
  - [ ] file_search/manager.rs
  - [ ] clipboard_history/manager.rs
  - [ ] + 13 other files
  - [ ] Replace with proper `?` operator or `match` statements

### Performance - Database (High Impact)
- [ ] **Add critical database indices** (1 hour)
  ```sql
  -- ai.rs
  CREATE INDEX IF NOT EXISTS idx_ai_generations_created ON ai_generations(created);
  CREATE INDEX IF NOT EXISTS idx_ai_conversations_updated ON ai_conversations(updated_at);

  -- clipboard_history
  CREATE INDEX IF NOT EXISTS idx_clipboard_content_type ON clipboard_history(content_type);
  CREATE INDEX IF NOT EXISTS idx_clipboard_pinned ON clipboard_history(is_pinned);
  CREATE INDEX IF NOT EXISTS idx_clipboard_last_copied ON clipboard_history(last_copied_at);

  -- snippets
  CREATE INDEX IF NOT EXISTS idx_snippets_keyword ON snippets(keyword);
  ```

- [ ] **Fix N+1 query in file_search/indexer.rs** (3 hours)
  - [ ] Add `get_all_file_timestamps()` method returning HashMap
  - [ ] Replace loop with single query + in-memory lookups
  - [ ] Test with large file systems (>10k files)

- [ ] **Implement batch database operations** (2 hours)
  - [ ] Wrap file_search indexing in transactions
  - [ ] Batch INSERT operations in manager.rs
  - [ ] Expected: 50x faster bulk indexing

### Performance - Blocking Operations
- [ ] **Fix CPU monitor blocking sleep** (3 hours)
  - [ ] Create background thread with Arc<Mutex<CpuInfo>>
  - [ ] Update state every 200ms
  - [ ] Commands return cached value instantly
  - [ ] File: system_monitors.rs:get_cpu_info()

---

## 🔴 Critical Priority (Week 1-2)

### Window Management (CRITICAL MISSING FEATURE)
- [ ] **Create window_manager.rs module** (2 weeks)
  - [ ] Add x11rb dependency for X11 support
  - [ ] Implement X11 window detection and manipulation
  - [ ] Add commands:
    - [ ] `move_window_to_left_half()`
    - [ ] `move_window_to_right_half()`
    - [ ] `center_window()`
    - [ ] `maximize_window()`
    - [ ] `move_to_next_desktop()`
  - [ ] Create Wayland support (compositor-specific)
    - [ ] Detect compositor (Sway, GNOME, KDE)
    - [ ] Sway IPC socket integration
    - [ ] GNOME D-Bus extensions
    - [ ] KDE KWin scripts
  - [ ] Add frontend UI in command palette
  - [ ] Test on multiple desktop environments

### System Commands (CRITICAL MISSING FEATURE)
- [ ] **Create system_commands.rs module** (1 week)
  - [ ] `shutdown()` - systemctl poweroff
  - [ ] `restart()` - systemctl reboot
  - [ ] `sleep()` - systemctl suspend
  - [ ] `lock_screen()` - loginctl lock-session
  - [ ] `set_volume(level)` - pactl integration
  - [ ] `volume_up()` / `volume_down()` / `volume_mute()`
  - [ ] `empty_trash()` - rm -rf ~/.local/share/Trash/*
  - [ ] `eject_drive(device)` - udisksctl unmount
  - [ ] Add commands to command palette
  - [ ] Add confirmation dialogs for destructive operations
  - [ ] Test on GNOME, KDE, other DEs

### Global Hotkeys (CRITICAL MISSING FEATURE)
- [ ] **Extend hotkey system for per-command bindings** (1 week)
  - [ ] Create hotkey_manager.rs module
  - [ ] Add global shortcut registration for custom commands
  - [ ] Implement keybinding storage (SQLite or config file)
  - [ ] Create settings UI for hotkey configuration
  - [ ] Add conflict detection (duplicate keybindings)
  - [ ] Commands to support:
    - [ ] Clipboard History (Cmd+Shift+C)
    - [ ] Snippets (Cmd+Shift+S)
    - [ ] File Search (Cmd+Shift+F)
    - [ ] System Monitors (Cmd+Shift+M)
    - [ ] AI Chat (Cmd+Shift+A)
  - [ ] Test with multiple simultaneous hotkeys

---

## 🟡 High Priority (Week 3-4)

### Performance Optimization
- [ ] **Replace shell commands with native D-Bus** (3 days)
  - [ ] Add `zbus` dependency
  - [ ] quick_toggles.rs: Replace nmcli with NetworkManager D-Bus
  - [ ] quick_toggles.rs: Replace rfkill with native API
  - [ ] Benchmark performance improvement (expected 10x faster)

- [ ] **Implement parallel database initialization** (1 day)
  - [ ] Add rayon dependency
  - [ ] lib.rs:setup() - parallel manager initialization
  - [ ] Or implement lazy initialization with OnceCell
  - [ ] Measure startup time improvement

- [ ] **Implement SQLite FTS5 for snippet search** (2 days)
  - [ ] Create `snippets_fts` virtual table
  - [ ] Migrate existing snippet search to FTS5
  - [ ] Add triggers to keep FTS in sync
  - [ ] Test search performance on large datasets

- [ ] **Optimize app cache invalidation** (2 days)
  - [ ] cache.rs: Track per-file modification times
  - [ ] Only re-parse changed .desktop files
  - [ ] Test with frequent app installations

### Extension System Improvements
- [ ] **Improve extension compatibility detection** (1 week)
  - [ ] Add more heuristics for macOS-only code
  - [ ] Create compatibility rating system (Compatible/Partial/Incompatible)
  - [ ] Show ratings in Extensions UI
  - [ ] Add warning dialogs for incompatible extensions

- [ ] **Create Linux extension guidelines** (2 days)
  - [ ] Document path differences
  - [ ] List unavailable APIs (AppleScript, Swift bindings)
  - [ ] Provide Linux alternatives
  - [ ] Create example Linux-native extension

- [ ] **Fork top 10 popular extensions** (2 weeks)
  - [ ] Identify most-used Raycast extensions
  - [ ] Create Linux-compatible versions
  - [ ] Host on GitHub
  - [ ] Add to Flareup extension store

### Testing Infrastructure
- [ ] **Expand test coverage** (3 days)
  - [x] Frontend testing infrastructure exists (vitest + testing-library)
  - [x] Extensions.svelte has comprehensive tests (293 lines)
  - [x] CommandPalette.svelte has comprehensive tests (472 lines)
  - [ ] Add Rust unit tests (currently 0 coverage - critical gap)
    - [ ] Write tests for snippets/engine.rs (placeholder expansion)
    - [ ] Write tests for extension_shims.rs (path translation)
    - [ ] Write tests for frecency.rs (scoring algorithm)
    - [ ] Write tests for soulver.rs (calculator)
  - [ ] Add test dependencies to Cargo.toml (proptest, mockall)
  - [ ] Target: 60% coverage for critical Rust modules

- [ ] **Enhance existing CI pipeline** (2 days)
  - [x] nightly.yml exists (builds AppImage daily)
  - [ ] Add `cargo test` step to workflow
  - [ ] Add `pnpm test:unit` step to workflow
  - [ ] Run `cargo clippy -- -D warnings`
  - [ ] Run `cargo fmt -- --check`
  - [ ] Create PR-triggered workflow (not just nightly/manual)

---

## 🟡 High Priority (Week 5-6)

### Wayland Improvements
- [ ] **Improve Wayland compositor support** (1 week)
  - [ ] Add compositor detection function
  - [ ] Implement Sway IPC integration
    - [ ] Window manipulation via IPC
    - [ ] Workspace switching
  - [ ] Implement GNOME Shell D-Bus extensions
  - [ ] Implement KDE KWin script interface
  - [ ] Add selected text access for Wayland
  - [ ] Test on each compositor

### Code Quality & Refactoring
- [ ] **Refactor large modules** (3 days)
  - [ ] Split ai.rs (726 lines) into:
    - [ ] ai/mod.rs
    - [ ] ai/client.rs
    - [ ] ai/storage.rs
    - [ ] ai/types.rs
  - [ ] Split extensions.rs (631 lines) into:
    - [ ] extensions/loader.rs
    - [ ] extensions/compatibility.rs
    - [ ] extensions/types.rs
  - [ ] Extract from lib.rs (661 lines):
    - [ ] hotkey.rs
    - [ ] window.rs

- [ ] **Address TODO comments in TypeScript/Svelte** (2 days)
  - Note: No TODOs found in Rust code
  - [ ] assets.ts:32,52,57 - Improve icon resolution heuristic
  - [ ] assets.ts:57 - Implement adjustContrast
  - [ ] CommandDeeplinkConfirm.svelte:33 - Implement "always open"
  - [ ] nodes/shared/actions.ts:7 - Improve function naming
  - [ ] sidecar/src/api/cache.ts:37 - Understand and document the fix

### Documentation
- [ ] **Create user documentation** (1 week)
  - [ ] docs/user-guide/installation.md
  - [ ] docs/user-guide/quickstart.md
  - [ ] docs/user-guide/keyboard-shortcuts.md
  - [ ] docs/user-guide/features/snippets.md
  - [ ] docs/user-guide/features/clipboard.md
  - [ ] docs/user-guide/features/ai-chat.md
  - [ ] docs/user-guide/troubleshooting.md
  - [ ] Extension compatibility list

- [ ] **Create developer documentation** (3 days)
  - [ ] docs/developer/architecture.md
  - [ ] docs/developer/building.md
  - [ ] docs/developer/contributing.md
  - [ ] docs/developer/extension-development.md
  - [ ] Generate API docs with `cargo doc`

---

## 🟢 Medium Priority (Future Releases)

### UI/UX Improvements
- [ ] **Accessibility audit** (2 days)
  - [ ] Verify BaseList.svelte has proper ARIA roles
  - [ ] Add aria-live regions for AI streaming
  - [ ] Test with Orca screen reader
  - [ ] Verify WCAG AA color contrast
  - [ ] Fix any keyboard trap issues

- [ ] **UI polish** (1 week)
  - [ ] Smooth animations for view transitions
  - [ ] Loading state improvements
  - [ ] Empty state illustrations
  - [ ] Error message improvements
  - [ ] Consistency pass on all components

### Feature Completeness
- [ ] **Menu Bar Extra / System Tray** (1 week)
  - [ ] Add system tray icon
  - [ ] Quick actions menu
  - [ ] Status indicators
  - [ ] Test on GNOME, KDE, Cinnamon

- [ ] **Fallback commands** (2 days)
  - [ ] Add fallback command configuration
  - [ ] UI for setting default actions
  - [ ] Test fallback logic

- [ ] **Extension hot reload** (3 days)
  - [ ] Watch extension directories for changes
  - [ ] Reload extensions without app restart
  - [ ] Show notification on reload

- [ ] **Trash management** (1 day)
  - [ ] `show_trash()` command
  - [ ] `restore_from_trash(file)` command
  - [ ] Integration with file manager

### Terminal Detection Improvements
- [ ] **Improve snippet terminal detection** (2 days)
  - [ ] Add heuristic fallback (check for "term" in WM_CLASS)
  - [ ] Add process TTY detection
  - [ ] Add user override settings
  - [ ] Test with various terminals
  - [ ] Address "not 100 percent happy" from commit c052e1a

### Security Improvements
- [ ] **Extension security** (3 days)
  - [ ] Create allowlist of verified extensions
  - [ ] Add permission system for extensions
  - [ ] Sandbox extension execution more strictly
  - [ ] Add extension code review process

- [ ] **Permission auditing** (1 day)
  - [ ] Document all required permissions
  - [ ] Add runtime permission checks
  - [ ] Improve udev rules documentation
  - [ ] Add permission troubleshooting guide

---

## 🔵 Low Priority (Nice to Have)

### Advanced Features (from FEATURE_IDEAS.md)
- [ ] **Keyboard Maestro-like macros** (4 weeks)
  - [ ] Design macro system architecture
  - [ ] Implement macro recorder
  - [ ] Create macro editor UI
  - [ ] Add trigger system (hotkey, schedule, event)

- [ ] **Scheduled actions/automations** (2 weeks)
  - [ ] Cron-like scheduler backend
  - [ ] UI for schedule management
  - [ ] Action templates
  - [ ] Notification system

- [ ] **Webhooks and remote triggers** (2 weeks)
  - [ ] HTTP server for webhook endpoints
  - [ ] Webhook configuration UI
  - [ ] Authentication/security
  - [ ] Event payload parsing

- [ ] **Headless/background extensions** (1 week)
  - [ ] Extension lifecycle management
  - [ ] Background process monitoring
  - [ ] Resource usage limits

- [ ] **File actions/contextual actions** (1 week)
  - [ ] Right-click integration with file manager
  - [ ] Action registry system
  - [ ] Custom action creation UI

- [ ] **Chained commands/pipes** (2 weeks)
  - [ ] Command pipeline syntax
  - [ ] Data passing between commands
  - [ ] Pipeline editor UI

### Optimization & Polish
- [ ] **Bundle size optimization** (2 days)
  - [ ] Run source-map-explorer
  - [ ] Implement code splitting
  - [ ] Lazy load settings view
  - [ ] Tree-shake unused components
  - [ ] Target: <1MB initial bundle

- [ ] **Memory profiling** (1 day)
  - [ ] Profile memory usage during typical workflows
  - [ ] Identify memory leaks
  - [ ] Optimize large data structures
  - [ ] Add memory usage monitoring

- [ ] **Startup time optimization** (2 days)
  - [ ] Profile startup sequence
  - [ ] Lazy load non-critical modules
  - [ ] Optimize database connections
  - [ ] Target: <500ms startup time

### Platform Support
- [ ] **Test on additional desktop environments** (1 week)
  - [ ] Cinnamon
  - [ ] MATE
  - [ ] XFCE
  - [ ] i3
  - [ ] Sway
  - [ ] Hyprland
  - [ ] Document compatibility matrix

- [ ] **Cross-distro testing** (1 week)
  - [ ] Ubuntu (latest + LTS)
  - [ ] Fedora
  - [ ] Arch Linux
  - [ ] openSUSE
  - [ ] Debian
  - [ ] Create distro-specific packages

---

## ✅ Completed Work

### Recent Completions (from git log)
- ✅ Extension fixes (commit c052e1a) - partial, marked as "not 100 percent happy"
- ✅ AI chat conversation saver (commit c6d628b)
- ✅ AI temperature control (commit 1357ec7)
- ✅ Multi-AI provider support with Ollama (commit 3f0040f)
- ✅ AI chat view integration (commit 7045c21)

### Core Features Implemented
- ✅ Command palette with fuzzy search
- ✅ Calculator (SoulverCore integration)
- ✅ Clipboard history with encryption
- ✅ Snippets with rich placeholders
- ✅ AI integration (OpenRouter + Ollama)
- ✅ File search and indexing
- ✅ System monitors (CPU, memory, disk, battery)
- ✅ Quick toggles (WiFi, Bluetooth, Dark Mode)
- ✅ GitHub OAuth integration
- ✅ Extension loading system
- ✅ Frecency ranking
- ✅ Deep linking support
- ✅ Settings UI
- ✅ Secure credential storage (system keyring)

### Infrastructure Implemented
- ✅ Frontend testing infrastructure (vitest + testing-library)
- ✅ Extensions.svelte comprehensive tests (293 lines)
- ✅ CommandPalette.svelte comprehensive tests (472 lines)
- ✅ CI/CD pipeline for nightly AppImage builds
- ✅ Monorepo structure with shared protocol package

---

## 📊 Progress Tracking

### Critical Path to Raycast Replacement
**Phase 1: Core Stability** (2-3 weeks)
- 0% complete

**Phase 2: Feature Parity** (4-6 weeks)
- Window Management: Not started
- System Commands: Not started
- Global Hotkeys: Not started

**Phase 3: Polish** (2-3 weeks)
- 0% complete

**Phase 4: Advanced Features** (8-12 weeks)
- 0% complete

### Overall Completion Estimate
- **Current State:** ~60% Raycast feature parity
- **Target:** 95% Raycast feature parity + Linux-specific enhancements
- **Estimated Timeline:** 3-6 months with focused development

---

## 🎯 Immediate Next Steps (This Week)

1. **Fix critical bugs** (Day 1)
   - Remove debug console.log (especially `console.log('null haha')`)
   - Add database indices
   - Replace println!/eprintln! with tracing

2. **Performance improvements** (Day 2-3)
   - Fix N+1 query in file_search/indexer.rs
   - Batch database operations
   - CPU monitor background thread

3. **Start window management** (Day 4-5)
   - Research X11 APIs (x11rb crate)
   - Create window_manager.rs module
   - Implement basic move_to_half

4. **Expand testing** (Day 5)
   - Add Rust unit tests (frontend tests already exist)
   - Add test steps to nightly.yml CI
   - Run existing frontend tests: `pnpm test:unit`

---

## 📝 Notes

- This TODO is based on comprehensive audit completed 2025-12-21
- See AUDIT_REPORT.md for detailed analysis and justifications
- Priorities may shift based on community feedback and usage patterns
- Mark items as completed by changing `[ ]` to `[x]`
- Update "Completed Work" section when finishing major features

**Created:** 2025-12-21 (Claude Sonnet 4.5)
**Reviewed:** 2025-12-21 (Claude Opus 4.5)

**Review Corrections Applied:**
- Testing: Frontend tests already exist - updated to "Expand coverage" not "Set up"
- CI: nightly.yml exists - updated to "Enhance" not "Create"
- TODOs: Clarified they're in TypeScript/Svelte only, none in Rust
- Logging: Added eprintln! count (~45 calls) to migration scope

**Next Review:** TBD after Phase 1 completion
