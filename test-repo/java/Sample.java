// Java sample file for AMP parser testing.
// Tests: classes, interfaces, methods, constructors, fields, enums, imports

package com.amp.testRepo;

import java.time.LocalDateTime;
import java.util.*;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Collectors;

// Enum definitions
enum Status {
    ACTIVE,
    INACTIVE,
    PENDING,
    SUSPENDED
}

enum UserRole {
    ADMIN,
    USER,
    GUEST
}

// Interface definitions
interface Repository<T> {
    Optional<T> findById(Long id);
    T save(T entity);
    boolean delete(Long id);
    List<T> findAll();
}

interface Validator {
    boolean validate();
    List<String> getErrors();
}

interface Logger {
    void info(String message);
    void error(String message);
    void warning(String message);
}

// Abstract base class
abstract class BaseEntity {
    protected Long id;
    protected LocalDateTime createdAt;
    protected LocalDateTime updatedAt;

    public BaseEntity() {
        this.createdAt = LocalDateTime.now();
        this.updatedAt = LocalDateTime.now();
    }

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public LocalDateTime getUpdatedAt() {
        return updatedAt;
    }

    public abstract boolean isValid();
}

// Model class
class User extends BaseEntity implements Validator {
    private String name;
    private String email;
    private Status status;
    private UserRole role;
    private List<String> errors;

    public User() {
        super();
        this.status = Status.ACTIVE;
        this.role = UserRole.USER;
        this.errors = new ArrayList<>();
    }

    public User(String name, String email) {
        this();
        this.name = name;
        this.email = email;
    }

    // Getters and setters
    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
        this.updatedAt = LocalDateTime.now();
    }

    public String getEmail() {
        return email;
    }

    public void setEmail(String email) {
        this.email = email;
        this.updatedAt = LocalDateTime.now();
    }

    public Status getStatus() {
        return status;
    }

    public void setStatus(Status status) {
        this.status = status;
        this.updatedAt = LocalDateTime.now();
    }

    public UserRole getRole() {
        return role;
    }

    public void setRole(UserRole role) {
        this.role = role;
    }

    // Business methods
    public boolean isActive() {
        return this.status == Status.ACTIVE;
    }

    @Override
    public boolean isValid() {
        return validate();
    }

    @Override
    public boolean validate() {
        errors.clear();

        if (name == null || name.trim().isEmpty()) {
            errors.add("Name cannot be empty");
        }

        if (email == null || !email.contains("@")) {
            errors.add("Invalid email format");
        }

        return errors.isEmpty();
    }

    @Override
    public List<String> getErrors() {
        return new ArrayList<>(errors);
    }

    public void updateStatus(Status newStatus) {
        this.status = newStatus;
        this.updatedAt = LocalDateTime.now();
    }

    public void suspend(String reason) {
        this.status = Status.SUSPENDED;
        this.updatedAt = LocalDateTime.now();
        System.out.println("User " + name + " suspended: " + reason);
    }

    @Override
    public String toString() {
        return String.format("User{id=%d, name='%s', email='%s', status=%s}",
                id, name, email, status);
    }
}

// Repository implementation
class UserRepository implements Repository<User> {
    private final Map<Long, User> users;
    private Long nextId;
    private final Logger logger;

    public UserRepository(Logger logger) {
        this.users = new ConcurrentHashMap<>();
        this.nextId = 1L;
        this.logger = logger;
    }

    @Override
    public Optional<User> findById(Long id) {
        return Optional.ofNullable(users.get(id));
    }

    @Override
    public User save(User entity) {
        if (!entity.validate()) {
            throw new IllegalArgumentException("Invalid user data: " + entity.getErrors());
        }

        if (entity.getId() == null) {
            entity.setId(nextId++);
        }

        entity.updatedAt = LocalDateTime.now();
        users.put(entity.getId(), entity);

        logger.info("User saved: " + entity.getName() + " (ID: " + entity.getId() + ")");
        return entity;
    }

    @Override
    public boolean delete(Long id) {
        if (users.remove(id) != null) {
            logger.info("User deleted: ID " + id);
            return true;
        }
        return false;
    }

    @Override
    public List<User> findAll() {
        return new ArrayList<>(users.values());
    }

    public List<User> findActiveUsers() {
        return users.values().stream()
                .filter(User::isActive)
                .collect(Collectors.toList());
    }

    public List<User> findByRole(UserRole role) {
        return users.values().stream()
                .filter(u -> u.getRole() == role)
                .collect(Collectors.toList());
    }
}

// Logger implementation
class ConsoleLogger implements Logger {
    @Override
    public void info(String message) {
        System.out.println("[INFO] " + LocalDateTime.now() + " - " + message);
    }

    @Override
    public void error(String message) {
        System.err.println("[ERROR] " + LocalDateTime.now() + " - " + message);
    }

    @Override
    public void warning(String message) {
        System.out.println("[WARN] " + LocalDateTime.now() + " - " + message);
    }
}

// Service class
class UserService {
    private final Repository<User> repository;
    private final Logger logger;

    public UserService(Repository<User> repository, Logger logger) {
        this.repository = repository;
        this.logger = logger;
    }

    public User createUser(String name, String email) {
        User user = new User(name, email);
        return repository.save(user);
    }

    public Optional<User> getUserById(Long id) {
        return repository.findById(id);
    }

    public boolean deleteUser(Long id) {
        return repository.delete(id);
    }

    public List<User> getAllUsers() {
        return repository.findAll();
    }

    public void promoteToAdmin(Long userId) {
        repository.findById(userId).ifPresent(user -> {
            user.setRole(UserRole.ADMIN);
            repository.save(user);
            logger.info("User " + user.getName() + " promoted to ADMIN");
        });
    }
}

// Utility class
class ValidationHelper {
    public static boolean isValidEmail(String email) {
        return email != null && email.contains("@") && email.contains(".");
    }

    public static boolean isValidName(String name) {
        return name != null && !name.trim().isEmpty() && name.length() >= 2;
    }

    public static String sanitizeInput(String input) {
        return input != null ? input.trim() : "";
    }
}

// Main class
public class Sample {
    public static void main(String[] args) {
        Logger logger = new ConsoleLogger();
        UserRepository repository = new UserRepository(logger);
        UserService service = new UserService(repository, logger);

        // Create users
        User user1 = service.createUser("Alice", "alice@example.com");
        User user2 = service.createUser("Bob", "bob@example.com");

        logger.info("Created user: " + user1);
        logger.info("Created user: " + user2);

        // Promote user to admin
        service.promoteToAdmin(user1.getId());

        // List all users
        List<User> users = service.getAllUsers();
        users.forEach(user -> logger.info("User: " + user.getName() + " - Status: " + user.getStatus()));
    }
}
