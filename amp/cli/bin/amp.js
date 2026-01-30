#!/usr/bin/env node

const { spawn } = require('child_process');
const { join } = require('path');
const { existsSync } = require('fs');

const BINARY_NAME = process.platform === 'win32' ? 'amp.exe' : 'amp';
const BINARY_PATH = join(__dirname, BINARY_NAME);

// Check if binary exists
if (!existsSync(BINARY_PATH)) {
  console.error('❌ AMP CLI binary not found.');
  console.error('   Try reinstalling: npm install -g @amp-protocol/cli');
  process.exit(1);
}

// Forward all arguments to the Rust binary
const child = spawn(BINARY_PATH, process.argv.slice(2), {
  stdio: 'inherit',
  windowsHide: false
});

child.on('exit', (code) => {
  process.exit(code || 0);
});

child.on('error', (error) => {
  console.error('❌ Failed to execute AMP CLI:', error.message);
  process.exit(1);
});
