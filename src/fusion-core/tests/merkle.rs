//! Merkle tree comprehensive tests matching OpenZeppelin StandardMerkleTree.

use alloy::primitives::{keccak256, B256};
use fusion_core::hashing::empty_obligations_root;
use fusion_core::merkle::{hash_pair, MerkleTree};

#[test]
fn test_empty_merkle_tree() {
    let tree = MerkleTree::from_leaves(vec![]);
    assert_eq!(tree.root(), empty_obligations_root());
    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
}

#[test]
fn test_single_leaf() {
    let leaf = keccak256(b"single_obligation");
    let tree = MerkleTree::from_leaves(vec![leaf]);
    assert_eq!(tree.root(), leaf);
    assert_eq!(tree.len(), 1);
    let proof = tree.proof_for_index(0).unwrap();
    assert!(proof.is_empty());
    assert!(MerkleTree::verify(&proof, tree.root(), leaf));
}

#[test]
fn test_two_leaves() {
    let a = keccak256(b"leaf_a");
    let b = keccak256(b"leaf_b");
    let tree = MerkleTree::from_leaves(vec![a, b]);

    let (first, second) = if a < b { (a, b) } else { (b, a) };
    assert_eq!(tree.root(), hash_pair(first, second));

    let proof0 = tree.proof_for_leaf(first).unwrap();
    assert_eq!(proof0, vec![second]);
    assert!(MerkleTree::verify(&proof0, tree.root(), first));

    let proof1 = tree.proof_for_leaf(second).unwrap();
    assert_eq!(proof1, vec![first]);
    assert!(MerkleTree::verify(&proof1, tree.root(), second));
}

#[test]
fn test_many_leaves_and_tamper_detection() {
    let leaves: Vec<B256> = (0..15)
        .map(|i| keccak256(format!("leaf_{i}").as_bytes()))
        .collect();

    let tree = MerkleTree::from_leaves(leaves.clone());
    assert_eq!(tree.len(), 15);

    for leaf in tree.sorted_leaves() {
        let proof = tree.proof_for_leaf(*leaf).unwrap();
        assert!(MerkleTree::verify(&proof, tree.root(), *leaf));

        // Tamper with leaf
        let fake_leaf = keccak256(b"fake");
        assert!(!MerkleTree::verify(&proof, tree.root(), fake_leaf));

        // Tamper with root
        let fake_root = keccak256(b"fake_root");
        assert!(!MerkleTree::verify(&proof, fake_root, *leaf));

        // Tamper with proof if non-empty
        if !proof.is_empty() {
            let mut bad_proof = proof.clone();
            bad_proof[0] = fake_leaf;
            assert!(!MerkleTree::verify(&bad_proof, tree.root(), *leaf));
        }
    }
}
