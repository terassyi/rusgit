use std::io;
use crate::object::blob::Blob;
use crate::index;

pub fn update_index(name: &str, mode: Option<&str>, hash: Option<&str>) -> io::Result<()> {
    let blob = Blob::from_name(name)?;
    let index = index::read_index(".git/index")?;
    match mode {
        Some(mode) => {
            // --cacheinfo
            // if mode matches Some(mode), hash should match Some(hash)
            let hash = hex::decode(hash.unwrap()).or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;
            if blob.as_bytes() != hash {
                println!("hash missmatched.");
                return Err(io::Error::from(io::ErrorKind::InvalidData))
            }
            let new_index = index::update_index_cacheinfo(index, mode, hash, name)?;
            index::write_index(".git/index", &new_index)?;
        },
        None => {
            let new_index = index::update_index(index, blob.calc_hash(), name)?;
            index::write_index(".git/index", &new_index)?;
        }
    }

    println!("update index add : {}", name);

    Ok(())
}
