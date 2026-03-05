/// A generic stack implementation with circular dependency detection.
#[derive(Debug, Clone)]
pub struct Stack<T: PartialEq + Clone> {
    items: Vec<T>,
}

impl<T: PartialEq + Clone> Stack<T> {
    /// Creates a new empty stack.
    #[must_use]
    pub const fn new() -> Self {
        Self { items: vec![] }
    }

    /// Pushes an item onto the top of the stack.
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes and returns the top item from the stack.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Finds a circular dependency starting from the given item.
    ///
    /// This method searches the stack for the given item and, if found, returns
    /// a vector containing the circular dependency path. The path includes all
    /// items from the first occurrence of the item to the end of the stack,
    /// plus the item itself at the end.
    #[must_use]
    pub fn find_circular_dependency(&self, item: &T) -> Option<Vec<T>> {
        let item_index = self.items.iter().position(|i| i == item)?;

        let mut circular_dependency = self.items[item_index..].to_vec();
        circular_dependency.push(item.clone());

        Some(circular_dependency)
    }
}
