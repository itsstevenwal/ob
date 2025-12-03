use crate::level::Level;
use crate::list::Node;
use crate::order::OrderInterface;
use std::collections::BTreeMap;

/// Represents one side of an orderbook (either bids or asks)
/// Uses a BTreeMap to maintain levels sorted by price
pub struct Side<O: OrderInterface> {
    is_bid: bool,

    /// BTreeMap of price -> Level, automatically sorted by price
    levels: BTreeMap<O::N, Level<O>>,
}

impl<O: OrderInterface> Side<O> {
    /// Creates a new empty side
    pub fn new(is_bid: bool) -> Self {
        Side {
            is_bid,
            levels: BTreeMap::new(),
        }
    }

    /// Returns the height of the side
    pub fn height(&self) -> usize {
        self.levels.len()
    }

    /// Returns true if this side is empty
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    pub fn insert_order(&mut self, order: O) -> *mut Node<O> {
        let price = order.price();
        if let Some(level) = self.levels.get_mut(&price) {
            let node_ptr = level.add_order(order);
            node_ptr
        } else {
            let mut level = Level::new(price);
            let node_ptr = level.add_order(order);
            self.levels.insert(price, level);
            node_ptr
        }
    }

    pub fn fill_order(&mut self, node_ptr: *mut Node<O>, fill: O::N) -> bool {
        let order = unsafe { &mut (*node_ptr).data };
        let price = order.price();
        let (removed, remove_tree) = if let Some(level) = self.levels.get_mut(&price) {
            let removed = level.fill_order(node_ptr, order, fill);

            (removed, level.is_empty())
        } else {
            panic!("level not found");
        };

        if remove_tree {
            self.levels.remove(&price);
        }

        removed
    }

    pub fn remove_order(&mut self, node_ptr: *mut Node<O>) {
        let price = unsafe { (*node_ptr).data.price() };

        let remove_tree = if let Some(level) = self.levels.get_mut(&price) {
            level.remove_order(node_ptr);
            level.is_empty()
        } else {
            panic!("order not found");
        };

        if remove_tree {
            self.levels.remove(&price);
        }
    }

    /// Returns an iterator over all orders in this side
    /// For bids: orders are returned with higher prices first
    /// For asks: orders are returned with lower prices first
    pub fn iter(&self) -> OrderIter<'_, O> {
        OrderIter {
            levels_iter: if self.is_bid {
                // For bids, iterate in descending order (higher price first)
                Box::new(self.levels.iter().rev())
            } else {
                // For asks, iterate in ascending order (lower price first)
                Box::new(self.levels.iter())
            },
            current_order_iter: None,
        }
    }

    /// Returns a mutable iterator over all orders in this side
    /// For bids: orders are returned with higher prices first
    /// For asks: orders are returned with lower prices first
    pub fn iter_mut(&mut self) -> OrderIterMut<'_, O> {
        OrderIterMut {
            levels_iter: if self.is_bid {
                // For bids, iterate in descending order (higher price first)
                Box::new(self.levels.iter_mut().rev())
            } else {
                // For asks, iterate in ascending order (lower price first)
                Box::new(self.levels.iter_mut())
            },
            current_order_iter: None,
        }
    }
}

/// Iterator over levels, supporting both forward and reverse iteration
/// Uses type erasure to unify forward and reverse iterators
type LevelIter<'a, T, O> = Box<dyn Iterator<Item = (&'a T, &'a Level<O>)> + 'a>;

/// Mutable iterator over levels, supporting both forward and reverse iteration
/// Uses type erasure to unify forward and reverse iterators
type LevelIterMut<'a, T, O> = Box<dyn Iterator<Item = (&'a T, &'a mut Level<O>)> + 'a>;

/// Iterator over orders in a side
pub struct OrderIter<'a, O: OrderInterface> {
    levels_iter: LevelIter<'a, O::N, O>,
    current_order_iter: Option<crate::list::Iter<'a, O>>,
}

impl<'a, O: OrderInterface> Iterator for OrderIter<'a, O> {
    type Item = &'a O;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current order iterator, try to get the next order from it
            if let Some(ref mut order_iter) = self.current_order_iter {
                if let Some(order) = order_iter.next() {
                    return Some(order);
                }
            }

            // Current level exhausted, move to next level
            self.current_order_iter = None;
            if let Some((_, level)) = self.levels_iter.next() {
                self.current_order_iter = Some(level.iter());
            } else {
                // No more levels
                return None;
            }
        }
    }
}

/// Mutable iterator over orders in a side
pub struct OrderIterMut<'a, O: OrderInterface> {
    levels_iter: LevelIterMut<'a, O::N, O>,
    current_order_iter: Option<crate::list::IterMut<'a, O>>,
}

impl<'a, O: OrderInterface> Iterator for OrderIterMut<'a, O> {
    type Item = &'a mut O;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current order iterator, try to get the next order from it
            if let Some(ref mut order_iter) = self.current_order_iter {
                if let Some(order) = order_iter.next() {
                    return Some(order);
                }
            }

            // Current level exhausted, move to next level
            self.current_order_iter = None;
            if let Some((_, level)) = self.levels_iter.next() {
                self.current_order_iter = Some(level.iter_mut());
            } else {
                // No more levels
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::TestOrder;

    #[test]
    fn test_new_side() {
        let side = Side::<TestOrder>::new(true);
        assert!(side.is_empty());
        assert_eq!(side.height(), 0);
    }

    #[test]
    fn test_insert_order() {
        let mut side = Side::<TestOrder>::new(true);
        let order = TestOrder::new("1", true, 100, 50);
        let _node_ptr = side.insert_order(order);

        assert!(!side.is_empty());
        assert_eq!(side.height(), 1);
    }

    #[test]
    fn test_insert_multiple_orders_same_price() {
        let mut side = Side::<TestOrder>::new(true);
        side.insert_order(TestOrder::new("1", true, 100, 50));
        side.insert_order(TestOrder::new("2", true, 100, 30));

        assert_eq!(side.height(), 1);
    }

    #[test]
    fn test_insert_orders_different_prices() {
        let mut side = Side::<TestOrder>::new(true);
        side.insert_order(TestOrder::new("1", true, 100, 50));
        side.insert_order(TestOrder::new("2", true, 200, 30));
        side.insert_order(TestOrder::new("3", true, 150, 20));

        assert_eq!(side.height(), 3);
    }

    #[test]
    fn test_remove_order() {
        let mut side = Side::<TestOrder>::new(true);
        let node_ptr = side.insert_order(TestOrder::new("1", true, 100, 50));
        side.insert_order(TestOrder::new("2", true, 100, 30));

        side.remove_order(node_ptr);
        assert_eq!(side.height(), 1);
    }

    #[test]
    fn test_remove_order_single_order() {
        let mut side = Side::<TestOrder>::new(true);
        let node_ptr = side.insert_order(TestOrder::new("1", true, 100, 50));

        side.remove_order(node_ptr);
        // Note: The level may still exist even if empty (implementation detail)
        // We just verify the order was removed by checking we can iterate
        let order_count: usize = side.iter().count();
        assert_eq!(order_count, 0);
    }

    #[test]
    fn test_iter_bids() {
        let mut side = Side::<TestOrder>::new(true);
        side.insert_order(TestOrder::new("1", true, 100, 50));
        side.insert_order(TestOrder::new("2", true, 300, 30));
        side.insert_order(TestOrder::new("3", true, 200, 20));

        // For bids, should iterate from highest price to lowest
        let prices: Vec<u64> = side.iter().map(|order| order.price()).collect();
        assert_eq!(prices, vec![300, 200, 100]);
    }

    #[test]
    fn test_iter_asks() {
        let mut side = Side::<TestOrder>::new(false);
        side.insert_order(TestOrder::new("1", false, 100, 50));
        side.insert_order(TestOrder::new("2", false, 300, 30));
        side.insert_order(TestOrder::new("3", false, 200, 20));

        // For asks, should iterate from lowest price to highest
        let prices: Vec<u64> = side.iter().map(|order| order.price()).collect();
        assert_eq!(prices, vec![100, 200, 300]);
    }

    #[test]
    fn test_iter_mut() {
        let mut side = Side::<TestOrder>::new(true);
        side.insert_order(TestOrder::new("1", true, 100, 50));
        side.insert_order(TestOrder::new("2", true, 200, 30));

        // Modify orders through mutable iterator
        for order in side.iter_mut() {
            // Can't directly modify BasicOrder fields, but we can test the iterator works
            let _ = order.price();
        }

        // Verify orders are still there
        assert_eq!(side.height(), 2);
    }

    #[test]
    fn test_height() {
        let mut side = Side::<TestOrder>::new(true);
        assert_eq!(side.height(), 0);

        side.insert_order(TestOrder::new("1", true, 100, 50));
        assert_eq!(side.height(), 1);

        side.insert_order(TestOrder::new("2", true, 200, 30));
        assert_eq!(side.height(), 2);

        side.insert_order(TestOrder::new("3", true, 100, 20));
        assert_eq!(side.height(), 2); // Same price, no new level
    }
}
