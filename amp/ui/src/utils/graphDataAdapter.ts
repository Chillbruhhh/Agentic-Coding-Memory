// Graph data transformation utilities for AMP Console
export interface AmpObject {
  id: string;
  type: string;
  kind?: string;
  name: string;
  path: string;
  language: string;
}

export interface AmpRelationship {
  in: string;
  out: string;
  relation_type?: string;
}

export interface GraphNode {
  id: string;
  name: string;
  kind: string;
  path: string;
  language: string;
  val: number;
  color: string;
  x?: number;
  y?: number;
  z?: number;
}

export interface GraphLink {
  source: string | { id: string; [key: string]: any };
  target: string | { id: string; [key: string]: any };
  type: string;
  value: number;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

// Symbol type to size mapping
export const getNodeSize = (kind: string): number => {
  switch (kind) {
    case 'function': return 8;
    case 'class': return 12;
    case 'method': return 6;
    case 'variable': return 4;
    case 'interface': return 10;
    case 'file': return 15;
    case 'directory': return 20;
    case 'note': return 9;
    case 'decision': return 11;
    case 'changeset': return 10;
    case 'artifact_core': return 14;
    default: return 5;
  }
};

// Symbol type to color mapping (cyberpunk theme)
export const getNodeColor = (kind: string): string => {
  switch (kind) {
    case 'function': return '#3b82f6'; // Blue
    case 'class': return '#ef4444'; // Red (primary)
    case 'method': return '#10b981'; // Green
    case 'variable': return '#f59e0b'; // Yellow
    case 'interface': return '#8b5cf6'; // Purple
    case 'file': return '#94a3b8'; // Soft slate
    case 'directory': return '#38bdf8'; // Sky blue
    case 'project': return '#ef4444'; // Bright red (repo core)
    case 'note': return '#6366f1'; // Indigo
    case 'decision': return '#7c3aed'; // Violet
    case 'changeset': return '#8b5cf6'; // Purple
    case 'artifact_core': return '#4f46e5'; // Indigo (core)
    default: return '#64748b'; // Slate
  }
};

// Transform AMP objects to 3d-force-graph format
export const transformAmpToGraph = (
  objects: AmpObject[],
  relationships: AmpRelationship[] = []
): GraphData => {
  // Filter for code symbols AND files/projects to show the full hierarchy
  const codeSymbolKinds = ['function', 'class', 'method', 'variable', 'interface'];
  const allowedTypes = ['symbol', 'Symbol', 'file', 'File', 'note', 'decision', 'changeset', 'artifact_core'];
  const allowedKinds = [...codeSymbolKinds, 'file', 'project', 'directory', 'note', 'decision', 'changeset', 'artifact_core'];

  const normalizeId = (value: string) =>
    value
      .replace(/^objects:/, '')
      .replace(/[⟨⟩]/g, '')
      .replace(/[`]/g, '')
      .replace(/[\u27E8\u27E9]/g, '');

  const getDisplayName = (obj: AmpObject) => {
    const rawName = obj.name || (obj as any).title || (obj.kind || obj.type || 'artifact');
    const kind = (obj.kind || obj.type || '').toLowerCase();
    const path = obj.path;
    if (path && (kind === 'file' || kind === 'directory')) {
      const normalized = path.replace(/\\/g, '/');
      const parts = normalized.split('/').filter(Boolean);
      if (parts.length > 0) {
        return parts[parts.length - 1];
      }
    }
    if (typeof rawName === 'string') {
      const normalized = rawName.replace(/\\/g, '/');
      if (normalized.includes('/')) {
        const parts = normalized.split('/').filter(Boolean);
        if (parts.length > 0) {
          return parts[parts.length - 1];
        }
      }
    }
    return rawName;
  };

  const nodes: GraphNode[] = objects
    .filter(obj => {
      const kind = (obj.kind || obj.type || '').toLowerCase();
      const type = (obj.type || '').toLowerCase();
      return allowedTypes.includes(type) && allowedKinds.includes(kind);
    })
    .map(obj => ({
      id: normalizeId(obj.id), // Normalize to match relationship format
      name: getDisplayName(obj),
      kind: (obj.kind || obj.type) as string,
      path: obj.path,
      language: obj.language,
      val: getNodeSize((obj.kind || obj.type) as string),
      color: getNodeColor((obj.kind || obj.type) as string)
    }));

  const nodeById = new Map(nodes.map(node => [node.id, node]));
  const fileHasDirParent = new Set<string>();
  relationships.forEach(rel => {
    const relType = (rel.relation_type || '').toLowerCase();
    if (relType !== 'defined_in') return;
    const sourceId = rel.in;
    const targetId = rel.out;
    const source = nodeById.get(sourceId);
    const target = nodeById.get(targetId);
    if (!source || !target) return;
    const sourceKind = (source.kind || '').toLowerCase();
    const targetKind = (target.kind || '').toLowerCase();
    if (sourceKind === 'directory' && targetKind === 'file') {
      fileHasDirParent.add(target.id);
    } else if (sourceKind === 'file' && targetKind === 'directory') {
      fileHasDirParent.add(source.id);
    }
  });

  const links: GraphLink[] = relationships
    .filter(rel => {
      // Only include links between existing nodes (use Map for O(1) lookup)
      const sourceNode = nodeById.get(rel.in);
      const targetNode = nodeById.get(rel.out);
      if (!sourceNode || !targetNode) return false;

      const relType = (rel.relation_type || '').toLowerCase();
      const sourceKind = (sourceNode.kind || '').toLowerCase();
      const targetKind = (targetNode.kind || '').toLowerCase();

      if (relType === 'defined_in') {
        const isProjectFile =
          (sourceKind === 'project' && targetKind === 'file') ||
          (sourceKind === 'file' && targetKind === 'project');
        if (isProjectFile) {
          const fileId = sourceKind === 'file' ? sourceNode.id : targetNode.id;
          if (fileHasDirParent.has(fileId)) return false;
        }
      }

      return true;
    })
    .map(rel => ({
      source: rel.in,   // 3d-force-graph expects 'source'
      target: rel.out,  // 3d-force-graph expects 'target'
      type: rel.relation_type || 'defined_in',
      value: 1
    }));

  return { nodes, links };
};

// Get symbol type statistics
export const getSymbolStats = (nodes: GraphNode[]) => {
  const stats = nodes.reduce((acc, node) => {
    acc[node.kind] = (acc[node.kind] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  return Object.entries(stats).map(([kind, count]) => ({
    kind,
    count,
    color: getNodeColor(kind)
  }));
};

