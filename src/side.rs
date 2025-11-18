use crate::level::Level;
use crate::order::OrderInterface;
use std::collections::BTreeMap;

/// Represents one side of an orderbook (either bids or asks)
/// Uses a BTreeMap to maintain levels sorted by price
pub struct Side<T: OrderInterface> {
    /// BTreeMap of price -> Level, automatically sorted by price
    levels: BTreeMap<u64, Level<T>>,
}

impl<T: OrderInterface> Side<T> {
    /// Creates a new empty side
    pub fn new() -> Self {
        Side {
            levels: BTreeMap::new(),
        }
    }

    /// Returns the number of price levels in this side
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Returns true if this side has no levels
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    /// Gets a reference to the level at the specified price
    /// Returns None if no level exists at that price
    pub fn get_level(&self, price: u64) -> Option<&Level<T>> {
        self.levels.get(&price)
    }

    /// Gets a mutable reference to the level at the specified price
    /// Returns None if no level exists at that price
    pub fn get_level_mut(&mut self, price: u64) -> Option<&mut Level<T>> {
        self.levels.get_mut(&price)
    }

    /// Gets or creates a level at the specified price
    /// If the level doesn't exist, creates a new empty level
    pub fn get_or_create_level(&mut self, price: u64) -> &mut Level<T> {
        self.levels
            .entry(price)
            .or_insert_with(|| Level::new(price))
    }

    /// Removes the level at the specified price
    /// Returns the removed level if it existed, None otherwise
    pub fn remove_level(&mut self, price: u64) -> Option<Level<T>> {
        self.levels.remove(&price)
    }

    /// Gets the best (highest) price level
    /// For bids, this would be the highest bid
    /// For asks, this would be the highest ask
    pub fn best_level(&self) -> Option<(&u64, &Level<T>)> {
        self.levels.iter().next_back()
    }

    /// Gets a mutable reference to the best (highest) price level
    pub fn best_level_mut(&mut self) -> Option<(&u64, &mut Level<T>)> {
        self.levels.iter_mut().next_back()
    }

    /// Gets the worst (lowest) price level
    /// For bids, this would be the lowest bid
    /// For asks, this would be the lowest ask
    pub fn worst_level(&self) -> Option<(&u64, &Level<T>)> {
        self.levels.iter().next()
    }

    /// Gets a mutable reference to the worst (lowest) price level
    pub fn worst_level_mut(&mut self) -> Option<(&u64, &mut Level<T>)> {
        self.levels.iter_mut().next()
    }

    /// Gets the best price (highest price in the BTreeMap)
    pub fn best_price(&self) -> Option<u64> {
        self.levels.keys().next_back().copied()
    }

    /// Gets the worst price (lowest price in the BTreeMap)
    pub fn worst_price(&self) -> Option<u64> {
        self.levels.keys().next().copied()
    }

    /// Removes the best (highest) price level
    /// Returns the removed level if it existed, None otherwise
    pub fn remove_best_level(&mut self) -> Option<Level<T>> {
        self.levels
            .keys()
            .next_back()
            .copied()
            .and_then(|price| self.levels.remove(&price))
    }

    /// Removes the worst (lowest) price level
    /// Returns the removed level if it existed, None otherwise
    pub fn remove_worst_level(&mut self) -> Option<Level<T>> {
        self.levels
            .keys()
            .next()
            .copied()
            .and_then(|price| self.levels.remove(&price))
    }

    /// Clears all levels from this side
    pub fn clear(&mut self) {
        self.levels.clear();
    }

    /// Returns an iterator over all levels (price, level) in ascending price order
    pub fn iter(&self) -> impl Iterator<Item = (&u64, &Level<T>)> {
        self.levels.iter()
    }

    /// Returns a mutable iterator over all levels (price, level) in ascending price order
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut Level<T>)> {
        self.levels.iter_mut()
    }

    /// Returns an iterator over all levels in descending price order
    pub fn iter_rev(&self) -> impl Iterator<Item = (&u64, &Level<T>)> {
        self.levels.iter().rev()
    }

    /// Returns a mutable iterator over all levels in descending price order
    pub fn iter_mut_rev(&mut self) -> impl Iterator<Item = (&u64, &mut Level<T>)> {
        self.levels.iter_mut().rev()
    }
}

impl<T: OrderInterface> Default for Side<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::BasicOrder;

    #[test]
    fn test_new_side() {
        let side = Side::<BasicOrder>::new();
        assert!(side.is_empty());
        assert_eq!(side.level_count(), 0);
    }

    #[test]
    fn test_get_or_create_level() {
        let mut side = Side::<BasicOrder>::new();
        let level = side.get_or_create_level(100);
        assert_eq!(level.price(), 100);
        assert_eq!(side.level_count(), 1);
    }

    #[test]
    fn test_get_level() {
        let mut side = Side::<BasicOrder>::new();
        side.get_or_create_level(100);

        let level = side.get_level(100);
        assert!(level.is_some());
        assert_eq!(level.unwrap().price(), 100);

        let level = side.get_level(200);
        assert!(level.is_none());
    }

    #[test]
    fn test_remove_level() {
        let mut side = Side::<BasicOrder>::new();
        side.get_or_create_level(100);
        side.get_or_create_level(200);

        let removed = side.remove_level(100);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().price(), 100);
        assert_eq!(side.level_count(), 1);

        let removed = side.remove_level(300);
        assert!(removed.is_none());
    }

    #[test]
    fn test_best_and_worst_level() {
        let mut side = Side::<BasicOrder>::new();
        side.get_or_create_level(100);
        side.get_or_create_level(300);
        side.get_or_create_level(200);

        let worst = side.worst_level();
        assert!(worst.is_some());
        assert_eq!(*worst.unwrap().0, 100);

        let best = side.best_level();
        assert!(best.is_some());
        assert_eq!(*best.unwrap().0, 300);
    }

    #[test]
    fn test_best_and_worst_price() {
        let mut side = Side::<BasicOrder>::new();
        side.get_or_create_level(100);
        side.get_or_create_level(300);
        side.get_or_create_level(200);

        assert_eq!(side.worst_price(), Some(100));
        assert_eq!(side.best_price(), Some(300));
    }

    #[test]
    fn test_remove_best_and_worst_level() {
        let mut side = Side::<BasicOrder>::new();
        side.get_or_create_level(100);
        side.get_or_create_level(300);
        side.get_or_create_level(200);

        let removed = side.remove_worst_level();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().price(), 100);
        assert_eq!(side.level_count(), 2);

        let removed = side.remove_best_level();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().price(), 300);
        assert_eq!(side.level_count(), 1);
    }

    #[test]
    fn test_add_order_to_level() {
        let mut side = Side::<BasicOrder>::new();
        let level = side.get_or_create_level(100);
        level.add_order(BasicOrder::new("1", true, 50));

        let level = side.get_level(100);
        assert!(level.is_some());
        assert_eq!(level.unwrap().total_quantity(), 50);
        assert_eq!(level.unwrap().order_count(), 1);
    }

    #[test]
    fn test_iter() {
        let mut side = Side::<BasicOrder>::new();
        side.get_or_create_level(100);
        side.get_or_create_level(300);
        side.get_or_create_level(200);

        let prices: Vec<u64> = side.iter().map(|(price, _)| *price).collect();
        assert_eq!(prices, vec![100, 200, 300]);
    }

    #[test]
    fn test_iter_rev() {
        let mut side = Side::<BasicOrder>::new();
        side.get_or_create_level(100);
        side.get_or_create_level(300);
        side.get_or_create_level(200);

        let prices: Vec<u64> = side.iter_rev().map(|(price, _)| *price).collect();
        assert_eq!(prices, vec![300, 200, 100]);
    }

    #[test]
    fn test_clear() {
        let mut side = Side::<BasicOrder>::new();
        side.get_or_create_level(100);
        side.get_or_create_level(200);

        side.clear();
        assert!(side.is_empty());
        assert_eq!(side.level_count(), 0);
    }
}
