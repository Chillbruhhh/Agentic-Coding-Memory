// Graph data transformation utilities for AMP Console
export interface AmpObject {
  id: string;
  type: string;
  kind: string;
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
  source: string;
  target: string;
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
    case 'file': return '#6b7280'; // Gray
    case 'directory': return '#4b5563'; // Dark gray
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
  const allowedTypes = ['symbol', 'Symbol']; // Support both cases
  const allowedKinds = [...codeSymbolKinds, 'file', 'project', 'directory'];
  
  const nodes: GraphNode[] = objects
    .filter(obj => allowedTypes.includes(obj.type) && allowedKinds.includes(obj.kind))
    .map(obj => ({
      id: obj.id.replace(/[⟨⟩]/g, ''), // Remove brackets to match relationship format
      name: obj.name,
      kind: obj.kind,
      path: obj.path,
      language: obj.language,
      val: getNodeSize(obj.kind),
      color: getNodeColor(obj.kind)
    }));

  const links: GraphLink[] = relationships
    .filter(rel => {
      // Only include links between existing nodes
      const sourceExists = nodes.some(n => n.id === rel.in);
      const targetExists = nodes.some(n => n.id === rel.out);
      
      // Debug: check what the missing source/target objects are
      if (!sourceExists) {
        const sourceObj = objects.find(obj => obj.id.replace(/[⟨⟩]/g, '') === rel.in);
        console.log('Missing source object:', sourceObj?.kind, sourceObj?.type, sourceObj?.name);
      }
      if (!targetExists) {
        const targetObj = objects.find(obj => obj.id.replace(/[⟨⟩]/g, '') === rel.out);
        console.log('Missing target object:', targetObj?.kind, targetObj?.type, targetObj?.name);
      }
      
      return sourceExists && targetExists;
    })
    .map(rel => ({
      source: rel.in,   // 3d-force-graph expects 'source'
      target: rel.out,  // 3d-force-graph expects 'target'
      type: rel.relation_type || 'defined_in',
      value: 1
    }));

  console.log(`Created ${links.length} links from ${relationships.length} relationships`);
  console.log('Sample nodes:', nodes.slice(0, 3).map(n => ({ id: n.id, name: n.name })));
  console.log('Sample relationships:', relationships.slice(0, 3));
  console.log('Sample links:', links.slice(0, 3));
  console.log('Node ID format:', nodes[0]?.id);
  console.log('Link source format:', links[0]?.source);

  // Force add a test link if we have nodes but no links
  if (nodes.length >= 2 && links.length === 0) {
    console.log('Adding test link between first two nodes');
    links.push({
      source: nodes[0].id,
      target: nodes[1].id,
      type: 'test',
      value: 1
    });
  }

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
