use std::ptr;

/// A node in the doubly linked list
pub struct Node<T> {
    pub data: T,
    pub prev: *mut Node<T>,
    pub next: *mut Node<T>,
}

impl<T> Node<T> {
    fn new(data: T) -> Box<Self> {
        Box::new(Node {
            data,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        })
    }
}

/// A doubly linked list implementation using unsafe raw pointers
pub struct List<T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
    length: usize,
}

impl<T> List<T> {
    /// Creates a new empty doubly linked list
    pub fn new() -> Self {
        List {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            length: 0,
        }
    }

    /// Returns the length of the list
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns true if the list is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Adds an element to the front of the list
    /// Returns the pointer address of the newly inserted node
    pub fn push_front(&mut self, data: T) -> *mut Node<T> {
        let new_node = Box::into_raw(Node::new(data));

        unsafe {
            if self.head.is_null() {
                // Empty list
                self.tail = new_node;
            } else {
                (*self.head).prev = new_node;
                (*new_node).next = self.head;
            }
            self.head = new_node;
        }

        self.length += 1;
        new_node
    }

    /// Adds an element to the back of the list
    /// Returns the pointer address of the newly inserted node
    pub fn push_back(&mut self, data: T) -> *mut Node<T> {
        let new_node = Box::into_raw(Node::new(data));

        unsafe {
            if self.tail.is_null() {
                // Empty list
                self.head = new_node;
            } else {
                (*self.tail).next = new_node;
                (*new_node).prev = self.tail;
            }
            self.tail = new_node;
        }

        self.length += 1;
        new_node
    }

    /// Removes and returns the pointer address from the front of the list
    /// Returns None if the list is empty
    /// Note: The caller is responsible for deallocating the node if needed
    pub fn pop_front(&mut self) -> Option<*mut Node<T>> {
        if self.head.is_null() {
            return None;
        }

        unsafe {
            let old_head = self.head;
            self.head = (*old_head).next;

            if self.head.is_null() {
                // This was the only node
                self.tail = ptr::null_mut();
            } else {
                (*self.head).prev = ptr::null_mut();
            }

            self.length -= 1;
            Some(old_head)
        }
    }

    /// Removes and returns the pointer address from the back of the list
    /// Returns None if the list is empty
    /// Note: The caller is responsible for deallocating the node if needed
    pub fn pop_back(&mut self) -> Option<*mut Node<T>> {
        if self.tail.is_null() {
            return None;
        }

        unsafe {
            let old_tail = self.tail;
            self.tail = (*old_tail).prev;

            if self.tail.is_null() {
                // This was the only node
                self.head = ptr::null_mut();
            } else {
                (*self.tail).next = ptr::null_mut();
            }

            self.length -= 1;
            Some(old_tail)
        }
    }

    /// Returns a reference to the front element without removing it
    pub fn front(&self) -> Option<&T> {
        if self.head.is_null() {
            None
        } else {
            unsafe { Some(&(*self.head).data) }
        }
    }

    /// Returns a mutable reference to the front element without removing it
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.head.is_null() {
            None
        } else {
            unsafe { Some(&mut (*self.head).data) }
        }
    }

    /// Returns a reference to the back element without removing it
    pub fn back(&self) -> Option<&T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&(*self.tail).data) }
        }
    }

    /// Returns a mutable reference to the back element without removing it
    pub fn back_mut(&mut self) -> Option<&mut T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&mut (*self.tail).data) }
        }
    }

    /// Removes all elements from the list
    pub fn clear(&mut self) {
        while let Some(node_ptr) = self.pop_front() {
            unsafe {
                let _ = Box::from_raw(node_ptr);
            }
        }
    }

    /// Removes the node at the given pointer from the list
    /// Returns the data from the removed node, or None if the pointer is null
    /// Note: The caller must ensure the pointer is valid and points to a node in this list
    pub fn remove(&mut self, node_ptr: *mut Node<T>) -> Option<T> {
        if node_ptr.is_null() {
            return None;
        }

        unsafe {
            let prev = (*node_ptr).prev;
            let next = (*node_ptr).next;

            // Update adjacent nodes' pointers
            if prev.is_null() {
                // Removing head
                self.head = next;
            } else {
                (*prev).next = next;
            }

            if next.is_null() {
                // Removing tail
                self.tail = prev;
            } else {
                (*next).prev = prev;
            }

            self.length -= 1;
            let boxed_node = Box::from_raw(node_ptr);
            Some(boxed_node.data)
        }
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(node_ptr) = self.pop_front() {
            unsafe {
                let _ = Box::from_raw(node_ptr);
            }
        }
    }
}

/// An iterator over the doubly linked list
pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front().map(|node_ptr| unsafe {
            let boxed_node = Box::from_raw(node_ptr);
            boxed_node.data
        })
    }
}

impl<T> List<T> {
    /// Consumes the list and returns an iterator over its elements
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let list: List<i32> = List::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_push_front() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(list.len(), 3);
        assert_eq!(*list.front().unwrap(), 3);
    }

    #[test]
    fn test_push_back() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(list.len(), 3);
        assert_eq!(*list.back().unwrap(), 3);
    }

    #[test]
    fn test_pop_front() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let node1 = list.pop_front().unwrap();
        assert_eq!(unsafe { (*node1).data }, 1);
        unsafe {
            let _ = Box::from_raw(node1);
        }

        let node2 = list.pop_front().unwrap();
        assert_eq!(unsafe { (*node2).data }, 2);
        unsafe {
            let _ = Box::from_raw(node2);
        }

        let node3 = list.pop_front().unwrap();
        assert_eq!(unsafe { (*node3).data }, 3);
        unsafe {
            let _ = Box::from_raw(node3);
        }

        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn test_pop_back() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let node1 = list.pop_back().unwrap();
        assert_eq!(unsafe { (*node1).data }, 3);
        unsafe {
            let _ = Box::from_raw(node1);
        }

        let node2 = list.pop_back().unwrap();
        assert_eq!(unsafe { (*node2).data }, 2);
        unsafe {
            let _ = Box::from_raw(node2);
        }

        let node3 = list.pop_back().unwrap();
        assert_eq!(unsafe { (*node3).data }, 1);
        unsafe {
            let _ = Box::from_raw(node3);
        }

        assert_eq!(list.pop_back(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn test_front_and_back() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(*list.front().unwrap(), 1);
        assert_eq!(*list.back().unwrap(), 3);
        assert_eq!(list.len(), 3); // Should not consume
    }

    #[test]
    fn test_front_mut_and_back_mut() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        *list.front_mut().unwrap() = 10;
        *list.back_mut().unwrap() = 30;

        assert_eq!(*list.front().unwrap(), 10);
        assert_eq!(*list.back().unwrap(), 30);
    }

    #[test]
    fn test_clear() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_into_iter() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let vec: Vec<i32> = list.into_iter().collect();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_mixed_operations() {
        let mut list = List::new();
        list.push_front(1);
        list.push_back(2);
        list.push_front(0);
        list.push_back(3);

        assert_eq!(list.len(), 4);
        assert_eq!(*list.front().unwrap(), 0);
        assert_eq!(*list.back().unwrap(), 3);

        let node1 = list.pop_front().unwrap();
        assert_eq!(unsafe { (*node1).data }, 0);
        unsafe {
            let _ = Box::from_raw(node1);
        }

        let node2 = list.pop_back().unwrap();
        assert_eq!(unsafe { (*node2).data }, 3);
        unsafe {
            let _ = Box::from_raw(node2);
        }

        let node3 = list.pop_front().unwrap();
        assert_eq!(unsafe { (*node3).data }, 1);
        unsafe {
            let _ = Box::from_raw(node3);
        }

        let node4 = list.pop_back().unwrap();
        assert_eq!(unsafe { (*node4).data }, 2);
        unsafe {
            let _ = Box::from_raw(node4);
        }
    }

    #[test]
    fn test_drop() {
        let mut list = List::new();
        for i in 0..100 {
            list.push_back(i);
        }
        // List should be properly cleaned up when it goes out of scope
    }
}
