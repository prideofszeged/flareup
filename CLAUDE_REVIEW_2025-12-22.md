# Flare Gap Analysis & Review
**Date:** 2025-12-22
**Reviewer:** Claude Opus 4.5
**Focus:** Extensions compatibility, Downloads Manager, overall gaps

---

## Executive Summary

Flare is approximately **60% feature-complete** compared to Raycast. The main pain points are:

1. **Extensions**: Many fail due to stub implementations and missing APIs
2. **Downloads Manager**: Does not exist
3. **System Integration**: Window management, system commands, per-command hotkeys missing
4. **Code Quality**: Unsafe `.unwrap()` calls can crash the app

---

## 1. Extensions: Why Many Don't Work

### 1.1 Critical: React Reconciler Stubs

**Location:** `sidecar/src/hostConfig.ts:383-412`

10 React Reconciler methods throw "Function not implemented" errors instead of being no-ops:

```typescript
resetFormInstance: function (): void {
    throw new Error('Function not implemented.');  // CRASHES extension
},
requestPostPaintCallback: function (): void {
    throw new Error('Function not implemented.');
},
shouldAttemptEagerTransition: function (): boolean {
    throw new Error('Function not implemented.');
},
// ... 7 more methods
```

**Impact:** Extensions using concurrent React features, forms, or suspense will crash.

**Fix:** Replace with no-op implementations that return safe defaults.

---

### 1.2 Critical: usePersistentState is Fake

**Location:** `sidecar/src/api/index.ts:97-103`

```typescript
usePersistentState: <T>(
    key: string,
    initialValue: T
): [T, React.Dispatch<React.SetStateAction<T>>, boolean] => {
    const [state, setState] = React.useState(initialValue);
    return [state, setState, false];  // Never persists! Always resets!
},
```

**Impact:** Extensions expecting state to persist between runs lose all data.

**Fix:** Implement actual persistence using LocalStorage API or backend storage.

---

### 1.3 Important: AppleScript Shim is Minimal

**Location:** `src-tauri/src/extension_shims.rs:80-114`

Only 4 AppleScript patterns are supported:

| Pattern | Linux Equivalent |
|---------|-----------------|
| `tell application "X" to activate` | `gtk-launch` / `xdg-open` |
| `tell application "X" to quit` | `pkill -f` |
| `display notification` | `notify-send` |
| `set volume N` | `pactl` / `amixer` |

Everything else returns:
```
"AppleScript not supported on Linux. Script: {script}"
```

**Common unsupported operations:**
- `tell application "System Events"` (keystroke simulation)
- `do shell script` (should map to child_process)
- `tell application "Finder"` (file operations)
- `tell application "Safari"` (browser control)
- Property access (`get name of application`)

---

### 1.4 Important: Missing/Incomplete APIs

| Raycast API | Status | Location |
|-------------|--------|----------|
| `Clipboard.copy/paste` | ✅ Works | `sidecar/src/api/clipboard.ts` |
| `Clipboard.read (HTML)` | ❌ Not supported | `src-tauri/src/clipboard.rs:42` |
| `LocalStorage` | ✅ Works | `sidecar/src/api/utils.ts` |
| `Cache` | ✅ Works | `sidecar/src/api/cache.ts` |
| `usePersistentState` | ❌ Stub only | `sidecar/src/api/index.ts:97` |
| `runAppleScript` | ⚠️ 4 patterns only | `src-tauri/src/extension_shims.rs` |
| `BrowserExtension` | ⚠️ CSS only, no JS eval | `sidecar/src/api/browserExtension.ts` |
| `getSelectedFinderItems` | ✅ Works (Linux equiv) | `sidecar/src/api/environment.ts` |
| `getSelectedText` | ✅ Works | `sidecar/src/api/environment.ts` |
| `showInFinder` | ✅ Works (xdg-open) | `sidecar/src/api/environment.ts` |
| `trash` | ✅ Works | `sidecar/src/api/environment.ts` |
| `OAuth` | ⚠️ Works but unclear packageName | `sidecar/src/api/oauth.ts:151` |
| `AI.ask` | ✅ Works | `sidecar/src/api/ai.ts` |

---

### 1.5 Path Translation Gaps

**Location:** `src-tauri/src/extension_shims.rs:17-74`

Path translation exists but is incomplete:

| macOS Path | Translated To |
|------------|--------------|
| `/Applications/X.app` | `/usr/share/applications/x.desktop` |
| `/Library/` | `/usr/lib/` |
| `~/Library/Application Support/` | `~/.local/share/` |
| `~/Library/Preferences/` | `~/.config/` |
| `/Users/` | `/home/` |

**Problem:** Many extensions hardcode paths without using Raycast APIs, so translation never happens.

---

### 1.6 Extension Compatibility Estimate

| Category | % Working | Notes |
|----------|-----------|-------|
| Pure UI (lists, forms, details) | 90% | Most work fine |
| Clipboard-based | 80% | HTML not supported |
| HTTP/API extensions | 95% | Work well |
| AppleScript automation | 10% | Only basic commands |
| Native binary bundled | 0% | macOS binaries fail |
| System Events | 5% | Almost nothing works |
| Browser control | 20% | CSS queries only |

---

## 2. Downloads Manager: Does Not Exist

### Current State

The file indexer watches `~/Downloads` (`src-tauri/src/file_search/indexer.rs:20`), but this is only for **file search**, not download management.

### What's Missing

| Feature | Status |
|---------|--------|
| Download progress tracking | ❌ Not implemented |
| Download pause/resume/cancel | ❌ Not implemented |
| Download history | ❌ Not implemented |
| Downloads UI view | ❌ Not implemented |
| Browser integration | ❌ Not implemented |
| Download notifications | ❌ Not implemented |

### Recommended Implementation

1. Create `src-tauri/src/downloads/` module:
   - `manager.rs` - Track active downloads
   - `history.rs` - SQLite storage for download history
   - `monitor.rs` - Watch ~/Downloads for new files

2. Create UI in `src/lib/components/DownloadsView.svelte`

3. Add commands:
   - `list_downloads` - Get download history
   - `open_download` - Open file/folder
   - `clear_download_history` - Clean up

---

## 3. Other Major Gaps

### 3.1 Critical Missing Features

| Feature | Priority | Effort | Notes |
|---------|----------|--------|-------|
| Window Management | Critical | 2 weeks | X11 via x11rb, Wayland per-compositor |
| System Commands | Critical | 1 week | shutdown, restart, sleep, lock |
| Per-Command Hotkeys | Critical | 1 week | Currently only global app toggle |
| System Tray | High | 3 days | No background indicator |

### 3.2 Code Quality Issues

#### Unsafe `.unwrap()` Calls (32+ instances)

**High-risk locations:**

| File | Risk | Issue |
|------|------|-------|
| `browser_extension.rs:170` | **Critical** | `TcpListener::bind().expect()` - crashes if port taken |
| `soulver.rs:10` | High | `CString::new().expect()` - crashes on invalid path |
| `snippets/engine.rs:22-28` | Medium | `Regex::new().unwrap()` - unlikely to fail |
| `snippets/manager.rs` | Medium | Many unwraps in tests |

**Fix:** Replace with `?` operator or `match` statements.

#### TcpListener Port Binding

**Location:** `src-tauri/src/browser_extension.rs:170`

```rust
let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
```

If port 7265 is already in use, the entire application crashes.

**Fix:**
```rust
let listener = match TcpListener::bind(&addr).await {
    Ok(l) => l,
    Err(e) => {
        tracing::error!("Failed to bind browser extension port: {}", e);
        return;
    }
};
```

---

## 4. TODO Comments in Codebase

### TypeScript/Svelte TODOs

| Location | Comment | Priority |
|----------|---------|----------|
| `src/lib/assets.ts:44` | `// TODO: better heuristic?` | Low |
| `src/lib/assets.ts:68` | `// TODO: better heuristic?` | Low |
| `src/lib/assets.ts:74` | `// TODO: actually handle adjustContrast` | Low |
| `src/lib/components/CommandDeeplinkConfirm.svelte:39` | `<!-- TODO: implement "always open" -->` | Medium |
| `src/lib/components/nodes/shared/actions.ts:8` | `// TODO: naming?` | Low |
| `sidecar/src/api/oauth.ts:151` | `// TODO: what does this mean?` (packageName) | Medium |

### Rust TODOs

No TODO comments found in Rust code.

---

## 5. Performance Issues

### 5.1 N+1 Query in File Indexer

**Location:** `src-tauri/src/file_search/indexer.rs`

The indexer queries the database for each file's timestamp individually instead of batch querying.

**Fix:** Add `get_all_file_timestamps()` returning `HashMap<PathBuf, i64>`.

### 5.2 Missing Database Indices

**Required indices (from TODO.md):**

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

---

## 6. Recommended Action Plan

### Quick Wins (< 1 day each)

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 1 | Fix React Reconciler stubs (no-op, don't throw) | 1 hour | High |
| 2 | Implement `usePersistentState` properly | 2 hours | High |
| 3 | Add database indices | 30 min | Medium |
| 4 | Fix TcpListener crash on port conflict | 30 min | Medium |
| 5 | Add more AppleScript shims (open URL, clipboard) | 4 hours | Medium |

### Medium Term (1-2 weeks)

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 6 | Replace all `.unwrap()` with safe handling | 1 day | High |
| 7 | Create Downloads Manager module | 2 days | Medium |
| 8 | Window management (X11) | 1 week | High |
| 9 | System commands | 2 days | High |
| 10 | Per-command global hotkeys | 1 week | High |

### Long Term (1+ months)

- Wayland window management (compositor-specific)
- Full AppleScript parser/translator
- Extension compatibility scoring system
- Fork/adapt top 10 popular Raycast extensions for Linux

---

## 7. Files Referenced

| File | Purpose |
|------|---------|
| `sidecar/src/hostConfig.ts` | React Reconciler configuration |
| `sidecar/src/api/index.ts` | Raycast API exports |
| `sidecar/src/api/*.ts` | Individual API implementations |
| `src-tauri/src/extension_shims.rs` | macOS API compatibility |
| `src-tauri/src/browser_extension.rs` | WebSocket server |
| `src-tauri/src/file_search/indexer.rs` | File indexing |
| `src-tauri/src/clipboard.rs` | Clipboard operations |
| `TODO.md` | Existing task tracking |

---

## 8. Conclusion

Flare has a solid foundation but needs work in three areas:

1. **Extension Compatibility**: Quick fixes to `usePersistentState` and React Reconciler stubs would immediately improve compatibility
2. **Feature Gaps**: Downloads Manager, Window Management, and System Commands are the biggest missing features
3. **Stability**: Replace `.unwrap()` calls to prevent crashes

The estimated time to reach 90% Raycast parity is **2-3 months** of focused development.

---

*This review supplements the existing TODO.md with specific technical findings.*
