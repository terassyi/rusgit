// use std::fmt;
use wu_diff;
use crate::object::blob::Blob;

#[derive(Debug, Clone)]
pub struct DiffEntry {
    pub name: String,
    pub new: Blob,
    pub old: Blob,
    pub new_mode: u32,
    pub old_mode: u32,
}

impl DiffEntry {
    pub fn new(name: &str, new: Blob, old: Blob, new_mode: u32, old_mode: u32) -> DiffEntry {
        DiffEntry {
            name: String::from(name),
            new,
            old,
            new_mode,
            old_mode,
        }
    }

    pub fn is_modified(&self) -> bool {
        self.is_contents_modified() || self.is_mode_modified()
    }

    pub fn is_contents_modified(&self) -> bool {
        self.new.calc_hash() != self.old.calc_hash()
    }

    pub fn is_mode_modified(&self) -> bool {
        self.new_mode != self.old_mode
    }
    
    pub fn compare(&self) -> Vec<wu_diff::DiffResult> {
        let new: Vec<&str> = self.new.content.split('\n').collect();
        let old: Vec<&str> = self.old.content.split('\n').collect();
        wu_diff::diff(&old, &new)
    }
}

// impl fmt::Display for DiffEntry {
    // fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // }

// }
