use crate::level::Level;
use crate::list::Node;
use crate::order::{Id, Num, OrderInterface};
use std::collections::BTreeMap;

/// Represents one side of an orderbook (either bids or asks)
/// Uses a BTreeMap to maintain levels sorted by price
pub struct Side<T: Id, N: Num, O: OrderInterface<T, N>> {
    is_bid: bool,

    /// BTreeMap of price -> Level, automatically sorted by price
    levels: BTreeMap<N, Level<T, N, O>>,
}

impl<T: Id, N: Num, O: OrderInterface<T, N>> Side<T, N, O> {
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
        let price = *order.price();
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

    pub fn fill_order(&mut self, node_ptr: *mut Node<O>, fill: N) -> bool {
        let order = unsafe { &mut (*node_ptr).data };
        let price = *order.price();
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
    pub fn iter(&self) -> OrderIter<'_, T, N, O> {
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
    pub fn iter_mut(&mut self) -> OrderIterMut<'_, T, N, O> {
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
type LevelIter<'a, T: Id, N: Num, O: OrderInterface<T, N>> =
    Box<dyn Iterator<Item = (&'a N, &'a Level<T, N, O>)> + 'a>;

/// Mutable iterator over levels, supporting both forward and reverse iteration
/// Uses type erasure to unify forward and reverse iterators
type LevelIterMut<'a, T: Id, N: Num, O: OrderInterface<T, N>> =
    Box<dyn Iterator<Item = (&'a N, &'a mut Level<T, N, O>)> + 'a>;

/// Iterator over orders in a side
pub struct OrderIter<'a, T: Id, N: Num, O: OrderInterface<T, N>> {
    levels_iter: LevelIter<'a, T, N, O>,
    current_order_iter: Option<crate::list::Iter<'a, O>>,
}

impl<'a, T: Id, N: Num, O: OrderInterface<T, N>> Iterator for OrderIter<'a, T, N, O> {
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
pub struct OrderIterMut<'a, T: Id, N: Num, O: OrderInterface<T, N>> {
    levels_iter: LevelIterMut<'a, T, N, O>,
    current_order_iter: Option<crate::list::IterMut<'a, O>>,
}

impl<'a, T: Id, N: Num, O: OrderInterface<T, N>> Iterator for OrderIterMut<'a, T, N, O> {
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
