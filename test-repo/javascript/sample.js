/**
 * JavaScript sample file for AMP parser testing.
 * Tests: classes, functions, arrow functions, imports, exports
 */

const EventEmitter = require('events');
const fs = require('fs');

// Constants
const MAX_CONNECTIONS = 100;
const DEFAULT_TIMEOUT = 5000;

// Class definition
class ConnectionPool extends EventEmitter {
    constructor(maxSize = MAX_CONNECTIONS) {
        super();
        this.maxSize = maxSize;
        this.connections = [];
        this.available = [];
    }
    
    async acquire() {
        if (this.available.length > 0) {
            return this.available.pop();
        }
        
        if (this.connections.length < this.maxSize) {
            const conn = await this.createConnection();
            this.connections.push(conn);
            return conn;
        }
        
        throw new Error('Connection pool exhausted');
    }
    
    release(connection) {
        this.available.push(connection);
        this.emit('released', connection);
    }
    
    async createConnection() {
        return { id: Date.now(), active: true };
    }
    
    getStats() {
        return {
            total: this.connections.length,
            available: this.available.length,
            inUse: this.connections.length - this.available.length
        };
    }
}

// Factory function
function createUser(name, email) {
    return {
        id: generateId(),
        name,
        email,
        createdAt: new Date(),
        isActive: true
    };
}

// Arrow functions
const generateId = () => Math.random().toString(36).substr(2, 9);

const validateEmail = (email) => {
    const regex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return regex.test(email);
};

// Higher-order function
const withRetry = (fn, maxRetries = 3) => {
    return async (...args) => {
        for (let i = 0; i < maxRetries; i++) {
            try {
                return await fn(...args);
            } catch (error) {
                if (i === maxRetries - 1) throw error;
                await sleep(1000 * Math.pow(2, i));
            }
        }
    };
};

// Async function
async function fetchData(url) {
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    return await response.json();
}

// Generator function
function* fibonacci(n) {
    let a = 0, b = 1;
    for (let i = 0; i < n; i++) {
        yield a;
        [a, b] = [b, a + b];
    }
}

// Async generator
async function* streamData(items) {
    for (const item of items) {
        await sleep(100);
        yield item;
    }
}

// Utility functions
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

// Object with methods
const userService = {
    users: new Map(),
    
    create(name, email) {
        const user = createUser(name, email);
        this.users.set(user.id, user);
        return user;
    },
    
    findById(id) {
        return this.users.get(id);
    },
    
    delete(id) {
        return this.users.delete(id);
    },
    
    getAll() {
        return Array.from(this.users.values());
    }
};

// Module exports
module.exports = {
    ConnectionPool,
    createUser,
    generateId,
    validateEmail,
    withRetry,
    fetchData,
    fibonacci,
    streamData,
    userService
};

// ES6 exports (if using modules)
// export { ConnectionPool, createUser, generateId, validateEmail };
// export default userService;
