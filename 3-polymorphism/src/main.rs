use std::borrow::Cow;
use std::collections::HashMap;

// Given trait and struct - DO NOT MODIFY
trait Storage<K, V> {
    fn set(&mut self, key: K, val: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<V>;
}

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u64,
    email: Cow<'static, str>,
    activated: bool,
}


// STATIC DISPATCH IMPLEMENTATION


/// UserRepository with static dispatch (compile-time polymorphism)
/// Uses generic type parameter S that implements Storage<u64, User>
struct UserRepositoryStatic<S>
where
    S: Storage<u64, User>,
{
    storage: S,
}

impl<S> UserRepositoryStatic<S>
where
    S: Storage<u64, User>,
{
    /// Create a new repository with injected storage
    fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Add a new user to the repository
    fn add(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    /// Get a user by ID
    fn get(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    /// Update an existing user
    fn update(&mut self, user: User) {
        // In a real system, you might want to check if user exists first
        self.storage.set(user.id, user);
    }

    /// Remove a user by ID
    fn remove(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }
}


// DYNAMIC DISPATCH IMPLEMENTATION (using Trait Objects)

/// UserRepository with dynamic dispatch (runtime polymorphism)
/// Uses trait object Box<dyn Storage<u64, User>>
struct UserRepositoryDynamic {
    storage: Box<dyn Storage<u64, User>>,
}

impl UserRepositoryDynamic {
    /// Create a new repository with injected storage
    fn new(storage: Box<dyn Storage<u64, User>>) -> Self {
        Self { storage }
    }

    /// Add a new user to the repository
    fn add(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    /// Get a user by ID
    fn get(&self, id: u64) -> Option<&User> {
        self.storage.get(&id)
    }

    /// Update an existing user
    fn update(&mut self, user: User) {
        self.storage.set(user.id, user);
    }

    /// Remove a user by ID
    fn remove(&mut self, id: u64) -> Option<User> {
        self.storage.remove(&id)
    }
}


// CONCRETE STORAGE IMPLEMENTATIONS (for testing)


/// HashMap-based storage implementation
struct HashMapStorage {
    data: HashMap<u64, User>,
}

impl HashMapStorage {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Storage<u64, User> for HashMapStorage {
    fn set(&mut self, key: u64, val: User) {
        self.data.insert(key, val);
    }

    fn get(&self, key: &u64) -> Option<&User> {
        self.data.get(key)
    }

    fn remove(&mut self, key: &u64) -> Option<User> {
        self.data.remove(key)
    }
}

/// Vector-based storage implementation (less efficient, for demonstration)
struct VecStorage {
    data: Vec<(u64, User)>,
}

impl VecStorage {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl Storage<u64, User> for VecStorage {
    fn set(&mut self, key: u64, val: User) {
        // Update if exists, otherwise append
        if let Some(pos) = self.data.iter().position(|(k, _)| *k == key) {
            self.data[pos] = (key, val);
        } else {
            self.data.push((key, val));
        }
    }

    fn get(&self, key: &u64) -> Option<&User> {
        self.data
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    fn remove(&mut self, key: &u64) -> Option<User> {
        if let Some(pos) = self.data.iter().position(|(k, _)| k == key) {
            Some(self.data.remove(pos).1)
        } else {
            None
        }
    }
}

// HELPER FUNCTIONS FOR CREATING TEST USERS


fn create_user(id: u64, email: &'static str, activated: bool) -> User {
    User {
        id,
        email: Cow::Borrowed(email),
        activated,
    }
}

// MAIN FUNCTION (demonstration)

fn main() {
    println!("=== Static Dispatch Example ===");
    let mut repo_static = UserRepositoryStatic::new(HashMapStorage::new());
    
    let user1 = create_user(1, "alice@example.com", true);
    repo_static.add(user1.clone());
    
    if let Some(retrieved) = repo_static.get(1) {
        println!("Retrieved user: {:?}", retrieved);
    }
    
    println!("\n=== Dynamic Dispatch Example ===");
    let mut repo_dynamic = UserRepositoryDynamic::new(Box::new(HashMapStorage::new()));
    
    let user2 = create_user(2, "bob@example.com", false);
    repo_dynamic.add(user2.clone());
    
    if let Some(retrieved) = repo_dynamic.get(2) {
        println!("Retrieved user: {:?}", retrieved);
    }
    
    println!("\n=== Tests can be run with: cargo test ===");
}


// TESTS


#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // STATIC DISPATCH TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_static_add_and_get() {
        let mut repo = UserRepositoryStatic::new(HashMapStorage::new());
        let user = create_user(1, "test@example.com", true);
        
        repo.add(user.clone());
        
        let retrieved = repo.get(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &user);
    }

    #[test]
    fn test_static_get_nonexistent() {
        let repo = UserRepositoryStatic::new(HashMapStorage::new());
        assert!(repo.get(999).is_none());
    }

    #[test]
    fn test_static_update() {
        let mut repo = UserRepositoryStatic::new(HashMapStorage::new());
        let user = create_user(1, "old@example.com", false);
        repo.add(user);
        
        let updated_user = create_user(1, "new@example.com", true);
        repo.update(updated_user.clone());
        
        let retrieved = repo.get(1).unwrap();
        assert_eq!(retrieved.email, "new@example.com");
        assert!(retrieved.activated);
    }

    #[test]
    fn test_static_remove() {
        let mut repo = UserRepositoryStatic::new(HashMapStorage::new());
        let user = create_user(1, "test@example.com", true);
        repo.add(user.clone());
        
        let removed = repo.remove(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), user);
        
        // Should be gone now
        assert!(repo.get(1).is_none());
    }

    #[test]
    fn test_static_remove_nonexistent() {
        let mut repo = UserRepositoryStatic::new(HashMapStorage::new());
        assert!(repo.remove(999).is_none());
    }

    #[test]
    fn test_static_multiple_users() {
        let mut repo = UserRepositoryStatic::new(HashMapStorage::new());
        
        let user1 = create_user(1, "alice@example.com", true);
        let user2 = create_user(2, "bob@example.com", false);
        let user3 = create_user(3, "charlie@example.com", true);
        
        repo.add(user1.clone());
        repo.add(user2.clone());
        repo.add(user3.clone());
        
        assert_eq!(repo.get(1).unwrap(), &user1);
        assert_eq!(repo.get(2).unwrap(), &user2);
        assert_eq!(repo.get(3).unwrap(), &user3);
    }

    #[test]
    fn test_static_with_vec_storage() {
        // Demonstrates injection of different storage implementation
        let mut repo = UserRepositoryStatic::new(VecStorage::new());
        let user = create_user(1, "test@example.com", true);
        
        repo.add(user.clone());
        assert_eq!(repo.get(1).unwrap(), &user);
        
        repo.remove(1);
        assert!(repo.get(1).is_none());
    }

    // ------------------------------------------------------------------------
    // DYNAMIC DISPATCH TESTS
    // ------------------------------------------------------------------------

    #[test]
    fn test_dynamic_add_and_get() {
        let mut repo = UserRepositoryDynamic::new(Box::new(HashMapStorage::new()));
        let user = create_user(1, "test@example.com", true);
        
        repo.add(user.clone());
        
        let retrieved = repo.get(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &user);
    }

    #[test]
    fn test_dynamic_get_nonexistent() {
        let repo = UserRepositoryDynamic::new(Box::new(HashMapStorage::new()));
        assert!(repo.get(999).is_none());
    }

    #[test]
    fn test_dynamic_update() {
        let mut repo = UserRepositoryDynamic::new(Box::new(HashMapStorage::new()));
        let user = create_user(1, "old@example.com", false);
        repo.add(user);
        
        let updated_user = create_user(1, "new@example.com", true);
        repo.update(updated_user.clone());
        
        let retrieved = repo.get(1).unwrap();
        assert_eq!(retrieved.email, "new@example.com");
        assert!(retrieved.activated);
    }

    #[test]
    fn test_dynamic_remove() {
        let mut repo = UserRepositoryDynamic::new(Box::new(HashMapStorage::new()));
        let user = create_user(1, "test@example.com", true);
        repo.add(user.clone());
        
        let removed = repo.remove(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), user);
        
        assert!(repo.get(1).is_none());
    }

    #[test]
    fn test_dynamic_remove_nonexistent() {
        let mut repo = UserRepositoryDynamic::new(Box::new(HashMapStorage::new()));
        assert!(repo.remove(999).is_none());
    }

    #[test]
    fn test_dynamic_multiple_users() {
        let mut repo = UserRepositoryDynamic::new(Box::new(HashMapStorage::new()));
        
        let user1 = create_user(1, "alice@example.com", true);
        let user2 = create_user(2, "bob@example.com", false);
        let user3 = create_user(3, "charlie@example.com", true);
        
        repo.add(user1.clone());
        repo.add(user2.clone());
        repo.add(user3.clone());
        
        assert_eq!(repo.get(1).unwrap(), &user1);
        assert_eq!(repo.get(2).unwrap(), &user2);
        assert_eq!(repo.get(3).unwrap(), &user3);
    }

    #[test]
    fn test_dynamic_with_vec_storage() {
        // Demonstrates injection of different storage implementation
        let mut repo = UserRepositoryDynamic::new(Box::new(VecStorage::new()));
        let user = create_user(1, "test@example.com", true);
        
        repo.add(user.clone());
        assert_eq!(repo.get(1).unwrap(), &user);
        
        repo.remove(1);
        assert!(repo.get(1).is_none());
    }

    // COMPARISON TESTS (demonstrating both work identically)

    #[test]
    fn test_static_and_dynamic_produce_same_results() {
        // Static dispatch version
        let mut repo_static = UserRepositoryStatic::new(HashMapStorage::new());
        let user1 = create_user(1, "test@example.com", true);
        repo_static.add(user1.clone());
        
        // Dynamic dispatch version
        let mut repo_dynamic = UserRepositoryDynamic::new(Box::new(HashMapStorage::new()));
        let user2 = create_user(1, "test@example.com", true);
        repo_dynamic.add(user2.clone());
        
        // Both should return the same user
        assert_eq!(repo_static.get(1), repo_dynamic.get(1));
    }
}