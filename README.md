# RevelaCode Windows

**RevelaCode Desktop** is the Windows integration host for the existing RevelaCode ecosystem.

It packages the existing React/Vite frontend inside a native Tauri 2 desktop application. The build checks out the separately maintained RevelaCode Frontend, Backend and RevelaAI repositories at explicit refs. Their histories and deployment responsibilities remain intact.

## Current milestone

This repository establishes the desktop host and release pipeline. It includes:

- Tauri 2 Windows desktop shell.
- Existing React/Vite frontend built from \`musombi123/RevelaCode-Frontend\`.
- A local SQLite workspace store exposed through Tauri commands.
- Windows NSIS installer configuration with the WebView2 offline installer.
- A Windows GitHub Actions build that checks out and records the exact source revisions of all three companion repositories.
- A bootstrap script for local Windows development.

**Important:** this foundation does not yet bundle the Flask backend, FastAPI AI gateway or USECT Go daemons as local Windows sidecars. The existing frontend still uses its configured cloud APIs for online operations. The SQLite commands are available for the next frontend integration step; the existing hub screens have not yet been migrated to this store. Offline operation must be implemented and tested module by module.

## Architecture

\`\`\`text
RevelaCode-Windows
├── Tauri 2 host (Rust)
│   ├── Native window and desktop lifecycle
│   └── SQLite local workspace commands
├── workspace/frontend (build-time checkout)
│   └── Existing React/Vite application
├── workspace/backend (build-time checkout)
│   └── Existing Flask backend source
└── workspace/revelaai (build-time checkout)
    └── Existing FastAPI gateway and USECT core source
\`\`\`

The Windows host is an integration and packaging layer. It does not copy, fork or replace the source-of-truth repositories.

## Requirements for local development

- Windows 10/11 x64.
- Git.
- Node.js 22.
- Rust stable with the MSVC toolchain.
- Microsoft C++ Build Tools / Visual Studio Build Tools.
- WebView2 Runtime for development. The distributed NSIS installer embeds the WebView2 offline installer.

## Bootstrap

Run PowerShell from the repository root:

\`\`\`powershell
Set-ExecutionPolicy -Scope Process Bypass
.\\scripts\\bootstrap.ps1
npm run tauri dev
\`\`\`

The bootstrap script clones or updates the three companion repositories under \`workspace/\`.

To build a Windows installer locally:

\`\`\`powershell
npm run tauri build
\`\`\`

The resulting NSIS installer is generated under:

\`\`\`text
src-tauri/target/release/bundle/nsis/
\`\`\`

## CI builds

GitHub Actions builds on Windows. A manual run accepts independent refs for the three source repositories. The workflow records their resolved Git commit SHAs in the build artifact. Release tags matching \`v*\` publish the generated setup executable as a GitHub Release asset.

## Local data

The desktop host stores its SQLite database in the operating system's per-user application data directory, not inside the installation directory. Application updates must not delete this data.

Do not store passwords, JWTs, API keys, payment credentials or other secrets in the general workspace table. A separate operating-system credential-store integration is required before desktop token persistence is enabled.

## Offline and synchronisation contract

- Local records must be written to device storage before the UI reports them as saved.
- Offline writes must carry stable IDs and explicit ownership/tenant scope.
- Synchronisation must go through authenticated backend APIs; the desktop must never connect directly to MongoDB Atlas.
- Cloud validation and server-side permissions remain authoritative.
- Payments, payroll finalisation, access changes, approvals and other sensitive actions must not be replayed automatically from an offline queue.
- Conflicting edits must be detected and surfaced; do not silently overwrite another device's changes.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the integration roadmap.
