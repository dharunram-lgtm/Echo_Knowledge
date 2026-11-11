import type {
  DocumentRecord,
  DocumentSearchResult,
  DocumentSummary,
  ImportBatchResult,
  StorageInitResponse,
  StorageStats,
} from "@/types/documents";

export class DocumentService {
  /**
   * Initializes or verifies local SQLite storage and document directory
   */
  public static async initializeStorage(): Promise<StorageInitResponse> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<StorageInitResponse>("initialize_storage");
  }

  /**
   * Opens the native file picker dialog and imports selected .pdf, .md, and .txt files
   */
  public static async pickAndImportDocuments(): Promise<ImportBatchResult | null> {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        title: "Select Documents to Import into ECHO",
        filters: [
          {
            name: "Supported Documents (*.pdf, *.md, *.txt)",
            extensions: ["pdf", "md", "txt"],
          },
          {
            name: "PDF Documents (*.pdf)",
            extensions: ["pdf"],
          },
          {
            name: "Markdown Files (*.md)",
            extensions: ["md"],
          },
          {
            name: "Text Files (*.txt)",
            extensions: ["txt"],
          },
        ],
      });

      if (!selected) {
        return null;
      }

      const paths: string[] = Array.isArray(selected) ? selected : [selected];
      if (paths.length === 0) {
        return null;
      }

      return await this.importDocuments(paths);
    } catch (err) {
      console.error("Failed to pick or import documents:", err);
      throw err;
    }
  }

  /**
   * Passes file paths to Rust backend for validation, hashing, storage, and text extraction
   */
  public static async importDocuments(filePaths: string[]): Promise<ImportBatchResult> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<ImportBatchResult>("import_documents", { filePaths });
  }

  /**
   * Retrieves all document summaries (without full text payload for UI efficiency)
   */
  public static async getDocuments(): Promise<DocumentSummary[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<DocumentSummary[]>("get_documents");
  }

  /**
   * Retrieves a full document record including extracted text by ID
   */
  public static async getDocument(id: string): Promise<DocumentRecord> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<DocumentRecord>("get_document", { id });
  }

  /**
   * Performs SQL LIKE search across filename and extracted document text
   */
  public static async searchDocuments(query: string): Promise<DocumentSearchResult[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<DocumentSearchResult[]>("search_documents", { query });
  }

  /**
   * Deletes a document from the SQLite database and deletes the physical copy in ECHO's storage
   */
  public static async deleteDocument(id: string): Promise<boolean> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<boolean>("delete_document", { id });
  }

  /**
   * Retrieves aggregate storage statistics from SQLite for the Dashboard
   */
  public static async getStorageStats(): Promise<StorageStats> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<StorageStats>("get_storage_stats");
  }
}

export default DocumentService;

