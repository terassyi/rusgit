use std::io;
use crate::object::blob::Blob;
use crate::index;
use crate::cmd::GIT_INDEX;

pub fn update_index(name: &str, mode: Option<&str>, hash: Option<&str>) -> io::Result<()> {
    let blob = Blob::from_name(name)?;
    let index = index::read_index(GIT_INDEX)?;
    match mode {
        Some(mode) => {
            // --cacheinfo
            // if mode matches Some(mode), hash should match Some(hash)
            let hash = hex::decode(hash.unwrap()).or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;
            if blob.as_bytes() != hash {
                return Err(io::Error::from(io::ErrorKind::InvalidData))
            }
            let new_index = index::update_index_cacheinfo(index, mode, hash, name)?;
            index::write_index(GIT_INDEX, &new_index)?;
        },
        None => {
            let new_index = index::update_index(index, blob.calc_hash(), name)?;
            index::write_index(GIT_INDEX, &new_index)?;
        }
    }
    Ok(())
}
