---
name: rag-query
description: Query knowledge base using RAG (Retrieval-Augmented Generation) platform. Use when: (1) User asks for knowledge base information, (2) User needs to query specific documents, (3) Any request involving knowledge retrieval or document search.
homepage: https://github.com/openclaw/openclaw
metadata:
  {
    "openclaw":
      {
        "emoji": "🔍",
        "requires": { "bins": ["uv"] },
        "env": ["RAG_API_URL"],
        "primaryEnv": "RAG_API_URL",
        "install":
          [
            {
              "id": "uv-brew",
              "kind": "brew",
              "formula": "uv",
              "bins": ["uv"],
              "label": "Install uv (brew)",
            },
          ],
      },
  }
---

# RAG Query - Knowledge Base Search

Query your RAG platform knowledge base with natural language questions.

## Basic Query

```bash
uv run {baseDir}/scripts/main.py --question "你的问题"
```

## Advanced Options

Query with specific document:

```bash
uv run {baseDir}/scripts/main.py --question "问题" --doc-id "doc_123"
```

Custom API endpoint:

```bash
uv run {baseDir}/scripts/main.py --question "问题" --api-url "http://localhost:8080"
```

Custom timeout:

```bash
uv run {baseDir}/scripts/main.py --question "问题" --timeout 180
```

## Configuration

**Environment Variables:**

- `RAG_API_URL` - RAG service endpoint (default: http://192.168.18.57:5000)
- `RAG_TIMEOUT` - Request timeout in seconds (default: 120)

**Config File:**

Set in `~/.openclaw/openclaw.json`:

```json
{
  "skills": {
    "rag-query": {
      "env": {
        "RAG_API_URL": "http://your-rag-service:5000",
        "RAG_TIMEOUT": "180"
      }
    }
  }
}
```

## Usage Examples

**User:** "What is the RAG platform?"
**Execute:** `uv run {baseDir}/scripts/main.py --question "What is the RAG platform?"`

**User:** "How to upload documents?"
**Execute:** `uv run {baseDir}/scripts/main.py --question "How to upload documents?"`

**User:** "Search in document abc123: user manual"
**Execute:** `uv run {baseDir}/scripts/main.py --question "user manual" --doc-id "abc123"`

## Notes

- The script outputs the RAG response directly
- Supports Chinese and English queries
- Requires RAG service to be running and accessible
- Timeout errors indicate the RAG service is down or slow
