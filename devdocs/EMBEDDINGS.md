# Embedding Configuration Guide

AMP supports vector embeddings for semantic search using OpenAI or Ollama.

## Quick Setup

1. **Copy the example configuration:**
   ```bash
   cd amp/server
   cp .env.example .env
   ```

2. **Choose and configure a provider** (edit `.env`):

### Option A: OpenAI (Cloud-based)

```bash
EMBEDDING_PROVIDER=openai
OPENAI_API_KEY=sk-your-key-here
EMBEDDING_MODEL=text-embedding-3-small
EMBEDDING_DIMENSION=1536
```

**Models:**
- `text-embedding-3-small` - 1536 dims, $0.02/1M tokens (recommended)
- `text-embedding-3-large` - 3072 dims, $0.13/1M tokens (higher quality)

### Option B: Ollama (Local)

```bash
EMBEDDING_PROVIDER=ollama
OLLAMA_URL=http://localhost:11434
EMBEDDING_MODEL=nomic-embed-text
EMBEDDING_DIMENSION=768
```

**Setup Ollama:**
```bash
# Install Ollama from https://ollama.ai
ollama pull nomic-embed-text
ollama serve
```

**Models:**
- `nomic-embed-text` - 768 dims, fast, good quality
- `mxbai-embed-large` - 1024 dims, higher quality
- `all-minilm` - 384 dims, very fast

### Option C: Disabled (Default)

```bash
EMBEDDING_PROVIDER=none
```

## Testing

Run the comprehensive test:
```powershell
cd amp/scripts
.\test-embeddings-comprehensive.ps1
```

This will:
1. Create test objects with embeddings
2. Verify embeddings are generated
3. Test semantic search
4. Validate ranking quality

## How It Works

1. **Auto-generation**: Embeddings are automatically generated when objects are created or updated
2. **Semantic Search**: Text queries are converted to embeddings and matched using cosine similarity
3. **Graceful Degradation**: If embeddings are disabled, text search still works

## Troubleshooting

**No embeddings generated:**
- Check `.env` file exists in `amp/server/`
- Verify `EMBEDDING_PROVIDER` is set correctly
- For OpenAI: Check API key is valid
- For Ollama: Ensure Ollama is running (`ollama serve`)

**Slow embedding generation:**
- OpenAI: ~100-500ms per object (network latency)
- Ollama: ~50-200ms per object (local processing)
- Consider batch operations for large datasets

**Dimension mismatch:**
- Ensure `EMBEDDING_DIMENSION` matches your model
- OpenAI text-embedding-3-small: 1536
- Ollama nomic-embed-text: 768

## Cost Considerations

**OpenAI:**
- text-embedding-3-small: ~$0.02 per 1M tokens
- Average object: ~50-200 tokens
- 10,000 objects: ~$0.01-0.04

**Ollama:**
- Free (runs locally)
- Requires ~2GB RAM for nomic-embed-text
- GPU optional but recommended for speed
