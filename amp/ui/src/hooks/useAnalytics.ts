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
  requestLatency: {
    p99: number;
    p95: number;
    p50: number;
    avg: number;
    dataPoints: Array<{
      timestamp: string;
      latency: number;
    }>;
  };
  errorDistribution: Array<{
    label: string;
    count: number;
    percent: number;
    color: string;
  }>;
  systemEvents: Array<{
    time: string;
    event: string;
    origin: string;
    status: string;
    alert: boolean;
  }>;
}

export const useAnalytics = () => {
  const [analytics, setAnalytics] = useState<AnalyticsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [timeInterval, setTimeInterval] = useState<string>('1h');

  const normalizeAnalytics = (payload: any): AnalyticsData => {
    const requestLatency = payload.requestLatency || {};
    const dataPoints = requestLatency.dataPoints || requestLatency.data_points || [];

    return {
      totalObjects: payload.totalObjects ?? 0,
      totalRelationships: payload.totalRelationships ?? 0,
      objectsByType: payload.objectsByType || {},
      languageDistribution: payload.languageDistribution || {},
      recentActivity: payload.recentActivity || [],
      systemMetrics: payload.systemMetrics || {
        memoryUsage: 0,
        cpuUsage: 0,
        diskUsage: 0,
        uptime: '',
      },
      indexingStats: payload.indexingStats || {
        filesIndexed: 0,
        symbolsExtracted: 0,
        lastIndexTime: '',
        indexingSpeed: '',
      },
      requestLatency: {
        p99: requestLatency.p99 ?? 0,
        p95: requestLatency.p95 ?? 0,
        p50: requestLatency.p50 ?? 0,
        avg: requestLatency.avg ?? 0,
        dataPoints,
      },
      errorDistribution: payload.errorDistribution || [],
      systemEvents: payload.systemEvents || [],
    };
  };

  const fetchAnalytics = async () => {
    try {
      setError(null);

      const response = await fetch('http://localhost:8105/v1/analytics');
      if (!response.ok) {
        throw new Error(`Server responded with status: ${response.status}`);
      }

      const payload = await response.json();
      setAnalytics(normalizeAnalytics(payload));
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to fetch analytics data';
      setError(errorMsg);
      console.error('Failed to fetch analytics:', err);
    }
  };

  useEffect(() => {
    // Initial load only - no intervals to prevent flickering
    const initialLoad = async () => {
      setLoading(true);
      await fetchAnalytics();
      setLoading(false);
    };
    
    initialLoad();
  }, [timeInterval]);

  const changeTimeInterval = (interval: string) => {
    setTimeInterval(interval);
    // Refresh data immediately when interval changes
    fetchAnalytics();
  };

  return {
    analytics,
    loading,
    error,
    timeInterval,
    setTimeInterval: changeTimeInterval,
  };
};
