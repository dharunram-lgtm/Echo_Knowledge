# ECHO Knowledge — Privacy & Security Architecture

ECHO Knowledge is designed under a **local-first** security model. This document details how user data, documents, database records, and credentials are handled.

---

## 1. Local-First Data Storage

All user documents, extracted text, embeddings, chat history, and metadata reside entirely on the user's local filesystem.

### Storage Directory Locations
All user data is stored within the user profile directory outside the application binaries:

```text
%APPDATA%\com.echo.desktop\ECHO\
├── database\
│   └── echo.db           # SQLite database (documents, chunks, embeddings, tags, graph)
└── documents\
    └── imported\         # Local managed document vault (imported file copies)
```

- **SQLite Database (`echo.db`):** Uses SQLite with Write-Ahead Logging (WAL) for transactional integrity.
- **Document Vault (`imported/`):** When documents are imported, a copy is stored in this directory indexed by a deterministic SHA-256 content hash.
- **No Cloud Synchronization:** ECHO Knowledge does not synchronize, mirror, or upload your document vault or database to any developer-owned cloud storage.

---

## 2. API Key Management & Credential Isolation

ECHO Knowledge uses a **Bring Your Own Key (BYOK)** model for optional AI services (Google Gemini and Groq).

### Storage Mechanism
- **Windows Credential Manager / DPAPI:** API keys are stored in the native Windows Credential Manager under the `echo.knowledge.ai` service namespace using DPAPI encryption.
- **No Frontend Exposure:** The React frontend never stores API keys in `localStorage`, cookies, or JavaScript variables. The frontend only receives status indicators (e.g., `{ configured: true, provider: "gemini" }`).
- **No Plaintext Persistence:** Keys are not written to SQLite database tables or configuration JSON files.

---

## 3. Network Communication & Data Transmission

### Local Offline Capabilities
When operating without an AI provider configured:
- Document import, file extraction, local text search, collection organization, and reading workflows operate completely offline.
- No outbound network connections are initiated for local file operations.

### Cloud Provider Communication
When using AI-assisted features (semantic embeddings, Ask ECHO grounded answers, automated tagging):
- **Direct HTTPS Requests:** Requests are sent directly from the local application to the user's selected provider:
  - Google Gemini: `https://generativelanguage.googleapis.com`
  - Groq: `https://api.groq.com`
- **Data Transmitted:** Only relevant document chunks required to answer a specific query or generate embeddings are transmitted over TLS/HTTPS.
- **No Intermediary Proxies:** Requests do not pass through any proxy server operated by the ECHO development team.
- **Provider Policies:** Data sent to Gemini or Groq is governed by the respective provider's privacy policy and terms of service. Review their data retention settings in your provider dashboard.

---

## 4. Telemetry and Analytics

- **Zero Telemetry:** ECHO Knowledge contains **no** telemetry tracking, usage analytics, crash reporting pings, or background usage beacons.
- **No Third-Party Trackers:** The frontend bundle contains no analytics SDKs (no Google Analytics, PostHog, Mixpanel, or Sentry).
- **No User Account Requirement:** Using ECHO Knowledge requires no account registration, email address, or license validation server check.

---

## 5. Tauri 2 Desktop Sandboxing

ECHO Knowledge enforces strict capability controls via Tauri 2 permission policies (`src-tauri/capabilities/default.json`):
- **Denied Capabilities:** Arbitrary process execution, shell access, and broad filesystem access outside the designated app directories are denied.
- **Scoped Capabilities:** Only scoped file picker dialogs, system shell opener for external documentation links, and designated application storage commands are permitted.

---

## 6. Uninstallation and Data Retention

- **Application Removal:** Running the uninstaller removes the executable, Start Menu shortcuts, and application binaries from the user directory.
- **Vault Preservation:** By default, standard uninstallers do not delete the `%APPDATA%\com.echo.desktop\ECHO\` folder. This prevents accidental loss of personal documents and databases during version updates.
- **Complete Data Deletion:** To permanently remove all data, users can manually delete the `%APPDATA%\com.echo.desktop\ECHO\` directory and clear credentials under Windows Credential Manager (`echo.knowledge.ai`).
