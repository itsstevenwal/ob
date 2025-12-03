use ob::list::List;

#[test]
fn test_new() {
    let list: List<i32> = List::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn test_push_back() {
    let mut list = List::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    assert_eq!(list.len(), 3);
    assert_eq!(list.iter().last().unwrap(), &3);
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
fn test_drop() {
    let mut list = List::new();
    for i in 0..100 {
        list.push_back(i);
    }
    // List should be properly cleaned up when it goes out of scope
}

#[test]
fn test_iter() {
    let mut list = List::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, vec![&1, &2, &3]);

    // List should still have all elements
    assert_eq!(list.len(), 3);
}

#[test]
fn test_iter_mut() {
    let mut list = List::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    for item in list.iter_mut() {
        *item *= 2;
    }

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, vec![&2, &4, &6]);

    // List should still have all elements
    assert_eq!(list.len(), 3);
}

#[test]
fn test_remove_null_pointer() {
    let mut list = List::new();
    list.push_back(1);

    let result = list.remove(std::ptr::null_mut());
    assert_eq!(result, None);
    assert_eq!(list.len(), 1);
}

#[test]
fn test_remove_head() {
    let mut list = List::new();
    let node1 = list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    let removed = list.remove(node1);
    assert_eq!(removed, Some(1));
    assert_eq!(list.len(), 2);

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, vec![&2, &3]);
}

#[test]
fn test_remove_tail() {
    let mut list = List::new();
    list.push_back(1);
    list.push_back(2);
    let node3 = list.push_back(3);

    let removed = list.remove(node3);
    assert_eq!(removed, Some(3));
    assert_eq!(list.len(), 2);

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, vec![&1, &2]);
}

#[test]
fn test_remove_middle() {
    let mut list = List::new();
    list.push_back(1);
    let node2 = list.push_back(2);
    list.push_back(3);

    let removed = list.remove(node2);
    assert_eq!(removed, Some(2));
    assert_eq!(list.len(), 2);

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, vec![&1, &3]);
}

#[test]
fn test_remove_only_node() {
    let mut list = List::new();
    let node1 = list.push_back(1);

    let removed = list.remove(node1);
    assert_eq!(removed, Some(1));
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, Vec::<&i32>::new());
}

#[test]
fn test_remove_multiple_nodes() {
    let mut list = List::new();
    let node1 = list.push_back(1);
    let node2 = list.push_back(2);
    let node3 = list.push_back(3);
    let node4 = list.push_back(4);
    let node5 = list.push_back(5);

    // Remove middle node
    let removed = list.remove(node3);
    assert_eq!(removed, Some(3));
    assert_eq!(list.len(), 4);

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, vec![&1, &2, &4, &5]);

    // Remove tail
    let removed = list.remove(node5);
    assert_eq!(removed, Some(5));
    assert_eq!(list.len(), 3);

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, vec![&1, &2, &4]);

    // Remove head
    let removed = list.remove(node1);
    assert_eq!(removed, Some(1));
    assert_eq!(list.len(), 2);

    let vec: Vec<&i32> = list.iter().collect();
    assert_eq!(vec, vec![&2, &4]);

    // Remove remaining nodes
    let removed = list.remove(node2);
    assert_eq!(removed, Some(2));
    assert_eq!(list.len(), 1);

    let removed = list.remove(node4);
    assert_eq!(removed, Some(4));
    assert_eq!(list.len(), 0);
    assert!(list.is_empty());
}

