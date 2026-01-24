import React, { useState, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { HiRefresh } from 'react-icons/hi';
import { GiTrashCan } from 'react-icons/gi';
import { SiGraphql, SiObsidian } from 'react-icons/si';
import { BiVector } from 'react-icons/bi';
import { MdTimeline } from 'react-icons/md';
import { useArtifacts, ArtifactSummary } from '../hooks/useArtifacts';
import { Artifact, ArtifactType } from '../types/amp';

// Type labels for display
const TYPE_LABELS: Record<ArtifactType, string> = {
  decision: 'Decision',
  filelog: 'File Log',
  note: 'Note',
  changeset: 'Changeset'
};

export const Artifacts: React.FC = () => {
  const [selectedArtifact, setSelectedArtifact] = useState<ArtifactSummary | null>(null);
  const [selectedDetail, setSelectedDetail] = useState<Artifact | null>(null);
  const { artifacts, loading, error, refetch, fetchArtifactDetails, deleteArtifact, getLayerCounts } = useArtifacts();

  // Fetch details when artifact is selected
  useEffect(() => {
    if (selectedArtifact) {
      fetchArtifactDetails(selectedArtifact.id).then(detail => {
        setSelectedDetail(detail);
      });
    } else {
      setSelectedDetail(null);
    }
  }, [selectedArtifact, fetchArtifactDetails]);

  const layerCounts = getLayerCounts();

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return 'Unknown';
    // Check if date is valid and not a current-time fallback
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return 'Unknown';
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const renderMemoryLayerBadges = (artifact: ArtifactSummary) => (
    <div className="flex items-center gap-1">
      {artifact.memory_layers.graph && (
        <span className="p-1 rounded bg-red-950/30" title="Graph Layer">
          <SiGraphql className="w-3 h-3 text-red-400" />
        </span>
      )}
      {artifact.memory_layers.vector && (
        <span className="p-1 rounded bg-blue-950/30" title="Vector Layer">
          <BiVector className="w-3 h-3 text-blue-400" />
        </span>
      )}
      {artifact.memory_layers.temporal && (
        <span className="p-1 rounded bg-green-950/30" title="Temporal Layer">
          <MdTimeline className="w-3 h-3 text-green-400" />
        </span>
      )}
    </div>
  );

  const renderDetailContent = () => {
    if (!selectedDetail) {
      return (
        <div className="flex-1 flex items-center justify-center text-slate-500">
          Select an artifact to view details.
        </div>
      );
    }

    const handleDelete = async () => {
      if (!selectedArtifact) return;
      const confirmDelete = window.confirm(`Delete artifact "${selectedArtifact.title}"? This cannot be undone.`);
      if (!confirmDelete) return;
      const ok = await deleteArtifact(selectedArtifact.id);
      if (ok) {
        setSelectedArtifact(null);
        setSelectedDetail(null);
        refetch();
      }
    };

    switch (selectedDetail.type) {
      case 'decision':
        return (
          <div className="p-4 space-y-4 overflow-y-auto">
            <div className="flex items-center gap-2 mb-4">
              <SiObsidian className="w-5 h-5 text-red-500" />
              <h3 className="text-lg font-semibold text-slate-200 flex-1">{selectedDetail.title}</h3>
              <button
                onClick={handleDelete}
                className="p-1.5 rounded text-slate-400 hover:text-red-300 hover:bg-red-900/20 transition-colors"
                title="Delete artifact"
              >
                <GiTrashCan className="w-4 h-4" />
              </button>
              {selectedDetail.status && (
                <span className={`px-2 py-0.5 text-xs rounded-full ${
                  selectedDetail.status === 'accepted' ? 'bg-green-900/50 text-green-300' :
                  selectedDetail.status === 'proposed' ? 'bg-amber-900/50 text-amber-300' :
                  selectedDetail.status === 'deprecated' ? 'bg-red-900/50 text-red-300' :
                  'bg-slate-800 text-slate-400'
                }`}>
                  {selectedDetail.status}
                </span>
              )}
            </div>
            <div className="space-y-3">
              <div>
                <h4 className="text-xs uppercase text-slate-500 mb-1">Context</h4>
                <p className="text-sm text-slate-300 bg-slate-900/50 p-3 rounded">{selectedDetail.context}</p>
              </div>
              <div>
                <h4 className="text-xs uppercase text-slate-500 mb-1">Decision</h4>
                <p className="text-sm text-slate-300 bg-slate-900/50 p-3 rounded">{selectedDetail.decision}</p>
              </div>
              <div>
                <h4 className="text-xs uppercase text-slate-500 mb-1">Consequences</h4>
                <p className="text-sm text-slate-300 bg-slate-900/50 p-3 rounded">{selectedDetail.consequences}</p>
              </div>
              {selectedDetail.alternatives && selectedDetail.alternatives.length > 0 && (
                <div>
                  <h4 className="text-xs uppercase text-slate-500 mb-1">Alternatives Considered</h4>
                  <ul className="list-disc list-inside text-sm text-slate-400 bg-slate-900/50 p-3 rounded">
                    {selectedDetail.alternatives.map((alt, i) => (
                      <li key={i}>{alt}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          </div>
        );

      case 'filelog':
        return (
          <div className="p-4 space-y-4 overflow-y-auto">
            <div className="flex items-center gap-2 mb-4">
              <SiObsidian className="w-5 h-5 text-red-500" />
              <h3 className="text-lg font-semibold text-slate-200 flex-1">{selectedDetail.title}</h3>
              <button
                onClick={handleDelete}
                className="p-1.5 rounded text-slate-400 hover:text-red-300 hover:bg-red-900/20 transition-colors"
                title="Delete artifact"
              >
                <GiTrashCan className="w-4 h-4" />
              </button>
            </div>
            <div className="space-y-3">
              <div>
                <h4 className="text-xs uppercase text-slate-500 mb-1">File Path</h4>
                <code className="text-sm text-blue-400 bg-slate-900/50 p-2 rounded block font-mono">{selectedDetail.file_path}</code>
              </div>
              <div>
                <h4 className="text-xs uppercase text-slate-500 mb-1">Summary</h4>
                <p className="text-sm text-slate-300 bg-slate-900/50 p-3 rounded">{selectedDetail.summary}</p>
              </div>
              {selectedDetail.symbols && selectedDetail.symbols.length > 0 && (
                <div>
                  <h4 className="text-xs uppercase text-slate-500 mb-1">Symbols</h4>
                  <div className="flex flex-wrap gap-1">
                    {selectedDetail.symbols.map((sym, i) => (
                      <span key={i} className="text-xs px-2 py-1 bg-blue-900/30 text-blue-300 rounded font-mono">{sym}</span>
                    ))}
                  </div>
                </div>
              )}
              {selectedDetail.change_history && selectedDetail.change_history.length > 0 && (
                <div>
                  <h4 className="text-xs uppercase text-slate-500 mb-1">Change History</h4>
                  <div className="space-y-2">
                    {selectedDetail.change_history.map((change, i) => (
                      <div key={i} className="text-sm bg-slate-900/50 p-2 rounded">
                        <span className="text-slate-500">{formatDate(change.timestamp)}</span>
                        <span className="text-slate-400 mx-2">-</span>
                        <span className="text-slate-300">{change.description}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        );

      case 'note':
        return (
          <div className="p-4 space-y-4 overflow-y-auto">
            <div className="flex items-center gap-2 mb-4">
              <SiObsidian className="w-5 h-5 text-red-500" />
              <h3 className="text-lg font-semibold text-slate-200 flex-1">{selectedDetail.title}</h3>
              <button
                onClick={handleDelete}
                className="p-1.5 rounded text-slate-400 hover:text-red-300 hover:bg-red-900/20 transition-colors"
                title="Delete artifact"
              >
                <GiTrashCan className="w-4 h-4" />
              </button>
              {selectedDetail.category && (
                <span className={`px-2 py-0.5 text-xs rounded-full ${
                  selectedDetail.category === 'insight' ? 'bg-amber-900/50 text-amber-300' :
                  selectedDetail.category === 'todo' ? 'bg-blue-900/50 text-blue-300' :
                  selectedDetail.category === 'question' ? 'bg-purple-900/50 text-purple-300' :
                  selectedDetail.category === 'warning' ? 'bg-red-900/50 text-red-300' :
                  'bg-slate-800 text-slate-400'
                }`}>
                  {selectedDetail.category}
                </span>
              )}
            </div>
            <div className="prose prose-invert prose-sm max-w-none">
              <div className="text-sm text-slate-300 bg-slate-900/50 p-4 rounded">
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                  {selectedDetail.content || ''}
                </ReactMarkdown>
              </div>
            </div>
          </div>
        );

      case 'changeset':
        return (
          <div className="p-4 space-y-4 overflow-y-auto">
            <div className="flex items-center gap-2 mb-4">
              <SiObsidian className="w-5 h-5 text-red-500" />
              <h3 className="text-lg font-semibold text-slate-200 flex-1">{selectedDetail.title}</h3>
              <button
                onClick={handleDelete}
                className="p-1.5 rounded text-slate-400 hover:text-red-300 hover:bg-red-900/20 transition-colors"
                title="Delete artifact"
              >
                <GiTrashCan className="w-4 h-4" />
              </button>
            </div>
            <div className="space-y-3">
              <div>
                <h4 className="text-xs uppercase text-slate-500 mb-1">Description</h4>
                <p className="text-sm text-slate-300 bg-slate-900/50 p-3 rounded">{selectedDetail.description}</p>
              </div>
              {selectedDetail.diff_summary && (
                <div>
                  <h4 className="text-xs uppercase text-slate-500 mb-1">Diff Summary</h4>
                  <pre className="text-xs text-slate-300 bg-slate-900/50 p-3 rounded font-mono overflow-x-auto">{selectedDetail.diff_summary}</pre>
                </div>
              )}
              <div>
                <h4 className="text-xs uppercase text-slate-500 mb-1">Files Changed ({selectedDetail.files_changed.length})</h4>
                <ul className="space-y-1">
                  {selectedDetail.files_changed.map((file, i) => (
                    <li key={i} className="text-sm text-blue-400 font-mono bg-slate-900/50 px-2 py-1 rounded">{file}</li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b border-border-dark bg-panel-dark flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h2 className="text-sm font-bold text-slate-200 uppercase tracking-wider">Artifacts</h2>
          <button
            onClick={() => refetch()}
            className="p-1.5 rounded text-slate-400 hover:text-primary hover:bg-red-900/20 transition-colors"
            title="Refresh"
          >
            <HiRefresh className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
          </button>
        </div>

        {/* Memory Layer Stats */}
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-1 text-xs">
            <SiGraphql className="w-3.5 h-3.5 text-red-400" />
            <span className="text-slate-400">{layerCounts.graph}</span>
          </div>
          <div className="flex items-center gap-1 text-xs">
            <BiVector className="w-3.5 h-3.5 text-blue-400" />
            <span className="text-slate-400">{layerCounts.vector}</span>
          </div>
          <div className="flex items-center gap-1 text-xs">
            <MdTimeline className="w-3.5 h-3.5 text-green-400" />
            <span className="text-slate-400">{layerCounts.temporal}</span>
          </div>
        </div>

        {/* Artifact Count */}
        <div className="flex items-center gap-2">
          <div className="px-3 py-1.5 text-xs font-medium rounded bg-primary text-white flex items-center gap-1.5">
            <SiObsidian className="w-3.5 h-3.5" />
            {artifacts.length}
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-[380px_1fr] gap-0 overflow-hidden">
        {/* Artifact List */}
        <div className="border-r border-border-dark overflow-y-auto bg-panel-dark/50">
          {loading && artifacts.length === 0 ? (
            <div className="p-4 text-center text-slate-500">Loading artifacts...</div>
          ) : error ? (
            <div className="p-4 text-center text-red-400">{error}</div>
          ) : artifacts.length === 0 ? (
            <div className="p-4 text-center text-slate-500">
              No artifacts found.
            </div>
          ) : (
            <div className="divide-y divide-border-dark">
              {artifacts.map((artifact) => {
                const isSelected = selectedArtifact?.id === artifact.id;
                const typeLabel = TYPE_LABELS[artifact.type] || artifact.type;

                return (
                  <div
                    key={artifact.id}
                    onClick={() => setSelectedArtifact(artifact)}
                    className={`p-3 cursor-pointer transition-colors ${
                      isSelected
                        ? 'bg-red-950/30 border-l-2 border-primary'
                        : 'hover:bg-slate-800/50 border-l-2 border-transparent'
                    }`}
                  >
                    <div className="flex items-start gap-3">
                      <div className={`p-2 rounded ${isSelected ? 'bg-red-900/40' : 'bg-slate-800/50'}`}>
                        <SiObsidian className="w-4 h-4 text-red-500" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between gap-2">
                          <span className="text-sm font-medium text-slate-200 truncate">{artifact.title}</span>
                          {renderMemoryLayerBadges(artifact)}
                        </div>
                        <div className="text-xs text-slate-500 mt-0.5">
                          <span className="text-red-400/70">{typeLabel}</span>
                          <span className="mx-1">•</span>
                          {formatDate(artifact.created_at)}
                          {artifact.agent_id && <span className="ml-2">by {artifact.agent_id}</span>}
                        </div>
                        {artifact.preview && (
                          <p className="text-xs text-slate-400 mt-1 line-clamp-2">{artifact.preview}</p>
                        )}
                        {artifact.tags && artifact.tags.length > 0 && (
                          <div className="flex flex-wrap gap-1 mt-1">
                            {artifact.tags.slice(0, 3).map((tag, i) => (
                              <span key={i} className="text-[10px] px-1.5 py-0.5 bg-slate-800 text-slate-400 rounded">{tag}</span>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Detail Panel */}
        <div className="flex flex-col overflow-hidden bg-background-dark">
          {renderDetailContent()}
        </div>
      </div>
    </div>
  );
};
