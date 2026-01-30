#!/usr/bin/env node

const { execSync } = require('child_process');
const { existsSync, mkdirSync, copyFileSync, chmodSync } = require('fs');
const { join } = require('path');
const os = require('os');

const BINARY_NAME = process.platform === 'win32' ? 'amp.exe' : 'amp';
const BIN_DIR = join(__dirname, 'bin');
const TARGET_PATH = join(BIN_DIR, BINARY_NAME);

console.log('🚀 Installing AMP CLI...');

// Check if Rust/Cargo is installed
function hasRust() {
  try {
    execSync('cargo --version', { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

// Build the Rust binary
function buildBinary() {
  console.log('🔨 Building AMP CLI from source...');
  
  try {
    // Build in release mode
    execSync('cargo build --release', {
      cwd: __dirname,
      stdio: 'inherit'
    });
    
    // Create bin directory if it doesn't exist
    if (!existsSync(BIN_DIR)) {
      mkdirSync(BIN_DIR, { recursive: true });
    }
    
    // Copy the binary to bin directory
    const sourcePath = join(__dirname, 'target', 'release', BINARY_NAME);
    
    if (!existsSync(sourcePath)) {
      throw new Error(`Binary not found at ${sourcePath}`);
    }
    
    copyFileSync(sourcePath, TARGET_PATH);
    
    // Make executable on Unix systems
    if (process.platform !== 'win32') {
      chmodSync(TARGET_PATH, 0o755);
    }
    
    console.log('✅ AMP CLI installed successfully!');
    console.log('📋 Usage: amp --help');
    console.log('🎯 Index a project: amp index');
    console.log('📊 Check status: amp status');
    
  } catch (error) {
    console.error('❌ Failed to build AMP CLI:', error.message);
    process.exit(1);
  }
}

// Main installation logic
function install() {
  // Check if binary already exists (pre-built)
  if (existsSync(TARGET_PATH)) {
    console.log('✅ AMP CLI binary already installed');
    return;
  }
  
  // Check if Rust is available
  if (!hasRust()) {
    console.error('❌ Rust/Cargo not found. Please install Rust first:');
    console.error('   Visit: https://rustup.rs/');
    console.error('   Or run: curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh');
    process.exit(1);
  }
  
  // Build from source
  buildBinary();
}

// Run installation
install();
