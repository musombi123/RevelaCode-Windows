# Desktop architecture and integration contract

## Runtime model

The Windows application has one native host and one packaged React application:

- **Tauri 2 / Rust:** window lifecycle, native commands and the local data boundary.
- **Existing React/Vite frontend:** the user interface, built from the companion frontend repository at the selected revision.
- **SQLite:** per-user local workspace records, stored outside the installation directory.
- **Cloud APIs:** existing RevelaCode Backend and RevelaAI deployments remain authoritative for server-owned data and operations.

The Backend and RevelaAI repositories are checked out and revision-recorded by CI, but are not currently compiled into local Windows sidecars. Their local packaging is a later milestone requiring explicit runtime, configuration, health-check and shutdown contracts.

## Source-of-truth repositories

| Repository | Responsibility | Current Windows build use |
|---|---|---|
| RevelaCode-Frontend | React/Vite product UI | Built and bundled into Tauri |
| RevelaCode-Backend | Flask API, authentication, MongoDB access, Jumuiya | Checked out and revision-recorded; cloud API remains active |
| RevelaAI | FastAPI gateway, Go-based USECT services and portable tools | Checked out and revision-recorded; cloud AI remains active |

No desktop code may connect directly to MongoDB Atlas. All cloud writes must pass through authenticated backend APIs.

## Local workspace database

The first native storage boundary is SQLite in the current user's application data folder. The initial schema is intentionally small:

```sql
workspace_records(
  scope       TEXT NOT NULL,
  record_key  TEXT NOT NULL,
  value_json  TEXT NOT NULL,
  updated_at  TEXT NOT NULL,
  PRIMARY KEY(scope, record_key)
)
```

Records must be scoped to an authenticated user or an explicitly identified guest workspace. The initial native API supports put, get, list and delete operations. It is not yet a replacement for the frontend's existing IndexedDB cache or offline mutation queue.

Do not store passwords, JWTs, API keys, refresh tokens, payment credentials or other secrets in this table. A dedicated Windows Credential Manager/keyring integration is required before persistent desktop credentials are introduced.

## Synchronisation model

The intended next stage is an authenticated, idempotent sync protocol:

1. A permitted hub action is saved locally with a stable mutation ID, owner scope, entity type, entity ID, base version and payload.
2. UI reports the change as **Saved on this device**, not as cloud-confirmed.
3. On reconnection, the desktop sends changes to a dedicated authenticated sync API on RevelaCode Backend.
4. Backend validates identity, tenant ownership, permissions, allowed operation and idempotency before writing to the existing MongoDB collections.
5. Backend returns an acknowledged version and any server changes.
6. Desktop updates the local workspace and marks acknowledged mutations complete.
7. Conflicts are surfaced for resolution; last-write-wins must not silently discard business or school records.

Never automatically replay payment initiation, payment confirmation, payroll finalisation, invoices, refunds, role changes, approvals, moderation decisions or administrative actions. Such actions require live server authority and explicit confirmation.

## Offline capability levels

**Available from this foundation**
- Native desktop shell.
- Local SQLite workspace commands.
- Existing frontend bundled inside the desktop shell.
- Existing frontend caching behaviour, where it already works.

**Not yet implemented**
- Migration of existing hub caches and drafts into the SQLite API.
- Per-hub offline data models and sync conflict resolution.
- Local Flask backend sidecar and MongoDB-independent service mode.
- Local FastAPI AI gateway and provider/model runtime.
- Bundled ZERO, Genesis and Infinite Mind daemons managed by the desktop host.
- Local language model and offline inference runtime.
- Offline account provisioning and secure token storage.

An offline installer is not the same as an entirely offline ecosystem. Cloud services, MongoDB Atlas, M-Pesa, email, live weather, cloud media and cloud AI still need connectivity.

## Local service lifecycle (future)

When local backend and AI sidecars are introduced, the Rust host must:
- start only signed/version-matched executables from the installation resources;
- bind local-only services to loopback, never 0.0.0.0;
- allocate or validate ports and verify health before exposing capabilities;
- generate short-lived per-launch service credentials;
- avoid inheriting arbitrary host environment variables or cloud secrets;
- redact credentials and personal data from logs;
- shut down child processes cleanly on app exit;
- fail closed when any required service is unhealthy.

No local MongoDB server is planned. SQLite is a device workspace/cache; MongoDB Atlas remains the central cloud database.

## Build provenance

The Windows workflow checks out each companion repository at an explicit branch, tag or commit input. It writes the resolved Git SHAs to `artifacts/source-revisions.json` and uploads that manifest with the installer. Release automation must use a tagged, reviewed build rather than silently tracking mutable branches in production.
