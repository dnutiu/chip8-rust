use alloc::vec::Vec;
use core::fmt::Debug;

/// Implements a simple stack based on a vector.
#[derive(Debug)]
pub struct Stack<T> {
    storage: Vec<T>,
}

impl<T: Debug> Stack<T> {
    pub fn new() -> Self {
        Stack {
            storage: Vec::new(),
        }
    }

    /// Pushes an element on the top of the stack
    pub fn push(&mut self, element: T) {
        self.storage.push(element)
    }

    /// Returns the top element from the stack.
    pub fn pop(&mut self) -> Option<T> {
        self.storage.pop()
    }

    /// Returns the top element from the stack without removing it.
    pub fn peek(&self) -> Option<&T> {
        self.storage.last()
    }

    /// Size returns the size of the stack.
    pub fn size(&self) -> usize {
        self.storage.len()
    }

    /// Is Empty returns true if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.storage.len() == 0
    }
}

impl<T: Debug> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_operations() {
        // Given
        let mut stack = Stack::<u16>::new();

        // Then
        stack.push(15);

        assert_eq!(1, stack.size());
        assert_eq!(15u16, *stack.peek().unwrap());

        let element = stack.pop();
        assert_eq!(15u16, element.unwrap());
        assert_eq!(0, stack.size());
        assert_eq!(true, stack.is_empty());

        let element = stack.pop();
        assert_eq!(true, element.is_none())
    }
}
