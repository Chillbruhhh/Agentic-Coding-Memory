import { useState, useEffect } from 'react';

export interface CodebaseProject {
  id: string;
  name: string;
  path: string;
  description?: string;
  language_stats: Record<string, number>;
  total_files: number;
  total_symbols: number;
  last_indexed: string;
  file_tree: FileNode[];
}

export interface FileNode {
  name: string;
  type: 'file' | 'folder';
  path: string;
  size?: string;
  modified?: string;
  children?: FileNode[];
  language?: string;
  symbols?: Array<{
    name: string;
    type: string;
    signature?: string;
  }>;
}

export const useCodebases = () => {
  const [codebases, setCodebases] = useState<CodebaseProject[]>([]);
  const [objects, setObjects] = useState<any[]>([]);
  const [relationships, setRelationships] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Helper function to build file tree from AMP objects
  const buildFileTreeFromObjects = (objects: any[]): FileNode[] => {
    const pathMap: Record<string, FileNode> = {};
    const rootNodes: FileNode[] = [];

    // First pass: create all file/folder nodes
    objects.forEach(obj => {
      if (obj.path) {
        const pathParts = obj.path.split('/').filter(Boolean);
        let currentPath = '';

        pathParts.forEach((part: string, index: number) => {
          const parentPath = currentPath;
          currentPath = currentPath ? `${currentPath}/${part}` : `/${part}`;

          if (!pathMap[currentPath]) {
            const isFile = index === pathParts.length - 1 && obj.type === 'Symbol';
            
            pathMap[currentPath] = {
              name: part,
              type: isFile ? 'file' : 'folder',
              path: currentPath,
              children: isFile ? undefined : [],
              language: isFile ? obj.language : undefined,
              symbols: isFile ? [] : undefined
            };

            // Add to parent or root
            if (parentPath && pathMap[parentPath]) {
              pathMap[parentPath].children?.push(pathMap[currentPath]);
            } else if (!parentPath) {
              rootNodes.push(pathMap[currentPath]);
            }
          }

          // Add symbol to file if it's a symbol object
          if (obj.type === 'Symbol' && index === pathParts.length - 1) {
            pathMap[currentPath].symbols?.push({
              name: obj.name,
              type: 'function', // Default to function, could be enhanced
              signature: obj.signature
            });
          }
        });
      }
    });

    return rootNodes;
  };

  const fetchCodebases = async () => {
    try {
      setLoading(true);
      setError(null);

      // Fetch real codebase data from AMP server using query endpoint
      const queryResponse = await fetch('http://localhost:8105/v1/query', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          limit: 1000 // Just get objects without text filter
        })
      });
      
      if (!queryResponse.ok) {
        throw new Error(`AMP server error: ${queryResponse.status} ${queryResponse.statusText}`);
      }
      
      const queryResult = await queryResponse.json();
      console.log('Query result:', queryResult); // Debug log
      
      // Extract objects from the QueryResponse format
      let objects = [];
      if (queryResult.results && Array.isArray(queryResult.results)) {
        // Extract objects from QueryResult format: results[].object
        objects = queryResult.results.map((result: any) => result.object || result);
      } else if (Array.isArray(queryResult)) {
        objects = queryResult;
      } else {
        console.error('Unexpected response format:', queryResult);
        throw new Error(`Invalid response format. Expected results array.`);
      }
      
      console.log('Extracted objects:', objects.slice(0, 3)); // Debug first 3 objects
      
      if (objects.length === 0) {
        throw new Error('No parsed codebases found. Run CLI indexing first.');
      }
      
      // Fetch relationships from AMP server relationships endpoint
      const relationshipsResponse = await fetch('http://localhost:8105/v1/relationships?type=defined_in', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        }
      });
      
      let relationshipData = [];
      if (relationshipsResponse.ok) {
        const relationships = await relationshipsResponse.json();
        console.log('Fetched relationships:', relationships.length, relationships.slice(0, 3)); // Debug log
        
        relationshipData = relationships.map((rel: any) => {
          // Extract UUIDs from "objects:⟨uuid⟩" format
          const inId = rel.in?.replace(/^objects:⟨|⟩$/g, '') || rel.in;
          const outId = rel.out?.replace(/^objects:⟨|⟩$/g, '') || rel.out;
          console.log('Extracted relationship IDs:', inId, '->', outId); // Debug log
          return {
            in: inId,
            out: outId,
            relation_type: 'defined_in'
          };
        }).filter((rel: any) => rel.in && rel.out);
      } else {
        console.error('Failed to fetch relationships:', relationshipsResponse.status, relationshipsResponse.statusText);
      }
      
      setRelationships(relationshipData);
      setObjects(objects);
      console.log(`Found ${objects.length} objects and ${relationshipData.length} relationships`); // Debug log

      // Group objects by project_id to create codebases
      const projectGroups: Record<string, any[]> = {};
      objects.forEach((obj: any) => {
        if (!projectGroups[obj.project_id]) {
          projectGroups[obj.project_id] = [];
        }
        projectGroups[obj.project_id].push(obj);
      });

      // Convert to codebase format
      const realCodebases: CodebaseProject[] = Object.entries(projectGroups).map(([projectId, projectObjects]) => {
        console.log(`Processing project: ${projectId}`, projectObjects.slice(0, 3)); // Debug first 3 objects
        
        // Calculate language stats
        const languageStats: Record<string, number> = {};
        const languageCounts: Record<string, number> = {};
        
        projectObjects.forEach(obj => {
          // Handle different possible language field names
          const language = obj.language || obj.lang || obj.file_type;
          if (language) {
            languageCounts[language] = (languageCounts[language] || 0) + 1;
          }
        });

        const totalFiles = Object.values(languageCounts).reduce((a, b) => a + b, 0) || 1;
        Object.entries(languageCounts).forEach(([lang, count]) => {
          languageStats[lang] = Math.round((count / totalFiles) * 100);
        });

        // Build file tree from objects
        const fileTree = buildFileTreeFromObjects(projectObjects);

        // Count symbols - only count actual code symbols (function, class, method, variable)
        const codeSymbolKinds = ['function', 'class', 'method', 'variable', 'interface'];
        const totalSymbols = projectObjects.filter(obj => {
          console.log('Object type check:', obj.type, obj.kind, obj.name); // Debug log
          return obj.type === 'Symbol' && codeSymbolKinds.includes(obj.kind);
        }).length;
        
        console.log(`Total symbols found: ${totalSymbols} out of ${projectObjects.length} objects`); // Debug log

        // Get project name - handle undefined project_id
        const projectName = projectId && projectId !== 'undefined' 
          ? projectId.charAt(0).toUpperCase() + projectId.slice(1).replace(/[-_]/g, ' ')
          : `Python Project (${projectObjects.length} objects)`;

        return {
          id: projectId || `project-${Date.now()}`,
          name: projectName,
          path: `/${projectId || 'unknown'}`,
          description: `Parsed codebase with ${totalSymbols} symbols across ${Math.max(totalFiles, projectObjects.length)} files`,
          language_stats: Object.keys(languageStats).length > 0 ? languageStats : { 'Python': 100 },
          total_files: Math.max(totalFiles, projectObjects.length),
          total_symbols: totalSymbols,
          last_indexed: projectObjects[0]?.created_at || new Date().toISOString(),
          file_tree: fileTree
        };
      });

      setCodebases(realCodebases);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to connect to AMP server';
      setError(errorMsg);
      console.error('Failed to fetch codebases:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchCodebases();
  }, []);

  const refetch = () => {
    fetchCodebases();
  };

  return {
    codebases,
    objects,
    relationships,
    loading,
    error,
    refetch
  };
};
