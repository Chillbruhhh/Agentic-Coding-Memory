# AMP Codebase Embedding & Graph Indexing Strategy

**High-Level Design Rationale and Architecture**

------------------------------------------------------------------------

## Purpose

This document summarizes the agreed strategy for how **Agentic Memory
Protocol (AMP)** indexes and embeds a codebase using a hybrid of:

-   Graph modeling (structure and relationships)
-   Vector embeddings (semantic recall)
-   Incremental chunking (scalable full‑coverage embeddings)

The goal is to provide agents with: - High‑precision retrieval -
Scalable semantic recall - Deterministic provenance - Efficient
incremental updates - Low operational cost

This design supports both hackathon MVP delivery and long‑term
production scaling.

------------------------------------------------------------------------

## Mental Model

AMP treats a repository as **structured knowledge**, not raw text.

Two complementary systems work together:

  -----------------------------------------------------------------------
  System                                   Role
  ---------------------------------------- ------------------------------
  Graph Database                           Structural reasoning,
                                           dependency tracking,
                                           provenance

  Vector Index                             Semantic similarity and recall
  -----------------------------------------------------------------------

Vectors answer: \> "What content is semantically similar?"

Graphs answer: \> "What is connected, impacted, or owned?"

Agents always operate using both.

------------------------------------------------------------------------

## Codebase Parsing Pipeline

When a repository is indexed:

1.  Files are discovered and filtered
2.  Language parsers extract Symbols
3.  Graph relationships are created
4.  File content is chunked
5.  Chunks are embedded
6.  Objects are persisted into AMP

This creates a multi‑layer memory model.

------------------------------------------------------------------------

## Object Layers

### 1. File

Represents a physical file in the repository.

Stores: - Path - Language - Hash - Size - Modification timestamps

Graph edges: - Repo → CONTAINS → File

------------------------------------------------------------------------

### 2. Symbol (Primary Precision Layer)

Represents structured code elements: - Functions - Classes - Methods -
Modules - Interfaces

Why embed symbols: - Small semantic units - High precision recall -
Cheap embedding cost - Stable across refactors - Graph friendly

Graph edges: - File → CONTAINS → Symbol - Symbol → CALLS / DEPENDS_ON →
Symbol

Symbols always receive embeddings.

------------------------------------------------------------------------

### 3. FileChunk (Full Coverage Layer)

Represents chunked slices of file content.

Chunking strategy: - \~500 tokens per chunk - Optional overlap: 50--100
tokens - One file yields N chunks based on size

Examples: - 300 tokens → 1 chunk - 1200 tokens → 3 chunks

Why chunk files: - Full semantic coverage - Enables deep recall inside
large files - Avoids embedding entire file as one noisy vector

Chunk metadata: - file_path - chunk_index - token_range or line_range -
content_hash - language - embedding

Graph edges: - File → CONTAINS → FileChunk

Chunks are embedded incrementally.

------------------------------------------------------------------------

### 4. FileLog (Semantic Summary Layer)

Represents a compressed semantic summary of a file.

Contains: - File purpose - Key symbols - Important behaviors - Known
constraints - Agent notes

Why embed FileLogs: - Stable semantic anchor - Cheap to update -
Excellent navigation signal for agents - Reduces noisy chunk recall

FileLogs always receive embeddings.

------------------------------------------------------------------------

## Why Not Embed Entire Files as Single Vectors

Embedding whole files creates problems:

-   High noise from boilerplate
-   Poor locality in retrieval
-   Expensive re‑embedding on small edits
-   Weak precision for agents
-   Large token costs

Chunking + summaries preserve precision and scalability.

------------------------------------------------------------------------

## Incremental Update Strategy

When a file changes:

1.  Recompute file hash
2.  Re-chunk the file
3.  Re-embed only chunks whose hash changed
4.  Remove orphaned chunks
5.  Update related symbols
6.  Refresh FileLog summary if needed

This avoids full re-indexing.

------------------------------------------------------------------------

## Hybrid Retrieval Strategy

Queries operate across three layers:

1.  Vector similarity (Symbols + FileChunks + FileLogs)
2.  Graph traversal (dependencies, ownership, provenance)
3.  Text filtering (metadata and tags)

Hybrid scoring combines available signals gracefully.

Vector search is optional --- text + graph must still work.

------------------------------------------------------------------------

## MVP Scope (Hackathon)

### Required

-   Symbol parsing + embeddings
-   FileChunk chunking + embeddings
-   Graph relationships
-   Incremental updates
-   Hybrid query fallback behavior

### Optional

-   FileLog summarization (if time allows)
-   Chunk overlap tuning

------------------------------------------------------------------------

## Post‑Hackathon Enhancements

-   ChangeSet embeddings
-   Repo snapshot embeddings
-   Cross‑repo linking
-   Semantic diff embeddings
-   Visualization tooling
-   Advanced pruning policies

------------------------------------------------------------------------

## Why This Is Documented as Markdown

Markdown is used intentionally because:

-   Human readable and reviewable
-   Easy to version control
-   Portable across tooling
-   Friendly for agents to consume
-   Supports diagrams and examples
-   Serves as protocol documentation

This file acts as: - Design contract - Contributor onboarding
reference - Agent instruction artifact - Architectural source of truth

------------------------------------------------------------------------

## Summary

AMP indexes codebases as structured knowledge:

-   Graphs provide structure and reasoning
-   Vectors provide semantic recall
-   Chunking provides scalable coverage
-   Symbols provide precision anchors
-   FileLogs provide compressed intent
-   Incremental updates preserve performance

This architecture enables agents to operate with persistent,
explainable, and scalable memory.

------------------------------------------------------------------------
