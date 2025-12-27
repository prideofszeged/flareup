# Bug Triage Report

Generated: 2025-12-26

## Critical Bugs (Potential Crashes) - FIXED

### 1. ~~Encryption Key Corruption Panic~~ ✅ FIXED
**File:** `src-tauri/src/clipboard_history/encryption.rs:13`
**Issue:** `key_bytes.try_into().unwrap()` will panic if keyring contains corrupted data
**Fix Applied:** Replaced `.unwrap()` with `.map_err()` returning proper error

### 2. ~~Mutex Poison Panic~~ ✅ FIXED
**File:** `src-tauri/src/downloads/mod.rs` (8 instances)
**Issue:** `.expect("downloads manager mutex poisoned")` causes panic on poisoned mutex
**Fix Applied:** Added `lock_manager()` helper function with graceful error handling

## Medium Priority (Code Quality)

### 3. Non-Idiomatic Result Handling
**File:** `src-tauri/src/downloads/mod.rs:153`
**Issue:** `if result.is_err() || !result.unwrap().status.success()`
**Problem:** Confusing control flow, though not technically a bug
**Fix:** Use `match` or `if let` pattern instead

### 4. Future Rust Incompatibility
**Dependency:** `wl-clipboard-rs v0.8.1`
**Issue:** Contains code that will be rejected by future Rust versions
**Action:** Monitor for updates to `wl-clipboard-rs`

## Low Priority (TODOs & Tech Debt)

### TypeScript/Svelte TODOs
| Location | Comment | Priority |
|----------|---------|----------|
| `src/lib/assets.ts:44` | "TODO: better heuristic?" | Low |
| `src/lib/assets.ts:68` | "TODO: better heuristic?" | Low |
| `src/lib/assets.ts:74` | "TODO: actually handle adjustContrast" | Low |
| `src/lib/components/CommandDeeplinkConfirm.svelte:39` | "TODO: implement 'always open'" | Medium |
| `src/lib/components/nodes/shared/actions.ts:8` | "TODO: naming?" | Low |
| `sidecar/src/api/oauth.ts:151` | "TODO: what does this mean?" | Medium |
| `vitest-setup-client.ts:8` | "TODO: better method?" | Low |

### Accessibility Warning
**File:** `src/lib/components/QuickAiView.svelte:211`
**Issue:** noninteractive element with nonnegative tabIndex
**Fix:** Add appropriate ARIA role or use interactive element

### Formatting
- 26 files need Prettier formatting (`pnpm run lint --fix`)

## Verification Status

- **Rust compilation:** Clean (no errors)
- **TypeScript check:** 1 warning (a11y)
- **Linting:** 26 formatting issues (auto-fixable)
- **Git status:** 4 files modified (no content changes - likely line endings)

## Next Steps

1. Fix Critical Bug #1 (encryption.rs) - potential data loss
2. Fix Critical Bug #2 (downloads/mod.rs) - potential crash
3. Run `pnpm run lint --fix` to fix formatting
4. Address Medium priority TODOs
