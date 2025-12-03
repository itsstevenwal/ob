use ob::level::Level;
#[path = "common/mod.rs"]
mod common;
use common::BasicOrder;

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

    level.remove_order(node_ptr);
    assert_eq!(level.total_quantity(), 70);
    assert_eq!(level.order_count(), 2);
}

#[test]
fn test_remove_nonexistent_order() {
    let mut level = Level::<BasicOrder>::new(100);
    level.add_order(BasicOrder::new("1", true, 50, 50));

    let null_ptr = std::ptr::null_mut();
    level.remove_order(null_ptr);
    assert_eq!(level.total_quantity(), 50);
    assert_eq!(level.order_count(), 1);
}
