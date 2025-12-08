# Repository Guidelines

## Project Structure & Module Organization
The SvelteKit client lives in `src/` (`lib/` for UI primitives, `routes/` for pages) and reads static assets from `static/` plus marketing media in `images/`. Tauri-native code, Rust commands, and the SoulverCore wrapper are under `src-tauri/`, while the Node sidecar that executes Raycast extensions resides in `sidecar/` (build that workspace before packaging). Shared IPC contracts sit in `packages/protocol`.

## Build, Test, and Development Commands
- `pnpm dev`: run the Vite/SvelteKit client with hot reload.
- `pnpm tauri dev`: launch the full desktop shell (set `LD_LIBRARY_PATH` to `src-tauri/SoulverWrapper/...` before calling).
- `pnpm --filter sidecar build`: rebuild the Node plugin host binary consumed by Tauri.
- `swift build -c release --package-path src-tauri/SoulverWrapper`: compile the SoulverCore bridge.
- `pnpm build`: produce the production web bundle consumed by Tauri.
- `pnpm lint` / `pnpm format`: enforce ESLint + Prettier rules.

## Coding Style & Naming Conventions
Prettier is source of truth (`pnpm format`); it enforces tabs, 100-character lines, single quotes, and Tailwind-aware sorting. Favor TypeScript everywhere, keep Svelte component names in `PascalCase.svelte`, stores/utilities in `camelCase.ts`, and derived constants in `UPPER_SNAKE` only when immutable. Use the ESLint config (Svelte + TypeScript + import rules) to catch side effects and unused stores.

## Testing Guidelines
Vitest with `@testing-library/svelte` backs unit tests (`pnpm test` or `pnpm test:unit --run`). Mirror component filenames (`CommandPalette.svelte` → `CommandPalette.svelte.test.ts`) inside `src/lib`. Prefer user-level assertions (`screen.getByRole`) over implementation details and add regression tests whenever frecency logic or keyboard scopes change.

## Commit & Pull Request Guidelines
Commits follow conventional prefixes observed in history (`feat:`, `refactor:`, `fix:`). Keep messages in the imperative and scoped to a single concern (e.g., `feat: add snippet argument focus trap`). Pull requests should describe user-visible changes, list manual test steps, link tracking issues, and attach screenshots or GIFs for UI changes.

## Security & Configuration Notes
Recorder features hook into keyboard devices; remind testers to install the `99-flare.rules` udev entry before validating snippets. Never commit secrets—use `.env` entries loaded by Tauri, and prefer system keyrings for tokens such as OpenRouter keys.
