//! Parser for the text-export subset of Dagor BLK files.
//!
//! Entry order and duplicate names are preserved. The root block has an empty
//! name, and untyped properties have an empty `kind`. Includes are not loaded:
//! `include "path"` becomes a property named `include` with kind `t`.
//!
//! This is not a binary BLK reader. Arrays, matrices using bracket syntax, and
//! other non-export extensions are unsupported. Nesting is limited to 128
//! blocks. See [`parse`] for value and string handling.

mod parser;

pub use parser::{ParseError, parse};

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub name: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Block(Block),
    Property(Property),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: String,
    pub kind: String,
    pub value: String,
}

impl Block {
    /// Iterates over immediate child blocks in source order, including repeats.
    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.entries.iter().filter_map(|entry| match entry {
            Entry::Block(block) => Some(block),
            Entry::Property(_) => None,
        })
    }

    /// Iterates over immediate properties in source order, including repeats.
    pub fn properties(&self) -> impl Iterator<Item = &Property> {
        self.entries.iter().filter_map(|entry| match entry {
            Entry::Property(property) => Some(property),
            Entry::Block(_) => None,
        })
    }

    /// Returns the first immediate property with this name, not a descendant.
    pub fn property(&self, name: &str) -> Option<&Property> {
        self.properties().find(|property| property.name == name)
    }
}
