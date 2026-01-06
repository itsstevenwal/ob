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
        if self.head.is_null() || self.length == 0 {
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

    /// Removes the node at the given pointer from the list
    /// Returns the data from the removed node, or None if the pointer is null
    ///
    /// # Safety
    /// The caller must ensure the pointer is valid and points to a node in this list
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn remove(&mut self, node_ptr: *mut Node<T>) -> Option<T> {
        if node_ptr.is_null() || self.length == 0 {
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
        while self.pop_front().is_some() {}
    }
}

/// An iterator over the doubly linked list that consumes the list
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

/// An iterator over the doubly linked list that borrows the list
pub struct Iter<'a, T> {
    current: *mut Node<T>,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        unsafe {
            let data = &(*self.current).data;
            self.current = (*self.current).next;
            Some(data)
        }
    }
}

/// A mutable iterator over the doubly linked list that borrows the list mutably
pub struct IterMut<'a, T> {
    current: *mut Node<T>,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        unsafe {
            let data = &mut (*self.current).data;
            let next = (*self.current).next;
            self.current = next;
            Some(data)
        }
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}

impl<T> List<T> {
    /// Returns an iterator over the list that borrows the list
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: self.head,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns a mutable iterator over the list that borrows the list mutably
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            current: self.head,
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_default() {
        let list: List<i32> = List::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);

        let list: List<i32> = List::default();
        assert!(list.is_empty());
    }

    #[test]
    fn test_push_and_pop() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        assert_eq!(list.len(), 3);

        let node = list.pop_front().unwrap();
        assert_eq!(unsafe { (*node).data }, 1);
        unsafe {
            let _ = Box::from_raw(node);
        }

        let node = list.pop_front().unwrap();
        assert_eq!(unsafe { (*node).data }, 2);
        unsafe {
            let _ = Box::from_raw(node);
        }

        let node = list.pop_front().unwrap();
        assert_eq!(unsafe { (*node).data }, 3);
        unsafe {
            let _ = Box::from_raw(node);
        }

        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn test_iterators() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        // iter
        let vec: Vec<&i32> = list.iter().collect();
        assert_eq!(vec, vec![&1, &2, &3]);
        assert_eq!(list.len(), 3);

        // iter_mut
        for item in list.iter_mut() {
            *item *= 2;
        }
        let vec: Vec<&i32> = list.iter().collect();
        assert_eq!(vec, vec![&2, &4, &6]);

        // into_iter
        let vec: Vec<i32> = list.into_iter().collect();
        assert_eq!(vec, vec![2, 4, 6]);
    }

    #[test]
    fn test_remove() {
        // Null pointer
        let mut list = List::new();
        list.push_back(1);
        assert_eq!(list.remove(std::ptr::null_mut()), None);

        // Remove head, middle, tail, only node
        let mut list = List::new();
        let n1 = list.push_back(1);
        let n2 = list.push_back(2);
        let n3 = list.push_back(3);

        assert_eq!(list.remove(n2), Some(2)); // middle
        assert_eq!(list.iter().collect::<Vec<_>>(), vec![&1, &3]);

        assert_eq!(list.remove(n3), Some(3)); // tail
        assert_eq!(list.remove(n1), Some(1)); // head (now only)
        assert!(list.is_empty());
    }

    #[test]
    fn test_drop() {
        let mut list = List::new();
        for i in 0..100 {
            list.push_back(i);
        }
        // Cleanup handled by Drop
    }
}
