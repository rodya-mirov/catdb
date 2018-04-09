extern crate catdb_lib;

use catdb_lib::trees;

fn main() {
    let mut b_tree = trees::BTree::<u64>::new();

    for j in 5 .. 5_000_000_u64 {
        b_tree.find(&j);
    }
    for i in 0_u64 .. 10_000_000_u64 {
        b_tree.insert(i);
    }
    for j in 5 .. 5_000_000_u64 {
        b_tree.find(&j);
    }
}
