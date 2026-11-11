# ECHO Knowledge v1.0.0

Windows 11 x64 release · Tauri 2 + NSIS

## Highlights

- Local SQLite persistence for documents, chunks, embeddings, conversations, tags, concepts, and knowledge-graph state.
- PDF, Markdown, and plain-text import into a managed local document vault.
- Local filename/text search and document lifecycle management.
- Optional Gemini and Groq integrations for embeddings, grounded Ask ECHO answers, tagging, and relationship discovery.
- API keys stored in Windows Credential Manager through the Rust backend.
- Knowledge graph exploration and persisted graph layout state.
- Current-user NSIS installer with ECHO Knowledge Start Menu shortcut and registered uninstaller.

## Privacy and storage

Personal documents, SQLite databases, API keys, credentials, and user vault content are not bundled in the installer. User data is stored under `%APPDATA%\com.echo.desktop\ECHO\`; uninstalling the application does not intentionally remove that directory.

Cloud-provider requests are direct from the desktop to the provider selected by the user. ECHO does not provide a developer-owned backend or telemetry service.

## Known limitations

- Scanned/image-only PDFs are not OCR-processed.
- Cloud AI features require the user's own Gemini and/or Groq credentials and are subject to provider availability, quotas, and rate limits.
- WebView2 bootstrapper installation requires internet access if WebView2 is not already present.
- Clean-machine installation, uninstall, and reinstall validation are environment-dependent; see the release audit report supplied with this build.

## Artifact

- Installer: `ECHO_Knowledge_Windows11_v1.0.0_x64-setup.exe`
- Size: 4,068,201 bytes (3.88 MiB)
- SHA-256: `6B653A1D269C79E596737462C0F6843B17F07184913AD50ACA3EEABD61ED9E77`
