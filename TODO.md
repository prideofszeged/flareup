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

## Medium Priority (Code Quality) - FIXED

### 3. ~~Non-Idiomatic Result Handling~~ ✅ FIXED

**File:** `src-tauri/src/downloads/mod.rs:161`
**Issue:** `if result.is_err() || !result.unwrap().status.success()`
**Fix Applied:** Changed to idiomatic `result.map_or(true, |output| !output.status.success())`

### 4. Future Rust Incompatibility

**Dependency:** `wl-clipboard-rs v0.8.1`
**Issue:** Contains code that will be rejected by future Rust versions
**Action:** Monitor for updates to `wl-clipboard-rs`

## Low Priority (TODOs & Tech Debt)

### TypeScript/Svelte TODOs

| Location                                              | Comment                                | Priority |
| ----------------------------------------------------- | -------------------------------------- | -------- |
| `src/lib/assets.ts:44`                                | "TODO: better heuristic?"              | Low      |
| `src/lib/assets.ts:68`                                | "TODO: better heuristic?"              | Low      |
| `src/lib/assets.ts:74`                                | "TODO: actually handle adjustContrast" | Low      |
| `src/lib/components/CommandDeeplinkConfirm.svelte:39` | "TODO: implement 'always open'"        | Medium   |
| `src/lib/components/nodes/shared/actions.ts:8`        | "TODO: naming?"                        | Low      |
| `sidecar/src/api/oauth.ts:151`                        | "TODO: what does this mean?"           | Medium   |
| `vitest-setup-client.ts:8`                            | "TODO: better method?"                 | Low      |

### ~~Accessibility Warning~~ ✅ FIXED

**File:** `src/lib/components/QuickAiView.svelte:211`
**Issue:** noninteractive element with nonnegative tabIndex
**Fix Applied:** Changed to `role="dialog"` with proper ARIA attributes

### ~~Formatting~~ ✅ FIXED

- ~~26 files need Prettier formatting~~ - All files now formatted

## Verification Status

- **Rust compilation:** Clean (no errors)
- **TypeScript check:** Clean (0 errors, 0 warnings)
- **Prettier:** All files formatted
- **ESLint:** 71 errors (pre-existing, not related to this triage)

## Completed

1. ✅ Fix Critical Bug #1 (encryption.rs)
2. ✅ Fix Critical Bug #2 (downloads/mod.rs mutex)
3. ✅ Fix Medium Bug #3 (non-idiomatic Result handling)
4. ✅ Fix Accessibility warning (QuickAiView.svelte)
5. ✅ Run Prettier formatting

## Remaining

- Monitor `wl-clipboard-rs` for updates
- Address Low priority TODOs as needed
- Consider addressing 71 pre-existing ESLint errors (separate effort)
