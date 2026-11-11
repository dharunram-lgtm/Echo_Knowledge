# ECHO Knowledge v1.0.0 release audit

## Release configuration

- Product: ECHO Knowledge
- Version: 1.0.0
- Target: Windows x64
- Tauri identifier: `com.echo.desktop` (preserved for existing-user data compatibility)
- Bundle: NSIS, current-user install mode
- Start Menu folder: `ECHO Knowledge`
- WebView2: silent download bootstrapper when the runtime is missing
- Icon: `src-tauri/icons/icon.ico` plus the configured multi-resolution icon set

## Storage and privacy audit

- SQLite initializes under `%APPDATA%\com.echo.desktop\ECHO\database\echo.db` with WAL mode and schema migrations through version 8.
- Imported PDFs, Markdown, and text files are copied to `%APPDATA%\com.echo.desktop\ECHO\documents\imported\`.
- API keys are handled by the Rust backend through `keyring` with the Windows-native backend and the `echo.knowledge.ai` service namespace.
- Gemini and Groq requests use direct HTTPS provider endpoints from Rust; no developer-owned backend is bundled.
- The installer payload was checked for the developer test runner, sample documents, SQLite filenames, credentials, and secret-like filenames; none were present in the final artifact.
- Personal documents, user SQLite databases, API keys, credentials, and vault contents are not bundled.

## Build results

| Check | Result |
|---|---|
| `npx tsc --noEmit` | PASS |
| `npm run build` | PASS; existing chunk-size/dynamic-import warnings only |
| `cargo check --manifest-path src-tauri/Cargo.toml` | PASS |
| `cargo check --manifest-path src-tauri/Cargo.toml --features test-runner --bin test_runner` | PASS |
| `npm run tauri build -- --bundles nsis` | PASS |

## Installer artifact

- Path: `src-tauri\target\release\bundle\nsis\ECHO_Knowledge_Windows11_v1.0.0_x64-setup.exe`
- Size: 4,068,201 bytes (3.88 MiB)
- SHA-256: `6B653A1D269C79E596737462C0F6843B17F07184913AD50ACA3EEABD61ED9E77`

## Installation tests

- Silent install to a workspace-local test directory: PASS, exit code 0.
- Installed payload: `echo.exe`, `uninstall.exe`, `WebView2Loader.dll`; no `test_runner.exe`: PASS.
- Silent uninstall: PASS, exit code 0.
- User database remained after uninstall: PASS; `echo.db` remained under `%APPDATA%\com.echo.desktop\ECHO\database\`.
- Installed GUI launch: NOT VERIFIED; the process returned Windows exit code `-1073740791` in the current automation session.
- SQLite, PDF/TXT/Markdown import, search, Ask ECHO, provider switching, graph rendering, restart persistence, and reinstallation access were not interactively verified in a separate Windows user or clean Windows 11 environment.
- The Rust verification runner passed schema initialization, TXT/Markdown/PDF import, duplicate detection, search, preview, and SQLite persistence. Its Windows Credential Manager test could not complete in this automation session because Windows returned `ERROR_NO_SUCH_LOGON_SESSION`; this is recorded as unverified rather than a product pass.

## Release blockers

1. Interactive launch and end-to-end desktop feature testing still require a usable Windows desktop session or a separate clean Windows 11 account/environment.
2. Gemini/Groq live API behavior remains untested without user-supplied credentials and network access.
3. Do not create tag `v1.0.0` or publish the GitHub release until these results are reviewed and approved.
