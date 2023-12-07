use hdk::prelude::*;
use hex;


pub use mere_memory_types::{
    MemoryEntry,
    MemoryBlockEntry,
    SequencePosition,
};
pub use mere_memory::{
    ToInput,
};
use crate::{
    LinkTypes,
};
use crate::errors::{ ErrorKinds };


pub const TAG_MEMORY: &'static str = "memory";
pub type AppResult<T> = Result<T, ErrorKinds>;

fn now() -> AppResult<u64> {
    sys_time()
	.map( |t| (t.as_micros() / 1000) as u64 )
	.map_err(ErrorKinds::HDKError)
}


const BLOCK_SIZE : usize = 2_097_152; // 2MB

pub fn remember_bytes(bytes: &Vec<u8>) -> AppResult<EntryHash> {
    debug!("Creating entries for remembering ({} bytes)", bytes.len() );

    let memory_size = bytes.len();
    let hash = mere_memory_types::calculate_hash( &bytes );
    let chunks = bytes.chunks( BLOCK_SIZE );
    let block_count = chunks.len();

    let mut blocks : Vec<EntryHash> = vec![];
    for (i, chunk) in chunks.enumerate() {
	let block_addr = create_memory_block_entry(MemoryBlockEntry{
	    sequence: SequencePosition {
		position: (i+1) as u64,
		length: block_count as u64,
	    },
	    bytes: chunk.to_owned(),
	})?;

	blocks.push( block_addr );
    }

    create_memory_entry(CreateInput {
	hash: hash,
	memory_size: memory_size as u64,
	block_addresses: blocks,
    })
}


pub fn retrieve_bytes(addr: EntryHash) -> AppResult<Vec<u8>> {
    let memory_info = get_memory_entry( addr )?;

    let mut chunks = vec![];
    for block_addr in memory_info.block_addresses.iter() {
	let block = get_memory_block_entry( block_addr.to_owned() )?;
	chunks.push( block.bytes.to_owned() );
    }

    Ok( chunks.into_iter().flatten().collect() )
}


pub fn memory_exists(bytes: &Vec<u8>) -> AppResult<bool> {
    let hash = mere_memory_types::calculate_hash( bytes );
    let path = make_hash_path( &hash )?;

    let links = get_links( path.path_entry_hash()?, LinkTypes::ByHash, Some(LinkTag::new( TAG_MEMORY )) )?;

    Ok( links.len() > 0 )
}


pub fn make_hash_path(hash: &[u8; 32]) -> AppResult<Path> {
    let path = Path::from( hex::encode( hash ) );

    Ok( path )
}


#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub hash: [u8; 32],
    pub memory_size: u64,
    pub block_addresses: Vec<EntryHash>,
}

pub fn create_memory_entry(input: CreateInput) -> AppResult<EntryHash> {
    debug!("Creating 'MemoryEntry' ({} bytes): {}", input.memory_size, input.block_addresses.len() );
    let pubkey = agent_info()?.agent_initial_pubkey;
    let default_now = now()?;

    let memory = MemoryEntry {
	author: pubkey.clone(),
	published_at: default_now,
	hash: hex::encode( input.hash ),
	memory_size: input.memory_size,
	block_addresses: input.block_addresses,
    };
    let entry_hash = hash_entry( &memory )?;

    create_entry( memory.to_input() )?;

    let path = make_hash_path( &input.hash )?;

    create_link( path.path_entry_hash()?, entry_hash.to_owned(), LinkTypes::ByHash, LinkTag::new( TAG_MEMORY ) )?;

    Ok( entry_hash )
}



pub fn get_memory_entry(addr: EntryHash) -> AppResult<MemoryEntry> {
    debug!("Get memory: {}", addr );
    let element = get( addr.clone(), GetOptions::latest() )?
	.ok_or( ErrorKinds::EntryNotFoundError(addr.clone()) )?;
    let memory = MemoryEntry::try_from( &element )?;

    Ok(	memory )
}


pub fn create_memory_block_entry(block: MemoryBlockEntry) -> AppResult<EntryHash> {
    debug!("Creating 'MemoryBlockEntry' ({}/{}): {}", block.sequence.position, block.sequence.length, block.bytes.len() );

    create_entry( block.to_input() )?;

    Ok( hash_entry( &block )? )
}



pub fn get_memory_block_entry(addr: EntryHash) -> AppResult<MemoryBlockEntry> {
    debug!("Get 'MemoryBlockEntry': {}", addr );
    let element = get( addr.clone(), GetOptions::latest() )?
	.ok_or( ErrorKinds::EntryNotFoundError(addr.clone()) )?;
    let block = MemoryBlockEntry::try_from( &element )?;

    Ok(	block )
}
