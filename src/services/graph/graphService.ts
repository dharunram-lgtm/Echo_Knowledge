import { invoke } from "@tauri-apps/api/core";
import type {
  ConceptRelationshipRecord,
  ConceptNeighborRecord,
  ConceptDocumentRecord,
  GraphStats,
  ConceptNodePositionRecord,
  NodePositionInput,
  GraphViewport,
  KnowledgeCluster,
  SuggestedRelationship,
  ConceptExplorationResult,
  TimelineEvent,
  PotentialConflict,
  KnowledgeInsights,
} from "../../types/graph";

export class GraphService {
  /**
   * Retrieves all concept relationships with an optional limit
   */
  public static async getAllRelationships(limit?: number): Promise<ConceptRelationshipRecord[]> {
    return await invoke<ConceptRelationshipRecord[]>("get_all_relationships", { limit });
  }

  /**
   * Retrieves all incoming and outgoing neighbor relationships for a specific concept
   */
  public static async getConceptNeighbors(conceptId: string): Promise<ConceptNeighborRecord[]> {
    return await invoke<ConceptNeighborRecord[]>("get_concept_neighbors", { conceptId });
  }

  /**
   * Retrieves all documents that reference a specific concept
   */
  public static async getConceptDocuments(conceptId: string): Promise<ConceptDocumentRecord[]> {
    return await invoke<ConceptDocumentRecord[]>("get_concept_documents", { conceptId });
  }

  /**
   * Retrieves all relationships discovered from a specific source document
   */
  public static async getDocumentRelationships(documentId: string): Promise<ConceptRelationshipRecord[]> {
    return await invoke<ConceptRelationshipRecord[]>("get_document_relationships", { documentId });
  }

  /**
   * Deletes a relationship record
   */
  public static async deleteConceptRelationship(id: string): Promise<void> {
    return await invoke<void>("delete_concept_relationship", { id });
  }

  /**
   * Retrieves high-level graph statistics
   */
  public static async getGraphStats(): Promise<GraphStats> {
    return await invoke<GraphStats>("get_graph_stats");
  }

  /**
   * Triggers AI relationship discovery for a document
   */
  public static async discoverDocumentRelationships(documentId: string): Promise<ConceptRelationshipRecord[]> {
    return await invoke<ConceptRelationshipRecord[]>("discover_document_relationships", { documentId });
  }

  /**
   * Discovers relationships across all documents
   */
  public static async discoverAllRelationships(): Promise<number> {
    return await invoke<number>("discover_all_relationships");
  }

  /**
   * Saves concept node positions to SQLite
   */
  public static async saveNodePositions(positions: NodePositionInput[]): Promise<void> {
    return await invoke<void>("save_node_positions", { positions });
  }

  /**
   * Retrieves all saved concept node positions from SQLite
   */
  public static async getNodePositions(): Promise<ConceptNodePositionRecord[]> {
    return await invoke<ConceptNodePositionRecord[]>("get_node_positions");
  }

  /**
   * Saves the graph canvas viewport (zoom and pan coordinates) to SQLite
   */
  public static async saveGraphViewport(viewport: GraphViewport): Promise<void> {
    return await invoke<void>("save_graph_viewport", { viewport });
  }

  /**
   * Retrieves the saved graph canvas viewport from SQLite
   */
  public static async getGraphViewport(): Promise<GraphViewport | null> {
    return await invoke<GraphViewport | null>("get_graph_viewport");
  }

  // --- Phase 4.3 Knowledge Intelligence Engine Methods ---

  /**
   * Retrieves all detected knowledge clusters
   */
  public static async getKnowledgeClusters(): Promise<KnowledgeCluster[]> {
    return await invoke<KnowledgeCluster[]>("get_knowledge_clusters");
  }

  /**
   * Recomputes knowledge clusters using graph community detection
   */
  public static async recomputeClusters(): Promise<KnowledgeCluster[]> {
    return await invoke<KnowledgeCluster[]>("recompute_clusters");
  }

  /**
   * Retrieves suggested hidden relationships
   */
  public static async getSuggestedRelationships(status?: string): Promise<SuggestedRelationship[]> {
    return await invoke<SuggestedRelationship[]>("get_suggested_relationships", { status });
  }

  /**
   * Discovers hidden connections between disconnected concepts
   */
  public static async discoverHiddenConnections(): Promise<SuggestedRelationship[]> {
    return await invoke<SuggestedRelationship[]>("discover_hidden_connections");
  }

  /**
   * Accepts a suggested relationship and inserts it into confirmed relationships
   */
  public static async acceptSuggestedRelationship(id: string): Promise<ConceptRelationshipRecord> {
    return await invoke<ConceptRelationshipRecord>("accept_suggested_relationship", { id });
  }

  /**
   * Rejects a suggested relationship
   */
  public static async rejectSuggestedRelationship(id: string): Promise<void> {
    return await invoke<void>("reject_suggested_relationship", { id });
  }

  /**
   * Explores a concept with AI-powered questions and real citations
   */
  public static async exploreConcept(
    conceptId: string,
    queryType: string,
    customPrompt?: string,
    compareConceptId?: string
  ): Promise<ConceptExplorationResult> {
    return await invoke<ConceptExplorationResult>("explore_concept", {
      conceptId,
      queryType,
      customPrompt,
      compareConceptId,
    });
  }

  /**
   * Retrieves the knowledge evolution timeline
   */
  public static async getKnowledgeEvolutionTimeline(): Promise<TimelineEvent[]> {
    return await invoke<TimelineEvent[]>("get_knowledge_evolution_timeline");
  }

  /**
   * Detects potential conflicts / divergences across documents
   */
  public static async detectPotentialConflicts(): Promise<PotentialConflict[]> {
    return await invoke<PotentialConflict[]>("detect_potential_conflicts");
  }

  /**
   * Retrieves comprehensive knowledge insights metrics
   */
  public static async getKnowledgeInsights(): Promise<KnowledgeInsights> {
    return await invoke<KnowledgeInsights>("get_knowledge_insights");
  }
}

