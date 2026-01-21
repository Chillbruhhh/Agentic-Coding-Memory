/**
 * C++ sample file for AMP parser testing.
 * Tests: classes, structs, namespaces, templates, #include, using
 */

#include <iostream>
#include <string>
#include <vector>
#include <memory>
#include <map>
#include <algorithm>
#include <chrono>
#include <optional>

// Namespace definition
namespace amp {
namespace testRepo {

// Enum class definitions
enum class Status {
    Active,
    Inactive,
    Pending,
    Suspended
};

enum class UserRole {
    Admin,
    User,
    Guest
};

// Forward declarations
class User;
class UserRepository;

// Interface (abstract class)
template<typename T>
class IRepository {
public:
    virtual ~IRepository() = default;
    virtual std::optional<T> findById(long id) const = 0;
    virtual T save(const T& entity) = 0;
    virtual bool deleteById(long id) = 0;
    virtual std::vector<T> findAll() const = 0;
};

// Validator interface
class IValidator {
public:
    virtual ~IValidator() = default;
    virtual bool validate() const = 0;
    virtual std::vector<std::string> getErrors() const = 0;
};

// Base entity class
class BaseEntity {
protected:
    long id_;
    std::chrono::system_clock::time_point createdAt_;
    std::chrono::system_clock::time_point updatedAt_;

public:
    BaseEntity() 
        : id_(0), 
          createdAt_(std::chrono::system_clock::now()),
          updatedAt_(std::chrono::system_clock::now()) {}

    virtual ~BaseEntity() = default;

    long getId() const { return id_; }
    void setId(long id) { id_ = id; }
    
    auto getCreatedAt() const { return createdAt_; }
    auto getUpdatedAt() const { return updatedAt_; }

    virtual bool isValid() const = 0;
};

// User class
class User : public BaseEntity, public IValidator {
private:
    std::string name_;
    std::string email_;
    Status status_;
    UserRole role_;
    mutable std::vector<std::string> errors_;

public:
    User() : status_(Status::Active), role_(UserRole::User) {}

    User(const std::string& name, const std::string& email)
        : name_(name), email_(email), status_(Status::Active), role_(UserRole::User) {}

    // Copy constructor
    User(const User& other) = default;

    // Move constructor
    User(User&& other) noexcept = default;

    // Copy assignment
    User& operator=(const User& other) = default;

    // Move assignment
    User& operator=(User&& other) noexcept = default;

    // Getters
    const std::string& getName() const { return name_; }
    const std::string& getEmail() const { return email_; }
    Status getStatus() const { return status_; }
    UserRole getRole() const { return role_; }

    // Setters
    void setName(const std::string& name) {
        name_ = name;
        updatedAt_ = std::chrono::system_clock::now();
    }

    void setEmail(const std::string& email) {
        email_ = email;
        updatedAt_ = std::chrono::system_clock::now();
    }

    void setStatus(Status status) {
        status_ = status;
        updatedAt_ = std::chrono::system_clock::now();
    }

    void setRole(UserRole role) {
        role_ = role;
    }

    // Business methods
    bool isActive() const {
        return status_ == Status::Active;
    }

    bool isValid() const override {
        return validate();
    }

    bool validate() const override {
        errors_.clear();

        if (name_.empty()) {
            errors_.push_back("Name cannot be empty");
        }

        if (email_.empty() || email_.find('@') == std::string::npos) {
            errors_.push_back("Invalid email format");
        }

        return errors_.empty();
    }

    std::vector<std::string> getErrors() const override {
        return errors_;
    }

    void updateStatus(Status newStatus) {
        status_ = newStatus;
        updatedAt_ = std::chrono::system_clock::now();
    }

    void suspend(const std::string& reason) {
        status_ = Status::Suspended;
        updatedAt_ = std::chrono::system_clock::now();
        std::cout << "User " << name_ << " suspended: " << reason << std::endl;
    }

    // Friend function for output
    friend std::ostream& operator<<(std::ostream& os, const User& user);
};

// Output operator
std::ostream& operator<<(std::ostream& os, const User& user) {
    os << "User{id=" << user.id_ 
       << ", name='" << user.name_ 
       << "', email='" << user.email_ << "'}";
    return os;
}

// Repository implementation
class UserRepository : public IRepository<User> {
private:
    std::map<long, User> users_;
    long nextId_;

public:
    UserRepository() : nextId_(1) {}

    std::optional<User> findById(long id) const override {
        auto it = users_.find(id);
        if (it != users_.end()) {
            return it->second;
        }
        return std::nullopt;
    }

    User save(const User& entity) override {
        User user = entity;

        if (!user.validate()) {
            throw std::invalid_argument("Invalid user data");
        }

        if (user.getId() == 0) {
            user.setId(nextId_++);
        }

        users_[user.getId()] = user;
        std::cout << "[INFO] User saved: " << user.getName() 
                  << " (ID: " << user.getId() << ")" << std::endl;

        return user;
    }

    bool deleteById(long id) override {
        auto it = users_.find(id);
        if (it != users_.end()) {
            users_.erase(it);
            std::cout << "[INFO] User deleted: ID " << id << std::endl;
            return true;
        }
        return false;
    }

    std::vector<User> findAll() const override {
        std::vector<User> result;
        result.reserve(users_.size());
        
        for (const auto& [id, user] : users_) {
            result.push_back(user);
        }
        
        return result;
    }

    std::vector<User> findActiveUsers() const {
        std::vector<User> result;
        
        std::copy_if(users_.begin(), users_.end(), 
                     std::back_inserter(result),
                     [](const auto& pair) { return pair.second.isActive(); });
        
        return result;
    }
};

// Service class
class UserService {
private:
    std::shared_ptr<IRepository<User>> repository_;

public:
    explicit UserService(std::shared_ptr<IRepository<User>> repository)
        : repository_(std::move(repository)) {}

    User createUser(const std::string& name, const std::string& email) {
        User user(name, email);
        return repository_->save(user);
    }

    std::optional<User> getUserById(long id) const {
        return repository_->findById(id);
    }

    bool deleteUser(long id) {
        return repository_->deleteById(id);
    }

    std::vector<User> getAllUsers() const {
        return repository_->findAll();
    }

    void promoteToAdmin(long userId) {
        auto userOpt = repository_->findById(userId);
        if (userOpt.has_value()) {
            User user = userOpt.value();
            user.setRole(UserRole::Admin);
            repository_->save(user);
            std::cout << "[INFO] User " << user.getName() 
                      << " promoted to ADMIN" << std::endl;
        }
    }
};

// Utility namespace
namespace utils {

bool isValidEmail(const std::string& email) {
    return !email.empty() && email.find('@') != std::string::npos;
}

bool isValidName(const std::string& name) {
    return !name.empty() && name.length() >= 2;
}

std::string sanitizeInput(const std::string& input) {
    std::string result = input;
    result.erase(0, result.find_first_not_of(" \t\n\r"));
    result.erase(result.find_last_not_of(" \t\n\r") + 1);
    return result;
}

// Template function
template<typename T, typename Predicate>
std::vector<T> filter(const std::vector<T>& items, Predicate pred) {
    std::vector<T> result;
    std::copy_if(items.begin(), items.end(), std::back_inserter(result), pred);
    return result;
}

} // namespace utils

} // namespace testRepo
} // namespace amp

// Main function
int main() {
    using namespace amp::testRepo;

    auto repository = std::make_shared<UserRepository>();
    UserService service(repository);

    // Create users
    auto user1 = service.createUser("Alice", "alice@example.com");
    auto user2 = service.createUser("Bob", "bob@example.com");

    std::cout << "Created user: " << user1 << std::endl;
    std::cout << "Created user: " << user2 << std::endl;

    // Promote user to admin
    service.promoteToAdmin(user1.getId());

    // List all users
    auto users = service.getAllUsers();
    std::cout << "\nAll users:" << std::endl;
    for (const auto& user : users) {
        std::cout << "  " << user << std::endl;
    }

    return 0;
}
