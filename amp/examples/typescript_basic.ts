/**
 * AMP TypeScript SDK Example
 * Demonstrates basic usage of the Agent Memory Protocol client.
 */

import { v4 as uuidv4 } from 'uuid';

// Note: This will work after SDK generation
// import { Configuration, ObjectsApi, QueryApi } from 'amp-client';

interface AmpObject {
  id: string;
  type: 'symbol' | 'decision' | 'changeset' | 'run';
  tenant_id: string;
  project_id: string;
  created_at: string;
  updated_at: string;
  provenance: {
    agent: string;
    model?: string;
    tools?: string[];
    summary: string;
  };
  links?: Array<{ type: string; target: string }>;
  embedding?: number[];
}

interface Symbol extends AmpObject {
  type: 'symbol';
  name: string;
  kind: 'file' | 'module' | 'class' | 'function' | 'variable' | 'type';
  path: string;
  language: string;
  content_hash?: string;
  signature?: string;
  documentation?: string;
}

interface Decision extends AmpObject {
  type: 'decision';
  title: string;
  problem: string;
  rationale: string;
  outcome: string;
  status?: 'proposed' | 'accepted' | 'rejected' | 'superseded';
}

/**
 * Mock client for demonstration until SDK is generated
 */
class MockAmpClient {
  async createSymbol(
    name: string,
    kind: Symbol['kind'],
    path: string,
    language: string
  ): Promise<Symbol> {
    const symbol: Symbol = {
      id: uuidv4(),
      type: 'symbol',
      tenant_id: 'default',
      project_id: 'example_project',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      provenance: {
        agent: 'typescript_example',
        summary: `Created symbol ${name}`,
      },
      name,
      kind,
      path,
      language,
    };
    
    console.log(`Created symbol: ${symbol.name} (${symbol.id})`);
    return symbol;
  }

  async createDecision(
    title: string,
    problem: string,
    rationale: string,
    outcome: string
  ): Promise<Decision> {
    const decision: Decision = {
      id: uuidv4(),
      type: 'decision',
      tenant_id: 'default',
      project_id: 'example_project',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      provenance: {
        agent: 'typescript_example',
        summary: `Made decision: ${title}`,
      },
      title,
      problem,
      rationale,
      outcome,
      status: 'accepted',
    };
    
    console.log(`Created decision: ${decision.title} (${decision.id})`);
    return decision;
  }

  async queryObjects(text: string, limit: number = 10) {
    console.log(`Querying for: '${text}' (limit: ${limit})`);
    return {
      results: [],
      trace_id: uuidv4(),
      total_count: 0,
      execution_time_ms: 42,
    };
  }
}

async function main() {
  console.log('🚀 AMP TypeScript SDK Example');
  console.log('='.repeat(40));

  // Initialize client
  const client = new MockAmpClient();

  // 1. Create some symbols
  console.log('\n📝 Creating symbols...');
  const mainFn = await client.createSymbol(
    'main',
    'function',
    'src/main.rs',
    'rust'
  );

  const configStruct = await client.createSymbol(
    'Config',
    'type',
    'src/config.rs',
    'rust'
  );

  // 2. Create a decision
  console.log('\n🤔 Making a decision...');
  const decision = await client.createDecision(
    'Choose web framework for AMP',
    'Need a fast, type-safe web framework for the Rust server',
    'Axum provides excellent performance and integrates well with tokio ecosystem',
    'Use Axum as the web framework'
  );

  // 3. Query the memory
  console.log('\n🔍 Querying memory...');
  const results = await client.queryObjects('rust functions');
  console.log(
    `Query returned ${results.total_count} results in ${results.execution_time_ms}ms`
  );

  console.log('\n✅ Example completed!');
  console.log('\nNext steps:');
  console.log('1. Start the AMP server: cd server && cargo run');
  console.log('2. Generate the real SDK: ./scripts/generate-sdks.sh');
  console.log('3. Replace MockAmpClient with the generated SDK');
}

// Run the example
main().catch(console.error);
