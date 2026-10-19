//! OpenZeppelin-compatible StandardMerkleTree implementation (§8.2).
//!
//! Features sorted-pair hashing and flat-array complete binary tree construction.

use crate::hashing::empty_obligations_root;
use alloy::primitives::{keccak256, B256};

/// Errors encountered in Merkle tree operations.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum MerkleError {
    #[error("Leaf index {0} out of bounds (tree has {1} leaves)")]
    IndexOutOfBounds(usize, usize),
    #[error("Leaf not found in tree")]
    LeafNotFound,
}

/// Hashes two 32-byte nodes in sorted order (§8.2, compatible with OpenZeppelin StandardMerkleTree).
pub fn hash_pair(a: B256, b: B256) -> B256 {
    if a < b {
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(a.as_slice());
        buf[32..].copy_from_slice(b.as_slice());
        keccak256(buf)
    } else {
        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(b.as_slice());
        buf[32..].copy_from_slice(a.as_slice());
        keccak256(buf)
    }
}

/// Sorts leaf hashes ascending.
pub fn sort_leaves(mut leaves: Vec<B256>) -> Vec<B256> {
    leaves.sort();
    leaves
}

/// Complete binary Merkle tree with sorted pairs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleTree {
    sorted_leaves: Vec<B256>,
    tree: Vec<B256>,
}

impl MerkleTree {
    /// Builds a Merkle tree from arbitrary leaves.
    /// Sorts leaves ascending per OpenZeppelin StandardMerkleTree rules.
    pub fn from_leaves(leaves: Vec<B256>) -> Self {
        let sorted = sort_leaves(leaves);
        let n = sorted.len();

        if n == 0 {
            return Self {
                sorted_leaves: vec![],
                tree: vec![],
            };
        }

        if n == 1 {
            return Self {
                sorted_leaves: sorted.clone(),
                tree: vec![sorted[0]],
            };
        }

        let mut tree = vec![B256::ZERO; 2 * n - 1];

        let tree_len = tree.len();
        // Place leaves in tree at end: tree[tree.len() - 1 - i] = sorted_leaves[i]
        for (i, leaf) in sorted.iter().enumerate() {
            tree[tree_len - 1 - i] = *leaf;
        }

        // Fill internal nodes from bottom to top
        let start = tree.len() - 1 - n;
        for i in (0..=start).rev() {
            tree[i] = hash_pair(tree[2 * i + 1], tree[2 * i + 2]);
        }

        Self {
            sorted_leaves: sorted,
            tree,
        }
    }

    /// Returns the Merkle root.
    /// For empty tree, returns `EMPTY_OBLIGATIONS_ROOT`.
    pub fn root(&self) -> B256 {
        if self.sorted_leaves.is_empty() {
            empty_obligations_root()
        } else {
            self.tree[0]
        }
    }

    /// Returns the sorted leaves.
    pub fn sorted_leaves(&self) -> &[B256] {
        &self.sorted_leaves
    }

    /// Number of leaves in the tree.
    pub fn len(&self) -> usize {
        self.sorted_leaves.len()
    }

    /// Returns true if tree has no leaves.
    pub fn is_empty(&self) -> bool {
        self.sorted_leaves.is_empty()
    }

    /// Generates proof for the leaf at `leaf_index` within `sorted_leaves`.
    pub fn proof_for_index(&self, leaf_index: usize) -> Result<Vec<B256>, MerkleError> {
        let n = self.sorted_leaves.len();
        if leaf_index >= n {
            return Err(MerkleError::IndexOutOfBounds(leaf_index, n));
        }
        if n <= 1 {
            return Ok(vec![]);
        }

        let mut index = self.tree.len() - 1 - leaf_index;
        let mut proof = Vec::new();

        while index > 0 {
            let pair_index = if index.is_multiple_of(2) {
                index - 1
            } else {
                index + 1
            };
            proof.push(self.tree[pair_index]);
            index = (index - 1) / 2;
        }

        Ok(proof)
    }

    /// Generates proof for a specific leaf value.
    pub fn proof_for_leaf(&self, leaf: B256) -> Result<Vec<B256>, MerkleError> {
        let idx = self
            .sorted_leaves
            .iter()
            .position(|&l| l == leaf)
            .ok_or(MerkleError::LeafNotFound)?;
        self.proof_for_index(idx)
    }

    /// Verifies a sorted-pair Merkle proof against a leaf and root.
    pub fn verify(proof: &[B256], root: B256, leaf: B256) -> bool {
        let mut computed = leaf;
        for sibling in proof {
            computed = hash_pair(computed, *sibling);
        }
        computed == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_leaf_root_equals_leaf() {
        let leaf = keccak256(b"some leaf");
        let tree = MerkleTree::from_leaves(vec![leaf]);
        assert_eq!(tree.root(), leaf);
        assert_eq!(tree.proof_for_index(0).unwrap(), Vec::<B256>::new());
        assert!(MerkleTree::verify(&[], tree.root(), leaf));
    }

    #[test]
    fn empty_tree_returns_empty_root() {
        let tree = MerkleTree::from_leaves(vec![]);
        assert_eq!(tree.root(), empty_obligations_root());
    }
}
