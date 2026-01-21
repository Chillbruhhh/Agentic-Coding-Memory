/**
 * TypeScript sample file for AMP parser testing.
 * Tests: interfaces, classes, functions, types, imports, exports
 */

import { EventEmitter } from 'events';
import * as fs from 'fs';

// Type definitions
export type UserId = string | number;
export type Status = 'active' | 'inactive' | 'pending';

// Interface definitions
export interface User {
    id: UserId;
    name: string;
    email: string;
    status: Status;
    createdAt: Date;
}

export interface Repository<T> {
    findById(id: string): Promise<T | null>;
    save(entity: T): Promise<T>;
    delete(id: string): Promise<boolean>;
}

// Class with generics
export class InMemoryRepository<T extends { id: string }> implements Repository<T> {
    private data: Map<string, T> = new Map();
    
    constructor(private readonly name: string) {}
    
    async findById(id: string): Promise<T | null> {
        return this.data.get(id) || null;
    }
    
    async save(entity: T): Promise<T> {
        this.data.set(entity.id, entity);
        return entity;
    }
    
    async delete(id: string): Promise<boolean> {
        return this.data.delete(id);
    }
    
    getSize(): number {
        return this.data.size;
    }
}

// Abstract class
export abstract class BaseService {
    protected logger: Console;
    
    constructor() {
        this.logger = console;
    }
    
    abstract initialize(): Promise<void>;
    
    protected log(message: string): void {
        this.logger.log(`[${this.constructor.name}] ${message}`);
    }
}

// Concrete implementation
export class UserService extends BaseService {
    private repository: Repository<User>;
    
    constructor(repository: Repository<User>) {
        super();
        this.repository = repository;
    }
    
    async initialize(): Promise<void> {
        this.log('UserService initialized');
    }
    
    async createUser(name: string, email: string): Promise<User> {
        const user: User = {
            id: Date.now().toString(),
            name,
            email,
            status: 'active',
            createdAt: new Date()
        };
        
        return await this.repository.save(user);
    }
    
    async getUserById(id: UserId): Promise<User | null> {
        return await this.repository.findById(id.toString());
    }
}

// Utility functions
export function validateEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
}

export const formatDate = (date: Date): string => {
    return date.toISOString().split('T')[0];
};

// Higher-order function
export function createLogger(prefix: string) {
    return (message: string) => {
        console.log(`[${prefix}] ${message}`);
    };
}

// Async generator
export async function* fetchUsers(ids: UserId[]): AsyncGenerator<User> {
    for (const id of ids) {
        // Simulated async fetch
        yield {
            id,
            name: `User ${id}`,
            email: `user${id}@example.com`,
            status: 'active',
            createdAt: new Date()
        };
    }
}

// Namespace
export namespace Utils {
    export function isValidId(id: UserId): boolean {
        return id !== null && id !== undefined;
    }
    
    export function generateId(): string {
        return Math.random().toString(36).substr(2, 9);
    }
}
