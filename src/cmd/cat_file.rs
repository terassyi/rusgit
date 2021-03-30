use std::io;
use std::str;
use std::fs::File;
use std::io::Read;
use libflate::zlib::{Encoder, Decoder};

use crate::object::{Object, ObjectType};
use crate::object::blob::Blob;
use crate::cmd::RUSGIT_OBJECTS_DIR;

pub enum CatFileType {
    Type,
    Size,
    Print,
}

impl CatFileType {
    pub fn from(opt: &str) -> Option<Self> {
        match opt {
            "t" => Some(CatFileType::Type),
            "s" => Some(CatFileType::Size),
            "p" => Some(CatFileType::Print),
            _ => None
        }
    }
}

pub fn cat_file(sha1: &str, opt: CatFileType) -> io::Result<()> {
    // match option
    let path = hash_key_to_path(sha1);
    match opt {
        CatFileType::Type => {
            // rusgit cat-file -t <hash key> 
            cat_file_t(&path)?;
            
        },
        CatFileType::Size => {
            // rusgit cat-file -s <hash key>
            cat_file_s(&path)?;
        },
        CatFileType::Print => {
            // rusgit cat-file -p <hash key>
            cat_file_p(&path)?;
        }
    };

    Ok(())
}

fn cat_file_p(path: &str) -> io::Result<()> {
    let obj = file_to_object(path)?;
    match obj {
        Object::Blob(blob) => print!("{}", blob.content),
        Object::Commit(commit) => print!("{}", commit),
        Object::Tree(tree) => print!("{}", tree),
    };
    Ok(())
}

fn cat_file_t(path: &str) -> io::Result<()> {
    let obj = file_to_object(path)?;
    println!("{}", obj.typ().to_string());
    Ok(())
}

fn cat_file_s(path: &str) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // decode
    let mut decoder = Decoder::new(&buf[..])?;
    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;
    let size = Object::size(&data).unwrap();
    println!("{}", size);
    Ok(())
}

pub fn hash_key_to_path(sha1: &str) -> String {
    let (dir, file) = sha1.split_at(2);
    // format!("{}/{}/{}", RUSGIT_OBJECTS_DIR, dir, file)
    format!(".git/objects/{}/{}", dir, file)
}

pub fn file_to_object(path: &str) -> io::Result<Object> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // decode
    let mut decoder = Decoder::new(&buf[..])?;
    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;
    let obj= Object::new(&data);
    Ok(obj)
}
