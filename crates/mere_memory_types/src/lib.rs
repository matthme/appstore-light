//! Other Resources
//!
//! - Source code - [github.com/mjbrisebois/hc-zome-mere-memory](https://github.com/mjbrisebois/hc-zome-mere-memory)
//! - Cargo package - [crates.io/crates/mere_memory_types](https://crates.io/crates/mere_memory_types)
//!

use hdi::prelude::*;
use sha2::{ Sha256, Digest };


/// Get the hash of the given bytes as a hex string
pub fn calculate_hash(bytes: &Vec<u8>) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update( bytes );

    hasher.finalize().into()
}


//
// Memory Entry
//
/// An Entry that represents a full byte-set by grouping a set of MemoryBlockEntry
///
/// Example values
/// ```ignore
/// use mere_memory_types::{ MemoryEntry };
///
/// MemoryEntry {
///     author: AgentPubKey::try_from("uhCAkNBaVvGRYmJUqsGNrfO8jC9Ij-t77QcmnAk3E3B8qh6TU09QN").unwrap(),
///     published_at: 1628013738224,
///     hash: "bdff630d3f1c11ef".to_string(),
///     memory_size: 712837,
///     block_addresses: vec![
///         EntryHash::try_from("uhCEkBh2fW3K2RE41X3MOO3LdrMUYPPXWPGtuDjwRrXQZk-94N7Ku").unwrap(),
///     ],
/// };
/// ```
#[hdk_entry_helper]
#[derive(Clone)]
pub struct MemoryEntry {
    pub author: AgentPubKey,
    pub published_at: u64,
    pub hash: String,
    pub memory_size: u64,
    pub block_addresses: Vec<EntryHash>,
}


//
// Memory Block Entry
//
/// Indicates where a memory block fits in a byte-set
///
/// Example (indicating block 1 of a 2 block set)
/// ```
/// use mere_memory_types::SequencePosition;
///
/// SequencePosition {
///     position: 1, // Indexing is intended to start at 1, not 0
///     length: 2,
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SequencePosition {
    pub position: u64,
    pub length: u64,
}

/// An Entry that contains 1 part of a MemoryEntry byte-set
///
/// Example
/// ```
/// use mere_memory_types::{ MemoryBlockEntry, SequencePosition };
///
/// MemoryBlockEntry {
///     sequence: SequencePosition {
///         position: 1, // Indexing is intended to start at 1, not 0
///         length: 2,
///     },
///     bytes: vec![ 34, 129, 87, 2 ],
/// };
/// ```
#[hdk_entry_helper]
#[derive(Clone)]
pub struct MemoryBlockEntry {
    pub sequence: SequencePosition,
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}
