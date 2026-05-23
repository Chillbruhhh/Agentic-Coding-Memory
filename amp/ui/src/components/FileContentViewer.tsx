import React, { useEffect, useMemo, useRef, useState } from 'react';
import { HiCode, HiOutlineDocument, HiOutlineExclamationCircle, HiChevronDown, HiChevronRight } from 'react-icons/hi';
import Prism from 'prismjs';
import 'prismjs/themes/prism-tomorrow.css';
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-jsx';
import 'prismjs/components/prism-tsx';
import 'prismjs/components/prism-python';
import 'prismjs/components/prism-rust';
import 'prismjs/components/prism-go';
import 'prismjs/components/prism-java';
import 'prismjs/components/prism-csharp';
import 'prismjs/components/prism-c';
import 'prismjs/components/prism-cpp';
import 'prismjs/components/prism-ruby';
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-yaml';
import 'prismjs/components/prism-toml';
import 'prismjs/components/prism-markdown';
import 'prismjs/components/prism-css';
import { FileNode } from '../hooks/useCodebases';

interface FileContentViewerProps {
  file: FileNode | null;
}

interface FileContentResponse {
  path: string;
  content: string;
  chunks: string[];
}

interface FileLog {
  summary?: string | null;
  summary_markdown?: string | null;
  purpose?: string | null;
  dependencies?: string[] | null;
  key_symbols?: string[] | null;
  notes?: string | null;
  file_path?: string | null;
  project_id?: string | null;
  updated_at?: string | null;
}

interface FileLogResponse {
  file_log: FileLog;
}

const MAX_FETCH_CHARS = 200000;

const extToPrism: Record<string, string> = {
  ts: 'typescript',
  tsx: 'tsx',
  js: 'javascript',
  jsx: 'jsx',
  mjs: 'javascript',
  cjs: 'javascript',
  py: 'python',
  pyi: 'python',
  rs: 'rust',
  go: 'go',
  java: 'java',
  kt: 'kotlin',
  cs: 'csharp',
  c: 'c',
  h: 'c',
  hpp: 'cpp',
  cpp: 'cpp',
  cc: 'cpp',
  cxx: 'cpp',
  rb: 'ruby',
  sh: 'bash',
  bash: 'bash',
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  toml: 'toml',
  md: 'markdown',
  css: 'css',
  scss: 'css',
};

const detectLanguage = (file: FileNode): string => {
  const explicit = (file.language || '').toLowerCase();
  if (explicit && Prism.languages[explicit]) return explicit;
  const ext = file.name.split('.').pop()?.toLowerCase();
  if (ext && extToPrism[ext] && Prism.languages[extToPrism[ext]]) return extToPrism[ext];
  return 'clike';
};

const reflowIfFlattened = (raw: string): string => {
  if (!raw) return raw;
  const newlineCount = (raw.match(/\n/g) || []).length;
  if (newlineCount > raw.length / 200) return raw;
  return raw
    .replace(/\*\//g, '*/\n')
    .replace(/;\s*/g, ';\n')
    .replace(/\{\s*/g, '{\n')
    .replace(/\}\s*/g, '\n}\n')
    .replace(/\n{3,}/g, '\n\n');
};

type RightPanelTab = 'symbols' | 'filelog';

export const FileContentViewer: React.FC<FileContentViewerProps> = ({ file }) => {
  const [content, setContent] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [showPanel, setShowPanel] = useState<boolean>(true);
  const [panelTab, setPanelTab] = useState<RightPanelTab>('filelog');
  const [fileLog, setFileLog] = useState<FileLog | null>(null);
  const [logLoading, setLogLoading] = useState<boolean>(false);
  const [logError, setLogError] = useState<string | null>(null);
  const requestIdRef = useRef(0);
  const logRequestIdRef = useRef(0);

  useEffect(() => {
    if (!file || file.type === 'folder') {
      setContent('');
      setError(null);
      setLoading(false);
      return;
    }

    const reqId = ++requestIdRef.current;
    setLoading(true);
    setError(null);
    setContent('');

    const encoded = encodeURIComponent(file.path);
    fetch(`http://localhost:8105/v1/codebase/file-contents/${encoded}?max_chars=${MAX_FETCH_CHARS}`)
      .then(async (res) => {
        if (!res.ok) {
          if (res.status === 404) {
            throw new Error('File content not indexed yet. Re-run `amp index` to capture file contents.');
          }
          throw new Error(`Server error: ${res.status} ${res.statusText}`);
        }
        return (await res.json()) as FileContentResponse;
      })
      .then((data) => {
        if (reqId !== requestIdRef.current) return;
        setContent(reflowIfFlattened(data.content || ''));
      })
      .catch((err) => {
        if (reqId !== requestIdRef.current) return;
        setError(err instanceof Error ? err.message : 'Failed to load file content');
      })
      .finally(() => {
        if (reqId !== requestIdRef.current) return;
        setLoading(false);
      });
  }, [file?.path, file?.type]);

  useEffect(() => {
    if (!file || file.type === 'folder') {
      setFileLog(null);
      setLogError(null);
      setLogLoading(false);
      return;
    }

    const reqId = ++logRequestIdRef.current;
    setLogLoading(true);
    setLogError(null);
    setFileLog(null);

    const encoded = encodeURIComponent(file.path);
    const projectParam = file.project_id ? `?project_id=${encodeURIComponent(file.project_id)}` : '';
    fetch(`http://localhost:8105/v1/codebase/file-log-objects/${encoded}${projectParam}`)
      .then(async (res) => {
        if (!res.ok) {
          if (res.status === 404) {
            throw new Error('No file log generated for this file');
          }
          if (res.status === 409) {
            throw new Error('Ambiguous file path - log spans multiple projects');
          }
          throw new Error(`Server error: ${res.status} ${res.statusText}`);
        }
        return (await res.json()) as FileLogResponse;
      })
      .then((data) => {
        if (reqId !== logRequestIdRef.current) return;
        setFileLog(data.file_log || null);
      })
      .catch((err) => {
        if (reqId !== logRequestIdRef.current) return;
        setLogError(err instanceof Error ? err.message : 'Failed to load file log');
      })
      .finally(() => {
        if (reqId !== logRequestIdRef.current) return;
        setLogLoading(false);
      });
  }, [file?.path, file?.type, file?.project_id]);

  const language = useMemo(() => (file ? detectLanguage(file) : 'clike'), [file?.path, file?.language]);

  const highlightedHtml = useMemo(() => {
    if (!content) return '';
    const grammar = Prism.languages[language] || Prism.languages.clike;
    try {
      return Prism.highlight(content, grammar, language);
    } catch {
      return content
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
    }
  }, [content, language]);

  const lineCount = useMemo(() => content.split('\n').length, [content]);

  if (!file) {
    return (
      <div className="flex-1 flex items-center justify-center bg-background-dark text-slate-400">
        <div className="text-center">
          <HiOutlineDocument size={48} className="mx-auto mb-4 opacity-50" />
          <div>Select a file to view its content</div>
        </div>
      </div>
    );
  }

  if (file.type === 'folder') {
    return (
      <div className="flex-1 flex items-center justify-center bg-background-dark text-slate-400">
        <div className="text-center">
          <div className="text-4xl mb-4">📁</div>
          <div>Folder selected: {file.name}</div>
          <div className="text-sm mt-2">{file.children?.length || 0} items</div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-col bg-background-dark min-h-0">
      <div className="px-4 py-2 border-b border-border-dark bg-black/30 flex items-center justify-between text-xs">
        <div className="flex items-center gap-2 min-w-0">
          <HiCode className="text-primary shrink-0" />
          <span className="text-slate-300 font-mono truncate" title={file.path}>
            {file.path}
          </span>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {language && language !== 'clike' && (
            <span className="px-2 py-0.5 bg-primary/10 text-primary rounded uppercase tracking-wider">
              {language}
            </span>
          )}
          <button
            onClick={() => setShowPanel((v) => !v)}
            className="px-2 py-0.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded transition-colors"
            title="Toggle info panel"
          >
            {showPanel ? 'Hide info' : 'Show info'}
          </button>
          <span className="text-slate-500">read-only</span>
        </div>
      </div>

      <div className="flex-1 flex min-h-0 overflow-hidden">
        <div className="flex-1 overflow-auto bg-[#0b0a08] min-w-0">
          {loading && (
            <div className="p-6 text-slate-400 text-sm flex items-center gap-2">
              <div className="w-3 h-3 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
              Loading file content…
            </div>
          )}

          {!loading && error && (
            <div className="p-6 text-amber-300 text-sm flex items-start gap-2">
              <HiOutlineExclamationCircle className="text-amber-400 mt-0.5 shrink-0" size={18} />
              <div>
                <div className="font-medium mb-1">Couldn't load file content</div>
                <div className="text-slate-400">{error}</div>
              </div>
            </div>
          )}

          {!loading && !error && content && (
            <div className="flex font-mono text-[12.5px] leading-[1.55] min-h-full">
              <div
                className="select-none text-right text-slate-600 px-3 py-3 border-r border-border-dark bg-black/40 sticky left-0"
                aria-hidden
              >
                {Array.from({ length: lineCount }, (_, i) => (
                  <div key={i}>{i + 1}</div>
                ))}
              </div>
              <pre
                className={`flex-1 px-4 py-3 whitespace-pre overflow-visible !bg-transparent !m-0 language-${language}`}
              >
                <code
                  className={`language-${language}`}
                  dangerouslySetInnerHTML={{ __html: highlightedHtml }}
                />
              </pre>
            </div>
          )}

          {!loading && !error && !content && (
            <div className="p-6 text-slate-500 text-sm">No content available for this file.</div>
          )}
        </div>

        {showPanel && (
          <RightInfoPanel
            file={file}
            symbolCount={file.symbols?.length || 0}
            activeTab={panelTab}
            onTabChange={setPanelTab}
            fileLog={fileLog}
            logLoading={logLoading}
            logError={logError}
          />
        )}
      </div>
    </div>
  );
};

interface RightInfoPanelProps {
  file: FileNode;
  symbolCount: number;
  activeTab: RightPanelTab;
  onTabChange: (tab: RightPanelTab) => void;
  fileLog: FileLog | null;
  logLoading: boolean;
  logError: string | null;
}

const RightInfoPanel: React.FC<RightInfoPanelProps> = ({
  file,
  symbolCount,
  activeTab,
  onTabChange,
  fileLog,
  logLoading,
  logError,
}) => (
  <div className="w-[380px] shrink-0 border-l border-border-dark bg-panel-dark flex flex-col min-h-0">
    <div className="flex border-b border-border-dark bg-black/30 shrink-0">
      <PanelTabButton
        label={`File log${fileLog ? '' : logLoading ? ' …' : ''}`}
        active={activeTab === 'filelog'}
        onClick={() => onTabChange('filelog')}
      />
      <PanelTabButton
        label={`Symbols (${symbolCount})`}
        active={activeTab === 'symbols'}
        onClick={() => onTabChange('symbols')}
      />
    </div>

    <div className="flex-1 overflow-y-auto">
      {activeTab === 'filelog' ? (
        <FileLogSection log={fileLog} loading={logLoading} error={logError} />
      ) : (
        <SymbolsSection symbols={file.symbols || []} />
      )}
    </div>
  </div>
);

const PanelTabButton: React.FC<{ label: string; active: boolean; onClick: () => void }> = ({
  label,
  active,
  onClick,
}) => (
  <button
    onClick={onClick}
    className={`flex-1 px-3 py-2 text-xs font-bold uppercase tracking-wider transition-colors ${
      active
        ? 'text-primary border-b-2 border-primary bg-black/20'
        : 'text-slate-400 hover:text-slate-200 hover:bg-black/10'
    }`}
  >
    {label}
  </button>
);

const SymbolsSection: React.FC<{ symbols: NonNullable<FileNode['symbols']> }> = ({ symbols }) => {
  if (!symbols.length) {
    return <div className="p-4 text-xs text-slate-500">No symbols detected.</div>;
  }
  return (
    <div className="p-2 space-y-1">
      {symbols.map((symbol, index) => (
        <div
          key={`${symbol.name}-${index}`}
          className="px-2 py-1.5 bg-black/30 rounded border border-border-dark hover:border-primary/50 transition-colors"
        >
          <div className="flex items-center gap-2">
            <span className="font-mono text-xs text-slate-200 truncate">{symbol.name}</span>
            <span className="px-1.5 py-0.5 bg-slate-700 text-slate-300 text-[10px] rounded shrink-0">
              {symbol.type}
            </span>
          </div>
          {symbol.signature && (
            <div className="text-[11px] text-slate-500 font-mono mt-1 truncate" title={symbol.signature}>
              {symbol.signature}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

interface CollapsibleProps {
  title: string;
  count?: number;
  defaultOpen?: boolean;
  children: React.ReactNode;
}

const Collapsible: React.FC<CollapsibleProps> = ({ title, count, defaultOpen = true, children }) => {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className="border-b border-border-dark">
      <button
        onClick={() => setOpen((v) => !v)}
        className="w-full flex items-center gap-1.5 px-3 py-2 hover:bg-black/20 transition-colors"
      >
        {open ? <HiChevronDown size={14} className="text-slate-500" /> : <HiChevronRight size={14} className="text-slate-500" />}
        <span className="text-[11px] font-bold uppercase tracking-wider text-slate-300">{title}</span>
        {typeof count === 'number' && (
          <span className="text-[10px] text-slate-500">({count})</span>
        )}
      </button>
      {open && <div className="px-3 pb-3">{children}</div>}
    </div>
  );
};

// The FileLog `summary` field is sometimes a structured FILE_LOG v1 markdown
// blob. Extract the human-friendly Purpose paragraph instead of dumping the
// whole header.
const extractPurposeFromSummary = (text: string | null | undefined): string | null => {
  if (!text) return null;
  const match = text.match(/##\s*Purpose\s*\n([\s\S]*?)(?:\n##\s|$)/i);
  if (match) return match[1].trim();
  return null;
};

const FileLogSection: React.FC<{ log: FileLog | null; loading: boolean; error: string | null }> = ({
  log,
  loading,
  error,
}) => {
  if (loading) {
    return (
      <div className="p-4 text-xs text-slate-400 flex items-center gap-2">
        <div className="w-3 h-3 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
        Loading file log…
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 text-xs text-slate-500">
        <div className="flex items-start gap-1.5">
          <HiOutlineExclamationCircle className="text-slate-600 mt-0.5 shrink-0" size={14} />
          <div>{error}</div>
        </div>
      </div>
    );
  }

  if (!log) {
    return <div className="p-4 text-xs text-slate-500">No file log available.</div>;
  }

  const purposeFromSummary = extractPurposeFromSummary(log.summary || log.summary_markdown);
  const purpose = log.purpose || purposeFromSummary;
  const summary = log.summary_markdown || (purposeFromSummary ? null : log.summary);
  const deps = (log.dependencies || []).filter(Boolean);
  const symbols = (log.key_symbols || []).filter(Boolean);

  return (
    <div>
      {purpose && (
        <Collapsible title="Purpose" defaultOpen>
          <p className="text-xs text-slate-300 leading-relaxed whitespace-pre-wrap">{purpose}</p>
        </Collapsible>
      )}

      {summary && !purpose && (
        <Collapsible title="Summary" defaultOpen>
          <pre className="text-[11px] text-slate-400 leading-relaxed whitespace-pre-wrap font-mono">
            {summary}
          </pre>
        </Collapsible>
      )}

      {symbols.length > 0 && (
        <Collapsible title="Key symbols" count={symbols.length} defaultOpen>
          <div className="flex flex-wrap gap-1.5">
            {symbols.map((s, i) => (
              <span
                key={`${s}-${i}`}
                className="px-2 py-0.5 bg-black/40 border border-slate-700 rounded text-[11px] font-mono text-slate-300"
              >
                {s}
              </span>
            ))}
          </div>
        </Collapsible>
      )}

      {deps.length > 0 && (
        <Collapsible title="Dependencies" count={deps.length} defaultOpen={false}>
          <div className="space-y-1">
            {deps.map((d, i) => (
              <div
                key={`${d}-${i}`}
                className="px-2 py-1 bg-black/30 rounded text-[11px] font-mono text-slate-400 truncate"
                title={d}
              >
                {d}
              </div>
            ))}
          </div>
        </Collapsible>
      )}

      {log.notes && (
        <Collapsible title="Notes" defaultOpen={false}>
          <p className="text-xs text-slate-400 whitespace-pre-wrap">{log.notes}</p>
        </Collapsible>
      )}

      {log.updated_at && (
        <div className="px-3 py-2 text-[10px] text-slate-600 uppercase tracking-wider">
          Indexed {new Date(log.updated_at).toLocaleString()}
        </div>
      )}
    </div>
  );
};
