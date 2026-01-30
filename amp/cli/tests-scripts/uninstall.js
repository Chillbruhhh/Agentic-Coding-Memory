#!/usr/bin/env node

const { existsSync, unlinkSync, rmdirSync } = require('fs');
const { join } = require('path');

const BINARY_NAME = process.platform === 'win32' ? 'amp.exe' : 'amp';
const BIN_DIR = join(__dirname, 'bin');
const TARGET_PATH = join(BIN_DIR, BINARY_NAME);

console.log('🗑️  Uninstalling AMP CLI...');

try {
  // Remove binary if it exists
  if (existsSync(TARGET_PATH)) {
    unlinkSync(TARGET_PATH);
    console.log('✅ Binary removed');
  }
  
  // Remove bin directory if empty
  if (existsSync(BIN_DIR)) {
    try {
      rmdirSync(BIN_DIR);
      console.log('✅ Cleaned up bin directory');
    } catch {
      // Directory not empty, that's okay
    }
  }
  
  console.log('✅ AMP CLI uninstalled successfully');
} catch (error) {
  console.error('⚠️  Error during uninstall:', error.message);
}
