import React, { useEffect, useMemo, useState } from 'react';
import { HiPlay, HiRefresh, HiClock, HiOutlineDocumentText, HiExclamation, HiStatusOnline, HiStatusOffline, HiDatabase } from 'react-icons/hi';
import { useRuns, RunDetail, RunSummary } from '../hooks/useRuns';
import { useConnections } from '../hooks/useConnections';
import { CachePanel } from './CachePanel';

type SessionTab = 'live' | 'history';

const formatDate = (isoString?: string) => {
  if (!isoString) return 'Unknown';
  const date = new Date(isoString);
  if (isNaN(date.getTime())) return 'Unknown';
  return date.toLocaleString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
};

const formatDuration = (durationMs?: number) => {
  if (!durationMs || durationMs <= 0) return 'Unknown';
  const totalSeconds = Math.floor(durationMs / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  if (minutes === 0) return `${seconds}s`;
  return `${minutes}m ${seconds}s`;
};

const normalizeStatus = (status?: string) => (status || 'unknown').toLowerCase();

const statusStyles = (status?: string) => {
  switch (normalizeStatus(status)) {
    case 'running':
      return 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30';
    case 'completed':
      return 'bg-sky-500/20 text-sky-300 border-sky-500/30';
    case 'failed':
      return 'bg-red-500/20 text-red-300 border-red-500/30';
    case 'cancelled':
      return 'bg-amber-500/20 text-amber-300 border-amber-500/30';
    default:
      return 'bg-slate-700/40 text-slate-300 border-slate-600/40';
  }
};

const isLiveStatus = (status?: string) => normalizeStatus(status) === 'running';

export const Sessions: React.FC = () => {
  const { runs, loading, error, refetch, fetchRunDetails } = useRuns();
  const { isRunConnected, getConnectionForRun } = useConnections();
  const [activeTab, setActiveTab] = useState<SessionTab>('live');
  const [selectedRun, setSelectedRun] = useState<RunSummary | null>(null);
  const [selectedDetail, setSelectedDetail] = useState<RunDetail | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [showCache, setShowCache] = useState(false);
  const [expandedFocusEntries, setExpandedFocusEntries] = useState<Record<string, boolean>>({});

  const isRecentRun = (run: RunSummary) => {
    const timestamp = run.updated_at || run.created_at;
    if (!timestamp) return false;
    const time = new Date(timestamp).getTime();
    if (Number.isNaN(time)) return false;
    const thirtyMinutesMs = 30 * 60 * 1000;
    return Date.now() - time <= thirtyMinutesMs;
  };

  const liveRuns = useMemo(
    () => runs.filter(run => isRunConnected(run.id) || (isLiveStatus(run.status) && isRecentRun(run))),
    [runs, isRunConnected]
  );
  const historyRuns = useMemo(
    () => runs.filter(run => !isRunConnected(run.id) && !(isLiveStatus(run.status) && isRecentRun(run))),
    [runs, isRunConnected]
  );

  const visibleRuns = activeTab === 'live' ? liveRuns : historyRuns;

  useEffect(() => {
    if (!selectedRun) return;
    const updated = runs.find(run => run.id === selectedRun.id);
    if (updated) {
      setSelectedRun(updated);
    }
  }, [runs, selectedRun]);

  useEffect(() => {
    if (!selectedRun || !isLiveStatus(selectedRun.status)) {
      return;
    }
    const interval = setInterval(async () => {
      const detail = await fetchRunDetails(selectedRun.id);
      if (detail) {
        setSelectedDetail(detail);
      }
    }, 5000);
    return () => clearInterval(interval);
  }, [fetchRunDetails, selectedRun]);

  const handleSelectRun = async (run: RunSummary) => {
    setSelectedRun(run);
    setDetailLoading(true);
    const detail = await fetchRunDetails(run.id);
    setSelectedDetail(detail);
    setDetailLoading(false);
  };

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-slate-400">Loading sessions...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-red-400">Error: {error}</div>
      </div>
    );
  }

  const detail = selectedDetail || selectedRun;
  const focus = (detail as RunDetail | RunSummary | null)?.focus;
  const activeFocus = focus && focus.status !== 'completed' ? focus : null;

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="p-4 border-b border-border-dark bg-panel-dark flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h2 className="text-sm font-bold text-slate-200 uppercase tracking-wider">
            Agent Sessions
          </h2>
          <button
            onClick={refetch}
            className="p-1 hover:bg-white/5 rounded text-slate-400 hover:text-slate-200 transition-colors"
          >
            <HiRefresh size={16} />
          </button>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setActiveTab('live')}
            className={`px-3 py-1 rounded-full text-xs uppercase tracking-wider border ${
              activeTab === 'live'
                ? 'bg-red-950/40 text-primary border-primary/40'
                : 'text-slate-400 border-slate-700 hover:text-slate-200'
            }`}
          >
            Live {liveRuns.length}
          </button>
          <button
            onClick={() => setActiveTab('history')}
            className={`px-3 py-1 rounded-full text-xs uppercase tracking-wider border ${
              activeTab === 'history'
                ? 'bg-red-950/40 text-primary border-primary/40'
                : 'text-slate-400 border-slate-700 hover:text-slate-200'
            }`}
          >
            History {historyRuns.length}
          </button>
        </div>
      </div>

      <div className="flex-1 grid grid-cols-1 lg:grid-cols-[380px_1fr] gap-0 overflow-hidden">
        <div className="border-r border-border-dark overflow-y-auto p-4 space-y-4">
          {visibleRuns.length === 0 && (
            <div className="text-slate-500 text-sm">No sessions in this view.</div>
          )}
          {visibleRuns.map(run => {
            const connected = isRunConnected(run.id);
            const connection = getConnectionForRun(run.id);
            const connectionBorderClass = connected
              ? 'ring-2 ring-emerald-500/60 ring-offset-1 ring-offset-stone-900'
              : isLiveStatus(run.status)
              ? 'ring-2 ring-red-500/40 ring-offset-1 ring-offset-stone-900'
              : '';
            const isSelected = selectedRun?.id === run.id;
            const selectionBorderClass = isSelected
              ? connected
                ? 'border-l-emerald-500'
                : 'border-l-primary'
              : 'border-l-transparent';

            return (
              <button
                key={run.id}
                onClick={() => handleSelectRun(run)}
                className={`w-full text-left bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-4 border-l-4 shadow-lg transition-all hover:border-primary/50 rounded-lg ${connectionBorderClass} ${
                  selectionBorderClass
                }`}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="flex-1">
                    <div className="text-sm font-semibold text-slate-100 flex items-center gap-2">
                      {connected && (
                        <HiStatusOnline className="text-emerald-400 flex-shrink-0" size={14} title="Agent connected" />
                      )}
                      {!connected && isLiveStatus(run.status) && (
                        <HiStatusOffline className="text-red-400 flex-shrink-0" size={14} title="Agent disconnected" />
                      )}
                      <span>{run.input_summary || run.provenance?.summary || 'Untitled Session'}</span>
                    </div>
                    <div className="text-xs text-slate-500 mt-1">
                      Agent {connection?.agent_name || run.provenance?.agent || 'unknown'} - {run.project_id || 'default'}
                    </div>
                  </div>
                </div>
                <div className="mt-3 flex items-center justify-between text-xs text-slate-500">
                  <span>Started {formatDate(run.created_at)}</span>
                  <span>{run.duration_ms ? formatDuration(run.duration_ms) : (connected ? (connection?.agent_name || run.provenance?.agent || 'Connected') : 'Unknown')}</span>
                </div>
              </button>
            );
          })}
        </div>

        <div className="flex flex-col overflow-hidden">
          {!detail && (
            <div className="flex-1 flex items-center justify-center text-slate-500">
              Select a session to inspect details.
            </div>
          )}
          {detail && (
            <div className="flex-1 flex flex-col overflow-hidden">
              <div className="p-4 border-b border-border-dark bg-black/30 flex items-center justify-between">
                <div>
                  <div className="text-xs uppercase tracking-[0.2em] text-primary flex items-center gap-2">
                    Session
                    {isRunConnected(detail.id) && (
                      <span className="flex items-center gap-1 text-emerald-400">
                        <HiStatusOnline size={12} />
                        <span className="text-[10px]">Connected</span>
                      </span>
                    )}
                  </div>
                  <div className="text-lg font-semibold text-slate-100 mt-1">
                    {detail.input_summary || detail.provenance?.summary || 'Untitled Session'}
                  </div>
                  <div className="text-xs text-slate-500 mt-1">
                    Agent {getConnectionForRun(detail.id)?.agent_name || detail.provenance?.agent || 'unknown'} - {detail.project_id || 'default'}
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <button
                    onClick={() => setShowCache(!showCache)}
                    className={`flex items-center gap-1 px-3 py-1 rounded-full text-xs uppercase tracking-wider border transition-colors ${
                      showCache
                        ? 'bg-primary/20 text-primary border-primary/40'
                        : 'text-slate-400 border-slate-700 hover:text-slate-200 hover:border-slate-600'
                    }`}
                  >
                    <HiDatabase size={12} />
                    Cache
                  </button>
                </div>
              </div>

              {detailLoading && (
                <div className="p-4 text-slate-500 text-sm">Loading session details...</div>
              )}

              {!detailLoading && (
                <div className={`flex-1 overflow-hidden p-4 grid gap-4 ${
                  showCache ? 'grid-cols-1 xl:grid-cols-3' : 'grid-cols-1 xl:grid-cols-2'
                }`}>
                  <div className="flex flex-col border border-border-dark bg-panel-dark/60 rounded-lg p-4 overflow-hidden">
                    <div className="flex items-center gap-2 text-xs uppercase tracking-[0.2em] text-primary mb-3">
                      <HiPlay /> Current Focus
                    </div>
                  <div className="text-sm text-slate-200 leading-relaxed">
                      {activeFocus?.title || detail.input_summary || 'No active summary recorded.'}
                    </div>
                    {activeFocus?.plan && activeFocus.plan.length > 0 && (
                      <div className="mt-3 border border-border-dark rounded bg-black/40 p-3">
                        <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500 mb-2">
                          Plan
                        </div>
                        <ul className="text-xs text-slate-200 space-y-1">
                          {activeFocus.plan.map((step, idx) => (
                            <li key={`${step}-${idx}`} className="flex items-start gap-2">
                              <span className="text-slate-500">-</span>
                              <span>{step}</span>
                            </li>
                          ))}
                        </ul>
                      </div>
                    )}
                    <div className="mt-4 text-xs text-slate-500 space-y-2">
                      <div className="flex items-center gap-2">
                        <HiClock />
                        <span>Started {formatDate(detail.created_at)}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span>Duration {formatDuration(detail.duration_ms)}</span>
                      </div>
                      {detail.confidence !== undefined && (
                        <div className="flex items-center gap-2">
                          <span>Confidence {(detail.confidence * 100).toFixed(0)}%</span>
                        </div>
                      )}
                    </div>
                  </div>

                  <div className="flex flex-col border border-border-dark bg-panel-dark/60 rounded-lg p-4 overflow-hidden">
                    <div className="flex items-center gap-2 text-xs uppercase tracking-[0.2em] text-primary mb-3">
                      <HiOutlineDocumentText /> Recorded Output
                    </div>
                    <div className="flex-1 overflow-y-auto space-y-3">
                      {detail.outputs && detail.outputs.length > 0 ? (
                        detail.outputs.map((output, idx) => {
                          const metadata = output.metadata || {};
                          const isFocusOutput = metadata.kind === 'focus';
                          const filesChanged: string[] = metadata.files_changed || [];
                          const plan: string[] = metadata.plan || [];
                          const focusKey = `${detail.id || 'run'}-focus-${idx}`;
                          const isExpanded = expandedFocusEntries[focusKey] ?? false;
                          return (
                            <div key={`${output.type || 'output'}-${idx}`} className="border border-border-dark rounded bg-black/40 p-3">
                              {isFocusOutput ? (
                                <>
                                  <div className="flex items-start justify-between gap-3">
                                    <div>
                                      <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500 mb-1">
                                        Focus
                                      </div>
                                      <div className="text-sm text-slate-100">
                                        {metadata.title || 'Focus completed'}
                                      </div>
                                      <div className="text-xs text-slate-400 mt-1 line-clamp-2">
                                        {output.content || 'No summary provided.'}
                                      </div>
                                    </div>
                                    <button
                                      onClick={() =>
                                        setExpandedFocusEntries(prev => ({
                                          ...prev,
                                          [focusKey]: !isExpanded
                                        }))
                                      }
                                      className="px-2 py-1 text-[10px] uppercase tracking-[0.2em] rounded-full border border-primary/40 text-primary bg-primary/10 hover:bg-primary/20 transition-colors"
                                    >
                                      {isExpanded ? 'Hide focus' : 'View focus'}
                                    </button>
                                  </div>
                                  <div className="mt-2 flex items-center gap-2 text-[10px] text-slate-500 uppercase tracking-[0.2em]">
                                    {filesChanged.length > 0 && (
                                      <span>{filesChanged.length} file{filesChanged.length === 1 ? '' : 's'}</span>
                                    )}
                                    {metadata.completed_at && (
                                      <span>Completed {formatDate(metadata.completed_at)}</span>
                                    )}
                                  </div>
                                  {isExpanded && (
                                    <div className="mt-3 space-y-3">
                                      {plan.length > 0 && (
                                        <div className="text-xs text-slate-200">
                                          <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500 mb-1">
                                            Plan
                                          </div>
                                          <ul className="space-y-1">
                                            {plan.map((step, stepIdx) => (
                                              <li key={`${step}-${stepIdx}`} className="flex items-start gap-2">
                                                <span className="text-slate-500">-</span>
                                                <span>{step}</span>
                                              </li>
                                            ))}
                                          </ul>
                                        </div>
                                      )}
                                      {filesChanged.length > 0 && (
                                        <div className="text-xs text-slate-200">
                                          <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500 mb-1">
                                            Files Changed
                                          </div>
                                          <ul className="space-y-1">
                                            {filesChanged.map((file, fileIdx) => (
                                              <li key={`${file}-${fileIdx}`} className="text-slate-300">
                                                {file}
                                              </li>
                                            ))}
                                          </ul>
                                        </div>
                                      )}
                                    </div>
                                  )}
                                </>
                              ) : (
                                <>
                                  <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500 mb-2">
                                    {output.type || 'output'}
                                  </div>
                                  <div className="text-xs text-slate-200 whitespace-pre-wrap font-mono">
                                    {output.content || 'No content'}
                                  </div>
                                </>
                              )}
                            </div>
                          );
                        })
                      ) : (
                        <div className="text-sm text-slate-500">No recorded outputs yet.</div>
                      )}
                      {detail.errors && detail.errors.length > 0 && (
                        <div className="border border-red-900/40 bg-red-950/30 rounded p-3">
                          <div className="text-[10px] uppercase tracking-[0.2em] text-red-300 mb-2 flex items-center gap-2">
                            <HiExclamation /> Errors
                          </div>
                          {detail.errors.map((err, idx) => (
                            <div key={`${err.code || 'error'}-${idx}`} className="text-xs text-red-200 mb-2">
                              {err.message}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>

                  {showCache && (
                    <div className="flex flex-col border border-border-dark bg-panel-dark/60 rounded-lg overflow-hidden">
                      <CachePanel runId={detail.id} projectId={detail.project_id} />
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
