/**
 * Knowledge Engine Contracts
 *
 * Defines contracts for vector search and concept graph traversal.
 */

export interface SemanticSearchResult {
  chunkId: string;
  documentId: string;
  documentTitle: string;
  snippet: string;
  score: number;
}

export interface KnowledgeEngineContract {
  search(query: string, limit?: number): Promise<SemanticSearchResult[]>;
  getRelatedConcepts(nodeId: string): Promise<string[]>;
}

