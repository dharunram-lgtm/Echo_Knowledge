# ECHO — Project Structure & Directory Reference

This document outlines every major directory and file established during **Phase 1: Project Foundation & Architecture**, explaining their designated responsibilities and conventions.

---

## Root Directories

| Path | Purpose |
|---|---|
| `docs/` | Architectural guides, system design specifications, and onboarding documentation. |
| `src/` | React frontend application, UI components, state stores, feature modules, and design tokens. |
| `src-tauri/` | Tauri 2 native desktop backend in Rust, system IPC commands, and OS capabilities. |
| `public/` | Static static assets served directly without Vite transformation. |

---

## Detailed Frontend Structure (`src/`)

```
src/
├── app/
│   └── index.ts                 # Application environment detection and config constants
├── assets/                      # Static branding logos, icons, and vector artwork
├── components/
│   ├── shell/                   # Primary desktop shell layout components
│   │   ├── AppShell.tsx         # Root container orchestrating Sidebar, TitleBar, and Viewport
│   │   ├── Sidebar.tsx          # Navigation rail with route switching and collapse control
│   │   ├── TitleBar.tsx         # Windows-styled in-app header with search & theme switch
│   │   ├── NavigationItem.tsx   # Reusable Fluent navigation item button with active indicators
│   │   └── StatusBar.tsx        # Persistent bottom telemetry bar with system state
│   ├── ui/                      # Atomic Windows 11 Fluent design primitives
│   │   ├── Button.tsx           # Reusable Fluent button (primary, default, subtle)
│   │   ├── Card.tsx             # Fluent card container surface with hover states
│   │   ├── Badge.tsx            # Semantic status pills and counters
│   │   └── Input.tsx            # Fluent input field with focus accent underlines
│   ├── common/                  # Shared utility UI components
│   │   ├── ErrorBoundary.tsx    # Desktop error boundary catching React rendering faults
│   │   └── PlaceholderView.tsx  # Standardized placeholder layout for phase boundaries
│   └── documents/               # Document Management Components (Phase 2)
│       ├── DocumentPreviewModal.tsx # Document text inspector & metadata drawer
│       └── DuplicateNoticeModal.tsx # SHA-256 duplicate detection notification modal
├── pages/                       # Top-level screen views
│   ├── Dashboard.tsx            # Main system overview, metrics, and quick actions
│   ├── MyKnowledge.tsx          # Document repository and file management interface
│   ├── AskEcho.tsx              # AI query interface and synthesis dialogue
│   ├── KnowledgeGraph.tsx       # Concept network visualization and entity relationships
│   ├── Collections.tsx          # Notebooks, custom tags, and topic grouping
│   ├── Activity.tsx             # Event timeline, indexing log, and sync audit trail
│   └── Settings.tsx             # Application preferences, theme selector, and provider config
├── features/                    # Domain-driven feature slices
│   ├── onboarding/              # First-run experience and folder selection flow
│   ├── ai-providers/            # BYO provider contracts and metadata (OpenAI, Gemini, etc.)
│   ├── document-management/     # Document scanning, folder watchers, and file status
│   ├── knowledge/               # Knowledge graph entities, semantic links, and concepts
│   └── drive-sync/              # Optional Google Drive synchronization state
├── services/                    # Typed contracts for external and backend services
│   ├── ai/                      # AI provider completion and model contracts
│   ├── storage/                 # Local persistence service contracts
│   ├── documents/               # Document ingestion and parser contracts
│   └── knowledge/               # Semantic search and vector engine contracts
├── hooks/                       # Custom React hooks
│   ├── useTheme.tsx             # Theme state management and DOM synchronization
│   └── useAppInfo.ts            # Safe Tauri backend information retrieval hook
├── stores/                      # Client-side state stores
│   └── index.ts                 # Application state interface and initial values
├── types/                       # Shared TypeScript type definitions
│   ├── index.ts                 # Central type exports
│   ├── navigation.ts            # AppRoute union and NavigationItem interfaces
│   └── theme.ts                 # Theme mode and ThemeContextValue interfaces
├── utils/                       # Utility helper functions
│   └── index.ts                 # Classname combiner (`cn`) and byte formatters
└── styles/                      # Design system stylesheets
    ├── globals.css              # Root stylesheet importing Tailwind v4 and tokens
    ├── theme.css                # CSS variables for Light & Dark mode (charcoal palette)
    └── fluent.css               # Fluent components, elevations, and scrollbar styling
```

---

## Detailed Backend Structure (`src-tauri/`)

```
src-tauri/
├── capabilities/
│   └── default.json             # Tauri 2 security capabilities (core, opener, dialog)
├── icons/                       # Windows ICO, PNG, and multi-resolution application icons
├── src/
│   ├── bin/
│   │   └── test_runner.rs       # Automated Phase 2 integration test runner
│   ├── commands/                # Tauri IPC command modules
│   │   ├── mod.rs               # Commands module aggregator
│   │   ├── system.rs            # `get_app_info` command returning runtime info
│   │   └── documents.rs         # Import, search, preview, delete, stats commands
│   ├── database/
│   │   ├── mod.rs               # Persistence module aggregator
│   │   ├── connection.rs        # SQLite connection pool & AppStorage manager
│   │   ├── migrations.rs        # Versioned schema migrations runner (v1)
│   │   └── documents.rs         # SQLite document repository operations
│   ├── security/
│   │   └── mod.rs               # Architectural boundary for Windows Keyring & DPAPI
│   ├── lib.rs                   # Core library, plugin registration, and command bindings
│   └── main.rs                  # Windows executable entry point (hides console in release)
├── Cargo.toml                   # Rust crate configuration and dependencies
└── tauri.conf.json              # Tauri 2 desktop window, build, and bundle settings
```

