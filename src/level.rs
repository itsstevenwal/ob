use crate::list::{Iter, IterMut, List};
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

    /// Returns an iterator over the orders at this level
    pub fn iter(&self) -> Iter<'_, T> {
        self.orders.iter()
    }

    /// Returns a mutable iterator over the orders at this level
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.orders.iter_mut()
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
        let order = BasicOrder::new("1", true, 100, 50);

        level.add_order(order);
        assert_eq!(level.total_quantity(), 50);
        assert_eq!(level.order_count(), 1);
        assert!(!level.is_empty());
    }

    #[test]
    fn test_add_multiple_orders() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 100, 50));
        level.add_order(BasicOrder::new("2", true, 100, 30));
        level.add_order(BasicOrder::new("3", true, 100, 20));

        assert_eq!(level.total_quantity(), 100);
        assert_eq!(level.order_count(), 3);
    }

    #[test]
    fn test_remove_order() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 100, 50));
        let node_ptr = level.add_order(BasicOrder::new("2", true, 100, 30));
        level.add_order(BasicOrder::new("3", true, 100, 20));

        let removed = level.remove_order(node_ptr);
        assert_eq!(removed, Some(BasicOrder::new("2", true, 100, 30)));
        assert_eq!(level.total_quantity(), 70);
        assert_eq!(level.order_count(), 2);
    }

    #[test]
    fn test_remove_nonexistent_order() {
        let mut level = Level::<BasicOrder>::new(100);
        level.add_order(BasicOrder::new("1", true, 50, 50));

        let null_ptr = std::ptr::null_mut();
        let removed = level.remove_order(null_ptr);
        assert_eq!(removed, None);
        assert_eq!(level.total_quantity(), 50);
        assert_eq!(level.order_count(), 1);
    }
}
