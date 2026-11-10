import React, { useState, useEffect, useCallback, useMemo, useRef } from "react";
import {
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  useNodesState,
  useEdgesState,
  Background,
  BackgroundVariant,
  MarkerType,
  type Node,
  type Edge,
  type Viewport,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

import {
  Network,
  Sparkles,
  Loader2,
  CheckCircle2,
  AlertCircle,
  BarChart3,
  History,
} from "lucide-react";

import { GraphService } from "@/services/graph/graphService";
import { TagService } from "@/services/tags/tagService";
import { DocumentService } from "@/services/documents/documentService";
import type {
  ConceptRelationshipRecord,
  ConceptNeighborRecord,
  ConceptDocumentRecord,
  GraphStats,
  RelationshipType,
  GraphViewport,
  NodePositionInput,
  KnowledgeCluster,
  SuggestedRelationship,
} from "@/types/graph";
import type { ConceptRecord } from "@/types/tags";
import type { DocumentRecord } from "@/types/documents";

import { ConceptNode } from "@/components/graph/ConceptNode";
import { RelationshipEdge, RELATIONSHIP_COLOR_MAP } from "@/components/graph/RelationshipEdge";
import { GraphInspectorPanel } from "@/components/graph/GraphInspectorPanel";
import { GraphToolbar } from "@/components/graph/GraphToolbar";
import { ClusterFilterBar } from "@/components/graph/ClusterFilterBar";
import { ConceptExplorationModal } from "@/components/graph/ConceptExplorationModal";
import { HiddenConnectionsModal } from "@/components/graph/HiddenConnectionsModal";
import { KnowledgeEvolutionModal } from "@/components/graph/KnowledgeEvolutionModal";
import { KnowledgeInsightsModal } from "@/components/graph/KnowledgeInsightsModal";
import { DocumentPreviewModal } from "@/components/documents/DocumentPreviewModal";
import {
  calculateGraphLayout,
  placeNewConceptsWithoutCollision,
  type NodePosition,
} from "@/components/graph/graphLayout";

const nodeTypes = {
  conceptNode: ConceptNode,
};

const edgeTypes = {
  relationshipEdge: RelationshipEdge,
};

const KnowledgeGraphInner: React.FC = () => {
  const { fitView, zoomIn, zoomOut, setCenter, setViewport } = useReactFlow();

  // Data states
  const [stats, setStats] = useState<GraphStats | null>(null);
  const [concepts, setConcepts] = useState<ConceptRecord[]>([]);
  const [relationships, setRelationships] = useState<ConceptRelationshipRecord[]>([]);
  const [clusters, setClusters] = useState<KnowledgeCluster[]>([]);
  const [suggestedRelationships, setSuggestedRelationships] = useState<SuggestedRelationship[]>([]);

  // Selection states
  const [selectedConceptId, setSelectedConceptId] = useState<string | null>(null);
  const [selectedEdgeId, setSelectedEdgeId] = useState<string | null>(null);
  const [selectedClusterId, setSelectedClusterId] = useState<string | null>(null);
  const [neighbors, setNeighbors] = useState<ConceptNeighborRecord[]>([]);
  const [conceptDocs, setConceptDocs] = useState<ConceptDocumentRecord[]>([]);

  // Filter & Search states
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedType, setSelectedType] = useState<RelationshipType | "ALL">("ALL");
  const [confidenceThreshold, setConfidenceThreshold] = useState<number>(60);
  const [connectedOnly, setConnectedOnly] = useState<boolean>(false);
  const [maxNodes, setMaxNodes] = useState<number>(100);

  // UI & Loading states
  const [isLoading, setIsLoading] = useState(true);
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [isRecomputingClusters, setIsRecomputingClusters] = useState(false);
  const [notification, setNotification] = useState<{ type: "success" | "error" | "info"; message: string } | null>(null);
  const [previewDoc, setPreviewDoc] = useState<DocumentRecord | null>(null);
  const [isInspectorOpen, setIsInspectorOpen] = useState(true);

  // Phase 4.3 Modal states
  const [isExploreModalOpen, setIsExploreModalOpen] = useState(false);
  const [conceptToExplore, setConceptToExplore] = useState<ConceptRecord | null>(null);
  const [isHiddenConnectionsOpen, setIsHiddenConnectionsOpen] = useState(false);
  const [isEvolutionModalOpen, setIsEvolutionModalOpen] = useState(false);
  const [isInsightsModalOpen, setIsInsightsModalOpen] = useState(false);

  // React Flow state
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);

  // Persistent Layout & Viewport Refs
  const savedPositionsRef = useRef<Map<string, NodePosition>>(new Map());
  const savedViewportRef = useRef<GraphViewport | null>(null);
  const hasRestoredViewport = useRef<boolean>(false);
  const saveDebounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const viewportDebounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Debounced persistence helper for node coordinates
  const scheduleSavePositions = useCallback(() => {
    if (saveDebounceTimer.current) {
      clearTimeout(saveDebounceTimer.current);
    }
    saveDebounceTimer.current = setTimeout(async () => {
      const posArray: NodePositionInput[] = [];
      for (const [concept_id, pos] of savedPositionsRef.current.entries()) {
        posArray.push({ concept_id, x: pos.x, y: pos.y });
      }
      if (posArray.length > 0) {
        await GraphService.saveNodePositions(posArray).catch((err) => {
          console.error("Failed to save node positions:", err);
        });
      }
    }, 400);
  }, []);

  // Debounced persistence helper for canvas viewport
  const handleMoveEnd = useCallback((_: unknown, viewport: Viewport) => {
    savedViewportRef.current = viewport;
    if (viewportDebounceTimer.current) {
      clearTimeout(viewportDebounceTimer.current);
    }
    viewportDebounceTimer.current = setTimeout(async () => {
      await GraphService.saveGraphViewport(viewport).catch((err) => {
        console.error("Failed to save graph viewport:", err);
      });
    }, 500);
  }, []);

  // 1. Fetch initial data, clusters, suggestions and saved layout from SQLite
  const loadGraphData = useCallback(async () => {
    try {
      setIsLoading(true);
      const [
        gStats,
        allConcepts,
        allRels,
        savedPosRecords,
        savedViewport,
        allClusters,
        allSuggestions,
      ] = await Promise.all([
        GraphService.getGraphStats().catch(() => null),
        TagService.getAllConcepts().catch(() => []),
        GraphService.getAllRelationships(500).catch(() => []),
        GraphService.getNodePositions().catch(() => []),
        GraphService.getGraphViewport().catch(() => null),
        GraphService.getKnowledgeClusters().catch(() => []),
        GraphService.getSuggestedRelationships().catch(() => []),
      ]);

      // Populate positions map from SQLite
      const posMap = new Map<string, NodePosition>();
      for (const p of savedPosRecords) {
        posMap.set(p.concept_id, { x: p.x, y: p.y });
      }
      savedPositionsRef.current = posMap;
      savedViewportRef.current = savedViewport;

      setStats(gStats);
      setConcepts(allConcepts);
      setRelationships(allRels);
      setClusters(allClusters);
      setSuggestedRelationships(allSuggestions);
    } catch (err) {
      console.error("Failed to load knowledge graph data:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadGraphData();

    // Listen to real-time relationship discovery events
    let unlisten: (() => void) | undefined;
    import("@tauri-apps/api/event").then(({ listen }) => {
      listen("relationships:changed", () => {
        loadGraphData();
      }).then((fn) => {
        unlisten = fn;
  }).catch(() => { });
}).catch (() => { });

return () => {
  if (unlisten) unlisten();
  if (saveDebounceTimer.current) clearTimeout(saveDebounceTimer.current);
  if (viewportDebounceTimer.current) clearTimeout(viewportDebounceTimer.current);
};
  }, [loadGraphData]);

// 2. Fetch details for selected concept
useEffect(() => {
  if (!selectedConceptId) {
    setNeighbors([]);
    setConceptDocs([]);
    return;
  }

  let isMounted = true;
  Promise.all([
    GraphService.getConceptNeighbors(selectedConceptId).catch(() => []),
    GraphService.getConceptDocuments(selectedConceptId).catch(() => []),
  ]).then(([nbrs, docs]) => {
    if (isMounted) {
      setNeighbors(nbrs);
      setConceptDocs(docs);
    }
  });

  return () => {
    isMounted = false;
  };
}, [selectedConceptId]);

// Concept to Cluster mapping
const conceptToClusterMap = useMemo(() => {
  const map = new Map<string, KnowledgeCluster>();
  for (const cl of clusters) {
    for (const cId of cl.concept_ids) {
      map.set(cId, cl);
    }
  }
  return map;
}, [clusters]);

const activeClusterConceptSet = useMemo(() => {
  if (!selectedClusterId) return null;
  const cl = clusters.find((c) => c.id === selectedClusterId);
  return cl ? new Set(cl.concept_ids) : null;
}, [clusters, selectedClusterId]);

// 3. Filter relationships and concepts based on user controls
const filteredData = useMemo(() => {
  const validRels = relationships.filter((r) => {
    if (selectedType !== "ALL" && r.relationship_type !== selectedType) {
      return false;
    }
    if (r.confidence * 100 < confidenceThreshold) {
      return false;
    }
    return true;
  });

  const connectedConceptIdSet = new Set<string>();
  for (const r of validRels) {
    connectedConceptIdSet.add(r.source_concept_id);
    connectedConceptIdSet.add(r.target_concept_id);
  }

  let validConcepts = concepts.filter((c) => {
    if (connectedOnly && !connectedConceptIdSet.has(c.id)) {
      return false;
    }
    return true;
  });

  if (maxNodes > 0 && validConcepts.length > maxNodes) {
    validConcepts = validConcepts
      .sort((a, b) => {
        const aConn = connectedConceptIdSet.has(a.id) ? 1 : 0;
        const bConn = connectedConceptIdSet.has(b.id) ? 1 : 0;
        return bConn - aConn;
      })
      .slice(0, maxNodes);
  }

  const finalConceptIdSet = new Set(validConcepts.map((c) => c.id));
  const finalRels = validRels.filter(
    (r) => finalConceptIdSet.has(r.source_concept_id) && finalConceptIdSet.has(r.target_concept_id)
  );

  return {
    concepts: validConcepts,
    relationships: finalRels,
  };
}, [concepts, relationships, selectedType, confidenceThreshold, connectedOnly, maxNodes]);

// Set of neighbor IDs for the currently selected concept
const neighborIdSet = useMemo(() => {
  const set = new Set<string>();
  if (!selectedConceptId) return set;
  for (const n of neighbors) {
    set.add(n.neighbor_concept_id);
  }
  return set;
}, [selectedConceptId, neighbors]);

// 4. Transform into React Flow Nodes and Edges with Persistent Collision-Free Layout
const updateGraphElements = useCallback((resetLayout = false) => {
  const { concepts: displayConcepts, relationships: displayRels } = filteredData;
  if (displayConcepts.length === 0) {
    setNodes([]);
    setEdges([]);
    return;
  }

  // Determine node coordinates
  let positionsToUse: Map<string, NodePosition>;

  if (resetLayout || savedPositionsRef.current.size === 0) {
    // Full recalculation with compact cluster layout and collision resolution
    positionsToUse = calculateGraphLayout(displayConcepts, displayRels, 900, 600);
    savedPositionsRef.current = positionsToUse;

    // Persist newly calculated coordinates immediately to SQLite
    const posArray: NodePositionInput[] = [];
    for (const [concept_id, pos] of positionsToUse.entries()) {
      posArray.push({ concept_id, x: pos.x, y: pos.y });
    }
    if (posArray.length > 0) {
      GraphService.saveNodePositions(posArray).catch(console.error);
    }
  } else {
    // Identify any newly added concepts that don't have saved positions
    const newConcepts = displayConcepts.filter((c) => !savedPositionsRef.current.has(c.id));
    if (newConcepts.length > 0) {
      // Place new concepts in compact collision-free slots without moving existing ones
      const updated = placeNewConceptsWithoutCollision(
        newConcepts,
        savedPositionsRef.current,
        displayRels
      );
      savedPositionsRef.current = updated;

      // Persist newly placed concepts
      const newPosArray: NodePositionInput[] = newConcepts.map((c) => ({
        concept_id: c.id,
        x: updated.get(c.id)!.x,
        y: updated.get(c.id)!.y,
      }));
      GraphService.saveNodePositions(newPosArray).catch(console.error);
      positionsToUse = updated;
    } else {
      positionsToUse = savedPositionsRef.current;
    }
  }

  // Degree map
  const degreeMap = new Map<string, number>();
  for (const r of displayRels) {
    degreeMap.set(r.source_concept_id, (degreeMap.get(r.source_concept_id) || 0) + 1);
    degreeMap.set(r.target_concept_id, (degreeMap.get(r.target_concept_id) || 0) + 1);
  }

  const q = searchQuery.toLowerCase().trim();

  // Map to Flow Nodes
  const newNodes: Node[] = displayConcepts.map((c) => {
    const isSelected = selectedConceptId === c.id;
    const isNeighbor = neighborIdSet.has(c.id);
    const cluster = conceptToClusterMap.get(c.id);
    const isClusterMatch = activeClusterConceptSet ? activeClusterConceptSet.has(c.id) : false;
    const isDimmed =
      (!!selectedConceptId && !isSelected && !isNeighbor) ||
      (!!activeClusterConceptSet && !isClusterMatch);

    const isSearchMatch = Boolean(
      q && (c.name.toLowerCase().includes(q) || (c.description && c.description.toLowerCase().includes(q)))
    );

    const pos = positionsToUse.get(c.id) || { x: 500, y: 350 };

    return {
      id: c.id,
      type: "conceptNode",
      position: pos,
      data: {
        concept: c,
        degree: degreeMap.get(c.id) || 0,
        isSelected,
        isNeighbor,
        isDimmed,
        isSearchMatch,
        clusterColor: cluster?.color,
        isClusterMatch,
      },
    };
  });

  // Map to Flow Edges
  const newEdges: Edge[] = displayRels.map((r) => {
    const isConnectedToSelected =
      selectedConceptId === r.source_concept_id || selectedConceptId === r.target_concept_id;
    const isEdgeSelected = selectedEdgeId === r.id;
    const isDimmed =
      (!!selectedConceptId && !isConnectedToSelected) ||
      (!!selectedEdgeId && !isEdgeSelected) ||
      (!!activeClusterConceptSet &&
        (!activeClusterConceptSet.has(r.source_concept_id) ||
          !activeClusterConceptSet.has(r.target_concept_id)));

    const style = RELATIONSHIP_COLOR_MAP[r.relationship_type] || RELATIONSHIP_COLOR_MAP.RELATED_TO;

    return {
      id: r.id,
      source: r.source_concept_id,
      target: r.target_concept_id,
      type: "relationshipEdge",
      data: {
        relationship: r,
        isSelected: isEdgeSelected || isConnectedToSelected,
        isDimmed,
      },
      markerEnd: {
        type: MarkerType.ArrowClosed,
        width: 14,
        height: 14,
        color: isEdgeSelected ? "#ffffff" : style.stroke,
      },
    };
  });

  setNodes(newNodes);
  setEdges(newEdges);

  // Viewport restoration logic (tight 8% padding to keep nodes readable and prominent)
  if (resetLayout) {
    setTimeout(() => fitView({ padding: 0.08, duration: 400, maxZoom: 1.15 }), 50);
  } else if (!hasRestoredViewport.current) {
    hasRestoredViewport.current = true;
    setTimeout(() => {
      const savedVp = savedViewportRef.current;
      if (savedVp && typeof savedVp.zoom === "number" && savedVp.zoom > 0) {
        setViewport({ x: savedVp.x, y: savedVp.y, zoom: savedVp.zoom }, { duration: 300 });
      } else {
        fitView({ padding: 0.08, duration: 400, maxZoom: 1.15 });
      }
    }, 50);
  }
}, [
  filteredData,
  selectedConceptId,
  selectedEdgeId,
  neighborIdSet,
  searchQuery,
  conceptToClusterMap,
  activeClusterConceptSet,
  fitView,
  setViewport,
  setNodes,
  setEdges,
]);

// Sync graph on filter changes or data updates
useEffect(() => {
  if (!isLoading) {
    updateGraphElements(false);
  }
}, [updateGraphElements, isLoading]);

// Node Drag Stop Handler (Persist updated position cleanly on drag end)
const handleNodeDragStop = useCallback(
  (_event: unknown, node: Node) => {
    savedPositionsRef.current.set(node.id, { x: node.position.x, y: node.position.y });
    scheduleSavePositions();
  },
  [scheduleSavePositions]
);

// Reset layout handler: recalculates, persists to SQLite, and fits view
const handleResetLayout = () => {
  updateGraphElements(true);
  setNotification({
    type: "success",
    message: "Layout reset with compact, collision-free cluster organization. Positions saved.",
  });
};

// Node selection handlers
const handleNodeClick = (_: React.MouseEvent, node: Node) => {
  setSelectedConceptId(node.id);
  setSelectedEdgeId(null);
  setIsInspectorOpen(true);
};

const handleNodeDoubleClick = (_: React.MouseEvent, node: Node) => {
  setSelectedConceptId(node.id);
  setSelectedEdgeId(null);
  setIsInspectorOpen(true);
  setCenter(node.position.x + 100, node.position.y + 45, { zoom: 1.25, duration: 400 });
};

// Edge selection handler
const handleEdgeClick = (_: React.MouseEvent, edge: Edge) => {
  setSelectedEdgeId(edge.id);
  setSelectedConceptId(null);
  setIsInspectorOpen(true);
};

// Pane click handler (deselects and resets focus)
const handlePaneClick = () => {
  setSelectedConceptId(null);
  setSelectedEdgeId(null);
};

// Focus a specific concept from search, inspector or insights
const handleFocusConcept = (conceptId: string) => {
  setSelectedConceptId(conceptId);
  setSelectedEdgeId(null);
  setIsInspectorOpen(true);

  const pos = savedPositionsRef.current.get(conceptId);
  if (pos) {
    setCenter(pos.x + 100, pos.y + 45, { zoom: 1.25, duration: 400 });
  }
};

// Open Document Preview Modal
const handleOpenDocument = async (documentId: string) => {
  try {
    const doc = await DocumentService.getDocument(documentId);
    if (doc) {
      setPreviewDoc(doc);
    }
  } catch (err) {
    console.error("Failed to load document preview:", err);
  }
};

// Open AI Concept Exploration
const handleOpenExplore = (concept: ConceptRecord) => {
  setConceptToExplore(concept);
  setIsExploreModalOpen(true);
};

// Recompute Clusters
const handleRecomputeClusters = async () => {
  try {
    setIsRecomputingClusters(true);
    const newClusters = await GraphService.recomputeClusters();
    setClusters(newClusters);
    setNotification({
      type: "success",
      message: `Graph community detection identified ${newClusters.length} knowledge clusters.`,
    });
  } catch (err) {
    console.error("Failed to recompute clusters:", err);
  } finally {
    setIsRecomputingClusters(false);
  }
};

// Accept suggested hidden connection
const handleAcceptSuggestion = (newRel: ConceptRelationshipRecord) => {
  setRelationships((prev) => [newRel, ...prev]);
  setSuggestedRelationships((prev) =>
    prev.map((s) => (s.id === newRel.id ? { ...s, status: "accepted" as const } : s))
  );
  setNotification({
    type: "success",
    message: `Accepted connection: ${newRel.relationship_type}. Added to confirmed graph!`,
  });
  // Refresh clusters with the new edge
  GraphService.recomputeClusters().then(setClusters).catch(console.error);
};

// Reject suggested hidden connection
const handleRejectSuggestion = (id: string) => {
  setSuggestedRelationships((prev) =>
    prev.map((s) => (s.id === id ? { ...s, status: "rejected" as const } : s))
  );
};

// Relationship Discovery action
const handleDiscoverAll = async () => {
  try {
    setIsDiscovering(true);
    setNotification({
      type: "info",
      message: "Scanning documents with Gemini to extract verified evidence-grounded relationships...",
    });
    const count = await GraphService.discoverAllRelationships();
    await loadGraphData();
    setNotification({
      type: "success",
      message: `Relationship discovery completed. Found ${count} evidence-backed relationships.`,
    });
    handleResetLayout();
  } catch (err) {
    setNotification({
      type: "error",
      message: `Discovery error: ${String(err)}`,
    });
  } finally {
    setIsDiscovering(false);
  }
};

// Relationship Deletion action
const handleDeleteRelationship = async (id: string) => {
  try {
    await GraphService.deleteConceptRelationship(id);
    setRelationships((prev) => prev.filter((r) => r.id !== id));
    if (selectedEdgeId === id) setSelectedEdgeId(null);
    if (selectedConceptId) {
      setNeighbors((prev) => prev.filter((n) => n.relationship_id !== id));
    }
    const newStats = await GraphService.getGraphStats().catch(() => null);
    if (newStats) setStats(newStats);
    GraphService.recomputeClusters().then(setClusters).catch(console.error);
  } catch (err) {
    console.error("Failed to delete relationship:", err);
  }
};

// Clear all filters
const handleClearFilters = () => {
  setSearchQuery("");
  setSelectedType("ALL");
  setConfidenceThreshold(60);
  setConnectedOnly(false);
  setMaxNodes(100);
  setSelectedConceptId(null);
  setSelectedEdgeId(null);
  setSelectedClusterId(null);
};

const isFiltersActive =
  !!searchQuery.trim() ||
  selectedType !== "ALL" ||
  confidenceThreshold > 60 ||
  connectedOnly ||
maxNodes !== 100 ||
  selectedClusterId !== null;

// Selected item objects for inspector
const activeSelectedConcept = useMemo(
  () => concepts.find((c) => c.id === selectedConceptId) || null,
  [concepts, selectedConceptId]
);

const activeSelectedEdge = useMemo(
  () => relationships.find((r) => r.id === selectedEdgeId) || null,
  [relationships, selectedEdgeId]
);

const pendingSuggestionsCount = useMemo(
  () => suggestedRelationships.filter((s) => s.status === "suggested").length,
  [suggestedRelationships]
);

return (
    <div className="h-full flex flex-col p-6 space-y-3 max-w-[1600px] mx-auto min-h-0">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-2 border-b border-[var(--border-subtle)] shrink-0">
        <div>
          <h1 className="text-xl font-semibold tracking-tight text-[var(--text-primary)] flex items-center gap-2">
            <span>Knowledge Graph</span>
            <span className="fluent-badge text-[10px] py-0.5 px-2 bg-[var(--accent-primary)]/10 text-[var(--accent-primary)] border-[var(--accent-primary)]/30 font-mono">
              Phase 4.3 Intelligence Engine
            </span>
          </h1>
          <p className="text-xs text-[var(--text-secondary)]">
            Explore concepts, discover hidden connections, track knowledge evolution, and explore topics with AI.
          </p>
        </div>

          {/* Global Intelligence Actions */}
          <div className="flex items-center gap-2 flex-wrap">
            {stats && (
                <span className="text-[11px] text-[var(--text-tertiary)] hidden xl:inline font-mono">
                  {stats.total_concepts} concepts • {stats.total_relationships} confirmed
                </span>
          )}

                {/* Hidden Connections Action Button */}
                <button
                  type="button"
                  onClick={() => setIsHiddenConnectionsOpen(true)}
                  className="fluent-button text-xs py-1.5 px-2.5 flex items-center gap-1.5 cursor-pointer relative"
                  title="Review proposed hidden connections between concepts"
                >
                  <Sparkles className="w-3.5 h-3.5 text-cyan-400" />
                  <span>Hidden Connections</span>
                  {pendingSuggestionsCount > 0 && (
                    <span className="w-4 h-4 rounded-full bg-cyan-500 text-black text-[9px] font-bold flex items-center justify-center">
                      {pendingSuggestionsCount}
                    </span>
                  )}
                </button>

                {/* Knowledge Evolution & Conflicts Action Button */}
                <button
                  type="button"
                  onClick={() => setIsEvolutionModalOpen(true)}
                  className="fluent-button text-xs py-1.5 px-2.5 flex items-center gap-1.5 cursor-pointer"
                  title="Inspect historical knowledge timeline and potential evidence conflicts"
                >
                  <History className="w-3.5 h-3.5 text-emerald-400" />
                  <span>Evolution</span>
                </button>

                {/* Knowledge Insights Dashboard Action Button */}
                <button
                  type="button"
                  onClick={() => setIsInsightsModalOpen(true)}
                  className="fluent-button text-xs py-1.5 px-2.5 flex items-center gap-1.5 cursor-pointer"
                  title="View high-level knowledge analytics and hub metrics"
                >
                  <BarChart3 className="w-3.5 h-3.5 text-[var(--accent-primary)]" />
                  <span>Insights</span>
                </button>

                {/* Discover Relationships Trigger */}
                <button
                  type="button"
                  onClick={handleDiscoverAll}
                  disabled={isDiscovering || isLoading}
                  className="fluent-button text-xs py-1.5 px-3 bg-[var(--accent-primary)] text-white hover:opacity-90 flex items-center gap-1.5 cursor-pointer shadow-[var(--shadow-rest)]"
                  title="Scan all documents with Gemini to extract evidence-backed relationships"
                >
                  {isDiscovering ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    <Sparkles className="w-3.5 h-3.5" />
                  )}
                  <span>{isDiscovering ? "Discovering..." : "Discover Relationships"}</span>
                </button>
              </div>
      </div>

          {/* Notification Toast */}
          {notification && (
            <div
          className={`p-3 rounded-[var(--radius-md)] text-xs flex items-center justify-between border shrink-0 ${notification.type === "success"
                  ? "bg-[var(--status-success)]/10 text-[var(--status-success)] border-[var(--status-success)]/30"
                  : notification.type === "error"
                    ? "bg-[var(--status-danger)]/10 text-[var(--status-danger)] border-[var(--status-danger)]/30"
                    : "bg-blue-500/10 text-blue-400 border-blue-500/30"
          }`}
        >
          <div className="flex items-center gap-2">
            {notification.type === "success" ? (
              <CheckCircle2 className="w-4 h-4 shrink-0" />
            ) : (
              <AlertCircle className="w-4 h-4 shrink-0" />
            )}
            <span>{notification.message}</span>
          </div>
          <button
            type="button"
            onClick={() => setNotification(null)}
            className="text-[11px] underline opacity-70 hover:opacity-100 ml-4 cursor-pointer"
          >
            Dismiss
          </button>
        </div>
      )}

        {/* Toolbar & Filter Controls */}
        <GraphToolbar
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          selectedType={selectedType}
          onSelectType={setSelectedType}
          confidenceThreshold={confidenceThreshold}
          onConfidenceChange={setConfidenceThreshold}
          connectedOnly={connectedOnly}
          onConnectedOnlyToggle={setConnectedOnly}
          maxNodes={maxNodes}
          onMaxNodesChange={setMaxNodes}
          onZoomIn={() => zoomIn({ duration: 300 })}
          onZoomOut={() => zoomOut({ duration: 300 })}
          onFitView={() => fitView({ padding: 0.08, duration: 400, maxZoom: 1.15 })}
          onResetLayout={handleResetLayout}
          onClearFilters={handleClearFilters}
          isFiltersActive={isFiltersActive}
        />

        {/* Stage 1: Knowledge Cluster Filter Bar */}
        <ClusterFilterBar
          clusters={clusters}
          selectedClusterId={selectedClusterId}
          onSelectCluster={(cid) => {
            setSelectedClusterId(cid);
            if (cid) {
              const cl = clusters.find((c) => c.id === cid);
              if (cl?.primary_concept_id) {
                handleFocusConcept(cl.primary_concept_id);
              }
            }
          }}
          onRecomputeClusters={handleRecomputeClusters}
          isRecomputing={isRecomputingClusters}
        />

        {/* Main Graph Viewport + Inspector Layout */}
        <div className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-3.5 min-h-0 overflow-hidden relative">
          {/* Left / Center: Interactive Canvas (8 or 12 cols based on inspector) */}
          <div
            className={`${isInspectorOpen ? "lg:col-span-8" : "lg:col-span-12"
              } fluent-card p-0 relative flex flex-col min-h-0 overflow-hidden border border-[var(--border-default)] bg-[var(--bg-app)] shadow-[var(--shadow-rest)] transition-all duration-300`}
          >
            {isLoading ? (
              <div className="absolute inset-0 z-20 flex flex-col items-center justify-center space-y-2 bg-[var(--bg-app)]/80 backdrop-blur-sm">
                <Loader2 className="w-6 h-6 animate-spin text-[var(--accent-primary)]" />
                <span className="text-xs text-[var(--text-secondary)]">Loading Knowledge Graph...</span>
              </div>
            ) : nodes.length === 0 ? (
              /* Empty State */
              <div className="absolute inset-0 z-10 flex flex-col items-center justify-center p-8 text-center space-y-3 bg-[var(--bg-app)]">
                <div className="w-12 h-12 rounded-full bg-[var(--bg-subtle)] border border-[var(--border-default)] flex items-center justify-center text-[var(--accent-primary)]">
                  <Network className="w-6 h-6" />
                </div>
                <div className="space-y-1 max-w-md">
                  <h3 className="text-sm font-semibold text-[var(--text-primary)]">
                    {isFiltersActive ? "No matching concepts found" : "No Knowledge Concepts Extracted Yet"}
                  </h3>
                  <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
                    {isFiltersActive
                        ? "Try adjusting your search query, lowering the confidence threshold, or clearing the cluster/type filters."
                        : "Import documents in My Knowledge to automatically extract concept nodes and discover semantic connections."}
                  </p>
                </div>

                {isFiltersActive ? (
                  <button
                    type="button"
                    onClick={handleClearFilters}
                    className="fluent-button text-xs py-1.5 px-3"
                  >
                    Clear All Filters
                  </button>
                ) : (
                  <button
                    type="button"
                    onClick={handleDiscoverAll}
                    disabled={isDiscovering}
                    className="fluent-button text-xs py-1.5 px-3 bg-[var(--accent-primary)] text-white hover:opacity-90 flex items-center gap-1.5"
                  >
                    <Sparkles className="w-3.5 h-3.5" />
                    <span>Run Relationship Discovery</span>
                  </button>
                )}
              </div>
            ) : null}

            {/* Canvas Component */}
            <div className="w-full h-full relative">
              <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                onNodeDragStop={handleNodeDragStop}
                onMoveEnd={handleMoveEnd}
                onNodeClick={handleNodeClick}
                onNodeDoubleClick={handleNodeDoubleClick}
                onEdgeClick={handleEdgeClick}
                onPaneClick={handlePaneClick}
                nodeTypes={nodeTypes}
                edgeTypes={edgeTypes}
                zoomOnScroll={true}
                panOnDrag={true}
                nodesDraggable={true}
                minZoom={0.35}
                maxZoom={2.5}
                proOptions={{ hideAttribution: true }}
                className="bg-transparent"
              >
                <Background
                  variant={BackgroundVariant.Dots}
                  gap={24}
                  size={1.5}
                  color="rgba(255, 255, 255, 0.07)"
                />
              </ReactFlow>

              {/* Quick Canvas Overlay Stats & Inspector Toggle */}
              <div className="absolute top-3 left-3 z-10 pointer-events-none flex items-center gap-2">
                <div className="fluent-badge text-[10px] bg-[var(--bg-elevated)]/90 backdrop-blur-md text-[var(--text-secondary)] border border-[var(--border-subtle)] shadow-sm px-2.5 py-1 flex items-center gap-2 font-mono">
                  <span>{nodes.length} nodes</span>
                  <span>•</span>
                  <span>{edges.length} edges</span>
                  {selectedConceptId && (
                    <>
                      <span>•</span>
                      <span className="text-[var(--accent-primary)] font-bold">1 focused</span>
                    </>
                  )}
                  {selectedClusterId && (
                    <>
                      <span>•</span>
                      <span className="text-emerald-400 font-bold">1 cluster filtered</span>
                    </>
                  )}
                </div>
              </div>

              {!isInspectorOpen && (
                <button
                  type="button"
                  onClick={() => setIsInspectorOpen(true)}
                  className="absolute top-3 right-3 z-10 fluent-button text-xs py-1 px-2.5 bg-[var(--bg-elevated)]/90 backdrop-blur-md shadow-sm border border-[var(--border-default)]"
                  title="Open details inspector"
                >
                  Open Inspector
                </button>
              )}
            </div>
          </div>

          {/* Right: Concept / Relationship Inspector Panel (4 cols) */}
          {isInspectorOpen && (
            <div className="lg:col-span-4 h-full min-h-0 flex flex-col">
              <GraphInspectorPanel
                selectedConcept={activeSelectedConcept}
                selectedEdge={activeSelectedEdge}
                neighbors={neighbors}
                conceptDocs={conceptDocs}
                onClose={() => {
                  setSelectedConceptId(null);
                  setSelectedEdgeId(null);
                  setIsInspectorOpen(false);
                }}
                onSelectConcept={handleFocusConcept}
                onOpenDocument={handleOpenDocument}
                onDeleteRelationship={handleDeleteRelationship}
                onFocusConcept={handleFocusConcept}
                onExploreConcept={handleOpenExplore}
              />
            </div>
          )}
        </div>

        {/* Stage 3: AI Concept Exploration Modal */}
        <ConceptExplorationModal
          concept={conceptToExplore}
          isOpen={isExploreModalOpen}
          onClose={() => {
            setIsExploreModalOpen(false);
            setConceptToExplore(null);
          }}
          onOpenDocument={handleOpenDocument}
          allConcepts={concepts}
        />

        {/* Stage 2: Hidden Connection Discovery Modal */}
        <HiddenConnectionsModal
          isOpen={isHiddenConnectionsOpen}
          onClose={() => setIsHiddenConnectionsOpen(false)}
          suggestions={suggestedRelationships}
          onAccept={handleAcceptSuggestion}
          onReject={handleRejectSuggestion}
          onRefresh={() => {
            GraphService.getSuggestedRelationships().then(setSuggestedRelationships).catch(console.error);
          }}
          onOpenDocument={handleOpenDocument}
        />

        {/* Stage 4: Knowledge Evolution & Conflicts Modal */}
        <KnowledgeEvolutionModal
          isOpen={isEvolutionModalOpen}
          onClose={() => setIsEvolutionModalOpen(false)}
          onOpenDocument={handleOpenDocument}
          onFocusConcept={handleFocusConcept}
        />

        {/* Stage 5: Knowledge Insights Dashboard Modal */}
        <KnowledgeInsightsModal
          isOpen={isInsightsModalOpen}
          onClose={() => setIsInsightsModalOpen(false)}
          onFocusConcept={handleFocusConcept}
          onOpenDocument={handleOpenDocument}
          onSelectCluster={(cid) => {
            setSelectedClusterId(cid);
            const cl = clusters.find((c) => c.id === cid);
            if (cl?.primary_concept_id) {
              handleFocusConcept(cl.primary_concept_id);
            }
          }}
        />

        {/* Document Content Preview Modal */}
        <DocumentPreviewModal
          document={previewDoc}
          isOpen={!!previewDoc}
          onClose={() => setPreviewDoc(null)}
        />
    </div>
    );
};

export const KnowledgeGraph: React.FC = () => {
  return (
    <ReactFlowProvider>
      <KnowledgeGraphInner />
    </ReactFlowProvider>
    );
};

    export default KnowledgeGraph;
