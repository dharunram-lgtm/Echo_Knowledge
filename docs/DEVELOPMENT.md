# ECHO — Developer Guide & Setup

This guide walks through establishing your local development environment, running development servers, validating builds, and troubleshooting common desktop toolchain requirements.

---

## 1. System Requirements

- **Operating System:** Windows 10 (Build 19041+) or Windows 11
- **Node.js:** Node.js v20.x or v24.x with npm v10+
- **Rust Toolchain:** Rust `1.80+` installed via `rustup`
- **Microsoft C++ Build Tools:** Visual Studio 2022 Build Tools (MSVC v143 toolset + Windows 10/11 SDK)

---

## 2. Environment Setup

### 2.1 Installing Rust (if not present)

Download and run [rustup-init.exe](https://rustup.rs/) or install via winget:

```powershell
winget install --id Rustlang.Rustup -e
```

Verify your toolchain:

```powershell
rustc --version
cargo --version
```

### 2.2 Installing Visual Studio C++ Build Tools (Required for Windows linking)

Windows Tauri desktop builds require the Microsoft C++ linker (`link.exe`) and the Windows SDK.

Install the build tools using winget:

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Alternatively, download the Visual Studio Installer and select:
- **Workload:** "Desktop development with C++"
- **Components:** MSVC v143 C++ x64/x86 build tools and Windows 10/11 SDK.

---

## 3. Frontend Development Workflow

### Install Dependencies

```powershell
npm install
```

### Run Frontend in Vite Development Server

The frontend dev server launches rapidly with Hot Module Replacement (HMR):

```powershell
npm run dev
```

Visit [http://localhost:1420](http://localhost:1420) in your browser. The UI gracefully falls back to local web mock state when Tauri runtime APIs are not detected.

### Type Check & Production Bundle Build

To verify TypeScript strict compliance and produce optimized bundles:

```powershell
npm run build
```

This runs `tsc` followed by `vite build`.

---

## 4. Desktop Application Development

To run the complete desktop application with native windowing and Rust IPC:

```powershell
npm run tauri dev
```

This command:
1. Starts the Vite frontend server on port 1420.
2. Compiles the Rust backend crate (`src-tauri`).
3. Spawns the native Windows application shell with WebView2.

---

## 5. Architectural Quality Standards

- **TypeScript Strict Mode:** Maintain `strict: true`, `noUnusedLocals: true`, and `noUnusedParameters: true`. Never use `any` when explicit types can be modeled.
- **No Hardcoded Secrets:** Never place API keys, private tokens, or test credentials into frontend code or version-controlled files.
- **Restrictive Capabilities:** Add permissions to `src-tauri/capabilities/default.json` only when explicitly justified.
- **Fluent UI Discipline:** Adhere to Windows 11 Fluent design principles. Avoid heavy neon gradients, arbitrary drop shadows, or un-native animations.

