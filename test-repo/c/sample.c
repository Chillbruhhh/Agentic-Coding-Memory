/**
 * C sample file for AMP parser testing.
 * Tests: functions, structs, enums, typedefs, #include directives
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>

// Macro definitions
#define MAX_NAME_LENGTH 100
#define MAX_EMAIL_LENGTH 200
#define MAX_USERS 1000

// Enum definitions
typedef enum {
    STATUS_ACTIVE,
    STATUS_INACTIVE,
    STATUS_PENDING,
    STATUS_SUSPENDED
} Status;

typedef enum {
    ROLE_ADMIN,
    ROLE_USER,
    ROLE_GUEST
} UserRole;

// Struct definitions
typedef struct {
    long id;
    char name[MAX_NAME_LENGTH];
    char email[MAX_EMAIL_LENGTH];
    Status status;
    UserRole role;
    long created_at;
    long updated_at;
} User;

typedef struct {
    User* users[MAX_USERS];
    int count;
    long next_id;
} UserRepository;

typedef struct {
    const char* message;
    int code;
} Error;

// Function declarations
User* user_create(const char* name, const char* email);
void user_destroy(User* user);
bool user_validate(const User* user, Error* error);
bool user_is_active(const User* user);
void user_update_status(User* user, Status status);
void user_print(const User* user);

UserRepository* repository_create(void);
void repository_destroy(UserRepository* repo);
User* repository_save(UserRepository* repo, User* user);
User* repository_find_by_id(UserRepository* repo, long id);
bool repository_delete(UserRepository* repo, long id);
int repository_count(UserRepository* repo);
User** repository_find_active(UserRepository* repo, int* count);

bool validate_email(const char* email);
bool validate_name(const char* name);
long get_current_timestamp(void);
void log_info(const char* message);
void log_error(const char* message);

// User functions implementation
User* user_create(const char* name, const char* email) {
    User* user = (User*)malloc(sizeof(User));
    if (user == NULL) {
        return NULL;
    }
    
    user->id = 0;
    strncpy(user->name, name, MAX_NAME_LENGTH - 1);
    user->name[MAX_NAME_LENGTH - 1] = '\0';
    strncpy(user->email, email, MAX_EMAIL_LENGTH - 1);
    user->email[MAX_EMAIL_LENGTH - 1] = '\0';
    user->status = STATUS_ACTIVE;
    user->role = ROLE_USER;
    user->created_at = get_current_timestamp();
    user->updated_at = user->created_at;
    
    return user;
}

void user_destroy(User* user) {
    if (user != NULL) {
        free(user);
    }
}

bool user_validate(const User* user, Error* error) {
    if (user == NULL) {
        if (error != NULL) {
            error->message = "User is NULL";
            error->code = -1;
        }
        return false;
    }
    
    if (!validate_name(user->name)) {
        if (error != NULL) {
            error->message = "Invalid name";
            error->code = 1;
        }
        return false;
    }
    
    if (!validate_email(user->email)) {
        if (error != NULL) {
            error->message = "Invalid email";
            error->code = 2;
        }
        return false;
    }
    
    return true;
}

bool user_is_active(const User* user) {
    return user != NULL && user->status == STATUS_ACTIVE;
}

void user_update_status(User* user, Status status) {
    if (user != NULL) {
        user->status = status;
        user->updated_at = get_current_timestamp();
    }
}

void user_print(const User* user) {
    if (user == NULL) {
        printf("User is NULL\n");
        return;
    }
    
    printf("User{id=%ld, name='%s', email='%s', status=%d}\n",
           user->id, user->name, user->email, user->status);
}

// Repository functions implementation
UserRepository* repository_create(void) {
    UserRepository* repo = (UserRepository*)malloc(sizeof(UserRepository));
    if (repo == NULL) {
        return NULL;
    }
    
    repo->count = 0;
    repo->next_id = 1;
    
    for (int i = 0; i < MAX_USERS; i++) {
        repo->users[i] = NULL;
    }
    
    return repo;
}

void repository_destroy(UserRepository* repo) {
    if (repo == NULL) {
        return;
    }
    
    for (int i = 0; i < repo->count; i++) {
        if (repo->users[i] != NULL) {
            user_destroy(repo->users[i]);
        }
    }
    
    free(repo);
}

User* repository_save(UserRepository* repo, User* user) {
    if (repo == NULL || user == NULL) {
        return NULL;
    }
    
    Error error;
    if (!user_validate(user, &error)) {
        log_error(error.message);
        return NULL;
    }
    
    if (repo->count >= MAX_USERS) {
        log_error("Repository is full");
        return NULL;
    }
    
    if (user->id == 0) {
        user->id = repo->next_id++;
        user->created_at = get_current_timestamp();
    }
    
    user->updated_at = get_current_timestamp();
    repo->users[repo->count++] = user;
    
    char log_msg[256];
    snprintf(log_msg, sizeof(log_msg), "User saved: %s (ID: %ld)", user->name, user->id);
    log_info(log_msg);
    
    return user;
}

User* repository_find_by_id(UserRepository* repo, long id) {
    if (repo == NULL) {
        return NULL;
    }
    
    for (int i = 0; i < repo->count; i++) {
        if (repo->users[i] != NULL && repo->users[i]->id == id) {
            return repo->users[i];
        }
    }
    
    return NULL;
}

bool repository_delete(UserRepository* repo, long id) {
    if (repo == NULL) {
        return false;
    }
    
    for (int i = 0; i < repo->count; i++) {
        if (repo->users[i] != NULL && repo->users[i]->id == id) {
            user_destroy(repo->users[i]);
            
            // Shift remaining users
            for (int j = i; j < repo->count - 1; j++) {
                repo->users[j] = repo->users[j + 1];
            }
            
            repo->users[repo->count - 1] = NULL;
            repo->count--;
            
            return true;
        }
    }
    
    return false;
}

int repository_count(UserRepository* repo) {
    return repo != NULL ? repo->count : 0;
}

User** repository_find_active(UserRepository* repo, int* count) {
    if (repo == NULL || count == NULL) {
        return NULL;
    }
    
    User** active_users = (User**)malloc(sizeof(User*) * repo->count);
    if (active_users == NULL) {
        return NULL;
    }
    
    int active_count = 0;
    for (int i = 0; i < repo->count; i++) {
        if (user_is_active(repo->users[i])) {
            active_users[active_count++] = repo->users[i];
        }
    }
    
    *count = active_count;
    return active_users;
}

// Utility functions implementation
bool validate_email(const char* email) {
    if (email == NULL || strlen(email) == 0) {
        return false;
    }
    
    return strchr(email, '@') != NULL;
}

bool validate_name(const char* name) {
    if (name == NULL || strlen(name) < 2) {
        return false;
    }
    
    return true;
}

long get_current_timestamp(void) {
    return (long)time(NULL);
}

void log_info(const char* message) {
    printf("[INFO] %s\n", message);
}

void log_error(const char* message) {
    fprintf(stderr, "[ERROR] %s\n", message);
}

// Main function
int main(void) {
    UserRepository* repo = repository_create();
    if (repo == NULL) {
        log_error("Failed to create repository");
        return 1;
    }
    
    // Create users
    User* user1 = user_create("Alice", "alice@example.com");
    User* user2 = user_create("Bob", "bob@example.com");
    
    // Save users
    repository_save(repo, user1);
    repository_save(repo, user2);
    
    // Print all users
    printf("\nAll users:\n");
    for (int i = 0; i < repo->count; i++) {
        user_print(repo->users[i]);
    }
    
    // Find active users
    int active_count;
    User** active_users = repository_find_active(repo, &active_count);
    printf("\nActive users: %d\n", active_count);
    free(active_users);
    
    // Cleanup
    repository_destroy(repo);
    
    return 0;
}
