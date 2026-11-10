export type RelationshipType =
  | 'RELATED_TO'
  | 'IS_A'
  | 'PART_OF'
  | 'USES'
  | 'IMPLEMENTS'
  | 'REQUIRES'
  | 'COMPARED_WITH'
  | 'INCLUDES';

export interface ConceptRelationshipRecord {
  id: string;
  source_concept_id: string;
  source_concept_name: string | null;
  target_concept_id: string;
  target_concept_name: string | null;
  relationship_type: RelationshipType;
  confidence: number;
  evidence: string;
  source_document_id: string | null;
  source_document_name: string | null;
  source_chunk_id: string | null;
  discovery_method: string;
  created_at: string;
  updated_at: string;
}

export interface ConceptNeighborRecord {
  relationship_id: string;
  relationship_type: RelationshipType;
  direction: 'outgoing' | 'incoming';
  neighbor_concept_id: string;
  neighbor_concept_name: string;
  neighbor_category: string;
  confidence: number;
  evidence: string;
  source_document_id: string | null;
  source_document_name: string | null;
}

export interface ConceptDocumentRecord {
  document_id: string;
  document_filename: string;
  concept_id: string;
  concept_name: string;
  confidence: number;
  evidence: string | null;
  created_at: string;
}

export interface GraphStats {
  total_concepts: number;
  total_relationships: number;
  connected_documents: number;
  relationship_type_counts: [string, number][];
}

export interface ConceptNodePositionRecord {
  concept_id: string;
  x: number;
  y: number;
  updated_at: string;
}

export interface NodePositionInput {
  concept_id: string;
  x: number;
  y: number;
}

export interface GraphViewport {
  zoom: number;
  x: number;
  y: number;
}

// Phase 4.3 Knowledge Intelligence Engine Types
export interface KnowledgeCluster {
  id: string;
  name: string;
  description: string | null;
  color: string;
  concept_ids: string[];
  primary_concept_id: string | null;
  size: number;
  updated_at: string;
}

export interface SuggestedRelationship {
  id: string;
  source_concept_id: string;
  source_concept_name: string | null;
  target_concept_id: string;
  target_concept_name: string | null;
  relationship_type: RelationshipType;
  confidence: number;
  evidence: string;
  explanation: string;
  source_document_id: string | null;
  source_document_name: string | null;
  status: 'suggested' | 'accepted' | 'rejected';
  created_at: string;
  updated_at: string;
}

export interface ConceptExplorationResult {
  concept_id: string;
  concept_name: string;
  query_type: string;
  response_text: string;
  citations: Array<{
    document_id: string;
    filename: string;
    file_type: string;
    chunk_id: string;
    chunk_index: number;
    snippet: string;
    similarity: number;
  }>;
  related_concepts: string[];
  insufficient_context: boolean;
}

export interface TimelineEvent {
  id: string;
  event_type: 'document_imported' | 'concept_discovered' | 'relationship_added' | 'concept_enriched';
  title: string;
  description: string;
  timestamp: string;
  document_id?: string | null;
  document_name?: string | null;
  concept_id?: string | null;
  concept_name?: string | null;
  metadata?: string | null;
}

export interface PotentialConflict {
  id: string;
  concept_id: string;
  concept_name: string;
  statement_a: string;
  doc_a_id: string | null;
  doc_a_filename: string;
  doc_a_imported_at: string | null;
  statement_b: string;
  doc_b_id: string | null;
  doc_b_filename: string;
  doc_b_imported_at: string | null;
  explanation: string;
  created_at: string;
}

export interface KnowledgeInsights {
  total_concepts: number;
  total_confirmed_relationships: number;
  total_suggested_relationships: number;
  total_clusters: number;
  most_connected_concepts: [string, string, number][];
  largest_clusters: [string, string, number, string][];
  isolated_concepts: [string, string][];
  recently_discovered_concepts: [string, string, string][];
  recently_added_relationships: [string, string, string, string][];
  top_source_documents: [string, string, number][];
  potential_conflicts_count: number;
}


