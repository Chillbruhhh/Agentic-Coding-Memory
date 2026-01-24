import { useCallback, useEffect, useState } from 'react';
import { Artifact, ArtifactType, ArtifactFilters, MemoryLayers } from '../types/amp';

export interface ArtifactSummary {
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
  // Preview fields
  preview?: string;
}

const extractObjects = (payload: any): any[] => {
  if (payload?.results && Array.isArray(payload.results)) {
    return payload.results.map((result: any) => result.object || result);
  }
  if (Array.isArray(payload)) {
    return payload;
  }
  return [];
};

const normalizeArtifact = (raw: any): ArtifactSummary => {
  // Determine artifact type from the raw object
  const type = (raw.type || raw.artifact_type || 'note').toLowerCase() as ArtifactType;

  // Generate preview based on type (agent-authored artifacts only)
  let preview = '';
  switch (type) {
    case 'decision':
      preview = raw.decision || raw.context || '';
      break;
    case 'note':
      preview = raw.content || '';
      break;
    case 'changeset':
      preview = raw.description || '';
      break;
  }

  // Truncate preview
  if (preview.length > 150) {
    preview = preview.substring(0, 147) + '...';
  }

  return {
    id: raw.id,
    type,
    title: raw.title || raw.name || `${type} artifact`,
    created_at: raw.created_at || '',
    updated_at: raw.updated_at || raw.created_at || '',
    agent_id: raw.agent_id || raw.provenance?.agent,
    run_id: raw.run_id,
    project_id: raw.project_id,
    memory_layers: raw.memory_layers || {
      graph: true,
      vector: Boolean(raw.embedding || raw.embeddings),
      temporal: Boolean(raw.created_at)
    },
    tags: raw.tags || [],
    preview
  };
};

export const useArtifacts = (filters?: ArtifactFilters) => {
  const [artifacts, setArtifacts] = useState<ArtifactSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasLoaded, setHasLoaded] = useState(false);

  const fetchArtifacts = useCallback(async () => {
    try {
      if (!hasLoaded) {
        setLoading(true);
      }
      setError(null);

      // Build filter for artifact types (excludes filelog - those are indexer output, not agent-authored)
      const typeFilters = filters?.type || ['decision', 'note', 'changeset', 'Decision', 'Note', 'ChangeSet', 'Changeset'];

      const response = await fetch('http://localhost:8105/v1/query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          limit: 500,
          filters: {
            type: typeFilters,
            ...(filters?.project_id && { project_id: filters.project_id }),
            ...(filters?.agent_id && { agent_id: filters.agent_id }),
            ...(filters?.run_id && { run_id: filters.run_id })
          }
        })
      });

      if (!response.ok) {
        throw new Error(`AMP server error: ${response.status} ${response.statusText}`);
      }

      const payload = await response.json();
      const objects = extractObjects(payload);

      // Filter and normalize artifacts (agent-authored only, no filelogs)
      const artifactTypes = ['decision', 'note', 'changeset'];
      const normalized = objects
        .filter((obj: any) => {
          const objType = (obj.type || '').toLowerCase();
          return artifactTypes.includes(objType);
        })
        .map(normalizeArtifact)
        .sort((a, b) => {
          const aTime = new Date(a.created_at || 0).getTime();
          const bTime = new Date(b.created_at || 0).getTime();
          return bTime - aTime;
        });

      setArtifacts(normalized);
      if (!hasLoaded) {
        setHasLoaded(true);
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to load artifacts';
      setError(errorMsg);
    } finally {
      if (!hasLoaded) {
        setLoading(false);
      }
    }
  }, [hasLoaded, filters]);

  const fetchArtifactDetails = useCallback(async (id: string): Promise<Artifact | null> => {
    try {
      const response = await fetch(`http://localhost:8105/v1/objects/${id}`);
      if (!response.ok) {
        throw new Error(`Failed to load artifact ${id}`);
      }
      const detail = await response.json();
      return detail as Artifact;
    } catch (err) {
      console.error('Failed to load artifact detail:', err);
      return null;
    }
  }, []);

  const deleteArtifact = useCallback(async (id: string): Promise<boolean> => {
    try {
      const response = await fetch(`http://localhost:8105/v1/artifacts/${id}`, {
        method: 'DELETE'
      });
      if (!response.ok) {
        throw new Error(`Failed to delete artifact ${id}`);
      }
      setArtifacts(prev => prev.filter(artifact => artifact.id !== id));
      return true;
    } catch (err) {
      console.error('Failed to delete artifact:', err);
      setError(err instanceof Error ? err.message : 'Failed to delete artifact');
      return false;
    }
  }, []);

  // Get counts by type (agent-authored artifacts only)
  const getTypeCounts = useCallback(() => {
    const counts: Record<string, number> = {
      decision: 0,
      note: 0,
      changeset: 0
    };
    artifacts.forEach(a => {
      if (counts[a.type] !== undefined) {
        counts[a.type]++;
      }
    });
    return counts;
  }, [artifacts]);

  // Get counts by memory layer
  const getLayerCounts = useCallback(() => {
    return {
      graph: artifacts.filter(a => a.memory_layers.graph).length,
      vector: artifacts.filter(a => a.memory_layers.vector).length,
      temporal: artifacts.filter(a => a.memory_layers.temporal).length
    };
  }, [artifacts]);

  useEffect(() => {
    fetchArtifacts();
    const interval = setInterval(fetchArtifacts, 5000);
    return () => clearInterval(interval);
  }, [fetchArtifacts]);

  return {
    artifacts,
    loading,
    error,
    refetch: fetchArtifacts,
    fetchArtifactDetails,
    deleteArtifact,
    getTypeCounts,
    getLayerCounts
  };
};
