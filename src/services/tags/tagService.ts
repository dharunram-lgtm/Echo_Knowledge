import { invoke } from "@tauri-apps/api/core";
import type { ConceptRecord, DocumentTagRecord, DocumentTaggingStatusRecord, TagRecord } from "@/types/tags";

export class TagService {
  /**
   * Retrieves all tags attached to a specific document
   */
  public static async getDocumentTags(documentId: string): Promise<DocumentTagRecord[]> {
    return invoke<DocumentTagRecord[]>("get_document_tags", { documentId });
  }

  /**
   * Retrieves all distinct tags across the entire repository
   */
  public static async getAllTags(): Promise<TagRecord[]> {
    return invoke<TagRecord[]>("get_all_tags");
  }

  /**
   * Retrieves all concepts for knowledge graph continuity
   */
  public static async getAllConcepts(): Promise<ConceptRecord[]> {
    return invoke<ConceptRecord[]>("get_all_concepts");
  }

  /**
   * Manually adds a user-created tag to a document
   */
  public static async addManualTag(
    documentId: string,
    tagName: string,
    category?: string
  ): Promise<DocumentTagRecord> {
    return invoke<DocumentTagRecord>("add_manual_tag", {
      documentId,
      tagName,
      category: category || "concept",
    });
  }

  /**
   * Removes a tag from a document
   */
  public static async removeDocumentTag(
    documentId: string,
    tagId: string
  ): Promise<boolean> {
    return invoke<boolean>("remove_document_tag", {
      documentId,
      tagId,
    });
  }

  /**
   * Re-runs AI tag extraction on a document
   */
  public static async retagDocument(documentId: string): Promise<DocumentTagRecord[]> {
    return invoke<DocumentTagRecord[]>("retag_document", { documentId });
  }

  /**
   * Finds all document IDs matching the selected tags (AND filter)
   */
  public static async getDocumentsByTags(tagNames: string[]): Promise<string[]> {
    return invoke<string[]>("get_documents_by_tags", { tagNames });
  }

  /**
   * Renames a tag globally across all associated documents
   */
  public static async renameTag(tagId: string, newName: string): Promise<TagRecord> {
    return invoke<TagRecord>("rename_tag", { tagId, newName });
  }

  /**
   * Retrieves all document tagging status records from SQLite
   */
  public static async getAllTaggingStatuses(): Promise<DocumentTaggingStatusRecord[]> {
    return invoke<DocumentTaggingStatusRecord[]>("get_all_tagging_statuses");
  }

  /**
   * Retries all failed or awaiting_key document tagging jobs
   */
  public static async retryFailedTagging(): Promise<number> {
    return invoke<number>("retry_failed_tagging");
  }

  /**
   * Retries or enqueues tagging for a single document
   */
  public static async retryTaggingForDocument(documentId: string): Promise<void> {
    return invoke<void>("retry_tagging_for_document", { documentId });
  }
}

export default TagService;

