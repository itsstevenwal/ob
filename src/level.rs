use crate::list::List;
use crate::order::OrderInterface;

/// Represents a price level in the orderbook
/// A level contains all orders at a specific price point
pub struct Level<T: OrderInterface> {
    /// The price of this level
    price: u64,
    /// List of orders at this price level
    orders: List<T>,
    /// Total quantity across all orders at this level (cached for performance)
    total_quantity: u64,
}

impl<T: OrderInterface> Level<T> {
    /// Creates a new empty level at the specified price
    pub fn new(price: u64) -> Self {
        Level {
            price,
            orders: List::new(),
            total_quantity: 0,
        }
    }

    /// Returns the price of this level
    pub fn price(&self) -> u64 {
        self.price
    }

    /// Returns the total quantity of all orders at this level
    pub fn total_quantity(&self) -> u64 {
        self.total_quantity
    }

    /// Returns the number of orders at this level
    pub fn order_count(&self) -> usize {
        self.orders.len()
    }

    /// Returns true if this level has no orders
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Adds an order to this level
    /// Orders are added to the back (FIFO order)
    /// Returns a pointer to the newly inserted node
    pub fn add_order(&mut self, order: T) -> *mut crate::list::Node<T> {
        self.total_quantity += order.quantity();
        self.orders.push_back(order)
    }

    /// Removes an order by its pointer
    /// Returns the removed order if found, None otherwise
    pub fn remove_order(&mut self, node_ptr: *mut crate::list::Node<T>) -> Option<T> {
        let removed = self.orders.remove(node_ptr);
        if let Some(ref order) = removed {
            self.total_quantity -= order.quantity();
        }
        removed
    }

    /// Removes the first order (FIFO) from this level
    /// Returns the removed order if the level is not empty, None otherwise
    pub fn remove_first_order(&mut self) -> Option<T> {
        let node_ptr = self.orders.pop_front()?;
        unsafe {
            let boxed_node = Box::from_raw(node_ptr);
            let order = boxed_node.data;
            self.total_quantity -= order.quantity();
            Some(order)
        }
    }

    /// Removes the last order from this level
    /// Returns the removed order if the level is not empty, None otherwise
    pub fn remove_last_order(&mut self) -> Option<T> {
        let node_ptr = self.orders.pop_back()?;
        unsafe {
            let boxed_node = Box::from_raw(node_ptr);
            let order = boxed_node.data;
            self.total_quantity -= order.quantity();
            Some(order)
        }
    }

    /// Gets a reference to the first order without removing it
    pub fn first_order(&self) -> Option<&T> {
        self.orders.front()
    }

    /// Gets a mutable reference to the first order without removing it
    pub fn first_order_mut(&mut self) -> Option<&mut T> {
        self.orders.front_mut()
    }

    /// Gets a reference to the last order without removing it
    pub fn last_order(&self) -> Option<&T> {
        self.orders.back()
    }

    /// Gets a mutable reference to the last order without removing it
    pub fn last_order_mut(&mut self) -> Option<&mut T> {
        self.orders.back_mut()
    }

    /// Reduces the quantity of an order by the specified amount
    /// If the quantity becomes zero or negative, the order is removed
    /// Returns the actual quantity reduced (may be less than requested if order quantity is less)
    pub fn reduce_order_quantity(&mut self, order_id: &str, quantity: u64) -> Option<u64> {
        // Find the order and reduce its quantity
        // Since we can't easily iterate and mutate, we'll need to remove and re-add
        // For better performance, we could use a different approach, but this works
        let mut found = false;
        let mut actual_reduction = 0u64;

        // We need to find the order, modify it, and put it back
        // This is a limitation of the current List API - we can't easily find and modify
        // For now, we'll use a simple approach: remove, modify, re-add
        let mut orders_to_keep = List::new();
        let mut found_order = None;

        while let Some(node_ptr) = self.orders.pop_front() {
            unsafe {
                let boxed_node = Box::from_raw(node_ptr);
                let order = boxed_node.data;

                if order.id() == order_id && !found {
                    found = true;
                    if order.quantity() > quantity {
                        // Reduce quantity
                        actual_reduction = quantity;
                        let mut modified = order;
                        modified.fill(quantity);
                        found_order = Some(modified);
                    } else {
                        // Remove order entirely
                        actual_reduction = order.quantity();
                        found_order = None;
                    }
                } else {
                    orders_to_keep.push_back(order);
                }
            }
        }

        // Rebuild the orders list
        while let Some(node_ptr) = orders_to_keep.pop_front() {
            unsafe {
                let boxed_node = Box::from_raw(node_ptr);
                self.orders.push_back(boxed_node.data);
            }
        }

        if let Some(order) = found_order {
            self.orders.push_back(order);
        }

        if found {
            self.total_quantity -= actual_reduction;
            Some(actual_reduction)
        } else {
            None
        }
    }

    /// Clears all orders from this level
    pub fn clear(&mut self) {
        self.orders.clear();
        self.total_quantity = 0;
    }
}

impl<T: OrderInterface> Default for Level<T> {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::BasicOrder;

    #[test]
    fn test_new_level() {
        let level = Level::<BasicOrder>::new(100);
        assert_eq!(level.price(), 100);
        assert_eq!(level.total_quantity(), 0);
        assert_eq!(level.order_count(), 0);
        assert!(level.is_empty());
    }

    #[test]
    fn test_add_order() {
        let mut level = Level::<BasicOrder>::new(100);
        let order = BasicOrder::new("1", true, 50);

        level.add_order(order);
        assert_eq!(level.total_quantity(), 50);
        assert_eq!(level.order_count(), 1);
        assert!(!level.is_empty());
    }

    #[test]
    fn test_add_multiple_orders() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));
        level.add_order(BasicOrder::new("2", true, 30));
        level.add_order(BasicOrder::new("3", true, 20));

        assert_eq!(level.total_quantity(), 100);
        assert_eq!(level.order_count(), 3);
    }

    #[test]
    fn test_remove_order_by_id() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));
        let node_ptr = level.add_order(BasicOrder::new("2", true, 30));
        level.add_order(BasicOrder::new("3", true, 20));

        let removed = level.remove_order(node_ptr);
        assert_eq!(removed, Some(BasicOrder::new("2", true, 30)));
        assert_eq!(level.total_quantity(), 70);
        assert_eq!(level.order_count(), 2);
    }

    #[test]
    fn test_remove_nonexistent_order() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));

        let null_ptr = std::ptr::null_mut();
        let removed = level.remove_order(null_ptr);
        assert_eq!(removed, None);
        assert_eq!(level.total_quantity(), 50);
        assert_eq!(level.order_count(), 1);
    }

    #[test]
    fn test_remove_first_order() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));
        level.add_order(BasicOrder::new("2", true, 30));

        let removed = level.remove_first_order();
        assert_eq!(removed, Some(BasicOrder::new("1", true, 50)));
        assert_eq!(level.total_quantity(), 30);
        assert_eq!(level.order_count(), 1);
    }

    #[test]
    fn test_remove_last_order() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));
        level.add_order(BasicOrder::new("2", true, 30));

        let removed = level.remove_last_order();
        assert_eq!(removed, Some(BasicOrder::new("2", true, 30)));
        assert_eq!(level.total_quantity(), 50);
        assert_eq!(level.order_count(), 1);
    }

    #[test]
    fn test_first_order() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));
        level.add_order(BasicOrder::new("2", true, 30));

        let first = level.first_order();
        assert_eq!(first, Some(&BasicOrder::new("1", true, 50)));
        assert_eq!(level.order_count(), 2); // Should not remove
    }

    #[test]
    fn test_reduce_order_quantity() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));
        level.add_order(BasicOrder::new("2", true, 30));

        let reduced = level.reduce_order_quantity("1", 20);
        assert_eq!(reduced, Some(20));
        assert_eq!(level.total_quantity(), 60);
        assert_eq!(level.order_count(), 2);

        let first = level.first_order();
        assert_eq!(first, Some(&BasicOrder::new("1", true, 30)));
    }

    #[test]
    fn test_reduce_order_quantity_to_zero() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));
        level.add_order(BasicOrder::new("2", true, 30));

        let reduced = level.reduce_order_quantity("1", 50);
        assert_eq!(reduced, Some(50));
        assert_eq!(level.total_quantity(), 30);
        assert_eq!(level.order_count(), 1);
    }

    #[test]
    fn test_reduce_order_quantity_more_than_available() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));

        let reduced = level.reduce_order_quantity("1", 100);
        assert_eq!(reduced, Some(50));
        assert_eq!(level.total_quantity(), 0);
        assert_eq!(level.order_count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50));
        level.add_order(BasicOrder::new("2", true, 30));

        level.clear();
        assert_eq!(level.total_quantity(), 0);
        assert_eq!(level.order_count(), 0);
        assert!(level.is_empty());
    }
}
