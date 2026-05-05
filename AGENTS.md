# Synaptic — Agent Notes

A desktop personal knowledge graph app. SvelteKit 5 frontend + Tauri v2 Rust backend. macOS-only releases.

## Commands

- `bun install` — install deps (use bun, not npm)
- `bun run dev` — frontend dev server on port 1420
- `bun run tauri dev` — full desktop app dev mode
- `bun run build` — production frontend build (outputs to `build/`)
- `bun run tauri build` — build desktop app
- `bun run check` — lint and format check with Biome (CI gate)
- `bun run format` — auto-format with Biome
- `bun run release patch|minor|major` — bump version, commit, tag, and push

There is no test suite. `svelte-check` is installed but not wired to a script.

## Architecture

- **Frontend**: `src/` — SvelteKit 2 with Svelte 5 runes (`$state`), Vite, static adapter
  - `src/routes/` — single layout + page (SPA behavior via `fallback: 'index.html'`)
  - `src/lib/store.svelte.ts` — global reactive state using Svelte 5 runes, not legacy stores
  - `src/lib/api.ts` — thin wrappers around Tauri `invoke()` for all Rust commands
- **Backend**: `src-tauri/src/` — Rust Tauri v2 app
  - `main.rs` — command definitions and Tauri builder setup
  - `indexer.rs`, `semantic.rs`, `pty.rs`, `settings.rs`, `watcher.rs` — domain modules
  - `src-tauri/tauri.conf.json` — Tauri config (version must stay in sync with `package.json` and `Cargo.toml`)
- **State**: Frontend state is plain `$state` objects; no Redux, no Svelte stores.

## Tooling Conventions

- **Biome** handles linting and formatting. Do not add ESLint or Prettier.
  - Uses EditorConfig (tabs, LF)
  - Single quotes in JS/TS
  - Svelte files disable `noUnusedVariables` and `noUnusedImports` (props and bindings often appear unused to the linter)
- **TypeScript**: strict mode. `tsconfig.json` extends `.svelte-kit/tsconfig.json`.
- **Imports**: ESM only (`"type": "module"`).

## Release Workflow

1. Run `bun run release <patch|minor|major>` locally. This syncs versions in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`, commits, tags, and pushes.
2. GitHub Actions (`release.yml`) triggers on `v*.*.*` tags, builds a universal macOS `.dmg`, and creates a **draft** release.
3. Manually review and publish the draft release on GitHub. Do not create a release from the tag directly or it will conflict with the workflow draft.

## Runtime Dependencies

The app can launch without these, but core features require them:
- **Neovim** — embedded terminal editing (`which::which` and hardcoded paths searched)
- **Ollama** — semantic search and note title generation (default `http://localhost:11434`)

## Things That Are Easy to Guess Wrong

- This is a **Tauri desktop app**, not a website. All file system, shell, and native access goes through Rust commands exposed via `invoke()`.
- `bun run dev` only starts the Vite frontend. To test Tauri APIs you need `bun run tauri dev`.
- The `playwright` devDependency is present but unused — there are no tests.
- Biome’s linter allows unused variables/imports in `.svelte` files by design; do not "fix" them.
- The version in three files (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`) must always match. Use the release script; do not bump manually.
