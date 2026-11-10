# ECHO — Your Personal AI Knowledge Operating System

ECHO is a privacy-first Windows desktop application designed to serve as an intelligent, local knowledge operating system. It enables users to index and organize personal documents locally, connect their own AI model providers (BYO API key or local Ollama), retrieve information via semantic search, explore concept connections through knowledge graphs, and optionally synchronize encrypted snapshots with Google Drive.

> **Project Phase:** Phase 1 — Project Foundation & Architecture

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
2. **Bring-Your-Own-Intelligence (BYO-AI):** Freedom to choose and switch between OpenAI, Anthropic Claude, Google Gemini, or 100% offline local models via Ollama.
3. **Secure Credential Storage:** API keys and authentication tokens are never stored in frontend code or plaintext files. They are managed through native OS credential boundaries (Windows DPAPI / Credential Vault).
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

## 🚀 Getting Started

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

- **Phase 1: Project Foundation & File Structure (Current)**
  - Project initialization, architecture specification, directory layout, strict typing, Windows 11 Fluent design tokens, and lightweight structural components.
- **Phase 2: Local Persistence & Document Ingestion**
  - SQLite database setup, folder watcher, document parsing (Markdown, PDF, Plaintext), and chunking engine.
- **Phase 3: AI Provider Integration & Key Vault**
  - Windows Credential Manager integration for API keys, OpenAI/Anthropic/Gemini/Ollama client drivers.
- **Phase 4: Semantic Search & Knowledge Graph**
  - Local vector embeddings, HNSW index, entity relationship extraction, interactive graph canvas.
- **Phase 5: Ask ECHO Synthesis & Chat Interface**
  - Local RAG pipeline, context compilation, streaming response synthesis with source citations.
- **Phase 6: Google Drive Synchronization**
  - Encrypted remote snapshots, OAuth synchronization, conflict resolution.
