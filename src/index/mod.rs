use std::fmt;
use std::io;
use std::str;
use std::io::Read;
use std::io::Write;
use std::fs::File;
use std::fs;
#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
use chrono::{DateTime, TimeZone, Utc};
use sha1::{Sha1, Digest};
use crate::cmd::cat_file::{file_to_object, hash_key_to_path};
use crate::object::Object;
use crate::object::blob::Blob;
use crate::index::diff::DiffEntry;

mod diff;

#[derive(Debug, Clone)]
pub struct Entry {
    pub c_time: DateTime<Utc>,
    pub m_time: DateTime<Utc>,
    pub dev: u32,
    pub inode: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u32,
    pub hash: Vec<u8>,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub path: String,
    pub entries: i32,
    pub subtrees: usize,
    pub hash: Vec<u8>
}

#[derive(Debug, Clone)]
pub struct Index {
    pub entries: Vec<Entry>,
    pub tree_entries: Vec<TreeEntry>,
}

impl Entry {
    pub fn new(c_time: DateTime<Utc>, m_time: DateTime<Utc>, 
                dev: u32, inode: u32, mode: u32, uid: u32, gid: u32, size: u32,
                hash: Vec<u8>, name: String) -> Entry {
        Entry {
            c_time,
            m_time,
            dev,
            inode,
            mode,
            uid,
            gid,
            size,
            hash,
            name,
        }
    }

    pub fn from(data: &[u8]) -> Option<Entry> {
        let c_time = hex_to_num(&data[0..4]);
        let c_time_nano = hex_to_num(&data[4..8]);
        let m_time = hex_to_num(&data[8..12]);
        let m_time_nano = hex_to_num(&data[12..16]);
        let dev = hex_to_num(&data[16..20]);
        let inode = hex_to_num(&data[20..24]);
        let mode = num_to_mode_num(hex_to_num(&data[24..28])).ok()?;
        let uid = hex_to_num(&data[28..32]);
        let gid = hex_to_num(&data[32..36]);
        let size = hex_to_num(&data[36..40]);
        let hash = Vec::from(&data[40..60]);
        let name_size = hex_to_num(&data[60..62]);
        let name = String::from_utf8(Vec::from(&data[62..(62 + name_size as usize)])).ok()?;
        Some(Entry {
            c_time: Utc.timestamp(c_time.into(), c_time_nano),
            m_time: Utc.timestamp(m_time.into(), m_time_nano),
            dev,
            inode,
            mode,
            uid,
            gid,
            size,
            hash,
            name,
        })
    }

    #[cfg(target_os = "linux")]
    fn from_name(hash: Vec<u8>, name: &str) -> io::Result<Entry> {
        let metadata = fs::metadata(name)?;
        let c_time = metadata.st_ctime() as u32;
        let c_time_nano = metadata.st_ctime_nsec() as u32;
        let m_time = metadata.st_mtime() as u32;
        let m_time_nano = metadata.st_mtime_nsec() as u32;
        Ok(Entry {
            c_time: Utc.timestamp(c_time.into(), c_time_nano),
            m_time: Utc.timestamp(m_time.into(), m_time_nano),
            dev: metadata.st_dev() as u32,
            inode: metadata.st_ino() as u32,
            mode: metadata.st_mode(),
            uid: metadata.st_uid(),
            gid: metadata.st_gid(),
            size: metadata.st_size() as u32,
            hash,
            name: String::from(name),
        })
    }

    #[cfg(target_os = "macos")]
    fn from_name(hash: Vec<u8>, name: &str) -> io::Result<Entry> {
        let metadata = fs::metadata(name)?;
        let c_time = metadata.st_ctime() as u32;
        let c_time_nano = metadata.st_ctime_nsec() as u32;
        let m_time = metadata.st_mtime() as u32;
        let m_time_nano = metadata.st_mtime_nsec() as u32;
        Ok(Entry {
            c_time: Utc.timestamp(c_time.into(), c_time_nano),
            m_time: Utc.timestamp(m_time.into(), m_time_nano),
            dev: metadata.st_dev() as u32,
            inode: metadata.st_ino() as u32,
            mode: metadata.st_mode(),
            uid: metadata.st_uid(),
            gid: metadata.st_gid(),
            size: metadata.st_size() as u32,
            hash,
            name: String::from(name),
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let c_time = self.c_time.timestamp() as u32;
        let c_time_nano = self.c_time.timestamp_subsec_nanos();
        let m_time = self.m_time.timestamp() as u32;
        let m_time_nano = self.m_time.timestamp_subsec_nanos();
        let metadata = [c_time, c_time_nano, m_time, m_time_nano, 
                        self.dev, self.inode, self.mode, self.uid, self.gid, self.size]
                .iter()
                .flat_map(|&d| Vec::from(d.to_be_bytes()))
                .collect::<Vec<u8>>();
        let name_size = self.name.len() as u16;
        let name = self.name.as_bytes();
        let name_offset = 62 + name_size as usize;
        let padding = (0..(8 - name_offset % 8)).map(|_| b'\0').collect::<Vec<u8>>();
        [metadata, self.hash.clone(), Vec::from(name_size.to_be_bytes()), name.to_vec(), padding].concat()
    }

    pub fn size(&self) -> usize {
        let size = 62 + self.name.len();
        size + (8 - size % 8)
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} 0\t{}", self.mode, hex::encode(&self.hash), self.name)
    }
}

impl TreeEntry {
    pub fn new(path: &str, hash: Vec<u8>, entries: i32, subtrees: usize) -> TreeEntry {
        TreeEntry {
            path: String::from(path),
            entries,
            subtrees,
            hash,
        }
    }

    pub fn from(data: &[u8]) -> Option<TreeEntry> {
        // let path_tail = data.iter_mut().position(|&mut d| d == b'\0')? as usize;
        let mut path_tail = 0;
        for (i, d) in data.iter().enumerate() {
            if *d == b'\0' {
                path_tail = i;
                break;
            }
        }
        let path = if path_tail == 0 { "." } else { str::from_utf8(&data[0..path_tail]).ok()? };
        let mut iter = data[(path_tail+1)..].split(|&d| d == b' ');
        let entries = str::from_utf8(iter.next()?).ok()?;
        let entries = entries
                            .parse::<i32>().ok()?;
        let mut lines = iter.next()?.split(|&d| d == b'\n');
        let subtrees = str::from_utf8(lines.next()?).ok()?;
        let subtrees = subtrees
                        .parse::<usize>().ok()?;
        let hash = lines.next()?.to_vec();
        Some(TreeEntry {
            path: String::from(path),
            entries,
            subtrees,
            hash,
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let content = if self.path == "." {
            format!("\0{} {}\n",
                        self.entries,
                        self.subtrees)
        } else {
            format!("{}\0{} {}\n",
                        self.path,
                        self.entries,
                        self.subtrees)
        };
        [Vec::from(content.as_bytes()), self.hash.clone()].concat()
    }

    fn size(&self) -> usize {
        let entries_counter = self.entries.to_string().len();
        let sub_trees_counter = self.subtrees.to_string().len();
        let path_len = if self.path == "." { 0 } else { self.path.len() };
        path_len + 23 + entries_counter + sub_trees_counter
    }
}

impl Index {
    pub fn new(entries: Vec<Entry>, tree_entries: Vec<TreeEntry>) -> Index {
        Index {
            entries,
            tree_entries,
        }
    }

    pub fn from(data: &[u8]) -> Option<Index> {
        if &data[0..4] != b"DIRC" { 
            return None;
        }
        if hex_to_num(&data[4..8]) != 2 {
            return None;
        }
        // entry
        let entry_size = hex_to_num(&data[8..12]);
        let entries = (0..entry_size).try_fold((0, Vec::new()), |(offset, mut entries), _| {
            let entry = Entry::from(&data[(12 + offset)..])?;
            let size = entry.size();
            entries.push(entry);
            Some((offset + size, entries))
        })
        .map(|(_, entries)| entries)?;
        let total = entries.iter().fold(0, |total, entry| total + entry.size()) + 12;
        if data.len() == total {
            return Some(Index::new(entries, Vec::new()));
        }
        if &data[total..(total+4)] != b"TREE" {
            return None;
        }
        let tree_entry_size = hex_to_num(&data[(total+4)..(total+8)]);
        let tree_entries = tree_entrties_from_bytes(&data[(total+8)..])?;
        Some(Index::new(entries, tree_entries))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let hdr = [*b"DIRC", [0x00, 0x00, 0x00, 0x02], (self.entries.len() as u32).to_be_bytes()].concat();
        let entries = self.entries.iter()
                        .flat_map(|d| d.as_bytes())
                        .collect::<Vec<u8>>();
        let content = [hdr, entries].concat();

        let tree_entry_hdr = [*b"TREE", (self.tree_entries_size() as u32).to_be_bytes()].concat();
        let tree_entrries = self.tree_entries.iter()
                            .flat_map(|e| e.as_bytes())
                            .collect::<Vec<u8>>();
        let tree_entries_content = [tree_entry_hdr, tree_entrries].concat();
        let total = [content, tree_entries_content].concat();
        let hash = Vec::from(Sha1::digest(&total).as_slice());
        [total, hash].concat()
    }

    fn tree_entries_size(&self) -> usize {
        self.tree_entries.iter().fold(0, |sum, e| sum + e.size())
    }

    pub fn diff(&self) -> io::Result<Vec<DiffEntry>> {
        let new_blobs = self.entries.iter()
                        .map(|e| Blob::from_name(&e.name).unwrap())
                        .collect::<Vec<Blob>>();
        let old_blobs = self.entries.iter()
                        .map(|e| {
                            let hash = hex::encode(e.hash.clone());
                            Blob::from_hash_file(&hash_key_to_path(&hash)).unwrap()
                        })
                        .collect::<Vec<Blob>>();
        let names = self.entries.iter()
                        .map(|e| e.name.clone())
                        .collect::<Vec<String>>();
        let diff_entries: Vec<DiffEntry> = (0..(names.len())).map(|i| DiffEntry::new(&names[i], new_blobs[i].clone(), old_blobs[i].clone()))
                                            .filter(|e| e.is_modified())
                                            .collect();
        Ok(diff_entries)
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries.iter()
            .try_for_each(|e| write!(f, "{}\n", e))
    }
}


pub fn tree_entrties_from_bytes(data: &[u8]) -> Option<Vec<TreeEntry>> {
    // <name>\0<entries> <sub trees>\n<hash><name>\0<entries> <sub trees>\n<hash><name>
    let mut tree_entries: Vec<TreeEntry> = Vec::new();
    let splitter_offsets: Vec<usize> = data.iter().enumerate()
                            .filter(|(_, &d)| d == b'\n' )
                            .map(|(off, _)| off + 21)
                            .collect();
    let mut tails: Vec<usize> = Vec::new();
    tails.push(splitter_offsets[0]); // first
    let mut i = splitter_offsets[0];
    for o in splitter_offsets {
        if o - i > 20 {
            tails.push(o);
        }
        i = o;
    }
    let mut head = 0;
    for tail in tails {
        let tree_entry = TreeEntry::from(&data[head..tail])?;
        tree_entries.push(tree_entry);
        head = tail;
    }
    Some(tree_entries)
}

pub fn read_index(index_path: &str) -> io::Result<Index> {
    let mut file = match File::open(index_path) {
        Ok(file) => file,
        Err(err) => {
            if err.kind() != io::ErrorKind::NotFound {
                return Err(err);
            }
            File::create(index_path)?;
            return Ok(Index::new(Vec::new(), Vec::new()));
        }
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let index = Index::from(&buf).ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
    Ok(index)
}

pub fn write_index(index_path: &str, index: &Index) -> io::Result<()> {
    let mut file = File::create(index_path)?;
    file.write(&mut index.as_bytes())?;

    Ok(())
}

pub fn update_index(index: Index, hash: Vec<u8>, name: &str) -> io::Result<Index> {
    let entry = Entry::from_name(hash, name)?;
    let mut entries: Vec<Entry> = index.entries.into_iter()
                    .filter(|e| e.name != entry.name && e.hash != entry.hash)
                    .collect();
    entries.push(entry);
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Index::new(entries, index.tree_entries))
}

pub fn update_index_cacheinfo(index: Index, mode: &str, hash: Vec<u8>, name: &str) -> io::Result<Index> {
    let mut entry = Entry::from_name(hash, name)?;
    entry.mode = mode_to_num(mode)?;
    let mut entries: Vec<Entry> = index.entries.into_iter()
                    .filter(|e| e.name != entry.name && e.hash != entry.hash)
                    .collect();
    entries.push(entry);
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Index::new(entries, index.tree_entries))

}

fn hex_to_num(data: &[u8]) -> u32 {
    data.iter().rev().fold((0u32, 1u32), |(sum, offset), &d| {
        (sum + (d as u32 * offset), offset << 8)
    }).0
}

pub fn num_to_mode(mode: u32) -> String {
    let mode = mode as u16;
    let file_type = mode >> 13;
    let (user, group, other) = {
        let permission = mode & 0x01ff;
        let user = (permission & 0x01c0) >> 6;
        let group = (permission & 0x0038) >> 3;
        let other = permission & 0x0007;

        (user, group, other)
    };

    format!("{:03b}{}{}{}", file_type, user, group, other)
}

fn mode_to_num(mode: &str) -> io::Result<u32> {
    let m = u32::from_str_radix(mode, 8).or(Err(io::Error::from(io::ErrorKind::InvalidData)))?;
    Ok(m)
}

fn num_to_mode_num(mode: u32) -> io::Result<u32> {
    let mode = num_to_mode(mode);
    mode.parse::<u32>().or(Err(io::Error::from(io::ErrorKind::InvalidData)))
}

#[cfg(test)]
mod tests {
    use super::Entry;
    use super::Index;
    #[test]
    fn test_hex_to_num() {
        assert_eq!(super::hex_to_num(&[0x00, 0x00, 0x81, 0xa4]), 33188);
    }
    #[test]
    fn test_num_to_mode() {
        assert_eq!(super::num_to_mode(33188), String::from("100644"));
    }
    #[test]
    fn test_mode_to_num() {
        assert_eq!(super::mode_to_num("100644").unwrap(), 33188);
    }
    #[test]
    fn test_num_to_mode_num() {
        assert_eq!(super::num_to_mode_num(33188).unwrap(), 100644);
    }
    #[test]
    fn test_entry_from() {
        let bytes = [
            0x60, 0x5e, 0xf0, 0xa5,
            0x08, 0x7a, 0x51, 0xf3, 0x60, 0x5e, 0xf0, 0xa5, 0x08, 0x7a, 0x51, 0xf3, 0x01, 0x00, 0x00, 0x04,
            0x03, 0x18, 0xb3, 0xb9, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
            0x00, 0x00, 0x00, 0x10, 0xeb, 0x30, 0x54, 0x75, 0xbb, 0x51, 0x92, 0x32, 0x00, 0x53, 0x1a, 0xc7,
            0xfe, 0xf4, 0x6d, 0x4b, 0x98, 0x7b, 0x25, 0x3c, 0x00, 0x0a, 0x2e, 0x67, 0x69, 0x74, 0x69, 0x67,
            0x6e, 0x6f, 0x72, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let entry = Entry::from(&bytes).unwrap();
        assert_eq!(entry.name, ".gitignore");
        assert_eq!(entry.mode, 100644);
    }
    #[test]
    fn test_entry_format() {
        let bytes = [
            0x60, 0x5e, 0xf0, 0xa5,
            0x08, 0x7a, 0x51, 0xf3, 0x60, 0x5e, 0xf0, 0xa5, 0x08, 0x7a, 0x51, 0xf3, 0x01, 0x00, 0x00, 0x04,
            0x03, 0x18, 0xb3, 0xb9, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
            0x00, 0x00, 0x00, 0x10, 0xeb, 0x30, 0x54, 0x75, 0xbb, 0x51, 0x92, 0x32, 0x00, 0x53, 0x1a, 0xc7,
            0xfe, 0xf4, 0x6d, 0x4b, 0x98, 0x7b, 0x25, 0x3c, 0x00, 0x0a, 0x2e, 0x67, 0x69, 0x74, 0x69, 0x67,
            0x6e, 0x6f, 0x72, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let entry = Entry::from(&bytes).unwrap();
        let entry_str = "100644 eb305475bb51923200531ac7fef46d4b987b253c 0\t.gitignore";
        let res = format!("{}", entry);
        assert_eq!(&res, entry_str);
    }

    const TREE_ENTRY: [u8; 29] = [
        0x73, 0x72, 0x63, 0x00, 0x31, 0x39, 0x20, 0x34, 0x0a, 0x66,
        0x1a, 0xee, 0x10, 0x8c, 0x1a, 0x40, 0x78, 0xb2, 0xfd, 0x85, 0x17, 0x29, 0x58, 0x6d, 0x6a, 0x6c,
        0x60, 0x0b, 0xfe,
    ];
    const TREE_ENTRY_ROOT: [u8; 26] = [
        0x00, 0x31, 0x39, 0x20, 0x34, 0x0a, 0x66,
        0x1a, 0xee, 0x10, 0x8c, 0x1a, 0x40, 0x78, 0xb2, 0xfd, 0x85, 0x17, 0x29, 0x58, 0x6d, 0x6a, 0x6c,
        0x60, 0x0b, 0xfe,
    ];

    const TREE_ENTRIES: [u8; 194] = [
        0x00, 0x32, 0x35, 0x20,
        0x31, 0x0a, 0x91, 0x18, 0x9e, 0x52, 0xf2, 0x56, 0x9a, 0x80, 0xeb, 0x07, 0x2f, 0x73, 0xd2, 0x0e,
        0x41, 0xa6, 0xb2, 0x0c, 0x0e, 0xef, 0x73, 0x72, 0x63, 0x00, 0x31, 0x39, 0x20, 0x34, 0x0a, 0x66,
        0x1a, 0xee, 0x10, 0x8c, 0x1a, 0x40, 0x78, 0xb2, 0xfd, 0x85, 0x17, 0x29, 0x58, 0x6d, 0x6a, 0x6c,
        0x60, 0x0b, 0xfe, 0x63, 0x6d, 0x64, 0x00, 0x31, 0x32, 0x20, 0x30, 0x0a, 0xb1, 0x69, 0xba, 0x43,
        0xbb, 0xaf, 0x58, 0xb1, 0x44, 0xb7, 0x57, 0xa6, 0x09, 0x85, 0xcf, 0xca, 0xcc, 0x0d, 0x27, 0x2b,
        0x72, 0x65, 0x66, 0x73, 0x00, 0x31, 0x20, 0x30, 0x0a, 0xfd, 0x16, 0x9d, 0x2b, 0xbd, 0x10, 0x7f,
        0xa0, 0xb6, 0xc9, 0x99, 0x5e, 0x15, 0xf5, 0x7f, 0x27, 0x1d, 0x3f, 0x2d, 0x12, 0x69, 0x6e, 0x64,
        0x65, 0x78, 0x00, 0x31, 0x20, 0x30, 0x0a, 0x11, 0x16, 0x11, 0x10, 0x15, 0xd7, 0x16, 0x23, 0x34,
        0x0c, 0xdf, 0x13, 0xf1, 0x68, 0x82, 0xfc, 0xfb, 0xac, 0x3e, 0xa1, 0x6f, 0x62, 0x6a, 0x65, 0x63,
        0x74, 0x00, 0x34, 0x20, 0x30, 0x0a, 0xc9, 0x38, 0x2a, 0x7c, 0xdb, 0x03, 0x3e, 0x17, 0x1b, 0xc4,
        0x9d, 0x9f, 0x57, 0x2a, 0xbc, 0xc2, 0xf8, 0xfc, 0x4a, 0x5e, 0x3e, 0x7a, 0xa2, 0xa4, 0xd9, 0x0e,
        0xab, 0x05, 0x2f, 0x48, 0xa4, 0x0e, 0xbc, 0xf0, 0x75, 0xec, 0x4f, 0xed, 0x3d, 0x51,
    ];
    use super::TreeEntry;
    #[test]
    fn test_tree_entry_from() {
        let tree_entry = TreeEntry::from(&TREE_ENTRY).unwrap();
        assert_eq!(tree_entry.path, "src");
        assert_eq!(tree_entry.entries, 19);
        assert_eq!(tree_entry.subtrees, 4);
        assert_eq!(hex::encode(tree_entry.hash), "661aee108c1a4078b2fd851729586d6a6c600bfe");
    }
    #[test]
    fn test_tree_entry_as_bytes() {
        let tree_entry = TreeEntry::from(&TREE_ENTRY).unwrap();
        assert_eq!(tree_entry.as_bytes(), TREE_ENTRY.to_vec());

    }
    #[test]
    fn test_tree_entry_size() {
        let tree_entry = TreeEntry::from(&TREE_ENTRY).unwrap();
        assert_eq!(tree_entry.size(), TREE_ENTRY.len());
    }
    #[test]
    fn test_tree_entry_from_root() {
        let tree_entry = TreeEntry::from(&TREE_ENTRY_ROOT).unwrap();
        assert_eq!(tree_entry.path, ".");
        assert_eq!(tree_entry.entries, 19);
        assert_eq!(tree_entry.subtrees, 4);
        assert_eq!(hex::encode(tree_entry.hash), "661aee108c1a4078b2fd851729586d6a6c600bfe");
    }
    #[test]
    fn test_tree_entry_as_bytes_root() {
        let tree_entry = TreeEntry::from(&TREE_ENTRY_ROOT).unwrap();
        assert_eq!(tree_entry.as_bytes(), TREE_ENTRY_ROOT.to_vec());

    }
    #[test]
    fn test_tree_entry_size_root() {
        let tree_entry = TreeEntry::from(&TREE_ENTRY_ROOT).unwrap();
        assert_eq!(tree_entry.size(), TREE_ENTRY_ROOT.len());
    }
    #[test]
    fn test_tree_entries() {
        let tree_entries = super::tree_entrties_from_bytes(&TREE_ENTRIES).unwrap();
        assert_eq!(tree_entries.len(), 6);
        assert_eq!(tree_entries[0].path, ".");
        assert_eq!(tree_entries[1].path, "src");
        assert_eq!(tree_entries[2].path, "cmd");
        assert_eq!(tree_entries[3].path, "refs");
        assert_eq!(tree_entries[4].path, "index");
        assert_eq!(tree_entries[5].path, "object");
    }

    const index_data: [u8; 2302] = [
        0x44,0x49,0x52,0x43,0x00,0x00,0x00,0x02,0x00,0x00,0x00,0x19,0x60,0x70,0x7f,0x04,
        0x37,0x3c,0x98,0xe7,0x60,0x70,0x7f,0x04,0x37,0x3c,0x98,0xe7,0x01,0x00,0x00,0x04,
        0x03,0x26,0x46,0x41,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x00,0x05,0x6b,0x87,0x10,0xa7,0x11,0xf3,0xb6,0x89,0x88,0x5a,0xa5,0xc2,
        0x6c,0x6c,0x06,0xbd,0xe3,0x48,0xe8,0x2b,0x00,0x0d,0x2e,0x64,0x6f,0x63,0x6b,0x65,
        0x72,0x69,0x67,0x6e,0x6f,0x72,0x65,0x00,0x00,0x00,0x00,0x00,0x60,0x70,0x7f,0x46,
        0x24,0xf8,0x72,0xf6,0x60,0x70,0x7f,0x46,0x24,0xf8,0x72,0xf6,0x01,0x00,0x00,0x04,
        0x03,0x18,0xb3,0xb9,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x00,0x1a,0xe3,0xda,0xab,0x27,0x79,0xd1,0xd0,0xdc,0x3e,0x77,0x38,0xb5,
        0xdf,0x94,0xe4,0x04,0xa9,0x57,0x8a,0xd1,0x00,0x0a,0x2e,0x67,0x69,0x74,0x69,0x67,
        0x6e,0x6f,0x72,0x65,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x60,0x63,0x13,0xd4,
        0x11,0x42,0x4d,0xac,0x60,0x63,0x13,0xd4,0x11,0x42,0x4d,0xac,0x01,0x00,0x00,0x04,
        0x03,0x18,0xb4,0x01,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x1d,0xfb,0x5c,0x81,0x55,0x5c,0x8b,0x0e,0xc5,0x3d,0x7e,0x1d,0xab,0xcd,
        0x8f,0x53,0x8c,0x5c,0x9b,0x8a,0x57,0x5c,0x00,0x0a,0x43,0x61,0x72,0x67,0x6f,0x2e,
        0x6c,0x6f,0x63,0x6b,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x60,0x63,0x13,0xce,
        0x0e,0xdd,0xf2,0xa0,0x60,0x63,0x13,0xce,0x0e,0xdd,0xf2,0xa0,0x01,0x00,0x00,0x04,
        0x03,0x18,0xb3,0xba,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x01,0x2f,0xc7,0x6a,0x26,0xce,0xdf,0xa6,0x11,0x0a,0xc5,0x6e,0x3d,0xd7,
        0xef,0x5f,0xb5,0xb4,0x48,0xa9,0x04,0xe3,0x00,0x0a,0x43,0x61,0x72,0x67,0x6f,0x2e,
        0x74,0x6f,0x6d,0x6c,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x60,0x75,0x09,0x28,
        0x32,0xb1,0x9d,0xe6,0x60,0x75,0x09,0x28,0x32,0xb1,0x9d,0xe6,0x01,0x00,0x00,0x04,
        0x03,0x26,0x40,0xee,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x00,0x65,0xb9,0x0e,0x58,0x27,0x54,0x0e,0x20,0x8d,0x42,0x18,0x3e,0x23,
        0x90,0x4b,0x37,0x3c,0x7a,0x03,0x0c,0xb2,0x00,0x0a,0x44,0x6f,0x63,0x6b,0x65,0x72,
        0x66,0x69,0x6c,0x65,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x60,0x70,0x85,0x04,
        0x0a,0xfb,0x45,0xca,0x60,0x70,0x85,0x04,0x0a,0xfb,0x45,0xca,0x01,0x00,0x00,0x04,
        0x03,0x26,0x41,0xc2,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x00,0xdf,0xe3,0xf7,0x42,0x3d,0x01,0xfe,0xcc,0xc4,0xe6,0x4b,0xd8,0xc2,
        0x4b,0xe4,0x7e,0xeb,0x77,0x3b,0x53,0x93,0x00,0x12,0x64,0x6f,0x63,0x6b,0x65,0x72,
        0x2d,0x63,0x6f,0x6d,0x70,0x6f,0x73,0x65,0x2e,0x79,0x6d,0x6c,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x60,0x74,0x4e,0xac,0x32,0x14,0x5d,0x1c,0x60,0x74,0x4e,0xac,
        0x32,0x14,0x5d,0x1c,0x01,0x00,0x00,0x04,0x03,0x27,0xbe,0x6f,0x00,0x00,0x81,0xa4,
        0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,0x00,0x00,0x01,0x3d,0xa4,0x50,0x3c,0x6a,
        0x11,0x04,0x89,0x17,0x63,0xfc,0x89,0xd5,0xb2,0xb4,0x37,0x2a,0xbf,0xb2,0x80,0xe2,
        0x00,0x0e,0x73,0x72,0x63,0x2f,0x63,0x6d,0x64,0x2f,0x61,0x64,0x64,0x2e,0x72,0x73,
        0x00,0x00,0x00,0x00,0x60,0x77,0x1c,0x62,0x30,0x31,0xf1,0x07,0x60,0x77,0x1c,0x62,
        0x30,0x31,0xf1,0x07,0x01,0x00,0x00,0x04,0x03,0x19,0x89,0x61,0x00,0x00,0x81,0xa4,
        0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,0x00,0x00,0x09,0x7b,0x90,0xfb,0x4c,0xc2,
        0x55,0x3a,0x66,0x68,0xd2,0xe6,0xcd,0xbe,0xcb,0x8b,0xe1,0x80,0x9e,0x3f,0x61,0x14,
        0x00,0x13,0x73,0x72,0x63,0x2f,0x63,0x6d,0x64,0x2f,0x63,0x61,0x74,0x5f,0x66,0x69,
        0x6c,0x65,0x2e,0x72,0x73,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x60,0x77,0x13,0xe8,
        0x1c,0xf1,0xe0,0xaf,0x60,0x77,0x13,0xe8,0x1c,0xf1,0xe0,0xaf,0x01,0x00,0x00,0x04,
        0x03,0x2b,0x14,0xcb,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x05,0x00,0x76,0x8b,0x86,0x7d,0x9a,0x78,0x1f,0x83,0xf9,0x0d,0x99,0x89,
        0x3a,0x83,0xc7,0x7c,0x52,0x5a,0x1e,0xa7,0x00,0x11,0x73,0x72,0x63,0x2f,0x63,0x6d,
        0x64,0x2f,0x63,0x6f,0x6d,0x6d,0x69,0x74,0x2e,0x72,0x73,0x00,0x60,0x76,0xf6,0x0b,
        0x04,0x7d,0x12,0xa6,0x60,0x76,0xf6,0x0b,0x04,0x7d,0x12,0xa6,0x01,0x00,0x00,0x04,
        0x03,0x28,0x7b,0x42,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x02,0x1a,0xf7,0x5c,0x19,0xbe,0x3f,0x5c,0xab,0xbf,0xbc,0xed,0x26,0xde,
        0xcb,0xf0,0x99,0xf2,0x12,0x1d,0x6c,0x3f,0x00,0x16,0x73,0x72,0x63,0x2f,0x63,0x6d,
        0x64,0x2f,0x63,0x6f,0x6d,0x6d,0x69,0x74,0x5f,0x74,0x72,0x65,0x65,0x2e,0x72,0x73,
        0x00,0x00,0x00,0x00,0x60,0x74,0x4a,0x85,0x17,0x3c,0x3d,0x60,0x60,0x74,0x4a,0x85,
        0x17,0x3c,0x3d,0x60,0x01,0x00,0x00,0x04,0x03,0x1c,0x79,0x08,0x00,0x00,0x81,0xa4,
        0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,0x00,0x00,0x02,0x0e,0x05,0xca,0x50,0x36,
        0x78,0xdb,0x12,0xb0,0x33,0xfb,0x1b,0xbb,0xf7,0x3c,0xff,0x74,0x46,0x43,0xdd,0x4a,
        0x00,0x16,0x73,0x72,0x63,0x2f,0x63,0x6d,0x64,0x2f,0x68,0x61,0x73,0x68,0x5f,0x6f,
        0x62,0x6a,0x65,0x63,0x74,0x2e,0x72,0x73,0x00,0x00,0x00,0x00,0x60,0x76,0xbf,0x97,
        0x15,0xa8,0xbf,0x9c,0x60,0x76,0xbf,0x97,0x15,0xa8,0xbf,0x9c,0x01,0x00,0x00,0x04,
        0x03,0x19,0x0d,0x55,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x02,0x18,0xca,0x90,0x64,0x86,0xa4,0xa4,0x2d,0x05,0x2e,0xc3,0x57,0x44,
        0xe0,0x76,0x03,0x89,0x89,0x21,0x0b,0x5b,0x00,0x0f,0x73,0x72,0x63,0x2f,0x63,0x6d,
        0x64,0x2f,0x69,0x6e,0x69,0x74,0x2e,0x72,0x73,0x00,0x00,0x00,0x60,0x77,0xc8,0x8e,
        0x34,0x2a,0x0a,0xda,0x60,0x77,0xc8,0x8e,0x34,0x2a,0x0a,0xda,0x01,0x00,0x00,0x04,
        0x03,0x2b,0x7b,0x84,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x0a,0xa7,0x1b,0x2d,0xd2,0xc4,0xd9,0x42,0x28,0x03,0x1e,0xb5,0x64,0xa6,
        0xe1,0x90,0x8e,0x69,0xca,0x4d,0xaf,0xa5,0x00,0x0e,0x73,0x72,0x63,0x2f,0x63,0x6d,
        0x64,0x2f,0x6c,0x6f,0x67,0x2e,0x72,0x73,0x00,0x00,0x00,0x00,0x60,0x74,0x46,0x8e,
        0x2d,0xc5,0x88,0x10,0x60,0x74,0x46,0x8e,0x2d,0xc5,0x88,0x10,0x01,0x00,0x00,0x04,
        0x03,0x26,0x0d,0x23,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x01,0x84,0x9e,0x8e,0x88,0xe2,0x89,0x38,0x79,0xa9,0xbe,0xa4,0x5d,0x25,
        0xc2,0x37,0xc8,0x75,0x5d,0x1a,0x39,0xaf,0x00,0x13,0x73,0x72,0x63,0x2f,0x63,0x6d,
        0x64,0x2f,0x6c,0x73,0x5f,0x66,0x69,0x6c,0x65,0x73,0x2e,0x72,0x73,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x60,0x77,0x12,0xfe,0x00,0x6d,0xf7,0x07,0x60,0x77,0x12,0xfe,
        0x00,0x6d,0xf7,0x07,0x01,0x00,0x00,0x04,0x03,0x19,0x23,0x4a,0x00,0x00,0x81,0xa4,
        0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,0x00,0x00,0x03,0x88,0x25,0xc3,0x06,0xcd,
        0x5e,0x41,0x7b,0xd4,0x88,0xba,0x3a,0x8f,0x22,0x39,0x6c,0x1e,0x99,0x84,0x9e,0x43,
        0x00,0x0e,0x73,0x72,0x63,0x2f,0x63,0x6d,0x64,0x2f,0x6d,0x6f,0x64,0x2e,0x72,0x73,
        0x00,0x00,0x00,0x00,0x60,0x74,0x4e,0xc8,0x2e,0xb2,0xbc,0xd9,0x60,0x74,0x4e,0xc8,
        0x2e,0xb2,0xbc,0xd9,0x01,0x00,0x00,0x04,0x03,0x24,0x6b,0x48,0x00,0x00,0x81,0xa4,
        0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,0x00,0x00,0x03,0xff,0x3a,0xca,0x99,0xc2,
        0xed,0xeb,0xf3,0x35,0xb5,0x0d,0x74,0xda,0x7d,0x5d,0xca,0x9d,0xa5,0x34,0xbe,0xe9,
        0x00,0x17,0x73,0x72,0x63,0x2f,0x63,0x6d,0x64,0x2f,0x75,0x70,0x64,0x61,0x74,0x65,
        0x5f,0x69,0x6e,0x64,0x65,0x78,0x2e,0x72,0x73,0x00,0x00,0x00,0x60,0x76,0xc5,0xc1,
        0x05,0x9a,0xb3,0x09,0x60,0x76,0xc5,0xc1,0x05,0x9a,0xb3,0x09,0x01,0x00,0x00,0x04,
        0x03,0x2a,0xaf,0x6a,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x00,0x81,0xcc,0x32,0xb9,0xb3,0x31,0xe6,0xb9,0x59,0x6f,0x8f,0x37,0x46,
        0xde,0xf5,0x1a,0xe2,0xd3,0x82,0x74,0xad,0x00,0x15,0x73,0x72,0x63,0x2f,0x63,0x6d,
        0x64,0x2f,0x75,0x70,0x64,0x61,0x74,0x65,0x5f,0x72,0x65,0x66,0x2e,0x72,0x73,0x00,
        0x00,0x00,0x00,0x00,0x60,0x75,0xc6,0x5d,0x0a,0x0d,0x86,0x48,0x60,0x75,0xc6,0x5d,
        0x0a,0x0d,0x86,0x48,0x01,0x00,0x00,0x04,0x03,0x28,0x2c,0x90,0x00,0x00,0x81,0xa4,
        0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,0x00,0x00,0x00,0xf4,0x2b,0xeb,0x5f,0x99,
        0x4b,0x1a,0x22,0xcd,0x31,0xeb,0xeb,0xac,0x9d,0x3d,0xeb,0x51,0x28,0xba,0xb4,0xc1,
        0x00,0x15,0x73,0x72,0x63,0x2f,0x63,0x6d,0x64,0x2f,0x77,0x72,0x69,0x74,0x65,0x5f,
        0x74,0x72,0x65,0x65,0x2e,0x72,0x73,0x00,0x00,0x00,0x00,0x00,0x60,0x75,0xc3,0xff,
        0x00,0x3d,0x5a,0xc9,0x60,0x75,0xc3,0xff,0x00,0x3d,0x5a,0xc9,0x01,0x00,0x00,0x04,
        0x03,0x24,0xd0,0x5a,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x4d,0xb7,0x61,0xdf,0x40,0xb9,0x4e,0x7c,0x5e,0x56,0x21,0x02,0xea,0x87,
        0x57,0x83,0x2f,0xcb,0x58,0xc3,0xe6,0xc9,0x00,0x10,0x73,0x72,0x63,0x2f,0x69,0x6e,
        0x64,0x65,0x78,0x2f,0x6d,0x6f,0x64,0x2e,0x72,0x73,0x00,0x00,0x60,0x77,0x13,0xb4,
        0x34,0x56,0xc3,0x8e,0x60,0x77,0x13,0xb4,0x34,0x56,0xc3,0x8e,0x01,0x00,0x00,0x04,
        0x03,0x18,0xb3,0xbc,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x1e,0x5d,0x56,0xdc,0x2d,0x73,0x5e,0x62,0x36,0x79,0x54,0xa0,0x04,0xe1,
        0x1a,0x5e,0x93,0x1f,0x2a,0xfc,0x97,0x58,0x00,0x0b,0x73,0x72,0x63,0x2f,0x6d,0x61,
        0x69,0x6e,0x2e,0x72,0x73,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x60,0x74,0x3c,0x73,
        0x30,0x72,0x95,0xac,0x60,0x74,0x3c,0x73,0x30,0x72,0x95,0xac,0x01,0x00,0x00,0x04,
        0x03,0x1a,0x39,0x13,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x05,0x60,0xcf,0x7d,0xe8,0xcf,0x33,0x1e,0x25,0x90,0x00,0x28,0x2e,0x4d,
        0x5d,0x5d,0x7d,0xd4,0x91,0xb0,0xf3,0xf0,0x00,0x12,0x73,0x72,0x63,0x2f,0x6f,0x62,
        0x6a,0x65,0x63,0x74,0x2f,0x62,0x6c,0x6f,0x62,0x2e,0x72,0x73,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x60,0x76,0x9a,0x9a,0x02,0x1e,0x7b,0xe4,0x60,0x76,0x9a,0x9a,
        0x02,0x1e,0x7b,0xe4,0x01,0x00,0x00,0x04,0x03,0x1a,0x39,0x1c,0x00,0x00,0x81,0xa4,
        0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,0x00,0x00,0x1c,0x3b,0xe9,0x69,0x84,0x78,
        0x9e,0xa5,0x03,0xc9,0xf2,0x29,0xf5,0x43,0x90,0x03,0xdf,0x06,0x3a,0xf9,0xa9,0x85,
        0x00,0x14,0x73,0x72,0x63,0x2f,0x6f,0x62,0x6a,0x65,0x63,0x74,0x2f,0x63,0x6f,0x6d,
        0x6d,0x69,0x74,0x2e,0x72,0x73,0x00,0x00,0x00,0x00,0x00,0x00,0x60,0x75,0x78,0x6f,
        0x1d,0x7d,0x42,0xfe,0x60,0x75,0x78,0x6f,0x1d,0x7d,0x42,0xfe,0x01,0x00,0x00,0x04,
        0x03,0x19,0xb2,0xbb,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x0d,0x5e,0x2c,0xa6,0x61,0xb6,0x34,0xcc,0xd1,0x79,0xb2,0x81,0x64,0x42,
        0x43,0x91,0x89,0xec,0x61,0xf3,0x20,0xad,0x00,0x11,0x73,0x72,0x63,0x2f,0x6f,0x62,
        0x6a,0x65,0x63,0x74,0x2f,0x6d,0x6f,0x64,0x2e,0x72,0x73,0x00,0x60,0x75,0xc7,0x15,
        0x20,0x7a,0x08,0xe4,0x60,0x75,0xc7,0x15,0x20,0x7a,0x08,0xe4,0x01,0x00,0x00,0x04,
        0x03,0x1a,0x39,0x1e,0x00,0x00,0x81,0xa4,0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,
        0x00,0x00,0x21,0x46,0xe5,0x27,0x96,0x13,0xa8,0xab,0x6d,0x12,0xb6,0xce,0x3d,0x71,
        0xf7,0xe3,0x0c,0xa0,0x06,0xf9,0x8e,0x68,0x00,0x12,0x73,0x72,0x63,0x2f,0x6f,0x62,
        0x6a,0x65,0x63,0x74,0x2f,0x74,0x72,0x65,0x65,0x2e,0x72,0x73,0x00,0x00,0x00,0x00,
        0x00,0x00,0x00,0x00,0x60,0x77,0x1c,0x60,0x10,0x33,0x45,0x7f,0x60,0x77,0x1c,0x60,
        0x10,0x33,0x45,0x7f,0x01,0x00,0x00,0x04,0x03,0x2a,0xc8,0xdd,0x00,0x00,0x81,0xa4,
        0x00,0x00,0x01,0xf5,0x00,0x00,0x00,0x14,0x00,0x00,0x08,0xbb,0x72,0x37,0x5b,0x9c,
        0x7c,0xf1,0xc2,0x75,0x8a,0x41,0xe5,0xa0,0x00,0x63,0x3b,0x6a,0x61,0x8a,0x43,0x99,
        0x00,0x0f,0x73,0x72,0x63,0x2f,0x72,0x65,0x66,0x73,0x2f,0x6d,0x6f,0x64,0x2e,0x72,
        0x73,0x00,0x00,0x00,
        0x54,0x52,0x45,0x45,0x00,0x00,0x00,0xae,0x00,0x32,0x35,0x20,
        0x31,0x0a,0x91,0x18,0x9e,0x52,0xf2,0x56,0x9a,0x80,0xeb,0x07,0x2f,0x73,0xd2,0x0e,
        0x41,0xa6,0xb2,0x0c,0x0e,0xef,0x73,0x72,0x63,0x00,0x31,0x39,0x20,0x34,0x0a,0x66,
        0x1a,0xee,0x10,0x8c,0x1a,0x40,0x78,0xb2,0xfd,0x85,0x17,0x29,0x58,0x6d,0x6a,0x6c,
        0x60,0x0b,0xfe,0x63,0x6d,0x64,0x00,0x31,0x32,0x20,0x30,0x0a,0xb1,0x69,0xba,0x43,
        0xbb,0xaf,0x58,0xb1,0x44,0xb7,0x57,0xa6,0x09,0x85,0xcf,0xca,0xcc,0x0d,0x27,0x2b,
        0x72,0x65,0x66,0x73,0x00,0x31,0x20,0x30,0x0a,0xfd,0x16,0x9d,0x2b,0xbd,0x10,0x7f,
        0xa0,0xb6,0xc9,0x99,0x5e,0x15,0xf5,0x7f,0x27,0x1d,0x3f,0x2d,0x12,0x69,0x6e,0x64,
        0x65,0x78,0x00,0x31,0x20,0x30,0x0a,0x11,0x16,0x11,0x10,0x15,0xd7,0x16,0x23,0x34,
        0x0c,0xdf,0x13,0xf1,0x68,0x82,0xfc,0xfb,0xac,0x3e,0xa1,0x6f,0x62,0x6a,0x65,0x63,
        0x74,0x00,0x34,0x20,0x30,0x0a,0xc9,0x38,0x2a,0x7c,0xdb,0x03,0x3e,0x17,0x1b,0xc4,
        0x9d,0x9f,0x57,0x2a,0xbc,0xc2,0xf8,0xfc,0x4a,0x5e,0x3e,0x7a,0xa2,0xa4,0xd9,0x0e,
        0xab,0x05,0x2f,0x48,0xa4,0x0e,0xbc,0xf0,0x75,0xec,0x4f,0xed,0x3d,0x51,
    ];
    #[test]
    fn test_index_from() {
        let index = Index::from(&index_data).unwrap();
        let entry_size = 25;
        assert_eq!(index.entries.len(), entry_size);
        assert_eq!(index.entries[0].name, ".dockerignore");
        assert_eq!(index.entries[11].name, "src/cmd/init.rs");
        assert_eq!(index.tree_entries.len(), 6);
    }
    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_entry_from_name() {
        let hash: Vec<u8> = vec![0x00, 0x00];
        let name = "Cargo.toml";
        let entry = Entry::from_name(hash, name).unwrap();
        assert_eq!(entry.name, "Cargo.toml");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_entry_from_name() {
        let hash: Vec<u8> = vec![0x00, 0x00];
        let name = "Cargo.toml";
        let entry = Entry::from_name(hash, name).unwrap();
        assert_eq!(entry.name, "Cargo.toml");
    }
    #[test]
    fn test_tree_entries_size() {
        let index = Index::from(&index_data).unwrap();
        assert_eq!(index.tree_entries_size(), 0xae);
    }
    #[test]
    fn test_index_as_bytes() {
        let index = Index::from(&index_data).unwrap();
        let res = index.as_bytes();
        assert_eq!(res.len(), 2302);
        assert_eq!(res, &index_data);
    }
    #[test]
    fn test_index_diff() {
        let index = Index::from(&index_data).unwrap();
        let _ = index.diff().unwrap();
        assert_eq!(true, true);
    }
    #[test]
    fn test_update_index() {
        let index = Index::new(vec![], vec![]);
        let new_index = super::update_index(index, Vec::from("hash".as_bytes()), "Cargo.toml").unwrap();
        assert_eq!(new_index.entries.len(), 1);
        assert_eq!(&super::num_to_mode(new_index.entries[0].mode), "100644");
        assert_eq!(&new_index.entries[0].name, "Cargo.toml");
    }
    #[test]
    fn test_update_index_cacheinfo() {
        let index = Index::new(vec![], vec![]);
        let new_index = super::update_index_cacheinfo(index, "100755", Vec::from("hash".as_bytes()), "Cargo.toml").unwrap();
        assert_eq!(new_index.entries.len(), 1);
        assert_eq!(&super::num_to_mode(new_index.entries[0].mode), "100755");
        assert_eq!(&new_index.entries[0].name, "Cargo.toml");
    }
}
