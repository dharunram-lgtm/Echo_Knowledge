# ECHO — Architecture & System Design

This document details the architectural boundaries, system layers, communication protocols, and security model of **ECHO: Your Personal AI Knowledge Operating System**.

---

## 1. High-Level Architecture

ECHO is architected around a dual-layer desktop paradigm:

```
┌─────────────────────────────────────────────────────────────┐
│                    REACT FRONTEND (UI)                      │
│                                                             │
│   Pages (Dashboard, MyKnowledge, AskEcho, Graph, Settings)  │
│                                │                            │
│   Components (AppShell, TitleBar, Sidebar, StatusBar)       │
│                                │                            │
│   Features & Services (Typed Contracts, State Stores)       │
└──────────────────────────────┬──────────────────────────────┘
                               │
                Tauri 2 IPC Bridge (JSON-RPC)
               [Restrictive Capabilities Schema]
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                    RUST BACKEND (DESKTOP)                   │
│                                                             │
│   Commands Layer (System, Ingestion, Query Handlers)        │
│                                │                            │
│   Security & Keyring (Windows DPAPI / Credential Vault)     │
│                                │                            │
│   Local Persistence (SQLite, Vector Embeddings, Documents)  │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Separation of Responsibilities

### Frontend (React + TypeScript + Tailwind)
- **Role:** Presentation, user interaction, reactive state, visual feedback, and view routing.
- **Strict Boundary:** The frontend contains **no API keys**, **no direct filesystem handles**, and **no ambient external network requests**.
- All operations requiring file access, secrets, or heavy background computation are delegated through typed IPC commands to the Rust backend.

### Backend (Rust + Tauri 2)
- **Role:** System-level capabilities, secure secret storage, local database transactions, background document indexing, and AI client communication.
- **Safety Guarantee:** Rust's memory safety and Tauri 2's granular capability permissions guarantee that the desktop web view cannot be exploited to access unauthorized host system files.

---

## 3. Security & Privacy Model

### 3.1 Zero-Knowledge / Local-First Boundary
- Documents imported into ECHO never leave the user's workstation without explicit consent.
- All embedding generation and vector search will prioritize local execution (via local embedding models or optional cloud provider endpoints explicitly selected by the user).

### 3.2 Keyring & Secret Isolation
- When users configure BYO API keys (e.g. OpenAI, Anthropic, Gemini), the credentials are transmitted over encrypted memory channels to the Rust backend.
- Keys are persisted in the **Windows Credential Manager** via DPAPI-protected APIs.
- Frontend JavaScript state only receives metadata indicators (e.g. `{ configured: true, provider: "openai" }`), never raw secret strings.

### 3.3 Restricted Tauri 2 Capabilities
- ECHO's default capability configuration (`src-tauri/capabilities/default.json`) explicitly restricts frontend permissions:
  - `core:default`: Basic window lifecycle and event messaging.
  - `opener:default`: Safe opening of explicit URLs in default system browser.
  - Arbitrary filesystem read/write is **denied**.
  - Arbitrary process execution is **denied**.

---

## 4. Design System Architecture

ECHO adopts an authentic **Windows 11 Fluent Design Language**:

1. **Charcoal Surfaces:** Dark theme uses authentic dark gray and charcoal surfaces (`#181819`, `#202022`, `#27272a`) avoiding high-contrast pure black and unnatural purple tints.
2. **Typography:** Styled with Microsoft's `Segoe UI Variable Text` and `Segoe UI Variable Display` font stacks for native legibility.
3. **Subtle Outlines & Corners:** Subtle border contrasts (`rgba(255,255,255,0.08)`) with standard 4px/6px/8px Windows 11 border radii.
4. **Theme Reactivity:** Supports instant switching between Light, Dark, and System Default themes without full-page reloads.

---

## 5. IPC Command Interface

### System Commands
| Command | Arguments | Return Type | Purpose |
|---|---|---|---|
| `get_app_info` | None | `AppInfo` | Retrieves application name, package version, active phase, and host OS platform. |

### Document Management & Storage Commands (Phase 2)
| Command | Arguments | Return Type | Purpose |
|---|---|---|---|
| `initialize_storage` | None | `StorageInitResponse` | Verifies and initializes local directory structures and SQLite schema. |
| `import_documents` | `file_paths: Vec<String>` | `ImportBatchResult` | Validates, hashes (SHA-256), deduplicates, copies files to managed storage, extracts text (.pdf, .md, .txt), and saves metadata. |
| `get_documents` | None | `Vec<DocumentSummary>` | Retrieves all document summaries for lightweight UI listings. |
| `get_document` | `id: String` | `DocumentRecord` | Retrieves complete document details including full extracted text. |
| `search_documents` | `query: String` | `Vec<DocumentSearchResult>` | Searches filename and extracted text with match categorization and snippets. |
| `delete_document` | `id: String` | `bool` | Safely removes document record from SQLite and managed file from local disk. |
| `get_storage_stats` | None | `StorageStats` | Aggregates document counts, total storage consumed, and breakdown by file type. |

---

## 6. SQLite Schema & Migration Architecture

Local persistence is managed using `rusqlite` in SQLite WAL (Write-Ahead Logging) mode.

- **Location:** `%APPDATA%/com.echo.desktop/ECHO/database/echo.db`
- **Managed Document Vault:** `%APPDATA%/com.echo.desktop/ECHO/documents/imported/`

```sql
CREATE TABLE IF NOT EXISTS documents (
    id TEXT PRIMARY KEY NOT NULL,
    filename TEXT NOT NULL,
    original_path TEXT NOT NULL,
    stored_path TEXT NOT NULL,
    file_type TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    extracted_text TEXT NOT NULL,
    content_hash TEXT NOT NULL UNIQUE,
    imported_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_documents_content_hash ON documents(content_hash);
CREATE INDEX IF NOT EXISTS idx_documents_filename ON documents(filename);
CREATE INDEX IF NOT EXISTS idx_documents_file_type ON documents(file_type);
CREATE INDEX IF NOT EXISTS idx_documents_imported_at ON documents(imported_at);
```

