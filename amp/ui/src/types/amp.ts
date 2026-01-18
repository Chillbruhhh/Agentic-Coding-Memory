export interface AmpObject {
  id: string;
  type: 'project' | 'directory' | 'file' | 'symbol';
  name: string;
  path?: string;
  kind?: string;
  language?: string;
  signature?: string;
  documentation?: string;
  created_at: string;
  updated_at: string;
}

export interface AmpRelationship {
  id: string;
  type: 'defined_in' | 'depends_on' | 'calls' | 'justified_by' | 'modifies' | 'implements' | 'produced';
  source_id: string;
  target_id: string;
  created_at: string;
}

export interface AmpQueryResponse {
  results: AmpObject[];
  total_count: number;
  trace_id: string;
}

export interface GraphNode extends AmpObject {
  x: number;
  y: number;
  z: number;
  children?: GraphNode[];
  parent?: GraphNode;
  collapsed?: boolean;
  level: number;
}

export interface GraphEdge {
  source: string;
  target: string;
  type: string;
}
