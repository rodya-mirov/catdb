use std::cmp::Ord;
use std::cmp::Ordering;

const BTREE_MIN_KEYS: usize = 15; // probably too small
const BTREE_MAX_KEYS: usize = 31; // should be 2*min+1

pub trait Key: Sized + Ord + Eq {
}

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
    root: Node<T>
}

impl <T: Key> BTree<T> {
    pub fn new() -> BTree<T> {
        BTree {
            num_keys: 0,
            root: Node::Leaf(LeafNode { keys: Vec::with_capacity(BTREE_MAX_KEYS), num_keys: 0 }),
        }
    }

    // TODO: what exactly is it finding? Probably want key -> data
    pub fn find(&self, key: &T) -> bool {
        let mut maybe_node: Option<&Node<T>> = Some(&self.root);

        // recursion would be more elegant but doing this helps manage references
        'main_loop: while let Some(current_node) = maybe_node {
            match current_node {
                // TODO: binary search
                & Node::Leaf(ref node) => {
                    for i in 0 .. node.num_keys {
                        match key.cmp(&node.keys[i]) {
                            Ordering::Less => { return false; },
                            Ordering::Equal => { return true; },
                            Ordering::Greater => {},
                        }
                    }

                    return false;
                }

                // TODO: binary search
                & Node::Internal(ref node) => {
                    for i in 0 .. node.num_keys {
                        match key.cmp(&node.keys[i]) {
                            Ordering::Less => {
                                maybe_node = Some(&node.children[i]);
                                continue 'main_loop;
                            },

                            Ordering::Equal => { return true; },
                            Ordering::Greater => {},
                        }
                    }

                    maybe_node = Some(&node.children[node.num_keys]);
                }
            }
        }

        false
    }

    pub fn insert(&mut self, key: T) -> bool {
        let (_inserted_at, success) = match self.root {
            Node::Internal(ref mut node) =>
                self.insert_at_internal_node(&mut node, key),

            Node::Leaf(ref mut node) =>
                self.insert_at_leaf_node(&mut node, key),
        };

        // then split if needed
        if success {
            panic!();
        }

        return success;
    }

    fn insert_at_leaf_node<'a>(&'a mut self, leaf: &'a mut LeafNode<T>, key: T) -> (NodeRefMut<'a, T>, bool) {
        for i in 0 .. leaf.num_keys {
            match key.cmp(&leaf.keys[i]) {
                Ordering::Less => {
                    // TODO: insert at i
                    panic!()
                },

                Ordering::Equal => { return (NodeRefMut::Leaf(leaf), false); },
                Ordering::Greater => {},
            }
        }

        // TODO: insert at end

        panic!()
    }

    fn insert_at_internal_node<'a>(&'a mut self, internal: &'a mut InternalNode<T>, key: T) -> (NodeRefMut<'a, T>, bool) {
        for i in 0 .. internal.num_keys {
            match key.cmp(&internal.keys[i]) {
                Ordering::Less => {
                    return match *internal.children[i] {
                        Node::Leaf(ref mut node) => 
                            self.insert_at_leaf_node(node, key),
                        Node::Internal(ref mut node) => 
                            self.insert_at_internal_node(node, key),
                    };
                }

                Ordering::Equal => {
                    return (NodeRefMut::Internal(internal), false);
                },

                Ordering::Greater => {},
            }
        }

        
        return match *internal.children[internal.num_keys] {
            Node::Leaf(ref mut node) => 
                self.insert_at_leaf_node(node, key),
            Node::Internal(ref mut node) => 
                self.insert_at_internal_node(node, key),
        };
    }

    pub fn size(&self) -> usize {
        return self.num_keys;
    }
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

    
}

