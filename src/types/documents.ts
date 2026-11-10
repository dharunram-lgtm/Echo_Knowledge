export type SupportedFileType = "pdf" | "md" | "txt";

export interface DocumentSummary {
  id: string;
  filename: string;
  original_path: string;
  stored_path: string;
  file_type: string;
  file_size: number;
  content_hash: string;
  imported_at: string;
  updated_at: string;
  text_length: number;
}

export interface DocumentRecord extends DocumentSummary {
  extracted_text: string;
}

export interface DocumentSearchResult {
  id: string;
  filename: string;
  file_type: string;
  file_size: number;
  imported_at: string;
  snippet: string;
  match_type: "filename" | "content" | "both";
}

export interface DuplicateNotice {
  filename: string;
  existing_filename: string;
  content_hash: string;
  imported_at: string;
  reason: string;
}

export interface FailedImportNotice {
  filename: string;
  path: string;
  error: string;
}

export interface ImportBatchResult {
  imported: DocumentSummary[];
  duplicates: DuplicateNotice[];
  failed: FailedImportNotice[];
  total_processed: number;
}

export interface StorageStats {
  total_documents: number;
  total_size_bytes: number;
  pdf_count: number;
  md_count: number;
  txt_count: number;
  last_imported_at: string | null;
}

export interface StorageInitResponse {
  initialized: boolean;
  db_path: string;
  documents_dir: string;
  schema_version: number;
}

