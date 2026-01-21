// C# sample file for AMP parser testing.
// Tests: classes, interfaces, methods, properties, enums, namespaces, using statements

using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;

namespace AMP.TestRepo
{
    // Enum definitions
    public enum Status
    {
        Active,
        Inactive,
        Pending,
        Suspended
    }

    public enum UserRole
    {
        Admin,
        User,
        Guest
    }

    // Interface definitions
    public interface IRepository<T> where T : class
    {
        Task<T> FindByIdAsync(int id);
        Task<T> SaveAsync(T entity);
        Task<bool> DeleteAsync(int id);
        Task<IEnumerable<T>> ListAsync();
    }

    public interface IValidator
    {
        bool Validate();
        IEnumerable<string> GetErrors();
    }

    // Abstract base class
    public abstract class BaseEntity
    {
        public int Id { get; set; }
        public DateTime CreatedAt { get; set; }
        public DateTime UpdatedAt { get; set; }

        protected BaseEntity()
        {
            CreatedAt = DateTime.UtcNow;
            UpdatedAt = DateTime.UtcNow;
        }

        public abstract bool IsValid();
    }

    // Model class
    public class User : BaseEntity, IValidator
    {
        public string Name { get; set; }
        public string Email { get; set; }
        public Status Status { get; set; }
        public UserRole Role { get; set; }
        private List<string> _errors;

        public User()
        {
            Status = Status.Active;
            Role = UserRole.User;
            _errors = new List<string>();
        }

        public User(string name, string email) : this()
        {
            Name = name;
            Email = email;
        }

        public bool IsActive => Status == Status.Active;

        public override bool IsValid()
        {
            return Validate();
        }

        public bool Validate()
        {
            _errors.Clear();

            if (string.IsNullOrWhiteSpace(Name))
                _errors.Add("Name cannot be empty");

            if (string.IsNullOrWhiteSpace(Email) || !Email.Contains("@"))
                _errors.Add("Invalid email format");

            return _errors.Count == 0;
        }

        public IEnumerable<string> GetErrors()
        {
            return _errors.AsReadOnly();
        }

        public void UpdateStatus(Status newStatus)
        {
            Status = newStatus;
            UpdatedAt = DateTime.UtcNow;
        }

        public void Suspend(string reason)
        {
            Status = Status.Suspended;
            UpdatedAt = DateTime.UtcNow;
            Console.WriteLine($"User {Name} suspended: {reason}");
        }
    }

    // Repository implementation
    public class UserRepository : IRepository<User>
    {
        private readonly Dictionary<int, User> _users;
        private int _nextId;
        private readonly ILogger _logger;

        public UserRepository(ILogger logger)
        {
            _users = new Dictionary<int, User>();
            _nextId = 1;
            _logger = logger;
        }

        public async Task<User> FindByIdAsync(int id)
        {
            await Task.Delay(10); // Simulate async operation
            
            if (_users.TryGetValue(id, out var user))
            {
                return user;
            }

            return null;
        }

        public async Task<User> SaveAsync(User entity)
        {
            if (!entity.Validate())
            {
                throw new InvalidOperationException("Invalid user data");
            }

            await Task.Delay(10); // Simulate async operation

            if (entity.Id == 0)
            {
                entity.Id = _nextId++;
                entity.CreatedAt = DateTime.UtcNow;
            }

            entity.UpdatedAt = DateTime.UtcNow;
            _users[entity.Id] = entity;

            _logger.Info($"User saved: {entity.Name} (ID: {entity.Id})");
            return entity;
        }

        public async Task<bool> DeleteAsync(int id)
        {
            await Task.Delay(10); // Simulate async operation
            
            if (_users.Remove(id))
            {
                _logger.Info($"User deleted: ID {id}");
                return true;
            }

            return false;
        }

        public async Task<IEnumerable<User>> ListAsync()
        {
            await Task.Delay(10); // Simulate async operation
            return _users.Values.ToList();
        }

        public async Task<IEnumerable<User>> GetActiveUsersAsync()
        {
            var allUsers = await ListAsync();
            return allUsers.Where(u => u.IsActive);
        }
    }

    // Logger interface and implementation
    public interface ILogger
    {
        void Info(string message);
        void Error(string message);
        void Warning(string message);
    }

    public class ConsoleLogger : ILogger
    {
        public void Info(string message)
        {
            Console.WriteLine($"[INFO] {DateTime.Now:yyyy-MM-dd HH:mm:ss} - {message}");
        }

        public void Error(string message)
        {
            Console.WriteLine($"[ERROR] {DateTime.Now:yyyy-MM-dd HH:mm:ss} - {message}");
        }

        public void Warning(string message)
        {
            Console.WriteLine($"[WARN] {DateTime.Now:yyyy-MM-dd HH:mm:ss} - {message}");
        }
    }

    // Service class
    public class UserService
    {
        private readonly IRepository<User> _repository;
        private readonly ILogger _logger;

        public UserService(IRepository<User> repository, ILogger logger)
        {
            _repository = repository;
            _logger = logger;
        }

        public async Task<User> CreateUserAsync(string name, string email)
        {
            var user = new User(name, email);
            return await _repository.SaveAsync(user);
        }

        public async Task<User> GetUserByIdAsync(int id)
        {
            return await _repository.FindByIdAsync(id);
        }

        public async Task<bool> DeleteUserAsync(int id)
        {
            return await _repository.DeleteAsync(id);
        }

        public async Task<IEnumerable<User>> GetAllUsersAsync()
        {
            return await _repository.ListAsync();
        }
    }

    // Utility class with static methods
    public static class ValidationHelper
    {
        public static bool IsValidEmail(string email)
        {
            return !string.IsNullOrWhiteSpace(email) && email.Contains("@");
        }

        public static bool IsValidName(string name)
        {
            return !string.IsNullOrWhiteSpace(name) && name.Length >= 2;
        }

        public static string SanitizeInput(string input)
        {
            return input?.Trim() ?? string.Empty;
        }
    }

    // Extension methods
    public static class UserExtensions
    {
        public static string GetDisplayName(this User user)
        {
            return $"{user.Name} ({user.Email})";
        }

        public static bool HasRole(this User user, UserRole role)
        {
            return user.Role == role;
        }
    }

    // Main program
    public class Program
    {
        public static async Task Main(string[] args)
        {
            var logger = new ConsoleLogger();
            var repository = new UserRepository(logger);
            var service = new UserService(repository, logger);

            // Create users
            var user1 = await service.CreateUserAsync("Alice", "alice@example.com");
            var user2 = await service.CreateUserAsync("Bob", "bob@example.com");

            logger.Info($"Created user: {user1.GetDisplayName()}");
            logger.Info($"Created user: {user2.GetDisplayName()}");

            // List all users
            var users = await service.GetAllUsersAsync();
            foreach (var user in users)
            {
                logger.Info($"User: {user.Name} - Status: {user.Status}");
            }
        }
    }
}
