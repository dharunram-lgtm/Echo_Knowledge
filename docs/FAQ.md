# ECHO Knowledge — Frequently Asked Questions (FAQ)

### 1. What is ECHO Knowledge?
ECHO Knowledge is a local-first, AI-powered personal knowledge management (PKM) and document retrieval desktop application built for Windows 11. It allows you to organize personal documents, perform semantic search, ask questions grounded in your document vault with source citations, and visualize conceptual relationships through interactive knowledge graphs.

### 2. Is ECHO Knowledge free and open source?
Yes. ECHO Knowledge is open-source software released under the [MIT License](https://github.com/dharunram-lgtm/Echo_Knowledge/blob/main/LICENSE). There are no paid subscription tiers or software licensing fees.

### 3. Does ECHO Knowledge require an account?
No. ECHO Knowledge does not require user registration, email sign-up, or cloud account creation. The application runs locally on your workstation.

### 4. Which operating systems are supported?
ECHO Knowledge currently targets **Windows 11 (64-bit)** and **Windows 10 (Build 19041+) 64-bit**. It uses native Windows desktop integrations (WebView2, Windows Credential Manager, and Fluent Design). macOS and Linux builds are not currently packaged.

### 5. Which document formats are supported?
ECHO Knowledge v1.1.0 supports:
- **PDF (`.pdf`)**: Native text extraction for text-based PDFs.
- **Markdown (`.md`)**: Formatted notes and documentation.
- **Plain Text (`.txt`)**: Raw textual documents.

### 6. Does ECHO Knowledge support scanned or image-based PDFs?
No. Version 1.1.0 does not include an Optical Character Recognition (OCR) engine. Image-only or scanned PDFs without selectable text streams cannot be extracted. Documents must contain text streams to be indexed.

### 7. Does ECHO Knowledge work without an AI provider?
Yes. Without configuring an AI provider, you can still:
- Import and store documents in the local vault.
- Organize documents into manual Collections.
- Perform local filename and full-text keyword searches across extracted text.
- Preview documents and manage local library items.

However, vector embeddings, semantic retrieval, grounded Ask ECHO answers, and automated relationship discovery require a configured AI provider API key.

### 8. How do I configure Google Gemini?
1. Obtain an API key from [Google AI Studio](https://aistudio.google.com/).
2. In ECHO Knowledge, open **Settings → AI Providers**.
3. Select **Google Gemini**, paste your API key, and click **Save Key**.
4. ECHO validates and saves the credential securely into Windows Credential Manager.

Gemini provides embedding generation (`text-embedding-004`) and language model generation (`gemini-1.5-flash` / `gemini-1.5-pro`).

### 9. How do I configure Groq?
1. Obtain an API key from [Groq Console](https://console.groq.com/).
2. In ECHO Knowledge, open **Settings → AI Providers**.
3. Select **Groq**, paste your API key, and click **Save Key**.
4. ECHO saves the credential into Windows Credential Manager.

Groq provides high-speed text inference (`llama-3.3-70b-versatile`, `mixtral-8x7b-32768`) for synthesis and tagging workflows.

### 10. Where is my personal data stored?
All user data is stored strictly on your local machine:
- **SQLite Database:** `%APPDATA%\com.echo.desktop\ECHO\database\echo.db`
- **Document Vault:** `%APPDATA%\com.echo.desktop\ECHO\documents\imported\`

ECHO Knowledge does not run a developer-hosted database or sync documents to any proprietary cloud.

### 11. Are my API keys stored securely?
Yes. API keys are stored in the native **Windows Credential Manager** using DPAPI encryption under the `echo.knowledge.ai` service namespace. The keys are never stored in browser `localStorage`, frontend state, or plain-text database fields.

### 12. Does ECHO Knowledge send my documents to the cloud?
When you use cloud AI features (e.g., generating embeddings or asking questions in Ask ECHO), the relevant text passages are sent directly from your desktop to the configured AI provider (Google Gemini or Groq) over an encrypted HTTPS connection. The data is never routed through any ECHO-managed intermediary server.

### 13. How do I install ECHO Knowledge?
1. Download the latest installer (`ECHO_Knowledge_Windows11_v1.1.0_x64-setup.exe`) from the [GitHub Releases](https://github.com/dharunram-lgtm/Echo_Knowledge/releases/latest) page.
2. Run the executable. It installs to your user directory, adds a Start Menu shortcut, and registers with Windows Apps & Features.
3. If Microsoft WebView2 is not already installed on your system, the installer will download it via Microsoft's official bootstrapper.

### 14. How do I uninstall ECHO Knowledge?
- Navigate to **Windows Settings → Apps → Installed apps**, locate **ECHO Knowledge**, and select **Uninstall**.
- The uninstaller removes application binaries and shortcuts.
- To also delete all imported documents and database files, manually delete `%APPDATA%\com.echo.desktop\ECHO\`.
