#!/usr/bin/env python3
# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "requests>=2.32.0",
# ]
# ///
"""
RAG Skill - Knowledge base Q&A using Retrieval-Augmented Generation.

Usage:
    python main.py --question "What is RAG?"
    python main.py -q "How to upload PDF?" --doc-id "doc_123"
    python main.py -q "Your question" --api-url "http://localhost:8080"

Environment Variables:
    RAG_API_URL    - RAG service endpoint (default: http://localhost:5000)
    RAG_TIMEOUT    - Request timeout in seconds (default: 120)
"""

import argparse
import io
import json
import os
import sys

from pathlib import Path
from typing import Optional

# Force UTF-8 output for Chinese/English support
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

# Default values
DEFAULT_API_URL = "http://localhost:5000"
DEFAULT_TIMEOUT = 120


def get_api_url(provided_url: Optional[str]) -> str:
    """Get API URL from argument first, then environment variable, then default."""
    if provided_url:
        return provided_url.rstrip('/')
    return os.environ.get('RAG_API_URL', DEFAULT_API_URL).rstrip('/')


def get_timeout(provided_timeout: Optional[int]) -> int:
    """Get timeout from argument first, then environment variable, then default."""
    if provided_timeout is not None:
        return provided_timeout
    try:
        return int(os.environ.get('RAG_TIMEOUT', str(DEFAULT_TIMEOUT)))
    except ValueError:
        return DEFAULT_TIMEOUT


def query_rag(
    question: str,
    api_url: str,
    doc_id: Optional[str] = None,
    timeout: int = DEFAULT_TIMEOUT,
) -> str:
    """
    Query the RAG service for an answer.

    Args:
        question: The user's question
        api_url: The RAG service endpoint
        doc_id: Optional document ID to limit search scope
        timeout: Request timeout in seconds

    Returns:
        The answer text, or error message
    """
    import requests

    url = f"{api_url}/api/chat"
    headers = {"Content-Type": "application/json"}
    payload = {"message": question}

    if doc_id:
        payload["doc_id"] = doc_id

    try:
        response = requests.post(
            url=url,
            headers=headers,
            json=payload,
            timeout=timeout
        )
        response.raise_for_status()

        result = response.json()

        # Handle different response formats
        if result.get("success") and "data" in result and "answer" in result["data"]:
            return result["data"]["answer"]
        elif result.get("code") == 0 and "data" in result and "answer" in result["data"]:
            return result["data"]["answer"]
        else:
            return f"Error: Invalid response format - {json.dumps(result, ensure_ascii=False)}"

    except requests.exceptions.Timeout:
        return f"Error: Request timeout after {timeout} seconds"
    except requests.exceptions.ConnectionError:
        return f"Error: Cannot connect to RAG service at {api_url}"
    except requests.exceptions.RequestException as e:
        return f"Error: Request failed - {str(e)}"
    except json.JSONDecodeError:
        return f"Error: Non-JSON response from server"
    except Exception as e:
        return f"Error: Unexpected error - {str(e)}"


def main():
    parser = argparse.ArgumentParser(
        description="Query knowledge base using RAG (Retrieval-Augmented Generation)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --question "What is RAG?"
  %(prog)s -q "How to upload PDF?" --doc-id "doc_123"
  %(prog)s -q "Your question" --api-url "http://localhost:8080"

Environment Variables:
  RAG_API_URL    API endpoint (default: http://localhost:5000)
  RAG_TIMEOUT    Timeout in seconds (default: 120)
        """
    )

    parser.add_argument(
        '--question', '-q',
        required=True,
        help='Your question in natural language'
    )
    parser.add_argument(
        '--doc-id', '-d',
        dest='doc_id',
        help='Limit search to specific document ID'
    )
    parser.add_argument(
        '--api-url', '-u',
        dest='api_url',
        help='RAG service endpoint (overrides RAG_API_URL env var)'
    )
    parser.add_argument(
        '--timeout', '-t',
        type=int,
        dest='timeout',
        help='Request timeout in seconds (overrides RAG_TIMEOUT env var)'
    )
    parser.add_argument(
        '--version', '-v',
        action='version',
        version='%(prog)s 1.0.0'
    )

    args = parser.parse_args()

    # Get configuration
    api_url = get_api_url(args.api_url)
    timeout = get_timeout(args.timeout)

    # Query RAG service
    answer = query_rag(
        question=args.question,
        api_url=api_url,
        doc_id=args.doc_id,
        timeout=timeout
    )

    # Output answer (OpenClaw will capture this)
    print(answer)


if __name__ == "__main__":
    main()
