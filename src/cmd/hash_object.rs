use std::io;
use std::fs::File;
use std::io::Read;

use crate::object::blob::Blob;
use crate::object::Object;

pub fn hash_object(path: &str, w: bool) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;
    let blob = Blob::from(&buf).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
    if !w {
        println!("{}", hex::encode(blob.calc_hash()));
    } else {
        let obj = Object::Blob(blob);
        obj.write()?;
    }
    Ok(())
}
