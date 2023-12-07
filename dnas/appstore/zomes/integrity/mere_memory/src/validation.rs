use hdi::prelude::*;
use mere_memory_types::{
    MemoryEntry,
    MemoryBlockEntry,
};
use crate::{
    EntryTypes,
};



#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op {
	Op::StoreRecord( store_record ) => {
	    if let Some(EntryType::App(AppEntryDef{ entry_index, zome_index, .. })) = store_record.record.action().entry_type() {
		if *zome_index != zome_info().unwrap().id {
		    // This Record does not belong to our Zome so we don't know how to validate it
		    return Ok(ValidateCallbackResult::Valid);
		}

		// debug!("Forwarding validation for StoreRecord->Action::Create->EntryType::App to validation handler");
		if let RecordEntry::Present(entry) = store_record.record.entry() {
		    if let Some(entry_type) = EntryTypes::deserialize_from_type(*zome_index, *entry_index, &entry )? {
			validate_record( entry_type, &store_record.record )
		    }
		    else {
			Ok(ValidateCallbackResult::Invalid(format!("No matching EntryTypes value for: {}/{}", zome_index.0, entry_index.0 )))
		    }
		}
		else {
		    Ok(ValidateCallbackResult::Invalid(format!("Record with AppEntryDef was expected to have a Present(entry): {:?}", store_record.record )))
		}
	    }
	    else if let Action::Delete(delete) = store_record.record.action() {
		let original_record = must_get_valid_record( delete.deletes_address.to_owned() )?;
		let original_action = original_record.signed_action.action();

		if let Some(EntryType::App(AppEntryDef{ entry_index, zome_index, .. })) = original_action.entry_type() {
		    if *zome_index != zome_info().unwrap().id {
			// This Record does not belong to our Zome so we don't know how to validate it
			return Ok(ValidateCallbackResult::Valid)
		    }

		    if let RecordEntry::Present(entry) = original_record.entry() {
			if let Some(entry_type) = EntryTypes::deserialize_from_type(*zome_index, *entry_index, &entry )? {
			    validate_record( entry_type, &store_record.record )
			}
			else {
			    Ok(ValidateCallbackResult::Invalid(format!("No matching EntryTypes value for: {}/{}", zome_index.0, entry_index.0 )))
			}
		    }
		    else {
			Ok(ValidateCallbackResult::Invalid(format!("Record with AppEntryDef was expected to have a Present(entry): {:?}", store_record.record )))
		    }
		}
		else {
		    // debug!("Ignoring Delete event of Action that doesn't contain EntryType::App: {:?}", original_action );
		    Ok(ValidateCallbackResult::Valid)
		}
	    }
	    else {
		// debug!("Ignoring Op::StoreRecord event that doesn't contain EntryType::App: {:?}", store_record.record );
		Ok(ValidateCallbackResult::Valid)
	    }
	},
	_ => {
	    // debug!("Ignoring Op event");
	    Ok(ValidateCallbackResult::Valid)
	},
    }
}

fn validate_record(entry_type: EntryTypes, record: &Record) -> ExternResult<ValidateCallbackResult> {
    // By the time we get here, we know it is for our Zome and the record contains an App Entry

    match record.action() {
	Action::Create(create) => {
	    // debug!("Running create validation for: {:?}", entry_type );
	    match entry_type {
		EntryTypes::Memory(entry) => validate_memory_create( create, entry ),
		EntryTypes::MemoryBlock(entry) => validate_memory_block_create( create, entry ),
	    }
	}
	Action::Update(update) => {
	    // debug!("Running update validation for: {:?}", entry_type );
	    match entry_type {
		EntryTypes::Memory(entry) => validate_memory_update( update, entry ),
		EntryTypes::MemoryBlock(entry) => validate_memory_block_update( update, entry ),
	    }
	},
	Action::Delete(delete) => {
	    // debug!("Running delete validation for: {:?}", entry_type );
	    match entry_type {
		EntryTypes::Memory(_) => validate_memory_delete( delete ),
		EntryTypes::MemoryBlock(_) => validate_memory_block_delete( delete ),
	    }
	},
	_ => {
	    // debug!("Nothing implemented for Action type");
	    Ok(ValidateCallbackResult::Invalid(format!("Unknown entry type: {:?}", entry_type )))
	},
    }
}



//
// Memory
//
fn validate_memory_create(_action: &action::Create, memory: MemoryEntry) -> ExternResult<ValidateCallbackResult> {
    let mut block_sum : u64 = 0;

    for block_addr in memory.block_addresses {
	let block : MemoryBlockEntry = must_get_entry( block_addr.to_owned() )?.try_into()?;
	block_sum = block_sum + (block.bytes.len() as u64);
    }

    if memory.memory_size != block_sum {
	Ok(ValidateCallbackResult::Invalid(format!("MemoryEntry 'memory_size' does not equal the sum of its blocks: {} != {}", memory.memory_size, block_sum )))
    }
    else {
	Ok(ValidateCallbackResult::Valid)
    }
}

fn validate_memory_update(_action: &action::Update, _memory: MemoryEntry) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(format!("Memory entries cannot be updated")))
}

fn validate_memory_delete(_action: &action::Delete) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(format!("Memory entries cannot be deleted")))
}



//
// Memory Block
//
fn validate_memory_block_create(_action: &action::Create, memory_block: MemoryBlockEntry) -> ExternResult<ValidateCallbackResult> {
    if memory_block.bytes.len() > 2_097_152 {
	return Ok(ValidateCallbackResult::Invalid("MemoryBlockEntry cannot be larger than 2MB (2,097,152 bytes)".to_string()));
    }
    else {
	Ok(ValidateCallbackResult::Valid)
    }
}

fn validate_memory_block_update(_action: &action::Update, _memory_block: MemoryBlockEntry) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(format!("MemoryBlock entries cannot be updated")))
}

fn validate_memory_block_delete(_action: &action::Delete) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(format!("MemoryBlock entries cannot be deleted")))
}
