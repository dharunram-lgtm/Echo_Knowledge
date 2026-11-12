# ECHO Knowledge v1.2.0

Windows 11 x64 release · Tauri 2 + NSIS

## Highlights

- **Critical Document Deletion Safety:** Fixed a data loss bug where clicking "Cancel" in the WebView confirmation still executed document deletion.
- **Unified Fluent Confirmation Modal:** Replaced native browser `window.confirm()` with `DeleteConfirmModal`, an accessible Fluent Design dialog featuring explicit destructive action warnings.
- **Guaranteed Cancellation Safety:** 
  - Clicking **Cancel** closes the dialog with zero destructive operations.
  - Pressing the **Escape** key dismisses the dialog safely without deleting.
  - Clicking the **Close (X)** button cancels without deleting.
  - Clicking the **modal backdrop** dismisses without deleting.
- **Focus Trap & Keyboard Accessibility:** Safe default focus on the Cancel button on open, with automatic focus restoration to the delete trigger button on dismissal.
- **In-Flight Guarding:** Prevents double submissions and concurrent clicks; displays loading spinner (`Deleting…`) while deletion is in flight.
- **Consistent Workflow Across Views:** Unified deletion safety behavior across both **My Knowledge** and **Collections**, while keeping collection document removal strictly separate from permanent deletion.
- **Documentation & Discoverability:** Added comprehensive feature, privacy, and FAQ guides with embedded application screenshots.

## Windows Installer

- **File:** `ECHO_Knowledge_Windows11_v1.2.0_x64-setup.exe`
- **Canonical NSIS Output:** `ECHO Knowledge_1.2.0_x64-setup.exe`
- **Release Version:** 1.2.0
- **Size:** 4,088,666 bytes (3.90 MiB)
- **SHA-256 Checksum:** `B31E90B3C6E82FD4038E0C4784657B014478F8836EEE122BBC8EC3DD8991CAEF`

## Privacy and Storage

All user documents, SQLite databases, and embeddings remain strictly local under `%APPDATA%\com.echo.desktop\ECHO\`. API keys are stored in Windows Credential Manager under `echo.knowledge.ai`. No telemetry or developer intermediary servers are used.

## Validation and Environment Notes

- Automated Invariant Tests: 10/10 passed (`node --test src/components/documents/__tests__/deleteSafety.test.mjs`).
- TypeScript Type Check: PASS (`tsc --noEmit`).
- Production Frontend Build: PASS (`vite build`).
- Rust Cargo Check: PASS (`cargo check`).
- Rust Cargo Unit Tests: Known Windows MSVC environment issue (`0xc0000139`, `STATUS_ENTRYPOINT_NOT_FOUND`) prevents `cargo test` execution; release binary was verified via build diagnostics and packaging pipelines.
