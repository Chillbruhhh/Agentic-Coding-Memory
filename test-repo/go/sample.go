// Go sample file for AMP parser testing.
// Tests: structs, interfaces, functions, methods, type declarations, imports

package main

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"time"
)

// Type definitions
type UserID int64
type Status string

const (
	StatusActive   Status = "active"
	StatusInactive Status = "inactive"
	StatusPending  Status = "pending"
)

// Error definitions
var (
	ErrNotFound     = errors.New("user not found")
	ErrInvalidInput = errors.New("invalid input")
	ErrDuplicate    = errors.New("duplicate entry")
)

// Interface definitions
type Repository interface {
	FindByID(ctx context.Context, id UserID) (*User, error)
	Save(ctx context.Context, user *User) error
	Delete(ctx context.Context, id UserID) error
	List(ctx context.Context) ([]*User, error)
}

type Validator interface {
	Validate() error
}

// Struct definitions
type User struct {
	ID        UserID    `json:"id"`
	Name      string    `json:"name"`
	Email     string    `json:"email"`
	Status    Status    `json:"status"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

type UserRepository struct {
	mu      sync.RWMutex
	users   map[UserID]*User
	nextID  UserID
	logger  Logger
}

type Logger interface {
	Info(msg string, args ...interface{})
	Error(msg string, args ...interface{})
}

type SimpleLogger struct{}

// Method implementations for User
func (u *User) IsActive() bool {
	return u.Status == StatusActive
}

func (u *User) Validate() error {
	if u.Name == "" {
		return fmt.Errorf("%w: name cannot be empty", ErrInvalidInput)
	}
	if u.Email == "" {
		return fmt.Errorf("%w: email cannot be empty", ErrInvalidInput)
	}
	return nil
}

func (u *User) UpdateStatus(status Status) {
	u.Status = status
	u.UpdatedAt = time.Now()
}

// Constructor function
func NewUser(name, email string) *User {
	return &User{
		Name:      name,
		Email:     email,
		Status:    StatusActive,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}
}

// Method implementations for UserRepository
func NewUserRepository(logger Logger) *UserRepository {
	return &UserRepository{
		users:  make(map[UserID]*User),
		nextID: 1,
		logger: logger,
	}
}

func (r *UserRepository) FindByID(ctx context.Context, id UserID) (*User, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	
	user, exists := r.users[id]
	if !exists {
		return nil, ErrNotFound
	}
	
	return user, nil
}

func (r *UserRepository) Save(ctx context.Context, user *User) error {
	if err := user.Validate(); err != nil {
		return err
	}
	
	r.mu.Lock()
	defer r.mu.Unlock()
	
	if user.ID == 0 {
		user.ID = r.nextID
		r.nextID++
		user.CreatedAt = time.Now()
	}
	
	user.UpdatedAt = time.Now()
	r.users[user.ID] = user
	
	r.logger.Info("User saved", "id", user.ID, "name", user.Name)
	return nil
}

func (r *UserRepository) Delete(ctx context.Context, id UserID) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	
	if _, exists := r.users[id]; !exists {
		return ErrNotFound
	}
	
	delete(r.users, id)
	r.logger.Info("User deleted", "id", id)
	return nil
}

func (r *UserRepository) List(ctx context.Context) ([]*User, error) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	
	users := make([]*User, 0, len(r.users))
	for _, user := range r.users {
		users = append(users, user)
	}
	
	return users, nil
}

func (r *UserRepository) GetActiveUsers(ctx context.Context) ([]*User, error) {
	allUsers, err := r.List(ctx)
	if err != nil {
		return nil, err
	}
	
	activeUsers := make([]*User, 0)
	for _, user := range allUsers {
		if user.IsActive() {
			activeUsers = append(activeUsers, user)
		}
	}
	
	return activeUsers, nil
}

// SimpleLogger implementation
func (l *SimpleLogger) Info(msg string, args ...interface{}) {
	fmt.Printf("[INFO] %s %v\n", msg, args)
}

func (l *SimpleLogger) Error(msg string, args ...interface{}) {
	fmt.Printf("[ERROR] %s %v\n", msg, args)
}

// Utility functions
func ValidateEmail(email string) bool {
	return len(email) > 0 && contains(email, "@")
}

func contains(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}

// Generic function (Go 1.18+)
func Filter[T any](items []T, predicate func(T) bool) []T {
	result := make([]T, 0)
	for _, item := range items {
		if predicate(item) {
			result = append(result, item)
		}
	}
	return result
}

func Map[T, U any](items []T, mapper func(T) U) []U {
	result := make([]U, len(items))
	for i, item := range items {
		result[i] = mapper(item)
	}
	return result
}

// Main function
func main() {
	logger := &SimpleLogger{}
	repo := NewUserRepository(logger)
	ctx := context.Background()
	
	// Create users
	user1 := NewUser("Alice", "alice@example.com")
	user2 := NewUser("Bob", "bob@example.com")
	
	repo.Save(ctx, user1)
	repo.Save(ctx, user2)
	
	// List users
	users, _ := repo.List(ctx)
	for _, user := range users {
		fmt.Printf("User: %s (%s)\n", user.Name, user.Email)
	}
}
