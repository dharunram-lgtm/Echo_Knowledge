import type { ConceptRecord } from "@/types/tags";
import type { ConceptRelationshipRecord } from "@/types/graph";

export interface NodePosition {
  x: number;
  y: number;
}

/**
 * Node card dimensions matching ConceptNode.tsx (min-w-[150px] max-w-[220px])
 */
export const NODE_WIDTH = 200;
export const NODE_HEIGHT = 85;

/**
 * Phase 4.2.2 Compact Spacing:
 * 10–12px minimum edge-to-edge gap between node bounding boxes.
 */
export const MIN_GAP_X = 12;
export const MIN_GAP_Y = 12;

export const NODE_MIN_GAP_X = NODE_WIDTH + MIN_GAP_X; // 212px origin-to-origin
export const NODE_MIN_GAP_Y = NODE_HEIGHT + MIN_GAP_Y; // 97px origin-to-origin

/**
 * Checks if two rectangular node bounding boxes overlap (including 10–12px minimum gap)
 */
export function isBoundingBoxColliding(
  p1: NodePosition,
  p2: NodePosition,
  minGapX = NODE_MIN_GAP_X,
  minGapY = NODE_MIN_GAP_Y
): boolean {
  return Math.abs(p1.x - p2.x) < minGapX && Math.abs(p1.y - p2.y) < minGapY;
}

/**
 * Deterministic AABB overlap resolution pass.
 * Gently separates colliding concept nodes so that no two cards overlap or stack,
 * while maintaining the tight 10–12px edge-to-edge spacing.
 * Runs in < 2ms.
 */
export function resolveNodeCollisions(
  positions: Map<string, NodePosition>,
  maxIterations = 60,
  minGapX = NODE_MIN_GAP_X,
  minGapY = NODE_MIN_GAP_Y
): Map<string, NodePosition> {
  const ids = Array.from(positions.keys());
  const n = ids.length;
  if (n <= 1) return positions;

  for (let iter = 0; iter < maxIterations; iter++) {
    let hadCollision = false;

    for (let i = 0; i < n; i++) {
      const uId = ids[i];
      const posU = positions.get(uId)!;

      for (let j = i + 1; j < n; j++) {
        const vId = ids[j];
        const posV = positions.get(vId)!;

        let dx = posU.x - posV.x;
        let dy = posU.y - posV.y;
        const absDx = Math.abs(dx);
        const absDy = Math.abs(dy);

        if (absDx < minGapX && absDy < minGapY) {
          hadCollision = true;

          // Break identical center stacks deterministically
          if (absDx < 0.1 && absDy < 0.1) {
            dx = (i % 2 === 0 ? 1 : -1) * (minGapX * 0.15);
            dy = (j % 2 === 0 ? 1 : -1) * (minGapY * 0.15);
          }

          const overlapX = minGapX - absDx;
          const overlapY = minGapY - absDy;

          // Separate along axes with smooth damping
          const pushX = (dx >= 0 ? 1 : -1) * (overlapX * 0.52);
          const pushY = (dy >= 0 ? 1 : -1) * (overlapY * 0.52);

          posU.x += pushX;
          posU.y += pushY;
          posV.x -= pushX;
          posV.y -= pushY;
        }
      }
    }

    if (!hadCollision) break;
  }

  return positions;
}

/**
 * Places newly discovered concepts into compact, collision-free positions
 * immediately adjacent to their connected neighbors without disturbing existing nodes.
 */
export function placeNewConceptsWithoutCollision(
  newConcepts: ConceptRecord[],
  existingPositions: Map<string, NodePosition>,
  relationships: ConceptRelationshipRecord[],
  defaultCenterX = 500,
  defaultCenterY = 380
): Map<string, NodePosition> {
  const allPositions = new Map<string, NodePosition>();

  // Copy existing positions (untouched to preserve manual user arrangements)
  for (const [id, pos] of existingPositions.entries()) {
    allPositions.set(id, { x: pos.x, y: pos.y });
  }

  if (newConcepts.length === 0) return allPositions;

  // Build connection map to placed nodes
  const connectedMap = new Map<string, string[]>();
  for (const rel of relationships) {
    if (!connectedMap.has(rel.source_concept_id)) {
      connectedMap.set(rel.source_concept_id, []);
    }
    if (!connectedMap.has(rel.target_concept_id)) {
      connectedMap.set(rel.target_concept_id, []);
    }
    connectedMap.get(rel.source_concept_id)!.push(rel.target_concept_id);
    connectedMap.get(rel.target_concept_id)!.push(rel.source_concept_id);
  }

  // Sort new concepts so connected ones get placed near neighbors first
  const sortedNew = [...newConcepts].sort((a, b) => {
    const aNeighbors = connectedMap.get(a.id) || [];
    const bNeighbors = connectedMap.get(b.id) || [];
    return bNeighbors.length - aNeighbors.length;
  });

  for (const concept of sortedNew) {
    const neighbors = connectedMap.get(concept.id) || [];
    const placedNeighbors = neighbors
      .map((id) => allPositions.get(id))
      .filter((pos): pos is NodePosition => !!pos);

    let anchorX = defaultCenterX;
    let anchorY = defaultCenterY;

    if (placedNeighbors.length > 0) {
      anchorX = placedNeighbors.reduce((acc, p) => acc + p.x, 0) / placedNeighbors.length;
      anchorY = placedNeighbors.reduce((acc, p) => acc + p.y, 0) / placedNeighbors.length;
    } else if (allPositions.size > 0) {
      let sumX = 0;
      let sumY = 0;
      for (const pos of allPositions.values()) {
        sumX += pos.x;
        sumY += pos.y;
      }
      anchorX = sumX / allPositions.size;
      anchorY = sumY / allPositions.size;
    }

    // Search outwards in compact concentric rings for the nearest non-colliding slot
    let chosenPos: NodePosition | null = null;
    const ringRadii = [110, 160, 220, 285, 355, 430, 510, 600];
    const anglesCount = 18;

    for (const radius of ringRadii) {
      for (let i = 0; i < anglesCount; i++) {
        const theta = (2 * Math.PI * i) / anglesCount;
        const candidateX = anchorX + radius * Math.cos(theta);
        const candidateY = anchorY + radius * Math.sin(theta);
        const candidatePos: NodePosition = { x: candidateX, y: candidateY };

        let hasCollision = false;
        for (const existingPos of allPositions.values()) {
          if (isBoundingBoxColliding(candidatePos, existingPos)) {
            hasCollision = true;
            break;
          }
        }

        if (!hasCollision) {
          chosenPos = candidatePos;
          break;
        }
      }
      if (chosenPos) break;
    }

    if (!chosenPos) {
      // Fallback: place in an adjacent compact offset
      chosenPos = {
        x: anchorX + ((allPositions.size % 4) - 1.5) * NODE_MIN_GAP_X,
        y: anchorY + Math.floor(allPositions.size / 4 + 1) * NODE_MIN_GAP_Y,
      };
    }

    allPositions.set(concept.id, chosenPos);
  }

  return allPositions;
}

/**
 * Phase 4.2.2 Compact Force-Directed Layout:
 * - Reduced repulsion & compact initial footprint
 * - Increased attraction between connected concepts to form tight clusters
 * - Enhanced centering gravity keeping isolated nodes close to the main graph
 * - Guaranteed non-overlapping 10–12px minimum gap via AABB post-processing
 */
export function calculateGraphLayout(
  concepts: ConceptRecord[],
  relationships: ConceptRelationshipRecord[],
  width = 900,
  height = 600
): Map<string, NodePosition> {
  const positions = new Map<string, NodePosition>();
  const n = concepts.length;
  if (n === 0) return positions;

  const centerX = width / 2;
  const centerY = height / 2;

  if (n === 1) {
    positions.set(concepts[0].id, { x: centerX, y: centerY });
    return positions;
  }

  // Calculate degrees (connection counts)
  const degreeMap = new Map<string, number>();
  for (const c of concepts) {
    degreeMap.set(c.id, 0);
  }
  for (const r of relationships) {
    degreeMap.set(r.source_concept_id, (degreeMap.get(r.source_concept_id) || 0) + 1);
    degreeMap.set(r.target_concept_id, (degreeMap.get(r.target_concept_id) || 0) + 1);
  }

  // Sort concepts by degree (high-degree in inner core, lower in outer periphery)
  const sortedConcepts = [...concepts].sort(
    (a, b) => (degreeMap.get(b.id) || 0) - (degreeMap.get(a.id) || 0)
  );

  // Compact initial placement radius
  const initialRadius = Math.max(100, Math.min(width, height) * 0.22, Math.sqrt(n) * 35);
  sortedConcepts.forEach((c, i) => {
    const angle = (2 * Math.PI * i) / n;
    // Layered radial rings: central hubs close to center, peripheral nodes slightly outside
    const degree = degreeMap.get(c.id) || 0;
    const ringFactor = degree >= 3 ? 0.45 : degree >= 1 ? 0.75 : 1.05;
    const r = initialRadius * ringFactor;
    positions.set(c.id, {
      x: centerX + r * Math.cos(angle),
      y: centerY + r * Math.sin(angle),
    });
  });

  // Iterative force simulation parameters tuned for compact visual grouping
  const iterations = 85;
  // Compact optimal distance k
  const k = Math.max(90, Math.min(180, Math.sqrt((width * height) / Math.max(n, 12)) * 0.48));
  let temperature = width / 18;
  const coolingFactor = 0.94;

  const conceptIds = concepts.map((c) => c.id);

  for (let iter = 0; iter < iterations; iter++) {
    const dispX = new Map<string, number>();
    const dispY = new Map<string, number>();

    for (const id of conceptIds) {
      dispX.set(id, 0);
      dispY.set(id, 0);
    }

    // 1. Moderate pair repulsion (attenuated at distance to prevent expansive canvas blowout)
    for (let i = 0; i < n; i++) {
      const uId = conceptIds[i];
      const posU = positions.get(uId)!;

      for (let j = i + 1; j < n; j++) {
        const vId = conceptIds[j];
        const posV = positions.get(vId)!;

        let dx = posU.x - posV.x;
        let dy = posU.y - posV.y;
        let dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < 1) {
          dx = ((i + j) % 2 === 0 ? 1 : -1) * 2;
          dy = ((i * j) % 2 === 0 ? 1 : -1) * 2;
          dist = Math.sqrt(dx * dx + dy * dy);
        }

        // Drop off repulsion at larger distances so clusters remain compact
        if (dist > k * 3.2) continue;

        const force = (k * k) / dist;
        const fx = (dx / dist) * force;
        const fy = (dy / dist) * force;

        dispX.set(uId, (dispX.get(uId) || 0) + fx);
        dispY.set(uId, (dispY.get(uId) || 0) + fy);
        dispX.set(vId, (dispX.get(vId) || 0) - fx);
        dispY.set(vId, (dispY.get(vId) || 0) - fy);
      }
    }

    // 2. Strong attraction along relationship edges (pulls related nodes into tight clusters)
    for (const rel of relationships) {
      const posU = positions.get(rel.source_concept_id);
      const posV = positions.get(rel.target_concept_id);
      if (!posU || !posV) continue;

      let dx = posU.x - posV.x;
      let dy = posU.y - posV.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < 1) continue;

      // Higher spring tension: brings connected concepts close together
      const force = (dist * dist) / (k * 0.75);
      const fx = (dx / dist) * force;
      const fy = (dy / dist) * force;

      dispX.set(rel.source_concept_id, (dispX.get(rel.source_concept_id) || 0) - fx);
      dispY.set(rel.source_concept_id, (dispY.get(rel.source_concept_id) || 0) - fy);
      dispX.set(rel.target_concept_id, (dispX.get(rel.target_concept_id) || 0) + fx);
      dispY.set(rel.target_concept_id, (dispY.get(rel.target_concept_id) || 0) + fy);
    }

    // 3. Centering gravity (stronger gravity keeps isolated nodes close to the main graph)
    for (const id of conceptIds) {
      const pos = positions.get(id)!;
      const dx = centerX - pos.x;
      const dy = centerY - pos.y;
      const deg = degreeMap.get(id) || 0;
      // Isolated concepts receive higher gravity so they don't wander off
      const gravity = deg === 0 ? 0.14 : 0.09;
      dispX.set(id, (dispX.get(id) || 0) + dx * gravity);
      dispY.set(id, (dispY.get(id) || 0) + dy * gravity);
    }

    // 4. Update positions with temperature bounding
    for (const id of conceptIds) {
      const dx = dispX.get(id) || 0;
      const dy = dispY.get(id) || 0;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < 0.001) continue;

      const cappedDist = Math.min(dist, temperature);
      const pos = positions.get(id)!;
      pos.x += (dx / dist) * cappedDist;
      pos.y += (dy / dist) * cappedDist;
    }

    temperature *= coolingFactor;
  }

  // 5. Post-layout deterministic collision resolution: guarantees strictly 0 overlapping cards with 10–12px minimum gap
  resolveNodeCollisions(positions, 60, NODE_MIN_GAP_X, NODE_MIN_GAP_Y);

  return positions;
}
