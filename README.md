# Ticketing Platform — Desktop

Offline-first ticketing & invoicing desktop application. **Tauri 2 + React 18 +
TypeScript** frontend with a **Rust + SQLite** local core. This repository is
the production foundation: a clean, layered architecture meant to be extended,
not a finished product.

> The `ticketing-desktop` project (sibling repo) is the **reference/demo** for
> the workflow and UI. This project re-establishes the same capabilities on a
> production-grade, layered base.

## Stack (pinned for stable desktop builds)

| Layer | Choice | Why |
|-------|--------|-----|
| Frontend | React **18.3**, TypeScript **5.6**, Vite **5** | Battle-tested, broad ecosystem compatibility, reliable EXE builds |
| Desktop shell | Tauri **2** | Small self-contained installers, native OS access |
| Local DB | **SQLite** via `sqlx` (migrations) | Zero-install, embedded, file-based |
| Logging | `tracing` | Structured, level-controlled via env |

## Architecture

The Rust core uses strict layering; dependencies point **inward**:

```
commands  (Tauri handlers, transport only)   src-tauri/src/commands/
   │  delegates to
service   (use-cases / orchestration)         src-tauri/src/service/
   │  uses
repository(all SQL lives here)                src-tauri/src/repository/
   │  over
db        (pool, app-data path, migrations)   src-tauri/src/db/
domain    (entities + pure business rules)    src-tauri/src/domain/
config / error                                cross-cutting
```

- **No SQL outside `repository/`.** No business rules outside `domain/`/`service/`.
  Commands are thin and convert errors at the edge.
- The **Ticket API** the UI consumes is `src/api/tickets.ts`, which maps 1:1 to
  the `ticket_*` Tauri commands — the single frontend↔core boundary.
- The local SQLite file is created automatically in the OS app-data directory on
  first launch and migrated from `src-tauri/migrations/`.

### Frontend layout

```
src/
  api/        Ticket API client (invoke wrappers)  ← ready, not yet rendered
  components/ shared UI building blocks            ← empty until designs land
  lib/        shared types + invoke helper
  App.tsx     placeholder only
```

> **Frontend is intentionally a placeholder.** UI work is deferred until the
> Figma designs are available. The API client + types exist so screens can be
> wired up quickly, but no components are built or rendered yet. Current focus is
> the `src-tauri` backend (CRUD + core).

## Prerequisites

- Node ≥ 18, Rust ≥ 1.77
- Linux build deps: `libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libayatana-appindicator3-dev build-essential file`
- Windows: WebView2 (preinstalled on Win 10/11) + MSVC build tools

## Develop

```bash
npm install
cp .env.example .env   # optional; defaults work
npm run tauri:dev
```

## Build installers

```bash
npm run tauri:build
```

Output: `src-tauri/target/release/bundle/` (`.msi`/`-setup.exe` on Windows,
`.AppImage`/`.deb` on Linux). To build the Windows installer without a Windows
machine, push to GitHub and run the **Build installers** workflow (Actions tab).

## Tests

```bash
cd src-tauri && cargo test    # domain rules unit-tested (no DB needed)
```

## Configuration

Read at startup from env (see `.env.example`): `CLOUD_API_URL`, `APP_LOG_LEVEL`,
`APP_DB_FILENAME`. All have safe defaults.

## Status

Foundation only — schema, layered core, ticket CRUD + validation, reference UI,
CI. Advanced features (auth, invoicing UI, cloud sync, backups) build on top of
this structure in later phases.
