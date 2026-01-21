import { useCallback, useEffect, useState } from 'react';

export interface RunSummary {
  id: string;
  project_id?: string;
  tenant_id?: string;
  created_at?: string;
  updated_at?: string;
  provenance?: {
    agent?: string;
    summary?: string;
  };
  input_summary?: string;
  status?: string;
  duration_ms?: number;
  confidence?: number;
}

export interface RunDetail extends RunSummary {
  outputs?: Array<{
    type?: string;
    content?: string;
    metadata?: any;
  }>;
  errors?: Array<{
    message?: string;
    code?: string;
    context?: any;
  }>;
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

const normalizeRun = (raw: any): RunSummary => ({
  id: raw.id,
  project_id: raw.project_id,
  tenant_id: raw.tenant_id,
  created_at: raw.created_at,
  updated_at: raw.updated_at,
  provenance: raw.provenance,
  input_summary: raw.input_summary,
  status: raw.status,
  duration_ms: typeof raw.duration_ms === 'number' ? raw.duration_ms : undefined,
  confidence: typeof raw.confidence === 'number' ? raw.confidence : undefined
});

export const useRuns = () => {
  const [runs, setRuns] = useState<RunSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasLoaded, setHasLoaded] = useState(false);

  const fetchRuns = useCallback(async () => {
    try {
      if (!hasLoaded) {
        setLoading(true);
      }
      setError(null);

      const response = await fetch('http://localhost:8105/v1/query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          limit: 200,
          filters: {
            type: ['Run', 'run']
          }
        })
      });

      if (!response.ok) {
        throw new Error(`AMP server error: ${response.status} ${response.statusText}`);
      }

      const payload = await response.json();
      const objects = extractObjects(payload);
      const normalized = objects
        .filter((obj: any) => (obj.type || '').toLowerCase() === 'run')
        .map(normalizeRun)
        .sort((a, b) => {
          const aTime = new Date(a.created_at || 0).getTime();
          const bTime = new Date(b.created_at || 0).getTime();
          return bTime - aTime;
        });

      setRuns(normalized);
      if (!hasLoaded) {
        setHasLoaded(true);
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to load runs';
      setError(errorMsg);
    } finally {
      if (!hasLoaded) {
        setLoading(false);
      }
    }
  }, [hasLoaded]);

  const fetchRunDetails = useCallback(async (id: string): Promise<RunDetail | null> => {
    try {
      const response = await fetch(`http://localhost:8105/v1/objects/${id}`);
      if (!response.ok) {
        throw new Error(`Failed to load run ${id}`);
      }
      const detail = await response.json();
      return detail as RunDetail;
    } catch (err) {
      console.error('Failed to load run detail:', err);
      return null;
    }
  }, []);

  useEffect(() => {
    fetchRuns();
    const interval = setInterval(fetchRuns, 5000);
    return () => clearInterval(interval);
  }, [fetchRuns]);

  return {
    runs,
    loading,
    error,
    refetch: fetchRuns,
    fetchRunDetails
  };
};
