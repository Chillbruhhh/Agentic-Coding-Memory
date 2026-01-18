import { useState, useEffect } from 'react';

export interface AnalyticsData {
  totalObjects: number;
  totalRelationships: number;
  objectsByType: Record<string, number>;
  languageDistribution: Record<string, number>;
  recentActivity: Array<{
    id: string;
    type: string;
    action: string;
    timestamp: string;
    details: string;
  }>;
  systemMetrics: {
    memoryUsage: number;
    cpuUsage: number;
    diskUsage: number;
    uptime: string;
  };
  indexingStats: {
    filesIndexed: number;
    symbolsExtracted: number;
    lastIndexTime: string;
    indexingSpeed: string;
  };
}

export const useAnalytics = () => {
  const [analytics, setAnalytics] = useState<AnalyticsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchAnalytics = async () => {
    try {
      setLoading(true);
      setError(null);

      // Try to fetch real analytics data
      try {
        const response = await fetch('http://localhost:8105/v1/analytics');
        if (response.ok) {
          const data = await response.json();
          setAnalytics(data);
          return;
        }
      } catch (fetchError) {
        console.log('Using mock analytics - server not available');
      }

      // Fallback to mock analytics data
      const mockAnalytics: AnalyticsData = {
        totalObjects: 931,
        totalRelationships: 924,
        objectsByType: {
          'Symbol': 687,
          'Decision': 12,
          'ChangeSet': 45,
          'Run': 187
        },
        languageDistribution: {
          'TypeScript': 342,
          'Rust': 289,
          'Markdown': 156,
          'JSON': 89,
          'TOML': 34,
          'YAML': 21
        },
        recentActivity: [
          {
            id: '1',
            type: 'Symbol',
            action: 'Created',
            timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
            details: 'KnowledgeGraph component indexed'
          },
          {
            id: '2',
            type: 'ChangeSet',
            action: 'Updated',
            timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
            details: 'UI components refactored'
          },
          {
            id: '3',
            type: 'Decision',
            action: 'Created',
            timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
            details: 'Industrial cyberpunk theme adopted'
          },
          {
            id: '4',
            type: 'Symbol',
            action: 'Created',
            timestamp: new Date(Date.now() - 45 * 60 * 1000).toISOString(),
            details: 'Analytics component implemented'
          },
          {
            id: '5',
            type: 'Run',
            action: 'Executed',
            timestamp: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
            details: 'Full codebase indexing completed'
          }
        ],
        systemMetrics: {
          memoryUsage: 67.3,
          cpuUsage: 23.8,
          diskUsage: 45.2,
          uptime: '2h 34m'
        },
        indexingStats: {
          filesIndexed: 156,
          symbolsExtracted: 931,
          lastIndexTime: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
          indexingSpeed: '47 files/sec'
        }
      };

      setAnalytics(mockAnalytics);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMsg);
      console.error('Failed to fetch analytics:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAnalytics();
    
    // Refresh analytics every 30 seconds
    const interval = setInterval(fetchAnalytics, 30000);
    return () => clearInterval(interval);
  }, []);

  const refetch = () => {
    fetchAnalytics();
  };

  return {
    analytics,
    loading,
    error,
    refetch
  };
};
