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

// Artifact types - created by agents during work
export type ArtifactType = 'decision' | 'filelog' | 'note' | 'changeset';

// Memory layer indicators for artifacts
export interface MemoryLayers {
  graph: boolean;      // Stored in knowledge graph with relationships
  vector: boolean;     // Has vector embeddings for semantic search
  temporal: boolean;   // Has temporal/time-series data
}

export interface ArtifactBase {
  id: string;
  type: ArtifactType;
  title: string;
  created_at: string;
  updated_at: string;
  agent_id?: string;
  run_id?: string;
  project_id?: string;
  memory_layers: MemoryLayers;
  tags?: string[];
}

// Decision artifact - ADR-style architectural decision record
export interface DecisionArtifact extends ArtifactBase {
  type: 'decision';
  context: string;           // Why was this decision needed?
  decision: string;          // What was decided?
  consequences: string;      // What are the implications?
  alternatives?: string[];   // What other options were considered?
  status?: 'proposed' | 'accepted' | 'deprecated' | 'superseded';
  linked_files?: string[];   // Files affected by this decision
}

// FileLog artifact - summary per file
export interface FileLogArtifact extends ArtifactBase {
  type: 'filelog';
  file_path: string;
  summary: string;           // High-level summary of the file
  symbols?: string[];        // Key symbols/functions in file
  dependencies?: string[];   // Files this depends on
  last_modified_by?: string; // Agent that last modified
  change_history?: {
    timestamp: string;
    agent_id: string;
    description: string;
  }[];
}

// Note artifact - freeform notes
export interface NoteArtifact extends ArtifactBase {
  type: 'note';
  content: string;           // Markdown content
  category?: 'insight' | 'todo' | 'question' | 'warning' | 'reference';
  linked_objects?: string[]; // IDs of related objects
}

// ChangeSet artifact - documents completed work
export interface ChangeSetArtifact extends ArtifactBase {
  type: 'changeset';
  description: string;       // What was changed
  diff_summary?: string;     // Summary of diffs
  files_changed: string[];   // List of modified files
  linked_decisions?: string[]; // Decision IDs that justified this change
}

// Union type for all artifacts
export type Artifact = DecisionArtifact | FileLogArtifact | NoteArtifact | ChangeSetArtifact;

// Artifact query/filter options
export interface ArtifactFilters {
  type?: ArtifactType[];
  project_id?: string;
  agent_id?: string;
  run_id?: string;
  tags?: string[];
  date_from?: string;
  date_to?: string;
  memory_layer?: 'graph' | 'vector' | 'temporal';
}
