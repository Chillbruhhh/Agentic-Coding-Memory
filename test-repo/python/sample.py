"""
Python sample file for AMP parser testing.
Tests: functions, classes, methods, imports, variables
"""

import os
import sys
from typing import List, Dict, Optional
from dataclasses import dataclass

# Module-level constant
MAX_RETRIES = 3
API_VERSION = "v1"

@dataclass
class User:
    """User data model"""
    id: int
    name: str
    email: str
    active: bool = True
    
    def to_dict(self) -> Dict[str, any]:
        """Convert user to dictionary"""
        return {
            "id": self.id,
            "name": self.name,
            "email": self.email,
            "active": self.active
        }

class DatabaseConnection:
    """Database connection manager"""
    
    def __init__(self, host: str, port: int):
        self.host = host
        self.port = port
        self._connection = None
    
    def connect(self) -> bool:
        """Establish database connection"""
        print(f"Connecting to {self.host}:{self.port}")
        self._connection = True
        return True
    
    def disconnect(self):
        """Close database connection"""
        if self._connection:
            self._connection = None
            print("Disconnected")
    
    def execute_query(self, query: str) -> List[Dict]:
        """Execute SQL query"""
        if not self._connection:
            raise ConnectionError("Not connected to database")
        return []

def calculate_fibonacci(n: int) -> int:
    """Calculate nth Fibonacci number"""
    if n <= 1:
        return n
    return calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)

def process_data(data: List[Dict], filter_fn=None) -> List[Dict]:
    """Process data with optional filter"""
    if filter_fn:
        return [item for item in data if filter_fn(item)]
    return data

async def fetch_user_data(user_id: int) -> Optional[User]:
    """Async function to fetch user data"""
    # Simulated async operation
    return User(id=user_id, name="Test User", email="test@example.com")

def main():
    """Main entry point"""
    db = DatabaseConnection("localhost", 5432)
    db.connect()
    
    users = [
        User(1, "Alice", "alice@example.com"),
        User(2, "Bob", "bob@example.com")
    ]
    
    for user in users:
        print(user.to_dict())
    
    db.disconnect()

if __name__ == "__main__":
    main()
