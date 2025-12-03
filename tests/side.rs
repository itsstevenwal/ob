use ob::side::Side;
#[path = "common/mod.rs"]
mod common;
use common::BasicOrder;

#[test]
fn test_new_side() {
    let side = Side::<BasicOrder>::new(true);
    assert!(side.is_empty());
    assert_eq!(side.height(), 0);
}

#[test]
fn test_insert_order() {
    let mut side = Side::<BasicOrder>::new(true);
    let order = BasicOrder::new("1", true, 100, 50);
    let _node_ptr = side.insert_order(order);

    assert!(!side.is_empty());
    assert_eq!(side.height(), 1);
}

#[test]
fn test_insert_multiple_orders_same_price() {
    let mut side = Side::<BasicOrder>::new(true);
    side.insert_order(BasicOrder::new("1", true, 100, 50));
    side.insert_order(BasicOrder::new("2", true, 100, 30));

    assert_eq!(side.height(), 1);
}

#[test]
fn test_insert_orders_different_prices() {
    let mut side = Side::<BasicOrder>::new(true);
    side.insert_order(BasicOrder::new("1", true, 100, 50));
    side.insert_order(BasicOrder::new("2", true, 200, 30));
    side.insert_order(BasicOrder::new("3", true, 150, 20));

    assert_eq!(side.height(), 3);
}

#[test]
fn test_remove_order() {
    let mut side = Side::<BasicOrder>::new(true);
    let node_ptr = side.insert_order(BasicOrder::new("1", true, 100, 50));
    side.insert_order(BasicOrder::new("2", true, 100, 30));

    side.remove_order(node_ptr);
    assert_eq!(side.height(), 1);
}

#[test]
fn test_remove_order_single_order() {
    let mut side = Side::<BasicOrder>::new(true);
    let node_ptr = side.insert_order(BasicOrder::new("1", true, 100, 50));

    side.remove_order(node_ptr);
    // Note: The level may still exist even if empty (implementation detail)
    // We just verify the order was removed by checking we can iterate
    let order_count: usize = side.iter().count();
    assert_eq!(order_count, 0);
}

#[test]
fn test_iter_bids() {
    let mut side = Side::<BasicOrder>::new(true);
    side.insert_order(BasicOrder::new("1", true, 100, 50));
    side.insert_order(BasicOrder::new("2", true, 300, 30));
    side.insert_order(BasicOrder::new("3", true, 200, 20));

    // For bids, should iterate from highest price to lowest
    let prices: Vec<u64> = side.iter().map(|order| order.price()).collect();
    assert_eq!(prices, vec![300, 200, 100]);
}

#[test]
fn test_iter_asks() {
    let mut side = Side::<BasicOrder>::new(false);
    side.insert_order(BasicOrder::new("1", false, 100, 50));
    side.insert_order(BasicOrder::new("2", false, 300, 30));
    side.insert_order(BasicOrder::new("3", false, 200, 20));

    // For asks, should iterate from lowest price to highest
    let prices: Vec<u64> = side.iter().map(|order| order.price()).collect();
    assert_eq!(prices, vec![100, 200, 300]);
}

#[test]
fn test_iter_mut() {
    let mut side = Side::<BasicOrder>::new(true);
    side.insert_order(BasicOrder::new("1", true, 100, 50));
    side.insert_order(BasicOrder::new("2", true, 200, 30));

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
    let mut side = Side::<BasicOrder>::new(true);
    assert_eq!(side.height(), 0);

    side.insert_order(BasicOrder::new("1", true, 100, 50));
    assert_eq!(side.height(), 1);

    side.insert_order(BasicOrder::new("2", true, 200, 30));
    assert_eq!(side.height(), 2);

    side.insert_order(BasicOrder::new("3", true, 100, 20));
    assert_eq!(side.height(), 2); // Same price, no new level
}

