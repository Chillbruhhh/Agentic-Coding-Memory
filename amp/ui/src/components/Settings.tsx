import React, { useState, useEffect } from 'react';
import { HiSave, HiRefresh, HiExclamation } from 'react-icons/hi';
import { BiServer, BiData, BiBrain } from 'react-icons/bi';

interface SettingsConfig {
  // Server Settings
  port: number;
  bindAddress: string;
  
  // Database Settings
  databaseUrl: string;
  dbUser: string;
  dbPass: string;
  
  // Embedding Provider
  embeddingProvider: 'openai' | 'openrouter' | 'ollama' | 'none';
  
  // OpenAI Settings
  openaiApiKey: string;
  openaiModel: string;
  openaiDimension: number;

  // OpenRouter Settings
  openrouterApiKey: string;
  openrouterModel: string;
  openrouterDimension: number;
  
  // Ollama Settings
  ollamaUrl: string;
  ollamaModel: string;
  ollamaDimension: number;

  // Index Model Settings
  indexProvider: 'openai' | 'openrouter' | 'ollama' | 'none';
  indexOpenaiModel: string;
  indexOpenrouterModel: string;
  indexOllamaModel: string;
  indexWorkers: number;
  indexRespectGitignore: boolean;
  
  // Legacy
  maxEmbeddingDimension: number;
}

export const Settings: React.FC = () => {
  const [config, setConfig] = useState<SettingsConfig>({
    port: 8105,
    bindAddress: '127.0.0.1',
    databaseUrl: 'ws://localhost:7505/rpc',
    dbUser: 'root',
    dbPass: 'root',
    embeddingProvider: 'none',
    openaiApiKey: '',
    openaiModel: 'text-embedding-3-small',
    openaiDimension: 1536,
    openrouterApiKey: '',
    openrouterModel: 'text-embedding-3-small',
    openrouterDimension: 1536,
    ollamaUrl: 'http://localhost:11434',
    ollamaModel: 'nomic-embed-text',
    ollamaDimension: 768,
    indexProvider: 'none',
    indexOpenaiModel: 'gpt-4o-mini',
    indexOpenrouterModel: 'openai/gpt-4o-mini',
    indexOllamaModel: 'llama3.1',
    indexWorkers: 4,
    indexRespectGitignore: true,
    maxEmbeddingDimension: 1536,
  });

  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [showOpenAiKey, setShowOpenAiKey] = useState(false);
  const [showOpenRouterKey, setShowOpenRouterKey] = useState(false);
  const [modelTab, setModelTab] = useState<'index' | 'embeddings'>('index');

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      setLoading(true);
      const response = await fetch('http://localhost:8105/v1/settings');
      if (!response.ok) throw new Error('Failed to load settings');
      const data = await response.json();
      setConfig(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async () => {
    try {
      setSaving(true);
      setError(null);
      setSuccess(false);
      
      const response = await fetch('http://localhost:8105/v1/settings', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      
      if (!response.ok) throw new Error('Failed to save settings');
      
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  const updateField = <K extends keyof SettingsConfig>(
    field: K,
    value: SettingsConfig[K]
  ) => {
    setConfig(prev => ({ ...prev, [field]: value }));
  };

  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-slate-400">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-6 max-w-5xl mx-auto space-y-6 w-full">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <h2 className="text-2xl font-display font-bold text-stone-100 mb-1 uppercase tracking-wider flex items-center gap-2">
            <span className="w-1 h-6 bg-primary inline-block"></span>
            System Configuration
          </h2>
          <p className="text-stone-500 text-sm font-mono">:: CONFIGURE AMP SERVER SETTINGS ::</p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={loadSettings}
            disabled={loading}
            className="px-4 py-2 bg-stone-800 border border-stone-700 text-stone-300 hover:bg-stone-700 hover:text-white transition-all flex items-center gap-2 text-sm font-mono uppercase disabled:opacity-50"
          >
            <HiRefresh className={loading ? 'animate-spin' : ''} />
            Reload
          </button>
          <button
            onClick={saveSettings}
            disabled={saving}
            className="px-4 py-2 bg-primary border border-red-600 text-black hover:bg-red-600 hover:text-white transition-all flex items-center gap-2 text-sm font-mono uppercase font-bold disabled:opacity-50 shadow-[0_0_10px_rgba(239,68,68,0.3)]"
          >
            <HiSave className={saving ? 'animate-pulse' : ''} />
            {saving ? 'Saving...' : 'Save Config'}
          </button>
        </div>
      </div>

      {/* Status Messages */}
      {error && (
        <div className="bg-red-900/20 border border-red-500 p-4 flex items-center gap-3">
          <HiExclamation className="text-red-500 text-xl flex-shrink-0" />
          <span className="text-red-400 text-sm font-mono">{error}</span>
        </div>
      )}

      {success && (
        <div className="bg-green-900/20 border border-green-500 p-4 flex items-center gap-3">
          <span className="text-green-400 text-sm font-mono">✓ Settings saved successfully</span>
        </div>
      )}

      {/* Server Settings */}
      <div className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-6 shadow-lg">
        <h3 className="text-lg font-display font-semibold flex items-center gap-2 text-stone-200 mb-6">
          <BiServer className="text-primary" />
          Server Settings
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Port</label>
            <input
              type="number"
              value={config.port}
              onChange={(e) => updateField('port', parseInt(e.target.value))}
              className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Bind Address</label>
            <input
              type="text"
              value={config.bindAddress}
              onChange={(e) => updateField('bindAddress', e.target.value)}
              className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
            />
          </div>
        </div>
      </div>

      {/* Database Settings */}
      <div className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-6 shadow-lg">
        <h3 className="text-lg font-display font-semibold flex items-center gap-2 text-stone-200 mb-6">
          <BiData className="text-primary" />
          Database Settings
        </h3>
        <div className="space-y-4">
          <div>
            <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Database URL</label>
            <input
              type="text"
              value={config.databaseUrl}
              onChange={(e) => updateField('databaseUrl', e.target.value)}
              placeholder="memory, file://amp.db, ws://localhost:7505/rpc"
              className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
            />
            <p className="text-xs text-stone-500 mt-1 font-mono">
              Examples: memory | file://amp.db | ws://localhost:7505/rpc
            </p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-mono text-stone-400 uppercase mb-2">DB User</label>
              <input
                type="text"
                value={config.dbUser}
                onChange={(e) => updateField('dbUser', e.target.value)}
                className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
              />
            </div>
            <div>
              <label className="block text-xs font-mono text-stone-400 uppercase mb-2">DB Password</label>
              <input
                type="password"
                value={config.dbPass}
                onChange={(e) => updateField('dbPass', e.target.value)}
                className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Model Settings */}
      <div className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-6 shadow-lg">
        <h3 className="text-lg font-display font-semibold flex items-center gap-2 text-stone-200 mb-6">
          <BiBrain className="text-primary" />
          Model Settings
        </h3>

        {/* Tabs */}
        <div className="flex gap-2 mb-6">
          {(['index', 'embeddings'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setModelTab(tab)}
              className={`px-4 py-2 text-sm font-mono uppercase transition-all ${
                modelTab === tab
                  ? 'bg-primary text-black border border-red-600 font-bold'
                  : 'bg-stone-800 text-stone-400 border border-stone-700 hover:bg-stone-700 hover:text-stone-200'
              }`}
            >
              {tab === 'index' ? 'Index Model' : 'Embeddings'}
            </button>
          ))}
        </div>

        {/* Index Model Settings */}
        {modelTab === 'index' && (
          <div className="space-y-6">
            <div>
              <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Provider</label>
              <div className="flex gap-2">
                {(['none', 'openai', 'openrouter', 'ollama'] as const).map((provider) => (
                  <button
                    key={provider}
                    onClick={() => updateField('indexProvider', provider)}
                    className={`px-4 py-2 text-sm font-mono uppercase transition-all ${
                      config.indexProvider === provider
                        ? 'bg-primary text-black border border-red-600 font-bold'
                        : 'bg-stone-800 text-stone-400 border border-stone-700 hover:bg-stone-700 hover:text-stone-200'
                    }`}
                  >
                    {provider}
                  </button>
                ))}
              </div>
            </div>

            <div>
              <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Index Workers</label>
              <input
                type="number"
                min={1}
                max={32}
                value={config.indexWorkers}
                onChange={(e) => updateField('indexWorkers', parseInt(e.target.value))}
                className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
              />
              <p className="text-xs text-stone-500 mt-2 font-mono">
                Higher values speed up indexing but increase API load.
              </p>
            </div>

            <div>
              <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Respect .gitignore</label>
              <button
                onClick={() => updateField('indexRespectGitignore', !config.indexRespectGitignore)}
                className={`px-4 py-2 text-sm font-mono uppercase transition-all ${
                  config.indexRespectGitignore
                    ? 'bg-primary text-black border border-red-600 font-bold'
                    : 'bg-stone-800 text-stone-400 border border-stone-700 hover:bg-stone-700 hover:text-stone-200'
                }`}
              >
                {config.indexRespectGitignore ? 'Enabled' : 'Disabled'}
              </button>
              <p className="text-xs text-stone-500 mt-2 font-mono">
                When enabled, files matched by .gitignore are skipped during indexing.
              </p>
            </div>

            {config.indexProvider === 'openai' && (
              <div className="space-y-4 border-t border-stone-800 pt-6">
                <h4 className="text-sm font-mono text-stone-300 uppercase">OpenAI Configuration</h4>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">API Key</label>
                  <div className="relative">
                    <input
                      type={showOpenAiKey ? 'text' : 'password'}
                      value={config.openaiApiKey}
                      onChange={(e) => updateField('openaiApiKey', e.target.value)}
                      placeholder="sk-..."
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none pr-20"
                    />
                    <button
                      onClick={() => setShowOpenAiKey(!showOpenAiKey)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-stone-500 hover:text-stone-300 font-mono uppercase"
                    >
                      {showOpenAiKey ? 'Hide' : 'Show'}
                    </button>
                  </div>
                </div>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Model</label>
                  <input
                    type="text"
                    value={config.indexOpenaiModel}
                    onChange={(e) => updateField('indexOpenaiModel', e.target.value)}
                    className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                  />
                </div>
              </div>
            )}

            {config.indexProvider === 'openrouter' && (
              <div className="space-y-4 border-t border-stone-800 pt-6">
                <h4 className="text-sm font-mono text-stone-300 uppercase">OpenRouter Configuration</h4>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">API Key</label>
                  <div className="relative">
                    <input
                      type={showOpenRouterKey ? 'text' : 'password'}
                      value={config.openrouterApiKey}
                      onChange={(e) => updateField('openrouterApiKey', e.target.value)}
                      placeholder="or-..."
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none pr-20"
                    />
                    <button
                      onClick={() => setShowOpenRouterKey(!showOpenRouterKey)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-stone-500 hover:text-stone-300 font-mono uppercase"
                    >
                      {showOpenRouterKey ? 'Hide' : 'Show'}
                    </button>
                  </div>
                </div>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Model</label>
                  <input
                    type="text"
                    value={config.indexOpenrouterModel}
                    onChange={(e) => updateField('indexOpenrouterModel', e.target.value)}
                    className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                  />
                </div>
              </div>
            )}

            {config.indexProvider === 'ollama' && (
              <div className="space-y-4 border-t border-stone-800 pt-6">
                <h4 className="text-sm font-mono text-stone-300 uppercase">Ollama Configuration</h4>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Ollama URL</label>
                  <input
                    type="text"
                    value={config.ollamaUrl}
                    onChange={(e) => updateField('ollamaUrl', e.target.value)}
                    placeholder="http://localhost:11434"
                    className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                  />
                </div>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Model</label>
                  <input
                    type="text"
                    value={config.indexOllamaModel}
                    onChange={(e) => updateField('indexOllamaModel', e.target.value)}
                    className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                  />
                </div>
              </div>
            )}

            {config.indexProvider === 'none' && (
              <div className="border-t border-stone-800 pt-6">
                <p className="text-sm text-stone-500 font-mono">
                  No index model configured. AI file logs will not be generated.
                </p>
              </div>
            )}
          </div>
        )}

        {/* Embedding Settings */}
        {modelTab === 'embeddings' && (
          <div className="space-y-6">
            <div>
              <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Provider</label>
              <div className="flex gap-2">
                {(['none', 'openai', 'openrouter', 'ollama'] as const).map((provider) => (
                  <button
                    key={provider}
                    onClick={() => updateField('embeddingProvider', provider)}
                    className={`px-4 py-2 text-sm font-mono uppercase transition-all ${
                      config.embeddingProvider === provider
                        ? 'bg-primary text-black border border-red-600 font-bold'
                        : 'bg-stone-800 text-stone-400 border border-stone-700 hover:bg-stone-700 hover:text-stone-200'
                    }`}
                  >
                    {provider}
                  </button>
                ))}
              </div>
            </div>

            {config.embeddingProvider === 'openai' && (
              <div className="space-y-4 border-t border-stone-800 pt-6">
                <h4 className="text-sm font-mono text-stone-300 uppercase">OpenAI Configuration</h4>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">API Key</label>
                  <div className="relative">
                    <input
                      type={showOpenAiKey ? 'text' : 'password'}
                      value={config.openaiApiKey}
                      onChange={(e) => updateField('openaiApiKey', e.target.value)}
                      placeholder="sk-..."
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none pr-20"
                    />
                    <button
                      onClick={() => setShowOpenAiKey(!showOpenAiKey)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-stone-500 hover:text-stone-300 font-mono uppercase"
                    >
                      {showOpenAiKey ? 'Hide' : 'Show'}
                    </button>
                  </div>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Model</label>
                    <input
                      type="text"
                      value={config.openaiModel}
                      onChange={(e) => updateField('openaiModel', e.target.value)}
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Dimension</label>
                    <input
                      type="number"
                      value={config.openaiDimension}
                      onChange={(e) => updateField('openaiDimension', parseInt(e.target.value))}
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                    />
                  </div>
                </div>
              </div>
            )}

            {config.embeddingProvider === 'openrouter' && (
              <div className="space-y-4 border-t border-stone-800 pt-6">
                <h4 className="text-sm font-mono text-stone-300 uppercase">OpenRouter Configuration</h4>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">API Key</label>
                  <div className="relative">
                    <input
                      type={showOpenRouterKey ? 'text' : 'password'}
                      value={config.openrouterApiKey}
                      onChange={(e) => updateField('openrouterApiKey', e.target.value)}
                      placeholder="or-..."
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none pr-20"
                    />
                    <button
                      onClick={() => setShowOpenRouterKey(!showOpenRouterKey)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 text-xs text-stone-500 hover:text-stone-300 font-mono uppercase"
                    >
                      {showOpenRouterKey ? 'Hide' : 'Show'}
                    </button>
                  </div>
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Model</label>
                    <input
                      type="text"
                      value={config.openrouterModel}
                      onChange={(e) => updateField('openrouterModel', e.target.value)}
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Dimension</label>
                    <input
                      type="number"
                      value={config.openrouterDimension}
                      onChange={(e) => updateField('openrouterDimension', parseInt(e.target.value))}
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                    />
                  </div>
                </div>
              </div>
            )}

            {config.embeddingProvider === 'ollama' && (
              <div className="space-y-4 border-t border-stone-800 pt-6">
                <h4 className="text-sm font-mono text-stone-300 uppercase">Ollama Configuration</h4>
                <div>
                  <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Ollama URL</label>
                  <input
                    type="text"
                    value={config.ollamaUrl}
                    onChange={(e) => updateField('ollamaUrl', e.target.value)}
                    placeholder="http://localhost:11434"
                    className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                  />
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Model</label>
                    <input
                      type="text"
                      value={config.ollamaModel}
                      onChange={(e) => updateField('ollamaModel', e.target.value)}
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Dimension</label>
                    <input
                      type="number"
                      value={config.ollamaDimension}
                      onChange={(e) => updateField('ollamaDimension', parseInt(e.target.value))}
                      className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
                    />
                  </div>
                </div>
              </div>
            )}

            {config.embeddingProvider === 'none' && (
              <div className="border-t border-stone-800 pt-6">
                <p className="text-sm text-stone-500 font-mono">
                  No embedding provider configured. Vector search will be disabled.
                </p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Advanced Settings */}
      <div className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] border border-stone-800 p-6 shadow-lg">
        <h3 className="text-lg font-display font-semibold flex items-center gap-2 text-stone-200 mb-6">
          Advanced Settings
        </h3>
        <div>
          <label className="block text-xs font-mono text-stone-400 uppercase mb-2">Max Embedding Dimension</label>
          <input
            type="number"
            value={config.maxEmbeddingDimension}
            onChange={(e) => updateField('maxEmbeddingDimension', parseInt(e.target.value))}
            className="w-full bg-stone-900 border border-stone-700 px-3 py-2 text-stone-200 font-mono text-sm focus:border-primary focus:outline-none"
          />
          <p className="text-xs text-stone-500 mt-1 font-mono">Legacy setting for maximum embedding dimension</p>
        </div>
      </div>
    </div>
  );
};
