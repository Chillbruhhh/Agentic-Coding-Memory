import React, { useEffect, useState, useCallback, useRef } from 'react';
import { HiLightBulb, HiCheckCircle, HiCode, HiExclamation, HiRefresh, HiChevronDown, HiChevronRight } from 'react-icons/hi';

interface CacheItem {
  kind: string;
  content: string;
  importance: number;
  file_ref?: string;
  created_at?: string;
}

interface CacheBlock {
  id: string;
  scope_id: string;
  sequence: number;
  status: 'open' | 'closed';
  items: CacheItem[];
  token_count: number;
  summary?: string;
  created_at?: string;
  closed_at?: string;
}

interface CachePanelProps {
  runId: string;
  projectId?: string;
}

type CacheScopeMode = 'session' | 'project' | 'all';

const normalizeItems = (items: any[]): CacheItem[] => items
  .filter(item => item && typeof item === 'object')
  .map(item => ({
    kind: item.kind || 'fact',
    content: item.content || '',
    importance: typeof item.importance === 'number' ? item.importance : 0.5,
    file_ref: item.file_ref,
    created_at: item.created_at,
  }));

const kindStyles: Record<string, string> = {
  fact: 'border-l-amber-500/50 bg-amber-950/20',
  decision: 'border-l-emerald-500/50 bg-emerald-950/20',
  snippet: 'border-l-sky-500/50 bg-sky-950/20',
  warning: 'border-l-red-500/50 bg-red-950/20',
};

export const CachePanel: React.FC<CachePanelProps> = ({ runId, projectId }) => {
  const [blocks, setBlocks] = useState<CacheBlock[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedBlocks, setExpandedBlocks] = useState<Set<string>>(new Set());
  const [scopeMode, setScopeMode] = useState<CacheScopeMode>('project');
  const [isInteracting, setIsInteracting] = useState(false);
  const scrollRef = useRef<HTMLDivElement | null>(null);

  // Try different scope patterns to find cache data
  const fetchCacheBlocks = useCallback(async () => {
    if (isInteracting) {
      return;
    }
    setLoading(true);
    setError(null);

    const scrollTop = scrollRef.current?.scrollTop ?? 0;

    const normalizedRunId = runId.replace(/^objects:/, '').replace(/[`⟨⟩]/g, '');
    const sessionScopes = [`run:${normalizedRunId}`, `session:${normalizedRunId}`];
    const primarySessionScope = sessionScopes[0];
    const projectScope = projectId ? `project:${projectId}` : 'project:amp';
    const scopePatterns = scopeMode === 'session'
      ? [primarySessionScope]
      : scopeMode === 'project'
      ? [projectScope]
      : [...sessionScopes, projectScope];

    let foundBlocks: CacheBlock[] = [];

    for (const scopeId of scopePatterns) {
      try {
        // Search for blocks with this scope
        const response = await fetch('http://localhost:8105/v1/cache/block/search', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            scope_id: scopeId,
            query: '*',
            limit: 20,
            include_open: true,
          }),
        });

        if (response.ok) {
          const data = await response.json();
          const matches = Array.isArray(data.matches) ? data.matches : [];
          if (matches.length > 0) {
            const detailed = await Promise.all(matches.map(async (match: any, index: number) => {
              if (!match?.block_id) return null;
              const detailResponse = await fetch(`http://localhost:8105/v1/cache/block/${encodeURIComponent(match.block_id)}`);
              if (!detailResponse.ok) return null;
              const detail = await detailResponse.json();
              return {
                id: detail.block_id,
                scope_id: scopeId,
                sequence: index + 1,
                status: detail.status || 'closed',
                items: normalizeItems(detail.items || []),
                token_count: detail.token_count || 0,
                summary: detail.summary,
                created_at: detail.created_at,
              } as CacheBlock;
            }));
            const scopedBlocks = detailed.filter(Boolean) as CacheBlock[];
            foundBlocks = [...foundBlocks, ...scopedBlocks];
          }
        }
      } catch (err) {
        console.debug(`Failed to fetch cache for scope ${scopeId}:`, err);
      }
    }

    // If no blocks found via search, try to get the current open block
    if (foundBlocks.length === 0 && scopePatterns.length > 0) {
      try {
        const currentScope = scopeMode === 'session' ? primarySessionScope : projectScope;
        const currentResponse = await fetch(`http://localhost:8105/v1/cache/block/current/${encodeURIComponent(currentScope)}`);
        if (currentResponse.ok) {
          const current = await currentResponse.json();
          foundBlocks = [{
            id: current.block_id,
            scope_id: currentScope,
            sequence: 1,
            status: current.status || 'open',
            items: normalizeItems(current.items || []),
            token_count: current.token_count || 0,
            summary: current.summary,
            created_at: current.created_at,
          }];
        }
      } catch (err) {
        console.debug('Failed to fetch current cache block:', err);
      }
    }

    // If no blocks found via search, try to get the legacy cache pack
    if (foundBlocks.length === 0 && scopePatterns.length > 0) {
      try {
        // Try to get cache pack which returns structured data
        const response = await fetch('http://localhost:8105/v1/cache/pack', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            scope_id: scopeMode === 'session' ? scopePatterns[0] : projectScope,
            token_budget: 2000,
          }),
        });

        if (response.ok) {
          const pack = await response.json();
          // Convert pack items to a pseudo-block for display
          if (pack.facts?.length || pack.decisions?.length || pack.snippets?.length || pack.warnings?.length) {
            const items: CacheItem[] = [
              ...(pack.facts || []).map((f: any) => ({ kind: 'fact', content: f.preview, importance: f.importance || 0.5 })),
              ...(pack.decisions || []).map((d: any) => ({ kind: 'decision', content: d.preview, importance: d.importance || 0.5 })),
              ...(pack.snippets || []).map((s: any) => ({ kind: 'snippet', content: s.preview, importance: s.importance || 0.5 })),
              ...(pack.warnings || []).map((w: any) => ({ kind: 'warning', content: w.preview, importance: w.importance || 0.5 })),
            ];

            foundBlocks = [{
              id: 'pack',
              scope_id: scopeMode === 'session' ? scopePatterns[0] : projectScope,
              sequence: 0,
              status: 'open',
              items,
              token_count: pack.token_count || 0,
              summary: pack.summary,
            }];
          }
        }
      } catch (err) {
        console.debug('Failed to fetch cache pack:', err);
      }
    }

    setBlocks(foundBlocks);
    setLoading(false);
    requestAnimationFrame(() => {
      if (scrollRef.current) {
        scrollRef.current.scrollTop = scrollTop;
      }
    });
  }, [runId, projectId, scopeMode, isInteracting]);

  useEffect(() => {
    fetchCacheBlocks();
    // Refresh every 10 seconds for live sessions
    const interval = setInterval(fetchCacheBlocks, 10000);
    return () => clearInterval(interval);
  }, [fetchCacheBlocks]);

  const toggleBlock = (blockId: string) => {
    setExpandedBlocks(prev => {
      const next = new Set(prev);
      if (next.has(blockId)) {
        next.delete(blockId);
      } else {
        next.add(blockId);
      }
      return next;
    });
  };

  // Flatten all items for display grouped by kind
  const allItems = blocks.flatMap(b => b.items);
  const itemsByKind = {
    fact: allItems.filter(i => i.kind === 'fact'),
    decision: allItems.filter(i => i.kind === 'decision'),
    snippet: allItems.filter(i => i.kind === 'snippet'),
    warning: allItems.filter(i => i.kind === 'warning'),
  };

  const totalItems = allItems.length;
  const totalTokens = blocks.reduce((sum, b) => sum + (b.token_count || 0), 0);

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="flex items-center justify-between px-3 py-2 border-b border-border-dark bg-black/30">
        <div className="flex items-center gap-2 text-xs uppercase tracking-[0.2em] text-primary">
          <HiLightBulb size={14} />
          Session Cache
        </div>
        <div className="flex items-center gap-3 text-xs text-slate-500">
          <select
            value={scopeMode}
            onChange={(event) => setScopeMode(event.target.value as CacheScopeMode)}
            className="bg-black/40 border border-border-dark text-slate-200 text-[10px] uppercase tracking-[0.2em] rounded px-2 py-1"
          >
            <option value="session">This Session</option>
            <option value="project">Project</option>
            <option value="all">All</option>
          </select>
          <span>{totalItems} items</span>
          <span>{totalTokens} tokens</span>
          <button
            onClick={fetchCacheBlocks}
            className="p-1 hover:bg-white/5 rounded text-slate-400 hover:text-slate-200 transition-colors"
          >
            <HiRefresh size={12} />
          </button>
        </div>
      </div>

      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-3 space-y-3"
        onMouseEnter={() => setIsInteracting(true)}
        onMouseLeave={() => setIsInteracting(false)}
        onWheel={() => setIsInteracting(true)}
      >
        {loading && (
          <div className="flex items-center justify-center p-4 text-slate-500 text-sm">
            Loading cache...
          </div>
        )}
        {!loading && error && (
          <div className="flex items-center justify-center p-4 text-red-400 text-sm">
            {error}
          </div>
        )}
        {!loading && !error && blocks.length === 0 && (
          <div className="flex flex-col items-center justify-center p-4 text-slate-500 text-sm">
            <span>No cache data for this session.</span>
            <button
              onClick={fetchCacheBlocks}
              className="mt-2 flex items-center gap-1 text-xs text-primary hover:text-primary/80"
            >
              <HiRefresh size={12} /> Refresh
            </button>
          </div>
        )}
        {!loading && !error && blocks.length > 0 && (
          <>
            {/* Warnings first if any */}
            {itemsByKind.warning.length > 0 && (
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-xs font-medium text-red-400">
                  <HiExclamation size={12} />
                  Warnings ({itemsByKind.warning.length})
                </div>
                {itemsByKind.warning.map((item, idx) => (
                  <div
                    key={`warning-${idx}`}
                    className={`border-l-2 ${kindStyles.warning} rounded-r p-2`}
                  >
                    <div className="text-xs text-slate-300 leading-relaxed">{item.content}</div>
                  </div>
                ))}
              </div>
            )}

            {/* Decisions */}
            {itemsByKind.decision.length > 0 && (
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-xs font-medium text-emerald-400">
                  <HiCheckCircle size={12} />
                  Decisions ({itemsByKind.decision.length})
                </div>
                {itemsByKind.decision.map((item, idx) => (
                  <div
                    key={`decision-${idx}`}
                    className={`border-l-2 ${kindStyles.decision} rounded-r p-2`}
                  >
                    <div className="text-xs text-slate-300 leading-relaxed">{item.content}</div>
                  </div>
                ))}
              </div>
            )}

            {/* Facts */}
            {itemsByKind.fact.length > 0 && (
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-xs font-medium text-amber-400">
                  <HiLightBulb size={12} />
                  Facts ({itemsByKind.fact.length})
                </div>
                {itemsByKind.fact.map((item, idx) => (
                  <div
                    key={`fact-${idx}`}
                    className={`border-l-2 ${kindStyles.fact} rounded-r p-2`}
                  >
                    <div className="text-xs text-slate-300 leading-relaxed">{item.content}</div>
                  </div>
                ))}
              </div>
            )}

            {/* Snippets */}
            {itemsByKind.snippet.length > 0 && (
              <div className="space-y-2">
                <div className="flex items-center gap-2 text-xs font-medium text-sky-400">
                  <HiCode size={12} />
                  Snippets ({itemsByKind.snippet.length})
                </div>
                {itemsByKind.snippet.map((item, idx) => (
                  <div
                    key={`snippet-${idx}`}
                    className={`border-l-2 ${kindStyles.snippet} rounded-r p-2`}
                  >
                    {item.file_ref && (
                      <div className="text-[10px] text-slate-500 mb-1 font-mono">{item.file_ref}</div>
                    )}
                    <pre className="text-xs text-slate-300 leading-relaxed whitespace-pre-wrap font-mono">
                      {item.content}
                    </pre>
                  </div>
                ))}
              </div>
            )}

            {/* Block details (collapsible) */}
            {blocks.length > 0 && blocks[0].id !== 'pack' && (
              <div className="mt-4 pt-3 border-t border-border-dark">
                <div className="text-xs text-slate-500 mb-2">Cache Blocks ({blocks.length})</div>
                {blocks.map(block => (
                  <div key={block.id} className="mb-2">
                    <button
                      onClick={() => toggleBlock(block.id)}
                      className="w-full flex items-center gap-2 text-xs text-slate-400 hover:text-slate-200 p-1 rounded hover:bg-white/5"
                    >
                      {expandedBlocks.has(block.id) ? <HiChevronDown size={12} /> : <HiChevronRight size={12} />}
                      <span className="font-mono">Block #{block.sequence}</span>
                      <span className={`px-1.5 py-0.5 rounded text-[10px] ${
                        block.status === 'open' ? 'bg-emerald-500/20 text-emerald-300' : 'bg-slate-700 text-slate-400'
                      }`}>
                        {block.status}
                      </span>
                      <span className="text-slate-500">{block.items.length} items</span>
                    </button>
                    {expandedBlocks.has(block.id) && block.summary && (
                      <div className="ml-5 mt-1 text-xs text-slate-500 italic">
                        {block.summary}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};
