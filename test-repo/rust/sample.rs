//! Rust sample file for AMP parser testing.
//! Tests: structs, enums, traits, impl blocks, functions, use statements

use std::collections::HashMap;
use std::fmt::{self, Display};
use std::sync::{Arc, Mutex};

// Type aliases
pub type UserId = u64;
pub type Result<T> = std::result::Result<T, Error>;

// Enum definitions
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Suspended { reason: String },
}

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    InvalidInput(String),
    DatabaseError(String),
}

// Struct definitions
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub status: Status,
    created_at: i64,
}

pub struct UserRepository {
    users: Arc<Mutex<HashMap<UserId, User>>>,
    next_id: Arc<Mutex<UserId>>,
}

// Trait definitions
pub trait Repository<T> {
    fn find_by_id(&self, id: UserId) -> Result<Option<T>>;
    fn save(&self, entity: T) -> Result<T>;
    fn delete(&self, id: UserId) -> Result<bool>;
}

pub trait Validator {
    fn validate(&self) -> Result<()>;
}

// Trait implementations
impl Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Active => write!(f, "Active"),
            Status::Inactive => write!(f, "Inactive"),
            Status::Pending => write!(f, "Pending"),
            Status::Suspended { reason } => write!(f, "Suspended: {}", reason),
        }
    }
}

impl User {
    pub fn new(id: UserId, name: String, email: String) -> Self {
        Self {
            id,
            name,
            email,
            status: Status::Active,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
    
    pub fn is_active(&self) -> bool {
        self.status == Status::Active
    }
    
    pub fn suspend(&mut self, reason: String) {
        self.status = Status::Suspended { reason };
    }
}

impl Validator for User {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(Error::InvalidInput("Name cannot be empty".to_string()));
        }
        if !self.email.contains('@') {
            return Err(Error::InvalidInput("Invalid email format".to_string()));
        }
        Ok(())
    }
}

impl UserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
    
    pub fn create_user(&self, name: String, email: String) -> Result<User> {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        
        let user = User::new(id, name, email);
        user.validate()?;
        
        let mut users = self.users.lock().unwrap();
        users.insert(id, user.clone());
        
        Ok(user)
    }
    
    pub fn get_active_users(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.values()
            .filter(|u| u.is_active())
            .cloned()
            .collect()
    }
}

impl Repository<User> for UserRepository {
    fn find_by_id(&self, id: UserId) -> Result<Option<User>> {
        let users = self.users.lock().unwrap();
        Ok(users.get(&id).cloned())
    }
    
    fn save(&self, entity: User) -> Result<User> {
        entity.validate()?;
        let mut users = self.users.lock().unwrap();
        users.insert(entity.id, entity.clone());
        Ok(entity)
    }
    
    fn delete(&self, id: UserId) -> Result<bool> {
        let mut users = self.users.lock().unwrap();
        Ok(users.remove(&id).is_some())
    }
}

// Free functions
pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

pub fn calculate_hash(input: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}

pub async fn fetch_user_data(id: UserId) -> Result<User> {
    // Simulated async operation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(User::new(id, "Test User".to_string(), "test@example.com".to_string()))
}

// Generic function
pub fn find_first<T, F>(items: &[T], predicate: F) -> Option<&T>
where
    F: Fn(&T) -> bool,
{
    items.iter().find(|item| predicate(item))
}

// Macro definition
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("[INFO] {}", format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        let user = User::new(1, "Alice".to_string(), "alice@example.com".to_string());
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Alice");
        assert!(user.is_active());
    }
    
    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com"));
        assert!(!validate_email("invalid-email"));
    }
}
