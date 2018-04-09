use std::cmp::Ord;
use std::cmp::Ordering;

use std::fmt::{Debug, Display};
use std::iter;
use std::mem;

const BTREE_MIN_KEYS: usize = 15; // probably too small? depends on disk model
const BTREE_MAX_KEYS: usize = 31; // should be 2*min+1; split if we hit this number of keys in a node

const BTREE_MEDIAN_INDEX: usize = BTREE_MIN_KEYS; // if we're fully loaded (i.e. at the "need to split" point) this is the index to remove and split on

pub trait Key: Sized + Ord + Eq {}

// TODO: macro for defining these
impl Key for u32 {}
impl Key for u64 {}
impl Key for i32 {}
impl Key for i64 {}

enum Node<T: Key> {
    Internal(InternalNode<T>),
    Leaf(LeafNode<T>),
}

enum NodeRef<'a, T: 'a + Key> {
    Internal(&'a InternalNode<T>),
    Leaf(&'a LeafNode<T>),
}

enum NodeRefMut<'a, T: 'a + Key> {
    Internal(&'a mut InternalNode<T>),
    Leaf(&'a mut LeafNode<T>),
}

struct InternalNode<T: Key> {
    keys: Vec<T>,
    children: Vec<Box<Node<T>>>,
    num_keys: usize,
}

struct LeafNode<T: Key> {
    keys: Vec<T>,
    num_keys: usize,
}

pub struct BTree<T: Key> {
    num_keys: usize,
    root: Node<T>,
}

struct InsertState {
    success: bool,
    must_split: bool,
}

struct SplitResult<T: Key> {
    median_key: T,
    right: Node<T>,
}

impl<T: Key> BTree<T> {
    pub fn new() -> BTree<T> {
        BTree {
            num_keys: 0,
            root: Node::Leaf(LeafNode {
                keys: Vec::with_capacity(BTREE_MAX_KEYS),
                num_keys: 0,
            }),
        }
    }

    // TODO: what exactly is it finding? Probably want key -> data
    pub fn find(&self, key: &T) -> bool {
        let mut maybe_node: Option<&Node<T>> = Some(&self.root);

        // recursion would be more elegant but doing this helps manage references
        'main_loop: while let Some(current_node) = maybe_node {
            match current_node {
                // TODO: binary search
                &Node::Leaf(ref node) => {
                    for i in 0..node.num_keys {
                        match key.cmp(&node.keys[i]) {
                            Ordering::Less => {
                                return false;
                            }
                            Ordering::Equal => {
                                return true;
                            }
                            Ordering::Greater => {}
                        }
                    }

                    return false;
                }

                // TODO: binary search
                &Node::Internal(ref node) => {
                    for i in 0..node.num_keys {
                        match key.cmp(&node.keys[i]) {
                            Ordering::Less => {
                                maybe_node = Some(&node.children[i]);
                                continue 'main_loop;
                            }

                            Ordering::Equal => {
                                return true;
                            }
                            Ordering::Greater => {}
                        }
                    }

                    maybe_node = Some(&node.children[node.num_keys]);
                }
            }
        }

        false
    }

    pub fn insert(&mut self, key: T) -> bool {
        let root_insert = insert_at_node(&mut self.root, key);

        // if self.root needs to split, do so
        if root_insert.must_split {
            let root_split = split_node(&mut self.root);
            let new_root = InternalNode {
                num_keys: 1,
                keys: Vec::with_capacity(BTREE_MAX_KEYS),
                children: Vec::with_capacity(BTREE_MAX_KEYS + 1),
            };

            let old_root = mem::replace(&mut self.root, Node::Internal(new_root));

            if let Node::Internal(ref mut root) = self.root {
                root.children.push(Box::new(old_root));
                root.keys.push(root_split.median_key);
                root.children.push(Box::new(root_split.right));
            }
        }

        if root_insert.success {
            self.num_keys += 1;
        }

        root_insert.success
    }

    pub fn size(&self) -> usize {
        return self.num_keys;
    }
}

impl<T: Key + Debug + Display> BTree<T> {
    pub fn draw_tree(&self) {
        print_node(&self.root, 0);
    }
}

fn print_node<T: Key + Debug + Display>(node: &Node<T>, depth: usize) {
    let spaces = iter::repeat(" ").take(depth).collect::<String>();
    match *node {
        Node::Leaf(ref leaf) => {
            println!(
                "{}Leaf: num_keys: {}, keys: {:?}",
                spaces, leaf.num_keys, leaf.keys
            );
        }

        Node::Internal(ref internal) => {
            println!(
                "{}Internal: num_keys: {}, keys: {:?}",
                spaces, internal.num_keys, internal.keys
            );
            for child_ref in internal.children.iter() {
                print_node(child_ref, depth + 2);
            }
        }
    }
}

fn split_node<T: Key>(node: &mut Node<T>) -> SplitResult<T> {
    match *node {
        Node::Leaf(ref mut leaf) => split_leaf_node(leaf),
        Node::Internal(ref mut internal) => split_internal_node(internal),
    }
}

fn split_internal_node<T: Key>(node: &mut InternalNode<T>) -> SplitResult<T> {
    let right_keys = node.keys
        .drain(BTREE_MEDIAN_INDEX + 1..)
        .collect::<Vec<_>>();
    let right_children = node.children
        .drain(BTREE_MEDIAN_INDEX + 1..)
        .collect::<Vec<_>>();

    let median_key = node.keys.remove(BTREE_MEDIAN_INDEX);

    let right = InternalNode {
        num_keys: right_keys.len(),
        keys: right_keys,
        children: right_children,
    };

    node.num_keys = node.keys.len();

    SplitResult {
        right: Node::Internal(right),
        median_key,
    }
}

fn split_leaf_node<T: Key>(node: &mut LeafNode<T>) -> SplitResult<T> {
    let right_keys = node.keys
        .drain(BTREE_MEDIAN_INDEX + 1..)
        .collect::<Vec<_>>();
    let median_key = node.keys.remove(BTREE_MEDIAN_INDEX);

    let right = LeafNode {
        num_keys: right_keys.len(),
        keys: right_keys,
    };

    node.num_keys = node.keys.len();

    SplitResult {
        right: Node::Leaf(right),
        median_key,
    }
}

fn insert_at_node<T: Key>(node: &mut Node<T>, key: T) -> InsertState {
    match *node {
        Node::Internal(ref mut internal) => insert_at_internal_node(internal, key),
        Node::Leaf(ref mut leaf) => insert_at_leaf_node(leaf, key),
    }
}

fn insert_at_internal_node<T: Key>(internal: &mut InternalNode<T>, key: T) -> InsertState {
    for i in 0..internal.num_keys {
        match key.cmp(&internal.keys[i]) {
            Ordering::Less => {
                let mut insert_state = insert_at_node(&mut *internal.children[i], key);

                if insert_state.must_split {
                    let split_result = split_node(&mut *internal.children[i]);

                    internal.keys.insert(i, split_result.median_key);
                    internal
                        .children
                        .insert(i + 1, Box::new(split_result.right));
                    internal.num_keys += 1;

                    insert_state.must_split = internal.num_keys >= BTREE_MAX_KEYS;
                }

                return insert_state;
            }

            Ordering::Equal => {
                return InsertState {
                    success: false,
                    must_split: false,
                };
            }

            Ordering::Greater => {}
        }
    }

    let mut insert_state = insert_at_node(&mut *internal.children[internal.num_keys], key);

    if insert_state.must_split {
        let split_result = split_node(&mut *internal.children[internal.num_keys]);

        internal.keys.push(split_result.median_key);
        internal.children.push(Box::new(split_result.right));
        internal.num_keys += 1;

        insert_state.must_split = internal.num_keys >= BTREE_MAX_KEYS;
    }

    insert_state
}

fn insert_at_leaf_node<T: Key>(leaf: &mut LeafNode<T>, key: T) -> InsertState {
    for i in 0..leaf.num_keys {
        match key.cmp(&leaf.keys[i]) {
            Ordering::Less => {
                leaf.keys.insert(i, key);
                leaf.num_keys += 1;
                return InsertState {
                    success: true,
                    must_split: leaf.num_keys >= BTREE_MAX_KEYS,
                };
            }

            Ordering::Equal => {
                return InsertState {
                    success: false,
                    must_split: false,
                };
            }
            Ordering::Greater => {}
        }
    }

    leaf.keys.insert(leaf.num_keys, key);
    leaf.num_keys += 1;

    return InsertState {
        success: true,
        must_split: leaf.num_keys >= BTREE_MAX_KEYS,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_test_u32() {
        let empty = BTree::<u32>::new();

        assert!(!empty.find(&1331));
        assert!(!empty.find(&642426344));

        assert_eq!(empty.size(), 0 as usize);
    }

    #[test]
    fn empty_test_i64() {
        let empty = BTree::<i64>::new();

        assert!(!empty.find(&1331));
        assert!(!empty.find(&642426344));

        assert_eq!(empty.size(), 0 as usize);
    }

    #[test]
    fn test_insert_u32() {
        let mut tree = BTree::<u32>::new();

        assert!(tree.size() == 0 as usize);

        assert!(tree.insert(123));

        assert!(tree.size() == 1 as usize);
        assert!(tree.find(&123));
        assert!(!tree.find(&43));
        assert!(!tree.find(&5278945));

        assert!(tree.insert(5278945));

        assert!(tree.size() == 2 as usize);
        assert!(tree.find(&123));
        assert!(!tree.find(&43));
        assert!(tree.find(&5278945));

        assert!(!tree.insert(5278945));

        assert!(tree.size() == 2 as usize);
        assert!(tree.find(&123));
        assert!(!tree.find(&43));
        assert!(tree.find(&5278945));
    }

    #[test]
    fn test_insert_more_i32() {
        let mut tree = BTree::<i32>::new();

        for i in 0..50 {
            assert!(tree.insert(i));
            assert_eq!(tree.size(), (i + 1) as usize);

            tree.draw_tree();

            for j in 0..1000 {
                assert_eq!(tree.find(&j), j <= i);
            }
        }
    }

    #[test]
    fn test_insert_out_of_order_i64() {
        let mut count = 0;
        let mut tree = BTree::<i64>::new();

        for x in 0..BTREE_MIN_KEYS + 1 {
            assert_eq!(tree.size(), count);
            assert!(!tree.find(&(x as i64)));

            tree.insert(x as i64);
            count += 1;

            assert_eq!(tree.size(), count);
            assert!(tree.find(&(x as i64)));
        }

        for x in BTREE_MAX_KEYS * 50..BTREE_MAX_KEYS * 60 {
            assert_eq!(tree.size(), count);
            assert!(!tree.find(&(x as i64)));

            tree.insert(x as i64);
            count += 1;

            assert_eq!(tree.size(), count);
            assert!(tree.find(&(x as i64)));
        }

        for x in BTREE_MIN_KEYS + 1..100 {
            assert_eq!(tree.size(), count);
            assert!(!tree.find(&(x as i64)));

            tree.insert(x as i64);
            count += 1;

            assert_eq!(tree.size(), count);
            assert!(tree.find(&(x as i64)));
        }
    }

    #[test]
    fn test_insert_much_more_u64() {
        let mut tree = BTree::<u64>::new();

        for i in 0..1000 {
            assert!(tree.insert(i));
            assert_eq!(tree.size(), (i + 1) as usize);

            for j in 0..1000 {
                assert_eq!(tree.find(&j), j <= i);
            }
        }
    }

}
