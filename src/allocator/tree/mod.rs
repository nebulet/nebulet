
mod tree;

/// And expandable heap backed by a binary search tree
pub struct Heap {
    tree: TreeList,
}