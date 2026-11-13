# ECHO Knowledge

ECHO Knowledge is a local-first, AI-powered knowledge management system and personal document intelligence desktop application for Windows 11. It allows you to organize personal documents, execute high-speed local and semantic searches, generate grounded AI answers with source citations, and explore conceptual relationships across an interactive knowledge graph.

Built with **Tauri 2**, **Rust**, **React 19**, **TypeScript**, and **SQLite**, ECHO Knowledge combines the performance and security of a local desktop application with optional cloud intelligence via Bring Your Own Key (BYOK) support for **Google Gemini** and **Groq**.

> **Latest Release:** [v1.2.1](https://github.com/dharunram-lgtm/Echo_Knowledge/releases/tag/v1.2.1) · Windows 11 / 10 x64 · Tauri 2 + NSIS

---

## 1. Project Overview

Modern personal knowledge management software often forces a trade-off: surrender your private documents to third-party cloud databases, or settle for basic desktop tools that lack intelligent semantic retrieval.

**ECHO Knowledge** bridges this gap through a local-first architecture:
- Your files, extracted text, metadata, embeddings, and chat histories remain strictly on your physical machine in a local SQLite database and document vault.
- When you choose to utilize AI synthesis or semantic embedding, ECHO communicates directly with your chosen AI provider using your personal API key.
- No developer-hosted proxy servers, no telemetry, no tracking beacons, and no mandatory user accounts.

---

## 2. Key Features

- **Local Document Management:** Import PDF, Markdown (`.md`), and plain-text (`.txt`) documents into a managed local vault.
- **SHA-256 Deduplication:** Automatic content hashing identifies duplicate files before ingestion, preventing redundant storage and indexing.
- **Document Collections (v1.2.1):** Organize documents into custom, persistent collections with dedicated color tags, document counts, and in-collection document previews.
- **Semantic Document Search:** Split documents into semantic chunks and retrieve information based on conceptual meaning using locally evaluated vector similarity (cosine similarity).
- **Fast Local Search:** Immediate keyword and filename search operating directly against local SQLite tables without requiring network connectivity.
- **Ask ECHO — Grounded AI Answers:** Interactive dialogue interface that constructs augmented prompts from retrieved document passages, providing grounded responses with verifiable source citations.
- **Interactive Knowledge Graphs:** Explore relationships, clusters, and concept evolution across your document library via an interactive canvas powered by `@xyflow/react`.
- **Bring Your Own Key (BYOK):** Seamless integration with Google Gemini (embeddings and generation) and Groq (high-speed generation) with direct-to-provider HTTPS communication.
- **Enterprise-Grade Credential Isolation:** API keys are stored in the native Windows Credential Manager via DPAPI; keys are never exposed to the frontend or stored in plaintext database tables.
- **Windows 11 Fluent Design:** Clean, native aesthetic adhering to Windows 11 Fluent Design principles, featuring charcoal surfaces, Segoe UI Variable typography, and dark/light theme switching.

---

## 3. Application Screenshots

### Dashboard

![ECHO Knowledge Dashboard](docs/screenshots/dashboard.png)

*The central dashboard showing document statistics, indexing status, AI provider connectivity, and local storage information.*

---

### My Knowledge

![ECHO Knowledge Document Management](docs/screenshots/my-knowledge.png)

*Manage imported documents, inspect indexing status, organize topics, and access document previews.*

---

### Collections

![ECHO Knowledge Collections Workspace](docs/screenshots/collections.png)

*Organize documents into persistent custom collections and access their contents.*

---

### Knowledge Graph

![ECHO Knowledge Graph Exploration](docs/screenshots/knowledge-graph.png)

*Explore concepts, discover relationships, and visualize connections across your knowledge base.*

---

## 4. How ECHO Knowledge Works

ECHO Knowledge implements a secure, local-first retrieval-augmented workflow:

```
┌────────────────────────────────────────────────────────┐
│               1. Local Document Ingestion              │
│  User Files (.pdf, .md, .txt) ──► SHA-256 Hash Check   │
│  ──► Copied to Local Vault ──► Text Parsed to SQLite   │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│               2. Chunking & Local Embeddings           │
│  Text Chunked ──► Gemini Embedding API (User Key)      │
│  ──► 768-dim Vectors Persisted in Local SQLite         │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│               3. Semantic Retrieval & Grounding        │
│  User Query ──► Local Cosine Similarity Search         │
│  ──► Top Relevant Chunks Assembled into Prompt         │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│               4. Direct Synthesis with Citations       │
│  Augmented Prompt ──► Gemini / Groq API (Direct HTTPS) │
│  ──► Grounded Response Rendered with Source Citations  │
└────────────────────────────────────────────────────────┘
```

---

## 5. Technology Stack

- **Desktop Runtime:** [Tauri 2](https://v2.tauri.app/) (Rust backend + Microsoft WebView2)
- **Frontend Framework:** [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) (Strict Mode)
- **Build Tool:** [Vite 8](https://vite.dev/)
- **Styling:** [Tailwind CSS v4](https://tailwindcss.com/) with Windows 11 Fluent tokens
- **Graph Visualization:** [@xyflow/react](https://reactflow.dev/)
- **Local Persistence:** [SQLite](https://www.sqlite.org/) via `rusqlite` (WAL Mode)
- **Icons:** [Lucide React](https://lucide.dev/)
- **Credential Storage:** Windows Credential Manager via native DPAPI

---

## 6. Architecture Overview

ECHO Knowledge enforces strict boundaries between its presentation layer and desktop system capabilities:

- **React Frontend:** Handles UI presentation, reactive state, and user interactions. Contains no file handles, no database connections, and no API keys.
- **Tauri IPC Bridge:** Communicates exclusively through typed IPC commands governed by a restrictive permissions schema (`src-tauri/capabilities/default.json`).
- **Rust Backend:** Executes file operations, database transactions, credential vault management, and direct HTTP communication with AI providers.

For an in-depth breakdown of system design and IPC commands, see [ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## 7. Download and Installation

### Official Windows Release (v1.2.1)

Download the official setup installer from the [GitHub Releases](https://github.com/dharunram-lgtm/Echo_Knowledge/releases/tag/v1.2.1) page:

- **Installer:** `ECHO_Knowledge_Windows11_v1.2.1_x64-setup.exe`
- **Release:** v1.2.1
- **SHA-256 Checksum:** `5F9D493669ABCA46CE8C60FA88686693785C550D69CA1C66ADBC78AE1EF1A64C`

### Installation Steps
1. Run `ECHO_Knowledge_Windows11_v1.1.0_x64-setup.exe`.
2. Follow the setup wizard to install the application for the current user.
3. Launch **ECHO Knowledge** from the Start Menu or desktop shortcut.
4. *(Optional)* Configure your Gemini or Groq API keys in **Settings → AI Providers** to unlock semantic search and grounded Q&A.

---

## 8. Requirements

- **Operating System:** Windows 11 (64-bit) or Windows 10 (Build 19041+) 64-bit.
- **Runtimes:** Microsoft Edge WebView2 (included in modern Windows 11; automatically bootstrapped during installation if missing) and Microsoft Visual C++ Redistributable.
- **Internet Access:** Required only during installation (if WebView2 bootstrapper is needed) and when transmitting prompts to configured AI providers. Local document indexing and keyword search function entirely offline.
- **AI Credentials:** An API key from [Google AI Studio](https://aistudio.google.com/) or [Groq Console](https://console.groq.com/) for AI-powered features.

---

## 9. Development Setup

To build and run ECHO Knowledge locally from source:

### Prerequisites
1. **Node.js:** v20.x or v24.x and npm
2. **Rust:** Stable Rust toolchain (`rustup`)
3. **C++ Build Tools:** Visual Studio 2022 Build Tools with "Desktop development with C++" workload (MSVC `link.exe`)

### Quick Start
```bash
# Clone the repository
git clone https://github.com/dharunram-lgtm/Echo_Knowledge.git
cd Echo_Knowledge

# Install dependencies
npm install

# Run TypeScript check and production bundle build
npm run build

# Launch the desktop application in development mode
npm run tauri dev
```

For complete instructions and troubleshooting, see [DEVELOPMENT.md](docs/DEVELOPMENT.md).

---

## 10. Privacy and Security

ECHO Knowledge is built with privacy as a foundational principle:

- **Local Storage:** Documents, database records, and embeddings are stored locally under `%APPDATA%\com.echo.desktop\ECHO\`.
- **Credential Protection:** API keys are stored securely in Windows Credential Manager using DPAPI encryption. Keys are never logged or stored in plaintext.
- **No Third-Party Intermediaries:** AI queries travel directly from your workstation to Google or Groq endpoints over TLS/HTTPS.
- **Zero Telemetry:** The application contains no tracking scripts, analytics libraries, or telemetry beacons.

For comprehensive privacy details, review [PRIVACY.md](docs/PRIVACY.md).

---

## 11. Documentation Links

- [System Architecture & Security Boundaries](docs/ARCHITECTURE.md)
- [Complete Feature Reference](docs/FEATURES.md)
- [Privacy & Security Specifications](docs/PRIVACY.md)
- [Frequently Asked Questions (FAQ)](docs/FAQ.md)
- [Developer & Build Guide](docs/DEVELOPMENT.md)
- [Project Directory Structure](docs/PROJECT_STRUCTURE.md)
- [Release Notes — v1.1.0](docs/RELEASE_NOTES_v1.1.0.md)
- [Release Notes — v1.0.0](docs/RELEASE_NOTES_v1.0.0.md)

---

## 12. Known Limitations

- **Scanned Documents:** Text extraction requires text-based PDFs. Image-only or scanned documents requiring OCR are not supported in v1.1.0.
- **Provider Quotas:** AI feature availability, model selection, rate limits, and latency depend directly on your Gemini or Groq account tier and quotas.
- **Platform Availability:** Packaged installers are provided for 64-bit Windows environments.
- **Local Scope:** Data is bound to the local Windows user profile; cloud synchronization across multiple devices is not supported.

---

## 13. Roadmap

- [x] Local SQLite persistence with schema migration system (v1.0.0)
- [x] Text extraction for PDF, Markdown, and TXT files (v1.0.0)
- [x] Windows Credential Manager DPAPI secret storage (v1.0.0)
- [x] Gemini embeddings and Groq/Gemini text generation (v1.0.0)
- [x] Grounded Ask ECHO Q&A with citations (v1.0.0)
- [x] Interactive Knowledge Graph exploration (v1.0.0)
- [x] Persistent document collections and preview drawer (v1.1.0)
- [ ] Local offline embedding models (ONNX Runtime / sentence-transformers)
- [ ] Optical Character Recognition (OCR) for scanned PDFs and images
- [ ] Exportable knowledge graph snapshots and reports

---

## 14. License

ECHO Knowledge is open-source software licensed under the [MIT License](LICENSE).
