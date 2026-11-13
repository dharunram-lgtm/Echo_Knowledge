# ECHO Knowledge v1.2.1

Windows 11 x64 release · Tauri 2 + NSIS

## Highlights

- **Knowledge Graph Orphan Concept & Edge Cleanup:** Fixed a data consistency issue where permanently deleting a document from My Knowledge or Collections left orphaned concept nodes (showing 0 referenced documents and 0 connected concepts) and dangling relationships in the Knowledge Graph.
- **Transactional Document Deletion:** Deletion of document metadata, chunks, embeddings, tags, associations, and exclusive concepts/relationships is now executed atomically inside a single SQLite transaction. If any step fails, changes are completely rolled back.
- **Safe Preservation of Shared Concepts:** Concepts referenced by more than one document are strictly preserved when any single source document is deleted.
- **Preservation of User-Defined Concepts:** Manually created concepts (`description = 'User-defined concept'`) are retained independently of document references.
- **Collection Non-Destructive Membership Safety:** Removing a document from a Collection does not delete the document or purge its concepts. Only permanent deletion triggers graph cleanup.
- **Dynamic Real-Time Graph Refresh:** Deletion now emits `relationships:changed` and `tagging:status_changed` Tauri events, prompting the Knowledge Graph view and Inspector to immediately reload and purge deleted nodes and edges without restarting.
- **Database Migration v10 (Automatic Orphan Sweep):** Automatically and safely sweeps legacy orphaned concepts and dangling relationships left behind by deletions performed prior to v1.2.1, ensuring existing databases are brought to full consistency upon launch while keeping user-defined concepts intact.

## Regression Test Suite

All 55 automated verification tests passed:
- Single-source concept deletion orphan cleanup.
- Shared concept preservation across multiple source documents.
- Orphaned relationship removal upon concept endpoint deletion.
- Collection membership removal non-destructive integrity.
- Migration v10 pre-existing orphan sweep and idempotence verification.

## Windows Installer

- **File:** `ECHO_Knowledge_Windows11_v1.2.1_x64-setup.exe`
- **Canonical NSIS Output:** `ECHO Knowledge_1.2.1_x64-setup.exe`
- **Release Version:** 1.2.1
- **Size:** 4,089,989 bytes (3.90 MiB)
- **SHA-256 Checksum:** `5F9D493669ABCA46CE8C60FA88686693785C550D69CA1C66ADBC78AE1EF1A64C`

## Privacy and Storage

All user documents, SQLite databases, and embeddings remain strictly local under `%APPDATA%\com.echo.desktop\ECHO\`. API keys are stored in Windows Credential Manager under `echo.knowledge.ai`. No telemetry or external server tracking is used.

## Validation Results

- Automated Invariant & Database Verification: 55/55 passed (`cargo run --bin test_runner --features="test-runner"`).
- TypeScript Type Check: PASS (`tsc --noEmit`).
- Production Frontend Build: PASS (`vite build`).
- Rust Cargo Check: PASS (`cargo check`).
- Git Diff Check: PASS (`git diff --check`).
