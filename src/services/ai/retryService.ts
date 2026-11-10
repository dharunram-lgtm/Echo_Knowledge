import { invoke } from "@tauri-apps/api/core";
import type { RetryOverview } from "../../types/retry";

export class RetryService {
  /**
   * Retrieves overall retry queue metrics and pause state
   */
  public static async getRetryOverview(): Promise<RetryOverview> {
    return await invoke<RetryOverview>("get_retry_overview");
  }

  /**
   * Globally pauses all automatic retries across indexing and tagging
   */
  public static async pauseAllRetries(): Promise<number> {
    return await invoke<number>("pause_all_retries");
  }

  /**
   * Resumes all paused retries across indexing and tagging
   */
  public static async resumeAllRetries(): Promise<number> {
    return await invoke<number>("resume_all_retries");
  }

  /**
   * Forces an immediate retry for all failed / retry_wait / paused operations
   */
  public static async retryAllFailed(): Promise<number> {
    return await invoke<number>("retry_all_failed");
  }

  /**
   * Triggers a manual immediate retry for a specific document
   */
  public static async retryDocumentOperation(
    documentId: string,
    operation: "indexing" | "tagging" | "both" = "both"
  ): Promise<void> {
    return await invoke<void>("retry_document_operation", { documentId, operation });
  }

  /**
   * Cancels automatic retry attempts for a specific document
   */
  public static async cancelDocumentRetry(
    documentId: string,
    operation: "indexing" | "tagging" | "both" = "both"
  ): Promise<void> {
    return await invoke<void>("cancel_document_retry", { documentId, operation });
  }
}

