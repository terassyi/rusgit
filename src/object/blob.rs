
use crate::object::{Object, ObjectType};

#[derive(Debug, Clone)]
pub struct Blob {
    pub size: usize,
    pub content: String,
}

impl Blob {
    pub fn new(content: &str) -> Self {
        Blob {
            size: content.len(),
            content: String::from(content),
        }
    }

    pub fn from(content: &[u8]) -> Option<Self> {
        let data = String::from_utf8(content.to_vec());
        match data {
            Ok(d) => {
                Some(Blob{
                    size: d.len(),
                    content: d,
                })
            },
            Err(_) => None
        }
    }

    pub fn typ(&self) -> ObjectType {
        ObjectType::Blob
    }
}
