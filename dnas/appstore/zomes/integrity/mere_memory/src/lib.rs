mod validation;

use hdi::prelude::*;
use mere_memory_types::{
    MemoryEntry,
    MemoryBlockEntry,
};

pub trait ToInput<T>
where
    ScopedEntryDefIndex: TryFrom<T, Error = WasmError>,
{
    fn to_input(&self) -> T;
}

impl ToInput<EntryTypes> for MemoryEntry {
    fn to_input(&self) -> EntryTypes {
	EntryTypes::Memory(self.clone())
    }
}

impl ToInput<EntryTypes> for MemoryBlockEntry {
    fn to_input(&self) -> EntryTypes {
	EntryTypes::MemoryBlock(self.clone())
    }
}


#[hdk_entry_defs]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    #[entry_def(required_validations = 5)]
    Memory(MemoryEntry),
    #[entry_def(required_validations = 5)]
    MemoryBlock(MemoryBlockEntry),
}


#[hdk_link_types]
pub enum LinkTypes {
    ByHash,
}
