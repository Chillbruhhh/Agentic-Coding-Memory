import { useCallback, useEffect, useState } from 'react';

export interface Connection {
  connection_id: string;
  agent_id: string;
  agent_name: string;
  run_id?: string;
  project_id?: string;
  status: 'connected' | 'disconnected';
  last_heartbeat: string;
  connected_at: string;
  expires_at: string;
}

export const useConnections = (pollInterval = 5000) => {
  const [connections, setConnections] = useState<Connection[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasLoaded, setHasLoaded] = useState(false);

  const fetchConnections = useCallback(async () => {
    try {
      if (!hasLoaded) {
        setLoading(true);
      }
      setError(null);

      const response = await fetch('http://localhost:8105/v1/connections');

      if (!response.ok) {
        throw new Error(`AMP server error: ${response.status} ${response.statusText}`);
      }

      const data: Connection[] = await response.json();

      // Filter to only show active connections (expires_at > now)
      const now = new Date();
      const activeConnections = data.filter(conn => {
        if (!conn.expires_at) return false;
        const expiresAt = new Date(conn.expires_at);
        return expiresAt > now && conn.status === 'connected';
      });

      setConnections(activeConnections);
      if (!hasLoaded) {
        setHasLoaded(true);
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to load connections';
      setError(errorMsg);
    } finally {
      if (!hasLoaded) {
        setLoading(false);
      }
    }
  }, [hasLoaded]);

  useEffect(() => {
    fetchConnections();
    const interval = setInterval(fetchConnections, pollInterval);
    return () => clearInterval(interval);
  }, [fetchConnections, pollInterval]);

  // Normalize run_id for comparison (handle "objects:" prefix variations)
  const normalizeRunId = (id: string | undefined): string => {
    if (!id) return '';
    return id.replace(/^objects:/, '').replace(/[`⟨⟩]/g, '');
  };

  // Helper to check if a specific run has an active connection
  const isRunConnected = useCallback(
    (runId: string): boolean => {
      const normalizedRunId = normalizeRunId(runId);
      return connections.some(conn => normalizeRunId(conn.run_id) === normalizedRunId);
    },
    [connections]
  );

  // Get connection info for a specific run
  const getConnectionForRun = useCallback(
    (runId: string): Connection | undefined => {
      const normalizedRunId = normalizeRunId(runId);
      return connections.find(conn => normalizeRunId(conn.run_id) === normalizedRunId);
    },
    [connections]
  );

  return {
    connections,
    loading,
    error,
    refetch: fetchConnections,
    isRunConnected,
    getConnectionForRun,
  };
};
