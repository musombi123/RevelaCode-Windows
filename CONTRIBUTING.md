# Contributing to RevelaCode Windows

Thank you for considering a contribution. Keep changes focused, verifiable, secure, and consistent with the existing architecture.

## Before starting

1. Read the repository README and relevant architecture/security documents.
2. Search existing issues and pull requests before opening a duplicate.
3. For substantial changes, propose the problem, solution, scope, and trade-offs in an issue or discussion first.
4. Get maintainer agreement before changing APIs, data ownership, security boundaries, persistence, licensing, or release behaviour.

## Development workflow

1. Create a feature branch from the current default branch.
2. Keep each pull request focused on one logical change.
3. Follow existing language, formatting, naming, and project structure.
4. Add or update tests for behaviour changes and regression fixes.
5. Update documentation, configuration examples, and migration notes.
6. Run relevant tests, linting, type checks, and builds where available.
7. Open a pull request with motivation, implementation notes, test evidence, risks, and screenshots/logs where useful.

## Engineering and security

- Never commit API keys, passwords, tokens, signing keys, private user data, `.env` files, or production database exports.
- Never weaken authentication, authorisation, tenant isolation, validation, rate limits, or audit logging to make a test pass.
- Preserve API contracts unless a reviewed migration is included.
- Avoid unrelated refactors and generated build artefacts unless explicitly tracked.
- Explain new dependencies and check maintenance, security, and licence compatibility.
- Submit only code and assets you have the right to contribute; include attribution and licence details.

## Project-specific constraints

Keep this repository as the Tauri integration/packaging host. Do not fork companion frontend/backend/AI source or create a second backend/database. Preserve local-data and offline-sync safety contracts.

## Pull request checklist

- [ ] Change is focused and solves the stated problem.
- [ ] Tests or verification steps are included and have been run.
- [ ] Documentation and examples are updated.
- [ ] No secrets, private data, or unlicensed assets are included.
- [ ] Compatibility, migrations, and security implications are documented.
- [ ] Known limitations and follow-up work are stated.

## Review

All contributions require maintainer review. A submitted pull request is not accepted until merged. By submitting, you confirm you have the right to contribute the work under the repository's applicable licence and have identified third-party material.
