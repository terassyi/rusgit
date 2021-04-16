use std::io;
use std::fs::File;
use std::io::Read;
use libflate::zlib::{Encoder, Decoder};
use sha1::{Sha1, Digest};
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

    pub fn from_name(name: &str) -> io::Result<Blob> {
        let mut file = File::open(name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let blob = Blob::from(&buf).ok_or(io::Error::from(io::ErrorKind::InvalidData))?;
        Ok(blob)
    }

    pub fn calc_hash(&self) -> Vec<u8> {
        Vec::from(Sha1::digest(&self.as_bytes()).as_slice())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let hdr = format!("{} {}\0", ObjectType::Blob.to_string(), self.size);
        let content = format!("{}{}", hdr, self.content);
        Vec::from(content.as_bytes())
    }

    pub fn typ(&self) -> ObjectType {
        ObjectType::Blob
    }

    pub fn is_modified(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn from_hash_file(path: &str) -> io::Result<Blob> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        // decode
        let mut decoder = Decoder::new(&buf[..])?;
        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;
        let mut iter = data.splitn(2, |&b| b == b'\0');
        iter.next().ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
        let d = iter.next().ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
        Blob::from(&d).ok_or(io::Error::from(io::ErrorKind::InvalidInput))
    }
}
