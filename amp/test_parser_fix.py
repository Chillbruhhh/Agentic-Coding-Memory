#!/usr/bin/env python3

import requests
import json
import tempfile
import os

# Test the codebase parser with a simple Python file
def test_parser_fix():
    # Create a test Python file
    test_code = '''
def hello_world():
    """A simple function"""
    print("Hello, world!")

class TestClass:
    """A test class"""
    
    def method_one(self):
        """A method"""
        return "method"
    
    def method_two(self):
        return "another method"

# A variable assignment
test_variable = "hello"
another_var = 42
'''
    
    # Write to temp file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
        f.write(test_code)
        temp_file = f.name
    
    try:
        # Test the parser endpoint
        url = "http://localhost:8105/v1/codebase/parse-file"
        
        with open(temp_file, 'r') as f:
            file_content = f.read()
        
        payload = {
            "file_path": temp_file,
            "content": file_content,
            "language": "python"
        }
        
        response = requests.post(url, json=payload)
        
        if response.status_code == 200:
            result = response.json()
            print("✅ Parser test successful!")
            print(f"Found {len(result.get('symbols', []))} symbols:")
            
            for symbol in result.get('symbols', []):
                print(f"  - {symbol['symbol_type']}: {symbol['name']} (line {symbol['start_line'] + 1})")
            
            # Check if we have proper symbol types (not "unknown")
            symbol_types = [s['symbol_type'] for s in result.get('symbols', [])]
            if 'unknown' in symbol_types:
                print("❌ Still finding 'unknown' symbol types")
                return False
            else:
                print("✅ All symbols have proper types!")
                return True
        else:
            print(f"❌ Parser test failed: {response.status_code}")
            print(response.text)
            return False
            
    finally:
        # Clean up
        os.unlink(temp_file)

if __name__ == "__main__":
    test_parser_fix()
