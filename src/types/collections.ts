/**
 * Collections Phase 1 — TypeScript types
 *
 * These types mirror the Rust structs in src-tauri/src/database/collections.rs
 * exactly. Field names match the serde serialization output from Rust.
 */

/** A manually curated collection of documents. */
export interface CollectionRecord {
  id: string;
  name: string;
  description: string | null;
  color: string | null;
  icon: string | null;
  created_at: string;
  updated_at: string;
  /** Live count of documents currently in this collection. */
  document_count: number;
}

/**
 * A document that is a member of a collection.
 * Mirrors CollectionDocumentRecord from Rust.
 * This is a document summary augmented with the membership `added_at` timestamp.
 */
export interface CollectionDocumentRecord {
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
  /** ISO-8601 timestamp of when this document was added to the collection. */
  added_at: string;
}

/** Payload for creating a new collection. */
export interface CreateCollectionArgs {
  name: string;
  description?: string | null;
  color?: string | null;
  icon?: string | null;
}

/** Payload for updating an existing collection. */
export interface UpdateCollectionArgs {
  id: string;
  name: string;
  description?: string | null;
  color?: string | null;
  icon?: string | null;
}

