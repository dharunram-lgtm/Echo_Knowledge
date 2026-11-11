# ECHO Knowledge

ECHO is a privacy-first Windows desktop application designed to serve as an intelligent, local knowledge operating system. It enables users to index and organize personal documents locally, connect their own Gemini or Groq provider with a user-supplied API key, retrieve information via semantic search, and explore concept connections through knowledge graphs.

> **Release:** v1.0.0 · Windows 11 x64 · Tauri 2 + NSIS

---

## 🛠 Technology Stack

- **Desktop Framework:** [Tauri 2](https://v2.tauri.app/) (Rust backend + WebView2 runtime)
- **Frontend Framework:** [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) (Strict Mode)
- **Build Tool:** [Vite 8](https://vite.dev/)
- **Styling & Design System:** [Tailwind CSS v4](https://tailwindcss.com/) + Windows 11 Fluent Design Language
- **Icons:** [Lucide React](https://lucide.dev/)
- **Package Manager:** npm

---

## 🏛 Core Design Principles

1. **Local-First & Zero Telemetry:** Personal documents, embeddings, and database indexes remain strictly on the user's physical machine.
2. **Bring-Your-Own-Intelligence (BYO-AI):** Choose between the supported Gemini and Groq providers using your own API credentials.
3. **Secure Credential Storage:** API keys are never stored in frontend code or plaintext files. They are managed through native OS credential boundaries (Windows Credential Manager / DPAPI).
4. **Restricted Desktop Capabilities:** Tauri permissions are strictly scoped to minimal IPC commands. No arbitrary filesystem, shell execution, or ambient network access is permitted.
5. **Authentic Windows 11 Fluent UI:** Dark charcoal surfaces, Segoe UI Variable typography, subtle translucent borders, and clean native desktop interactions without unnecessary purple gradients or excessive animations.

---

## 📁 Directory Structure Overview

```
echo-knowledge/
├── docs/                     # Architectural, structural, and developer documentation
│   ├── ARCHITECTURE.md       # High-level architecture and security boundaries
│   ├── PROJECT_STRUCTURE.md  # Detailed breakdown of directories and responsibilities
│   └── DEVELOPMENT.md        # Environment setup, build steps, and prerequisites
├── src/                      # Frontend React + TypeScript application
│   ├── app/                  # Application configuration and runtime initialization
│   ├── assets/               # Static assets, branding, and icons
│   ├── components/           # Reusable UI components
│   │   ├── shell/            # Core desktop shell (AppShell, Sidebar, TitleBar, StatusBar)
│   │   ├── ui/               # Atomic Fluent design components (Button, Card, Badge, Input)
│   │   └── common/           # Shared utilities (ErrorBoundary, PlaceholderView)
│   ├── pages/                # Top-level page views (Dashboard, MyKnowledge, AskEcho, etc.)
│   ├── features/             # Domain feature modules (onboarding, ai-providers, knowledge, etc.)
│   ├── services/             # Core service contracts (ai, storage, documents, knowledge)
│   ├── hooks/                # Custom React hooks (useTheme, useAppInfo, etc.)
│   ├── stores/               # Client-side state stores
│   ├── types/                # TypeScript type definitions and interfaces
│   ├── utils/                # Pure utility functions and class helpers
│   └── styles/               # Design tokens, Windows 11 Fluent CSS, and global styles
├── src-tauri/                # Desktop backend (Rust)
│   ├── capabilities/         # Scoped Tauri permission schemas (restrictive default)
│   ├── icons/                # Multi-resolution application icons
│   └── src/                  # Rust source code
│       ├── commands/         # Safe IPC command handlers (system, etc.)
│       ├── database/         # Local persistence & vector store architecture boundary
│       ├── security/         # Keyring isolation and credential management boundary
│       ├── lib.rs            # Application entry point and plugin registrations
│       └── main.rs           # Windows subsystem executable entry point
├── package.json              # Frontend dependencies and build scripts
├── tsconfig.json             # TypeScript strict configuration with path aliases
└── vite.config.ts            # Vite bundler configuration with Tailwind CSS v4
```

---

## 🚀 Windows 11 installation

Download `ECHO_Knowledge_Windows11_v1.0.0_x64-setup.exe` from the GitHub Releases page and run it. The installer is configured for current-user installation, creates an `ECHO Knowledge` Start Menu shortcut, and registers a standard uninstaller. It does not require Docker or a developer-hosted backend.

Windows 11 normally includes WebView2. If the runtime is missing or outdated, the installer uses Microsoft's WebView2 bootstrapper and therefore needs an internet connection during installation. No ECHO account is required.

Release installer SHA-256: `6B653A1D269C79E596737462C0F6843B17F07184913AD50ACA3EEABD61ED9E77`

### Local data and privacy

ECHO keeps its SQLite database, embeddings, conversations, tags, graph state, and imported document copies outside the installation directory:

```text
%APPDATA%\com.echo.desktop\ECHO\database\echo.db
%APPDATA%\com.echo.desktop\ECHO\documents\imported\
```

The installer does not bundle personal documents, SQLite databases, API keys, credentials, or local vault content. API keys are stored through Windows Credential Manager using the `echo.knowledge.ai` service namespace; they are not written to SQLite or frontend files. Uninstalling removes application files only and is designed to leave the ECHO data directory in place. Reinstalling v1.0.0 with the same identifier can therefore reuse existing local data.

ECHO has no developer-owned proxy or telemetry service. When a cloud provider is selected, document-derived prompts and embeddings are sent directly from the desktop to that provider's API according to the selected workflow. Review your provider's terms before importing sensitive material.

### API key setup

Open **Settings → AI Providers**, select Gemini or Groq, enter your own API key, and save it. ECHO stores the key in Windows Credential Manager. Gemini is used for embeddings and can also be selected for generation; Groq is available for generation and tagging/relationship workflows. Without a configured provider key, local document import and basic local storage remain available, but cloud-backed indexing, semantic retrieval, and Ask ECHO cannot complete.

### Requirements and limitations

- Windows 11 x64, WebView2 Runtime, and the Microsoft Visual C++ runtime available on the system.
- Internet access is required only for the WebView2 bootstrapper when needed and for selected cloud provider API calls.
- Supported imports are PDF, Markdown, and plain text. Scanned/image-only PDFs require OCR and are not supported by this release.
- API usage, quotas, rate limits, and model availability are controlled by Gemini/Groq and are not included with ECHO.
- User data is local to the Windows account and is not synchronized by this release.

## 🚀 Development setup

### Prerequisites

1. **Node.js**: v20 or v24+ with `npm`
2. **Rust**: Rust toolchain (installed via `rustup`)
3. **Windows C++ Build Tools**: Visual Studio 2022 Build Tools with "Desktop development with C++" workload (required for MSVC `link.exe`).

### Quick Start (Frontend Development & Verification)

```bash
# Install frontend dependencies
npm install

# Run TypeScript type check and production bundle build
npm run build

# Start the Vite development server
npm run dev
```

### Desktop Application Launch

```bash
# Run the Tauri desktop development shell
npm run tauri dev
```

---

## 🧭 Project Roadmap

### v1.0.0 capabilities

- Local SQLite storage for documents, extracted text, embeddings, conversations, tags, concepts, and graph state.
- PDF, Markdown, and plain-text import, local filename/text search, and document preview.
- Gemini embeddings and Gemini or Groq cloud-backed generation, tagging, and relationship discovery.
- Ask ECHO grounded retrieval with source citations and interactive knowledge-graph views.

Cloud-backed features require the user's own provider credentials. When enabled, relevant document content and queries are sent directly to the selected provider for AI processing. No cloud synchronization service is included in v1.0.0.

---

## 📄 License

ECHO is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

Third-party dependencies, runtimes, and bundled assets are subject to their respective open-source licenses.
