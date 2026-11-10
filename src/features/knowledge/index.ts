/**
 * Knowledge Feature Module
 *
 * Coordinates semantic search, graph relationship queries,
 * concept extraction, and citation synthesis.
 */

export interface KnowledgeNode {
  id: string;
  label: string;
  type: "document" | "concept" | "collection" | "tag";
  weight: number;
  documentId?: string;
}

export interface KnowledgeEdge {
  id: string;
  source: string;
  target: string;
  relationship: string;
  similarity?: number;
}

export interface KnowledgeGraphData {
  nodes: KnowledgeNode[];
  edges: KnowledgeEdge[];
}

