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

    // Helper to normalize paths for comparison
    const normalizePath = (path: string) => {
      return path.replace(/\\/g, '/').replace(/^\.\//, '').replace(/^\//, '');
    };
    const normalizeKind = (kind?: string) => (kind ? kind.toLowerCase() : '');

    // Separate objects by kind
    const projectObjs = objects.filter(obj => normalizeKind(obj.kind) === 'project');
    const dirObjs = objects.filter(obj => normalizeKind(obj.kind) === 'directory');
    const fileObjs = objects.filter(obj => normalizeKind(obj.kind) === 'file');
    const codeSymbols = objects.filter(obj => {
      const kind = normalizeKind(obj.kind);
      return kind && !['project', 'directory', 'file'].includes(kind);
    });

    console.log('Building file tree:', {
      directories: dirObjs.length,
      files: fileObjs.length,
      codeSymbols: codeSymbols.length
    });

    // Create directory nodes
    dirObjs.forEach(dir => {
      if (dir.path) {
        const normalizedPath = normalizePath(dir.path);
        pathMap[normalizedPath] = {
          name: dir.name,
          type: 'folder',
          path: normalizedPath,
          children: []
        };
      }
    });

    // Create file nodes
    fileObjs.forEach(file => {
      if (file.path) {
        const normalizedPath = normalizePath(file.path);
        pathMap[normalizedPath] = {
          name: file.name,
          type: 'file',
          path: normalizedPath,
          language: file.language,
          symbols: []
        };
      }
    });

    // Add code symbols to their parent files
    codeSymbols.forEach(symbol => {
      if (symbol.path) {
        const normalizedPath = normalizePath(symbol.path);
        const fileNode = pathMap[normalizedPath];
        if (fileNode && fileNode.type === 'file') {
          fileNode.symbols?.push({
            name: symbol.name,
            type: normalizeKind(symbol.kind),
            signature: symbol.signature
          });
        } else {
          console.warn('Symbol without matching file:', symbol.name, normalizedPath);
        }
      }
    });

    // Build hierarchy
    Object.values(pathMap).forEach(node => {
      const pathParts = node.path.split('/').filter(Boolean);
      if (pathParts.length === 1) {
        // Root level
        rootNodes.push(node);
      } else {
        // Find parent
        const parentPath = pathParts.slice(0, -1).join('/');
        const parent = pathMap[parentPath];
        if (parent && parent.children) {
          parent.children.push(node);
        } else {
          // No parent found, add to root
          console.warn('No parent found for:', node.path, 'expected parent:', parentPath);
          rootNodes.push(node);
        }
      }
    });

    console.log('Built file tree with', rootNodes.length, 'root nodes');
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

      objects = objects.filter(obj => {
        const objType = (obj.type || '').toLowerCase();
        return ['symbol', 'file', 'note', 'decision', 'changeset', 'artifact_core'].includes(objType);
      });

      if (objects.length === 0) {
        throw new Error('No parsed codebases found. Run CLI indexing first.');
      }
      
      // Fetch relationships from AMP server relationships endpoint
      const relationshipsResponse = await fetch('http://localhost:8105/v1/relationships', {
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
          const normalizeId = (value?: string) => {
            if (!value) return value;
            return value
              .replace(/^objects:/, '')
              .replace(/[⟨⟩]/g, '');
          };

          const inId = normalizeId(rel.in);
          const outId = normalizeId(rel.out);
          console.log('Extracted relationship IDs:', inId, '->', outId); // Debug log
          return {
            in: inId,
            out: outId,
            relation_type: rel.type || rel.relation_type || 'defined_in'
          };
        }).filter((rel: any) => rel.in && rel.out);
      } else {
        console.error('Failed to fetch relationships:', relationshipsResponse.status, relationshipsResponse.statusText);
      }
      
      setRelationships(relationshipData);
      setObjects(objects);
      console.log(`Found ${objects.length} objects and ${relationshipData.length} relationships`); // Debug log

      const codebaseObjects = objects.filter((obj: any) => {
        const objType = (obj.type || '').toLowerCase();
        return objType === 'symbol' || objType === 'file';
      });

      // Group codebase objects by project_id to create codebases
      const projectGroups: Record<string, any[]> = {};
      codebaseObjects.forEach((obj: any) => {
        if (!projectGroups[obj.project_id]) {
          projectGroups[obj.project_id] = [];
        }
        projectGroups[obj.project_id].push(obj);
      });

      const inferLanguage = (path: string | undefined) => {
        if (!path) return undefined;
        const parts = path.split('.');
        const ext = parts.length > 1 ? parts[parts.length - 1].toLowerCase() : '';
        switch (ext) {
          case 'py':
          case 'pyi':
            return 'python';
          case 'ts':
          case 'tsx':
            return 'typescript';
          case 'js':
          case 'jsx':
            return 'javascript';
          case 'rs':
            return 'rust';
          case 'go':
            return 'go';
          case 'java':
            return 'java';
          case 'kt':
          case 'kts':
            return 'kotlin';
          case 'cs':
            return 'csharp';
          case 'c':
            return 'c';
          case 'h':
          case 'hpp':
          case 'hh':
          case 'hxx':
          case 'cpp':
          case 'cxx':
          case 'cc':
            return 'cpp';
          case 'swift':
            return 'swift';
          case 'rb':
            return 'ruby';
          case 'php':
            return 'php';
          case 'sh':
          case 'bash':
            return 'shell';
          case 'md':
            return 'markdown';
          case 'json':
            return 'json';
          case 'toml':
            return 'toml';
          case 'yml':
          case 'yaml':
            return 'yaml';
          default:
            return undefined;
        }
      };

      // Convert to codebase format
      const realCodebases: CodebaseProject[] = Object.entries(projectGroups).map(([projectId, projectObjects]) => {
        console.log(`Processing project: ${projectId}`, projectObjects.slice(0, 3)); // Debug first 3 objects
        
        // Calculate language stats (code file distribution by size)
      const languageStats: Record<string, number> = {};
      const languageCounts: Record<string, number> = {};
      const projectFileObjects = projectObjects.filter(obj => (obj.kind || '').toLowerCase() === 'file');
      const totalFiles = projectFileObjects.length || 0;
      const codeLanguages = new Set([
        'python', 'typescript', 'javascript', 'rust', 'go', 'java', 'kotlin',
        'csharp', 'c', 'cpp', 'swift', 'ruby', 'php', 'shell'
      ]);

      const addLanguageWeight = (language: string | undefined, weight: number) => {
        if (!language) return;
        const key = language.toLowerCase();
        if (!codeLanguages.has(key)) return;
        languageCounts[key] = (languageCounts[key] || 0) + Math.max(weight, 1);
      };

      projectFileObjects.forEach((file) => {
        const rawPath = file.path || file.file_path;
        let language = file.language || file.lang || file.file_type;
        if (!language || language.toLowerCase() === 'unknown') {
          language = inferLanguage(rawPath);
        }
        const weight = typeof file.file_size === 'number'
          ? file.file_size
          : typeof file.line_count === 'number'
            ? file.line_count
            : 1;
        addLanguageWeight(language, weight);
      });

      const totalLanguageUnits = Object.values(languageCounts).reduce((sum, value) => sum + value, 0);
      Object.entries(languageCounts).forEach(([lang, count]) => {
        languageStats[lang] = totalLanguageUnits > 0 ? Math.round((count / totalLanguageUnits) * 100) : 0;
      });

      if (totalLanguageUnits === 0) {
        const fallbackCounts: Record<string, number> = {};
        projectFileObjects.forEach((file) => {
          let language = file.language || file.lang || file.file_type;
          if (!language || language.toLowerCase() === 'unknown') {
            language = inferLanguage(file.path || file.file_path);
          }
          if (!language) return;
          const key = language.toLowerCase();
          if (!codeLanguages.has(key)) return;
          fallbackCounts[key] = (fallbackCounts[key] || 0) + 1;
        });
        const fallbackTotal = Object.values(fallbackCounts).reduce((sum, value) => sum + value, 0);
        Object.entries(fallbackCounts).forEach(([lang, count]) => {
          languageStats[lang] = fallbackTotal > 0 ? Math.round((count / fallbackTotal) * 100) : 0;
        });
      }

        // Build file tree from objects
        const fileTree = buildFileTreeFromObjects(projectObjects);

        // Count symbols - only count actual code symbols (function, class, method, variable)
        const codeSymbolKinds = ['function', 'class', 'method', 'variable', 'interface', 'type'];
        const totalSymbols = projectObjects.filter(obj => {
          const objType = (obj.type || '').toLowerCase();
          const objKind = (obj.kind || '').toLowerCase();
          console.log('Object type check:', obj.type, obj.kind, obj.name); // Debug log
          return objType === 'symbol' && codeSymbolKinds.includes(objKind);
        }).length;
        
        console.log(`Total symbols found: ${totalSymbols} out of ${projectObjects.length} objects`); // Debug log

        // Get project name - look for the project Symbol object first
        const projectSymbol = projectObjects.find(obj => (obj.kind || '').toLowerCase() === 'project');
        const projectName = projectSymbol?.name || 
          (projectId && projectId !== 'undefined' 
            ? projectId.charAt(0).toUpperCase() + projectId.slice(1).replace(/[-_]/g, ' ')
            : `Python Project (${projectObjects.length} objects)`);

        const createdDates = projectObjects
          .map(obj => obj.created_at)
          .filter((value: string | undefined) => value);
        const lastIndexed = projectSymbol?.created_at || createdDates.sort().at(-1) || new Date().toISOString();

        return {
          id: projectId || `project-${Date.now()}`,
          name: projectName,
          path: `/${projectId || 'unknown'}`,
          description: `Parsed codebase with ${totalSymbols} symbols across ${Math.max(totalFiles, 0)} files`,
          language_stats: Object.keys(languageStats).length > 0 ? languageStats : {},
          total_files: totalFiles,
          total_symbols: totalSymbols,
          last_indexed: lastIndexed,
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

