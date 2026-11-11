/**
 * CollectionService — Tauri IPC bridge for Collections Phase 1.
 *
 * All commands are fire-once, strongly typed, and match the Rust command
 * signatures registered in src-tauri/src/lib.rs exactly.
 *
 * The service never touches the database directly; it only serializes arguments
 * and deserializes responses from the Rust backend.
 */

import type {
  CollectionDocumentRecord,
  CollectionRecord,
  CreateCollectionArgs,
  UpdateCollectionArgs,
} from "@/types/collections";

export class CollectionService {
  /** Returns all collections sorted by name (case-insensitive). */
  public static async listCollections(): Promise<CollectionRecord[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<CollectionRecord[]>("list_collections");
  }

  /**
   * Returns a single collection by id.
   * Throws if the collection does not exist.
   */
  public static async getCollection(id: string): Promise<CollectionRecord> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<CollectionRecord>("get_collection", { id });
  }

  /**
   * Creates a new collection and emits `collections:changed`.
   * Throws if the name is empty or exceeds 120 characters.
   */
  public static async createCollection(
    args: CreateCollectionArgs
  ): Promise<CollectionRecord> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<CollectionRecord>("create_collection", {
      name: args.name,
      description: args.description ?? null,
      color: args.color ?? null,
      icon: args.icon ?? null,
    });
  }

  /**
   * Updates a collection's name, description, color and icon.
   * Emits `collections:changed` on success.
   * Throws if the name is empty, the name exceeds 120 characters, or the id
   * does not exist.
   */
  public static async updateCollection(
    args: UpdateCollectionArgs
  ): Promise<CollectionRecord> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<CollectionRecord>("update_collection", {
      id: args.id,
      name: args.name,
      description: args.description ?? null,
      color: args.color ?? null,
      icon: args.icon ?? null,
    });
  }

  /**
   * Permanently deletes a collection and its membership records.
   * Does NOT delete the underlying documents.
   * Emits `collections:changed` on success.
   * Throws if the collection does not exist.
   */
  public static async deleteCollection(id: string): Promise<boolean> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<boolean>("delete_collection", { id });
  }

  /**
   * Adds one or more documents to a collection by their document IDs.
   * Duplicate memberships are silently ignored (INSERT OR IGNORE).
   * Throws if any document ID does not exist — the entire batch is rolled back.
   * Emits `collections:changed` on success.
   */
  public static async addDocumentsToCollection(
    collectionId: string,
    documentIds: string[]
  ): Promise<CollectionRecord> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<CollectionRecord>("add_documents_to_collection", {
      collectionId,
      documentIds,
    });
  }

  /**
   * Removes one or more documents from a collection.
   * Silently skips IDs that are not members.
   * Emits `collections:changed` on success.
   */
  public static async removeDocumentsFromCollection(
    collectionId: string,
    documentIds: string[]
  ): Promise<CollectionRecord> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<CollectionRecord>("remove_documents_from_collection", {
      collectionId,
      documentIds,
    });
  }

  /**
   * Returns all documents that are members of the given collection,
   * sorted by filename (case-insensitive).
   * Throws if the collection does not exist.
   */
  public static async getCollectionDocuments(
    collectionId: string
  ): Promise<CollectionDocumentRecord[]> {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<CollectionDocumentRecord[]>("get_collection_documents", {
      collectionId,
    });
  }
}

