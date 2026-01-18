import { useState, useEffect } from 'react';

export interface FileNode {
  name: string;
  type: 'file' | 'folder';
  path: string;
  size?: string;
  modified?: string;
  children?: FileNode[];
  language?: string;
}

export const useFileSystem = () => {
  const [fileTree, setFileTree] = useState<FileNode[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchFileSystem = async () => {
    try {
      setLoading(true);
      setError(null);

      // Try to fetch real file system data
      try {
        const response = await fetch('http://localhost:8105/v1/files');
        if (response.ok) {
          const data = await response.json();
          setFileTree(data);
          return;
        }
      } catch (fetchError) {
        console.log('Using mock file system - server not available');
      }

      // Fallback to enhanced mock data matching your actual project structure
      const mockFileTree: FileNode[] = [
        {
          name: 'amp',
          type: 'folder',
          path: '/amp',
          children: [
            {
              name: 'server',
              type: 'folder',
              path: '/amp/server',
              children: [
                {
                  name: 'src',
                  type: 'folder',
                  path: '/amp/server/src',
                  children: [
                    { 
                      name: 'main.rs', 
                      type: 'file', 
                      path: '/amp/server/src/main.rs', 
                      size: '3.2 KB', 
                      modified: '2h ago',
                      language: 'rust'
                    },
                    {
                      name: 'handlers',
                      type: 'folder',
                      path: '/amp/server/src/handlers',
                      children: [
                        { 
                          name: 'objects.rs', 
                          type: 'file', 
                          path: '/amp/server/src/handlers/objects.rs', 
                          size: '4.8 KB', 
                          modified: '1h ago',
                          language: 'rust'
                        },
                        { 
                          name: 'relationships.rs', 
                          type: 'file', 
                          path: '/amp/server/src/handlers/relationships.rs', 
                          size: '2.1 KB', 
                          modified: '3h ago',
                          language: 'rust'
                        }
                      ]
                    },
                    {
                      name: 'models',
                      type: 'folder',
                      path: '/amp/server/src/models',
                      children: [
                        { 
                          name: 'mod.rs', 
                          type: 'file', 
                          path: '/amp/server/src/models/mod.rs', 
                          size: '1.2 KB', 
                          modified: '4h ago',
                          language: 'rust'
                        }
                      ]
                    }
                  ]
                },
                { 
                  name: 'Cargo.toml', 
                  type: 'file', 
                  path: '/amp/server/Cargo.toml', 
                  size: '1.8 KB', 
                  modified: '5h ago',
                  language: 'toml'
                }
              ]
            },
            {
              name: 'ui',
              type: 'folder',
              path: '/amp/ui',
              children: [
                {
                  name: 'src',
                  type: 'folder',
                  path: '/amp/ui/src',
                  children: [
                    { 
                      name: 'App.tsx', 
                      type: 'file', 
                      path: '/amp/ui/src/App.tsx', 
                      size: '2.9 KB', 
                      modified: 'now',
                      language: 'typescript'
                    },
                    {
                      name: 'components',
                      type: 'folder',
                      path: '/amp/ui/src/components',
                      children: [
                        { 
                          name: 'KnowledgeGraph.tsx', 
                          type: 'file', 
                          path: '/amp/ui/src/components/KnowledgeGraph.tsx', 
                          size: '8.9 KB', 
                          modified: '10m ago',
                          language: 'typescript'
                        },
                        { 
                          name: 'FileExplorer.tsx', 
                          type: 'file', 
                          path: '/amp/ui/src/components/FileExplorer.tsx', 
                          size: '12.1 KB', 
                          modified: '15m ago',
                          language: 'typescript'
                        },
                        { 
                          name: 'Analytics.tsx', 
                          type: 'file', 
                          path: '/amp/ui/src/components/Analytics.tsx', 
                          size: '11.6 KB', 
                          modified: '20m ago',
                          language: 'typescript'
                        },
                        { 
                          name: 'Header.tsx', 
                          type: 'file', 
                          path: '/amp/ui/src/components/Header.tsx', 
                          size: '3.5 KB', 
                          modified: '25m ago',
                          language: 'typescript'
                        },
                        { 
                          name: 'Sidebar.tsx', 
                          type: 'file', 
                          path: '/amp/ui/src/components/Sidebar.tsx', 
                          size: '2.6 KB', 
                          modified: '30m ago',
                          language: 'typescript'
                        }
                      ]
                    },
                    {
                      name: 'hooks',
                      type: 'folder',
                      path: '/amp/ui/src/hooks',
                      children: [
                        { 
                          name: 'useAmpData.ts', 
                          type: 'file', 
                          path: '/amp/ui/src/hooks/useAmpData.ts', 
                          size: '2.8 KB', 
                          modified: '5m ago',
                          language: 'typescript'
                        }
                      ]
                    }
                  ]
                },
                { 
                  name: 'package.json', 
                  type: 'file', 
                  path: '/amp/ui/package.json', 
                  size: '1.2 KB', 
                  modified: '1h ago',
                  language: 'json'
                },
                { 
                  name: 'DESIGN.md', 
                  type: 'file', 
                  path: '/amp/ui/DESIGN.md', 
                  size: '8.5 KB', 
                  modified: '30m ago',
                  language: 'markdown'
                }
              ]
            },
            {
              name: 'cli',
              type: 'folder',
              path: '/amp/cli',
              children: [
                {
                  name: 'src',
                  type: 'folder',
                  path: '/amp/cli/src',
                  children: [
                    { 
                      name: 'main.rs', 
                      type: 'file', 
                      path: '/amp/cli/src/main.rs', 
                      size: '2.4 KB', 
                      modified: '6h ago',
                      language: 'rust'
                    },
                    { 
                      name: 'indexer.rs', 
                      type: 'file', 
                      path: '/amp/cli/src/indexer.rs', 
                      size: '5.1 KB', 
                      modified: '6h ago',
                      language: 'rust'
                    }
                  ]
                }
              ]
            },
            { 
              name: 'README.md', 
              type: 'file', 
              path: '/amp/README.md', 
              size: '4.2 KB', 
              modified: '2h ago',
              language: 'markdown'
            },
            { 
              name: 'DEVLOG.md', 
              type: 'file', 
              path: '/amp/DEVLOG.md', 
              size: '18.3 KB', 
              modified: '1h ago',
              language: 'markdown'
            }
          ]
        }
      ];

      setFileTree(mockFileTree);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMsg);
      console.error('Failed to fetch file system:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchFileSystem();
  }, []);

  const refetch = () => {
    fetchFileSystem();
  };

  return {
    fileTree,
    loading,
    error,
    refetch
  };
};
