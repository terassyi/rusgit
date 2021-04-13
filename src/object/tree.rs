
use sha1::{Sha1, Digest};
use std::str;
use std::fmt;
use std::io;
use std::path::Path;

use crate::object::{Object, ObjectType};
use crate::cmd::cat_file::{hash_key_to_path, file_to_object};
use crate::index;
use crate::cmd::GIT_INDEX;

#[derive(Debug, Clone)]
pub struct File {
    pub mode: usize,
    pub name: String,
    pub typ: ObjectType,
    pub hash: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub files: Vec<File>
}

impl File {
    fn new(mode: usize, hash: &[u8], name: &str, typ: ObjectType) -> Self {
        File {
            mode,
            name: String::from(name),
            typ,
            hash: hash.to_vec(),
        }
    }

    pub fn from(hdr: &[u8], hash: &[u8]) -> Option<Self> {
        let iterstr = str::from_utf8(hdr).ok()?;
        let mut iter = iterstr
                    .split_whitespace();
        let mode = iter.next()
                    .and_then(|d| d.parse::<usize>().ok())?;
        let name = iter.next()?;
        Some(File {
            mode,
            name: String::from(name),
            typ: is_dir(name),
            hash: hash.to_vec(),
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.mode, self.name);
        [header.as_bytes(), &self.hash].concat()
    }
}

fn is_dir(path: &str) -> ObjectType {
    let path = Path::new(path);
    match path.is_dir() {
        true => ObjectType::Tree,
        false => ObjectType::Blob,
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:>06} {} {}    {}",
            self.mode,
            self.typ.to_string(),
            hex::encode(&self.hash),
            self.name,
        )
    }
}

impl Tree {
    pub fn new(files: Vec<File>) -> Self {
        Tree {
            files
        }
    }

    pub fn from(data: &[u8]) -> Option<Self> {
        // <mode> <name>\0<hash><mode> <name>\0<hash>....<mode> <name>\0<hash>
        let mut files: Vec<File> = Vec::new();
        let splitter_offsets: Vec<usize> = data.iter().enumerate()
                                .filter(|(_, &d)| d == b'\0' )
                                .map(|(off, _)| off )
                                .collect();
        let mut offsets: Vec<usize> = Vec::new();
        let mut prev = 0;
        for i in splitter_offsets {
            if i - prev >= 20 {
                // \0 in hash
                offsets.push(i)
            }
            prev = i;
        }
        let mut head = 0;
        for offset in offsets.iter() {
            let hdr = &data[head..*offset];
            let hash = &data[(*offset + 1)..(*offset + 21)];
            let file = File::from(hdr, hash)?;
            files.push(file);
            head = *offset + 21;
        }
        Some(Tree::new(files))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let content: Vec<u8> = self.files.iter().flat_map(|x| x.encode()).collect();
        let header = format!("{} {}\0", ObjectType::Tree.to_string(), content.len());

        [header.as_bytes(), content.as_slice()].concat()
    }

    pub fn calc_hash(&self) -> Vec<u8> {
        Vec::from(Sha1::digest(&self.as_bytes()).as_slice())
    }

    pub fn typ(&self) -> ObjectType {
        ObjectType::Tree
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            (&self.files)
                .into_iter()
                .map(|f| format!("{}", f))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

pub fn write_tree() -> io::Result<Tree> {
    let index = index::read_index(GIT_INDEX)?;
    let entries: Vec<File> = index.entries.iter()
                    .map(|e| File::new(e.mode as usize, &e.hash, &e.name, ObjectType::Blob))
                    .collect();
    Ok(Tree::new(entries))
}

#[cfg(test)]
mod tests {
    use super::File;
    use super::Tree;
    use crate::object::ObjectType; 

    const FILE: [u8; 41] = [
        0x31, 0x30, 0x30, 0x36, 0x34, 0x34, 0x20,
        0x2e, 0x64, 0x6f, 0x63, 0x6b, 0x65, 0x72, 0x69, 0x67, 0x6e, 0x6f, 0x72, 0x65, 0x00, 0x6b, 0x87,
        0x10, 0xa7, 0x11, 0xf3, 0xb6, 0x89, 0x88, 0x5a, 0xa5, 0xc2, 0x6c, 0x6c, 0x06, 0xbd, 0xe3, 0x48,
        0xe8, 0x2b,
    ];
    
    const DIR: [u8; 30] = [
        0x34, 0x30, 0x30, 0x30, 0x30, 0x20, 0x73, 0x72,
        0x63, 0x00, 0x8e, 0x4e, 0x40, 0x00, 0x55, 0x72, 0x22, 0x26, 0x71, 0xa3, 0xc3, 0xc7, 0xf9, 0xea,
        0xda, 0x8f, 0xdf, 0x6c, 0x96, 0xf8,
    ];

    // 0x74, 0x72, 0x65, 0x65, 0x20, 0x32, 0x36, 0x39, 0x00,
    const TREE: [u8; 269] = [
        0x31, 0x30, 0x30, 0x36, 0x34, 0x34, 0x20, 0x2e, 0x64, 0x6f, 0x63, 0x6b, 0x65, 0x72, 0x69, 0x67, 0x6e, 0x6f, 0x72, 0x65, 0x00, 
            0x6b, 0x87, 0x10, 0xa7, 0x11, 0xf3, 0xb6, 0x89, 0x88, 0x5a, 0xa5, 0xc2, 0x6c, 0x6c, 0x06, 0xbd, 0xe3, 0x48, 0xe8, 0x2b, 
        0x31, 0x30, 0x30, 0x36, 0x34, 0x34, 0x20, 0x2e, 0x67, 0x69, 0x74, 0x69, 0x67, 0x6e, 0x6f, 0x72, 0x65, 0x00, 
            0xe3, 0xda, 0xab, 0x27, 0x79, 0xd1, 0xd0, 0xdc, 0x3e, 0x77, 0x38, 0xb5, 0xdf, 0x94, 0xe4, 0x04, 0xa9, 0x57, 0x8a, 0xd1, 
        0x31, 0x30, 0x30, 0x36, 0x34, 0x34, 0x20, 0x43, 0x61, 0x72, 0x67, 0x6f, 0x2e, 0x6c, 0x6f, 0x63, 0x6b, 0x00, 
            0x5c, 0x81, 0x55, 0x5c, 0x8b, 0x0e, 0xc5, 0x3d, 0x7e, 0x1d, 0xab, 0xcd, 0x8f, 0x53, 0x8c, 0x5c, 0x9b, 0x8a, 0x57, 0x5c, 
        0x31, 0x30, 0x30, 0x36, 0x34, 0x34, 0x20, 0x43, 0x61, 0x72, 0x67, 0x6f, 0x2e, 0x74, 0x6f, 0x6d, 0x6c, 0x00,
            0xc7, 0x6a, 0x26, 0xce, 0xdf, 0xa6, 0x11, 0x0a, 0xc5, 0x6e, 0x3d, 0xd7, 0xef, 0x5f, 0xb5, 0xb4, 0x48, 0xa9, 0x04, 0xe3, 
        0x31, 0x30, 0x30, 0x36, 0x34, 0x34, 0x20, 0x44, 0x6f, 0x63, 0x6b, 0x65, 0x72, 0x66, 0x69, 0x6c, 0x65, 0x00, 
            0x0b, 0x4c, 0x5d, 0xcb, 0xc0, 0x9d, 0x28, 0x9b, 0xfb, 0x7a, 0x07, 0x16, 0x9a, 0x9c, 0xfe, 0x71, 0x5e, 0x93, 0x2b, 0x51, 
        0x31, 0x30, 0x30, 0x36, 0x34, 0x34, 0x20, 0x64, 0x6f, 0x63, 0x6b, 0x65, 0x72, 0x2d, 0x63, 0x6f, 0x6d, 0x70, 0x6f, 0x73, 0x65, 0x2e, 0x79, 0x6d, 0x6c, 0x00, 
            0xe3, 0xf7, 0x42, 0x3d, 0x01, 0xfe, 0xcc, 0xc4, 0xe6, 0x4b, 0xd8, 0xc2, 0x4b, 0xe4, 0x7e, 0xeb, 0x77, 0x3b, 0x53, 0x93, 
        0x34, 0x30, 0x30, 0x30, 0x30, 0x20, 0x73, 0x72, 0x63, 0x00, 
            0x8e, 0x4e, 0x40, /* \0 in hash value */0x00, 0x55, 0x72, 0x22, 0x26, 0x71, 0xa3, 0xc3, 0xc7, 0xf9, 0xea, 0xda, 0x8f, 0xdf, 0x6c, 0x96, 0xf8, 
    ];
    #[test]
    fn test_file_from() {
        let file = File::from(&FILE[0..20], &FILE[21..]).unwrap();
        assert_eq!(file.mode, 100644);
        assert_eq!(file.name, ".dockerignore");
        assert_eq!(file.typ, ObjectType::Blob);
        assert_eq!(hex::encode(file.hash), "6b8710a711f3b689885aa5c26c6c06bde348e82b");
    }
    #[test]
    fn test_file_fmt() {
        let file_str = "100644 blob 6b8710a711f3b689885aa5c26c6c06bde348e82b    .dockerignore";
        let file = File::from(&FILE[0..21], &FILE[21..]).unwrap();
        let res = format!("{}", file);
        assert_eq!(res, file_str);
    }
    #[test]
    fn test_tree_from() {
        let tree = Tree::from(&TREE).unwrap();
        assert_eq!(tree.files.len(), 7);
        assert_eq!(tree.files[0].name, String::from(".dockerignore"));
        assert_eq!(tree.files[6].name, String::from("src"));
        assert_eq!(tree.files[1].mode, 100644);
    }
//     #[test]
//     fn test_tree_fmt() {
//         let tree_str = "040000 tree 586adda20ea6368204ec255813d7540e0f50eae3    cmd
// 100644 blob 621c8b5649992e727e38edf8a70ea56b38ddae1b    main.rs
// 040000 tree 3b74be79d3c861a3e114c608debbd7bdc4518ba6    object";
//         let tree = Tree::from(tree_str.as_bytes()).unwrap();
//         let res = format!("{}", tree);
//         assert_eq!(res, tree_str);

//     }
}
