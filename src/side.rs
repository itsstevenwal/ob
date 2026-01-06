use crate::{level::Level, list::Node, order::OrderInterface};
use std::collections::{btree_map, BTreeMap};

/// One side of an orderbook (bids or asks). Uses BTreeMap for price-sorted levels.
pub struct Side<O: OrderInterface> {
    is_bid: bool,
    levels: BTreeMap<O::N, Level<O>>,
}

impl<O: OrderInterface> Side<O> {
    #[inline]
    pub fn new(is_bid: bool) -> Self {
        Side {
            is_bid,
            levels: BTreeMap::new(),
        }
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.levels.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    #[inline(always)]
    pub fn insert_order(&mut self, order: O) -> *mut Node<O> {
        let price = order.price();
        if let Some(level) = self.levels.get_mut(&price) {
            level.add_order(order)
        } else {
            let mut level = Level::new(price);
            let node_ptr = level.add_order(order);
            self.levels.insert(price, level);
            node_ptr
        }
    }

    /// Fills an order and returns true if fully filled.
    /// Caller must ensure node_ptr is valid and in this side.
    #[inline(always)]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn fill_order(&mut self, node_ptr: *mut Node<O>, fill: O::N) -> bool {
        let order = unsafe { &mut (*node_ptr).data };
        let price = order.price();
        let level = self
            .levels
            .get_mut(&price)
            .expect("node_ptr must point to valid order in this side");
        let removed = level.fill_order(node_ptr, order, fill);
        if level.is_empty() {
            self.levels.remove(&price);
        }
        removed
    }

    /// Removes an order by its node pointer.
    /// Caller must ensure node_ptr is valid and in this side.
    #[inline(always)]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn remove_order(&mut self, node_ptr: *mut Node<O>) {
        let price = unsafe { (*node_ptr).data.price() };
        let level = self
            .levels
            .get_mut(&price)
            .expect("node_ptr must point to valid order in this side");
        level.remove_order(node_ptr);
        if level.is_empty() {
            self.levels.remove(&price);
        }
    }

    /// Bids: highest price first. Asks: lowest price first.
    #[inline]
    pub fn iter(&self) -> OrderIter<'_, O> {
        if self.is_bid {
            OrderIter::Rev(OrderIterRev {
                levels_iter: self.levels.iter().rev(),
                current_order_iter: None,
            })
        } else {
            OrderIter::Fwd(OrderIterFwd {
                levels_iter: self.levels.iter(),
                current_order_iter: None,
            })
        }
    }

    /// Bids: highest price first. Asks: lowest price first.
    #[inline]
    pub fn iter_mut(&mut self) -> OrderIterMut<'_, O> {
        if self.is_bid {
            OrderIterMut::Rev(OrderIterMutRev {
                levels_iter: self.levels.iter_mut().rev(),
                current_order_iter: None,
            })
        } else {
            OrderIterMut::Fwd(OrderIterMutFwd {
                levels_iter: self.levels.iter_mut(),
                current_order_iter: None,
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Iterators
// ─────────────────────────────────────────────────────────────────────────────

pub struct OrderIterFwd<'a, O: OrderInterface> {
    levels_iter: btree_map::Iter<'a, O::N, Level<O>>,
    current_order_iter: Option<crate::list::Iter<'a, O>>,
}

pub struct OrderIterRev<'a, O: OrderInterface> {
    levels_iter: std::iter::Rev<btree_map::Iter<'a, O::N, Level<O>>>,
    current_order_iter: Option<crate::list::Iter<'a, O>>,
}

pub enum OrderIter<'a, O: OrderInterface> {
    Fwd(OrderIterFwd<'a, O>),
    Rev(OrderIterRev<'a, O>),
}

impl<'a, O: OrderInterface> Iterator for OrderIterFwd<'a, O> {
    type Item = &'a O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut order_iter) = self.current_order_iter {
                if let Some(order) = order_iter.next() {
                    return Some(order);
                }
            }
            self.current_order_iter = None;
            if let Some((_, level)) = self.levels_iter.next() {
                self.current_order_iter = Some(level.iter());
            } else {
                return None;
            }
        }
    }
}

impl<'a, O: OrderInterface> Iterator for OrderIterRev<'a, O> {
    type Item = &'a O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut order_iter) = self.current_order_iter {
                if let Some(order) = order_iter.next() {
                    return Some(order);
                }
            }
            self.current_order_iter = None;
            if let Some((_, level)) = self.levels_iter.next() {
                self.current_order_iter = Some(level.iter());
            } else {
                return None;
            }
        }
    }
}

impl<'a, O: OrderInterface> Iterator for OrderIter<'a, O> {
    type Item = &'a O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OrderIter::Fwd(iter) => iter.next(),
            OrderIter::Rev(iter) => iter.next(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mutable Iterators
// ─────────────────────────────────────────────────────────────────────────────

pub struct OrderIterMutFwd<'a, O: OrderInterface> {
    levels_iter: btree_map::IterMut<'a, O::N, Level<O>>,
    current_order_iter: Option<crate::list::IterMut<'a, O>>,
}

pub struct OrderIterMutRev<'a, O: OrderInterface> {
    levels_iter: std::iter::Rev<btree_map::IterMut<'a, O::N, Level<O>>>,
    current_order_iter: Option<crate::list::IterMut<'a, O>>,
}

pub enum OrderIterMut<'a, O: OrderInterface> {
    Fwd(OrderIterMutFwd<'a, O>),
    Rev(OrderIterMutRev<'a, O>),
}

impl<'a, O: OrderInterface> Iterator for OrderIterMutFwd<'a, O> {
    type Item = &'a mut O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut order_iter) = self.current_order_iter {
                if let Some(order) = order_iter.next() {
                    return Some(order);
                }
            }
            self.current_order_iter = None;
            if let Some((_, level)) = self.levels_iter.next() {
                self.current_order_iter = Some(level.iter_mut());
            } else {
                return None;
            }
        }
    }
}

impl<'a, O: OrderInterface> Iterator for OrderIterMutRev<'a, O> {
    type Item = &'a mut O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut order_iter) = self.current_order_iter {
                if let Some(order) = order_iter.next() {
                    return Some(order);
                }
            }
            self.current_order_iter = None;
            if let Some((_, level)) = self.levels_iter.next() {
                self.current_order_iter = Some(level.iter_mut());
            } else {
                return None;
            }
        }
    }
}

impl<'a, O: OrderInterface> Iterator for OrderIterMut<'a, O> {
    type Item = &'a mut O;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OrderIterMut::Fwd(iter) => iter.next(),
            OrderIterMut::Rev(iter) => iter.next(),
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
        let _node_ptr = side.insert_order(TestOrder::new("1", true, 100, 50));
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
        let order_count: usize = side.iter().count();
        assert_eq!(order_count, 0);
    }

    #[test]
    fn test_iter_bids() {
        let mut side = Side::<TestOrder>::new(true);
        side.insert_order(TestOrder::new("1", true, 100, 50));
        side.insert_order(TestOrder::new("2", true, 300, 30));
        side.insert_order(TestOrder::new("3", true, 200, 20));
        let prices: Vec<u64> = side.iter().map(|order| order.price()).collect();
        assert_eq!(prices, vec![300, 200, 100]);
    }

    #[test]
    fn test_iter_asks() {
        let mut side = Side::<TestOrder>::new(false);
        side.insert_order(TestOrder::new("1", false, 100, 50));
        side.insert_order(TestOrder::new("2", false, 300, 30));
        side.insert_order(TestOrder::new("3", false, 200, 20));
        let prices: Vec<u64> = side.iter().map(|order| order.price()).collect();
        assert_eq!(prices, vec![100, 200, 300]);
    }

    #[test]
    fn test_iter_mut() {
        let mut side = Side::<TestOrder>::new(true);
        side.insert_order(TestOrder::new("1", true, 100, 50));
        side.insert_order(TestOrder::new("2", true, 200, 30));
        for order in side.iter_mut() {
            let _ = order.price();
        }
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
        assert_eq!(side.height(), 2);
    }
}
