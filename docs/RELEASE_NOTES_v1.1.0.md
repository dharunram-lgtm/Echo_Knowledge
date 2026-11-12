# ECHO Knowledge v1.1.0

Windows 11 x64 release · Tauri 2 + NSIS

## Highlights

- **Persistent Collections:** Create, edit, and delete custom document collections with dedicated color tags and persistent document counts.
- **In-Collection Preview & Management:** Preview full document text inside collections and safely remove documents from collections without deleting library files.
- **Desktop Experience & Branding:** Native application icon replacing default Tauri branding across all Windows assets (taskbar, Start Menu, window titlebar, and installer).
- **Direct API Provider Links:** Settings page features working external documentation and console links for Google AI Studio and Groq.
- **SQLite Migration v9:** Versioned database migration maintaining backward compatibility and schema integrity.

## Windows Installer

- **File:** `ECHO_Knowledge_Windows11_v1.1.0_x64-setup.exe`
- **Release:** v1.1.0
- **SHA-256 Checksum:** `3A1F9EF9DFF49A740FB9120B905F6BA6A67E9543E70F418FBD94B2B6D32F8C09`

## Privacy and Storage

All user documents, SQLite databases, and embeddings remain strictly local under `%APPDATA%\com.echo.desktop\ECHO\`. API keys are stored in Windows Credential Manager under `echo.knowledge.ai`. No telemetry or developer intermediary servers are used.
