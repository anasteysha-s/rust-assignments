// part_1.rs

/// Typestate pattern implementation for Post
/// States: New, Unmoderated, Published, Deleted

// Marker types for each state
pub struct New;
pub struct Unmoderated;
pub struct Published;
pub struct Deleted;

// The Post type parameterized by state
pub struct Post<State> {
    content: String,
    _state: std::marker::PhantomData<State>,
}

// Implementation for creating a new post
impl Post<New> {
    pub fn new(content: String) -> Self {
        Post {
            content,
            _state: std::marker::PhantomData,
        }
    }

    /// Transition from New to Unmoderated
    pub fn publish(self) -> Post<Unmoderated> {
        Post {
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

// Implementation for Unmoderated state
impl Post<Unmoderated> {
    /// Transition from Unmoderated to Published
    pub fn allow(self) -> Post<Published> {
        Post {
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }

    /// Transition from Unmoderated to Deleted
    pub fn deny(self) -> Post<Deleted> {
        Post {
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

// Implementation for Published state
impl Post<Published> {
    /// Transition from Published to Deleted
    pub fn delete(self) -> Post<Deleted> {
        Post {
            content: self.content,
            _state: std::marker::PhantomData,
        }
    }
}

// Common methods available in all states
impl<State> Post<State> {
    pub fn content(&self) -> &str {
        &self.content
    }
}

// Optionally, implement specific methods for Deleted state
impl Post<Deleted> {
    pub fn is_deleted(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_to_unmoderated_to_published() {
        let post = Post::new("Hello, World!".to_string());
        let post = post.publish();
        let post = post.allow();
        
        assert_eq!(post.content(), "Hello, World!");
    }

    #[test]
    fn test_new_to_unmoderated_to_deleted() {
        let post = Post::new("Spam content".to_string());
        let post = post.publish();
        let post = post.deny();
        
        assert!(post.is_deleted());
    }

    #[test]
    fn test_published_to_deleted() {
        let post = Post::new("Valid post".to_string());
        let post = post.publish();
        let post = post.allow();
        let post = post.delete();
        
        assert!(post.is_deleted());
    }

    #[test]
    fn test_content_accessible_in_all_states() {
        let post = Post::new("Test content".to_string());
        assert_eq!(post.content(), "Test content");
        
        let post = post.publish();
        assert_eq!(post.content(), "Test content");
        
        let post = post.allow();
        assert_eq!(post.content(), "Test content");
        
        let post = post.delete();
        assert_eq!(post.content(), "Test content");
    }

    // The following tests demonstrate compile-time errors
    // Uncomment them to see the compiler preventing invalid transitions
    
    // #[test]
    // fn test_cannot_delete_new_post() {
    //     let post = Post::new("Content".to_string());
    //     let post = post.delete(); // ERROR: no method named `delete` found
    // }

    // #[test]
    // fn test_cannot_deny_deleted_post() {
    //     let post = Post::new("Content".to_string());
    //     let post = post.publish();
    //     let post = post.deny();
    //     let post = post.deny(); // ERROR: no method named `deny` found
    // }

    // #[test]
    // fn test_cannot_allow_new_post() {
    //     let post = Post::new("Content".to_string());
    //     let post = post.allow(); // ERROR: no method named `allow` found
    // }

    // #[test]
    // fn test_cannot_publish_twice() {
    //     let post = Post::new("Content".to_string());
    //     let post = post.publish();
    //     let post = post.publish(); // ERROR: no method named `publish` found
    // }
}