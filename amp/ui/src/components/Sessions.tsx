import React, { useEffect, useMemo, useState } from 'react';
import { HiPlay, HiRefresh, HiClock, HiOutlineDocumentText, HiExclamation } from 'react-icons/hi';
import { useRuns, RunDetail, RunSummary } from '../hooks/useRuns';

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
  const [activeTab, setActiveTab] = useState<SessionTab>('live');
  const [selectedRun, setSelectedRun] = useState<RunSummary | null>(null);
  const [selectedDetail, setSelectedDetail] = useState<RunDetail | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);

  const liveRuns = useMemo(
    () => runs.filter(run => isLiveStatus(run.status)),
    [runs]
  );
  const historyRuns = useMemo(
    () => runs.filter(run => !isLiveStatus(run.status)),
    [runs]
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
          {visibleRuns.map(run => (
            <button
              key={run.id}
              onClick={() => handleSelectRun(run)}
              className={`w-full text-left bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-4 border-l-4 shadow-lg transition-all hover:border-primary/50 ${
                selectedRun?.id === run.id ? 'border-l-primary' : 'border-l-transparent'
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <div className="flex-1">
                  <div className="text-sm font-semibold text-slate-100">
                    {run.input_summary || run.provenance?.summary || 'Untitled Session'}
                  </div>
                  <div className="text-xs text-slate-500 mt-1">
                    {run.provenance?.agent || 'unknown'} · {run.project_id || 'default'}
                  </div>
                </div>
                <span className={`text-[10px] px-2 py-0.5 border rounded-full uppercase ${statusStyles(run.status)}`}>
                  {normalizeStatus(run.status)}
                </span>
              </div>
              <div className="mt-3 flex items-center justify-between text-xs text-slate-500">
                <span>Started {formatDate(run.created_at)}</span>
                <span>{formatDuration(run.duration_ms)}</span>
              </div>
            </button>
          ))}
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
                  <div className="text-xs uppercase tracking-[0.2em] text-primary">Session</div>
                  <div className="text-lg font-semibold text-slate-100 mt-1">
                    {detail.input_summary || detail.provenance?.summary || 'Untitled Session'}
                  </div>
                  <div className="text-xs text-slate-500 mt-1">
                    {detail.provenance?.agent || 'unknown'} · {detail.project_id || 'default'}
                  </div>
                </div>
                <span className={`text-[10px] px-2 py-0.5 border rounded-full uppercase ${statusStyles(detail.status)}`}>
                  {normalizeStatus(detail.status)}
                </span>
              </div>

              {detailLoading && (
                <div className="p-4 text-slate-500 text-sm">Loading session details...</div>
              )}

              {!detailLoading && (
                <div className="flex-1 overflow-hidden p-4 grid grid-cols-1 xl:grid-cols-2 gap-4">
                  <div className="flex flex-col border border-border-dark bg-panel-dark/60 rounded-lg p-4 overflow-hidden">
                    <div className="flex items-center gap-2 text-xs uppercase tracking-[0.2em] text-primary mb-3">
                      <HiPlay /> Current Focus
                    </div>
                    <div className="text-sm text-slate-200 leading-relaxed">
                      {detail.input_summary || 'No active summary recorded.'}
                    </div>
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
                        detail.outputs.map((output, idx) => (
                          <div key={`${output.type || 'output'}-${idx}`} className="border border-border-dark rounded bg-black/40 p-3">
                            <div className="text-[10px] uppercase tracking-[0.2em] text-slate-500 mb-2">
                              {output.type || 'output'}
                            </div>
                            <pre className="text-xs text-slate-200 whitespace-pre-wrap font-mono">
                              {output.content || 'No content'}
                            </pre>
                          </div>
                        ))
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
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
