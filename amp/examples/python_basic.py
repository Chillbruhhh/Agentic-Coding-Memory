#!/usr/bin/env python3
"""
AMP Python SDK Example
Demonstrates basic usage of the Agentic Memory Protocol client.
"""

import asyncio
import uuid
from datetime import datetime
from typing import List

# Note: This will work after SDK generation
# from amp_client import ApiClient, Configuration, ObjectsApi, QueryApi

class MockAmpClient:
    """Mock client for demonstration until SDK is generated"""
    
    async def create_symbol(self, name: str, kind: str, path: str, language: str):
        """Create a symbol object"""
        symbol = {
            "id": str(uuid.uuid4()),
            "type": "symbol",
            "tenant_id": "default",
            "project_id": "example_project",
            "created_at": datetime.utcnow().isoformat(),
            "updated_at": datetime.utcnow().isoformat(),
            "provenance": {
                "agent": "python_example",
                "summary": f"Created symbol {name}"
            },
            "name": name,
            "kind": kind,
            "path": path,
            "language": language
        }
        print(f"Created symbol: {symbol['name']} ({symbol['id']})")
        return symbol
    
    async def create_decision(self, title: str, problem: str, rationale: str, outcome: str):
        """Create a decision object"""
        decision = {
            "id": str(uuid.uuid4()),
            "type": "decision",
            "tenant_id": "default",
            "project_id": "example_project",
            "created_at": datetime.utcnow().isoformat(),
            "updated_at": datetime.utcnow().isoformat(),
            "provenance": {
                "agent": "python_example",
                "summary": f"Made decision: {title}"
            },
            "title": title,
            "problem": problem,
            "rationale": rationale,
            "outcome": outcome,
            "status": "accepted"
        }
        print(f"Created decision: {decision['title']} ({decision['id']})")
        return decision
    
    async def query_objects(self, text: str, limit: int = 10):
        """Query objects using text search"""
        print(f"Querying for: '{text}' (limit: {limit})")
        # Mock response
        return {
            "results": [],
            "trace_id": str(uuid.uuid4()),
            "total_count": 0,
            "execution_time_ms": 42
        }

async def main():
    """Demonstrate AMP usage patterns"""
    print("🚀 AMP Python SDK Example")
    print("=" * 40)
    
    # Initialize client
    client = MockAmpClient()
    
    # 1. Create some symbols
    print("\n📝 Creating symbols...")
    main_fn = await client.create_symbol(
        name="main",
        kind="function", 
        path="src/main.rs",
        language="rust"
    )
    
    config_struct = await client.create_symbol(
        name="Config",
        kind="type",
        path="src/config.rs", 
        language="rust"
    )
    
    # 2. Create a decision
    print("\n🤔 Making a decision...")
    decision = await client.create_decision(
        title="Choose database for AMP",
        problem="Need a database that supports vector search and graph relations",
        rationale="SurrealDB provides both vector indexing and graph capabilities in a single system",
        outcome="Use SurrealDB as the storage backend"
    )
    
    # 3. Query the memory
    print("\n🔍 Querying memory...")
    results = await client.query_objects("rust functions")
    print(f"Query returned {results['total_count']} results in {results['execution_time_ms']}ms")
    
    print("\n✅ Example completed!")
    print("\nNext steps:")
    print("1. Start the AMP server: cd server && cargo run")
    print("2. Generate the real SDK: ./scripts/generate-sdks.sh")
    print("3. Replace MockAmpClient with the generated SDK")

if __name__ == "__main__":
    asyncio.run(main())
