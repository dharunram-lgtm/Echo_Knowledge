# ECHO Knowledge — Feature Reference

This document provides a factual, detailed guide to the features implemented in **ECHO Knowledge v1.1.0**.

ECHO Knowledge is a local-first desktop application designed for Windows 11 that organizes documents, executes semantic searches, synthesizes answers using user-provided AI models, and maps conceptual relationships across documents.

---

## 1. Local Document Management & Ingestion

### Supported Formats
ECHO Knowledge natively processes three standard document formats:
- **PDF documents (`.pdf`)**: Extracts readable text streams using native parsing.
- **Markdown files (`.md`)**: Parses structured text, headings, and notes.
- **Plain-text files (`.txt`)**: Imports unformatted raw text.

> [!NOTE]
> Scanned or image-only PDFs require optical character recognition (OCR), which is **not supported** in v1.1.0. PDFs must contain selectable, extractable text.

### Ingestion Workflow
1. **Selection:** Users select files via the native Windows file picker (`@tauri-apps/plugin-dialog`).
2. **SHA-256 Content Hashing:** Every imported document is hashed before writing. Duplicate files with identical hashes are identified immediately, preventing duplicate ingestion and redundant storage.
3. **Local Document Vault:** Files are copied to a managed local vault directory under `%APPDATA%\com.echo.desktop\ECHO\documents\imported\`.
4. **Metadata & Text Extraction:** Extracted text, original paths, file sizes, and timestamps are recorded in the local SQLite database.

---

## 2. Document Collections (v1.1.0)

ECHO Knowledge v1.1.0 introduces persistent manual collections for structuring documents into projects, topics, or research themes:

- **Create, Rename & Delete Collections:** Users can establish custom collections with dedicated color tags and descriptions.
- **Multi-Collection Assignment:** A single document can belong to multiple collections simultaneously without duplicating underlying storage.
- **In-Collection Document Preview:** Preview full extracted text and metadata directly within the Collections workspace.
- **Safe Removal vs. Permanent Deletion:** Users can remove a document from a collection (preserving the file in the library) or choose permanent deletion with an explicit confirmation dialog.
- **Persistent Counts:** Collection counters and membership associations persist across application restarts in SQLite schema v9.

---

## 3. Local Search & Semantic Retrieval

ECHO Knowledge provides two distinct levels of search:

### Local Lexical Search
- Real-time search across document filenames and extracted text snippets.
- Runs entirely offline against local SQLite tables with no network latency or API calls.

### Semantic Search & Chunk Retrieval
- Documents are split into semantic chunks for vector representation.
- Embedding vectors (768-dimensional vectors generated via Gemini API) are stored locally in SQLite.
- Vector similarity scoring is computed using local cosine similarity against stored vectors.
- Matches are retrieved based on contextual meaning rather than exact word matching.

---

## 4. Ask ECHO — Grounded Q&A

**Ask ECHO** is an interactive research interface that generates answers grounded directly in the user's document vault:

- **Context Retrieval:** Queries retrieve the most relevant local document chunks using vector cosine similarity.
- **Grounded Prompt Construction:** Retrieved context passages are assembled into an augmented prompt sent directly to the configured AI provider.
- **Verifiable Source Citations:** Generated responses include explicit citations indicating which local documents and chunks informed each statement.
- **Conversation History:** Chat sessions are stored locally in SQLite, allowing users to review prior queries and answers.

---

## 5. Knowledge Graph & Relationship Mapping

ECHO Knowledge models concept relationships across your document repository:

- **Concept & Tag Extraction:** Extracts key topics, entities, and tags from ingested documents.
- **Interactive Graph Canvas:** Built with `@xyflow/react`, providing zoomable, pannable, and draggable node visualizations.
- **Persistent Layouts:** Node coordinates and layout positions are persisted in SQLite, preserving graph arrangements between sessions.
- **Filtering & Exploration:** Filter concepts by clusters, examine connections in an inspector panel, and explore relationships between overlapping themes.

---

## 6. Bring Your Own Key (BYOK) AI Integrations

ECHO Knowledge does not sell subscription tiers or run proprietary proxy servers. Users bring their own API keys:

| Provider | Supported Functions | Setup Requirement |
|---|---|---|
| **Google Gemini** | Embeddings (`text-embedding-004`), Text Generation (`gemini-1.5-flash`, `gemini-1.5-pro`) | API key from Google AI Studio |
| **Groq** | Fast Inference / Text Generation (`llama-3.3-70b-versatile`, `mixtral-8x7b-32768`) | API key from Groq Console |

- **Security:** API keys are never stored in plain text or browser local storage; they are stored in the **Windows Credential Manager** via the Rust backend.
- **Direct Transmission:** API requests travel directly from the user's desktop application to the respective provider's endpoints over HTTPS.

---

## 7. Windows 11 Fluent UI & Desktop Integration

- **Fluent Design Language:** Built with dark charcoal surfaces (`#181819`), Segoe UI Variable typography, and subtle border accents.
- **Theme Support:** Supports Dark, Light, and System Default themes.
- **Desktop Windowing:** Built on Tauri 2 with custom titlebar controls, status bar diagnostics, and native dialogs.
