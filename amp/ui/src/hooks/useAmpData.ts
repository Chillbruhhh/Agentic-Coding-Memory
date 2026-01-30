import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { AmpQueryResponse, AmpObject } from '../types/amp';

// Check if we're running in Tauri
const isTauri = typeof window !== 'undefined' && window.__TAURI_IPC__;

export const useAmpData = () => {
  const [data, setData] = useState<AmpObject[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      if (isTauri) {
        // Use Tauri IPC
        const response = await invoke<AmpQueryResponse>('get_amp_data');
        setData(response.results || []);
      } else {
        // Try direct HTTP call to AMP server
        try {
          const httpResponse = await fetch('http://localhost:8105/v1/objects');
          if (!httpResponse.ok) {
            throw new Error(`HTTP ${httpResponse.status}: ${httpResponse.statusText}`);
          }
          const objects = await httpResponse.json();
          setData(Array.isArray(objects) ? objects : []);
        } catch (fetchError) {
          // Fallback to enhanced mock data for demo
          console.log('Using enhanced mock data - AMP server not available');
          const mockData = [
            {
              id: '1',
              type: 'symbol' as const,
              name: 'main',
              path: 'amp/server/src/main.rs',
              language: 'rust',
              signature: 'fn main()',
              tenant_id: 'demo',
              project_id: 'amp',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              provenance: { source: 'tree-sitter', version: '1.0' }
            },
            {
              id: '2', 
              type: 'symbol' as const,
              name: 'KnowledgeGraph',
              path: 'amp/ui/src/components/KnowledgeGraph.tsx',
              language: 'typescript',
              signature: 'export const KnowledgeGraph: React.FC',
              tenant_id: 'demo',
              project_id: 'amp',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              provenance: { source: 'tree-sitter', version: '1.0' }
            },
            {
              id: '3',
              type: 'symbol' as const,
              name: 'Industrial Cyberpunk Theme',
              path: 'amp/ui/DESIGN.md',
              language: 'markdown',
              signature: 'UI Design System Choice',
              tenant_id: 'demo', 
              project_id: 'amp',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              provenance: { source: 'documentation', version: '1.0' }
            },
            {
              id: '4',
              type: 'symbol' as const,
              name: 'Professional UI Implementation',
              path: 'amp/ui/src/',
              language: 'typescript',
              signature: 'React + Tailwind + React Icons',
              tenant_id: 'demo',
              project_id: 'amp',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              provenance: { source: 'git', version: '1.0' }
            },
            {
              id: '5',
              type: 'symbol' as const,
              name: 'FileExplorer',
              path: 'amp/ui/src/components/FileExplorer.tsx',
              language: 'typescript',
              signature: 'export const FileExplorer: React.FC',
              tenant_id: 'demo',
              project_id: 'amp',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              provenance: { source: 'tree-sitter', version: '1.0' }
            },
            {
              id: '6',
              type: 'symbol' as const,
              name: 'Analytics',
              path: 'amp/ui/src/components/Analytics.tsx',
              language: 'typescript',
              signature: 'export const Analytics: React.FC',
              tenant_id: 'demo',
              project_id: 'amp',
              created_at: new Date().toISOString(),
              updated_at: new Date().toISOString(),
              provenance: { source: 'tree-sitter', version: '1.0' }
            }
          ];
          setData(mockData);
        }
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMsg);
      console.error('Failed to fetch AMP data:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const refetch = () => {
    fetchData();
  };

  return {
    data,
    loading,
    error,
    refetch
  };
};
