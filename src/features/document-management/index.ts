/**
 * Document Management Feature Module
 *
 * Coordinates local filesystem document scanning, metadata extraction,
 * folder watching, and file ingestion.
 */

export interface DocumentMetadata {
  id: string;
  title: string;
  path: string;
  fileSize: number;
  mimeType: string;
  createdAt: string;
  updatedAt: string;
  indexedAt?: string;
  tags: string[];
}

export interface FolderWatchConfig {
  folderPath: string;
  recursive: boolean;
  fileTypes: string[];
}

