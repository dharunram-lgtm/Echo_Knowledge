export type TagCategory = "technology" | "concept" | "topic";

export interface TagRecord {
  id: string;
  name: string;
  normalized_name: string;
  category: TagCategory;
  created_at: string;
}

export interface DocumentTagRecord {
  document_id: string;
  tag_id: string;
  name: string;
  normalized_name: string;
  category: TagCategory;
  is_ai_generated: boolean;
  confidence: number;
  evidence?: string | null;
  created_at: string;
}

export interface ConceptRecord {
  id: string;
  name: string;
  canonical_id: string;
  category: TagCategory;
  description?: string | null;
  created_at: string;
}

export interface DocumentConceptRecord {
  document_id: string;
  concept_id: string;
  name: string;
  canonical_id: string;
  category: TagCategory;
  confidence: number;
  evidence?: string | null;
  created_at: string;
}

export type TaggingState = "pending" | "tagging" | "tagged" | "failed" | "awaiting_key" | "untagged";

export interface DocumentTaggingStatusRecord {
  document_id: string;
  status: TaggingState;
  error_message?: string | null;
  content_hash: string;
  tags_count: number;
  last_attempt_at?: string | null;
  tagged_at?: string | null;
  attempt_count: number;
  next_retry_at?: string | null;
  retry_state: string;
  last_successful_attempt_at?: string | null;
  last_error?: string | null;
}


