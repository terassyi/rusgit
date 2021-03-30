
use std::str;
use std::fmt;

use crate::object::{Object, ObjectType};
use crate::cmd::cat_file::{hash_key_to_path, file_to_object};

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
        // let obj_type = iter.next()
                        // .and_then(|d| ObjectType::from(d))?;
        let path = hash_key_to_path(&hex::encode(hash));
        let obj = file_to_object(&path).ok()?; // high overhead...
        let name = iter.next()?;
        Some(File {
            mode,
            name: String::from(name),
            typ: obj.typ(),
            hash: hash.to_vec(),
        })
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
        let mut iter = data.split(|&d| d == b'\0');
        let files: Vec<File> = Vec::new();
        let mut hdr = iter.next()?;
        let files = iter.try_fold(files, |mut acc, x| {
            let (hash, nxt_hdr) = x.split_at(20);
            let file = File::from(hdr, hash)?;
            acc.push(file);
            hdr = nxt_hdr;
            Some(acc)
        })?;
        Some(Tree { files })
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

#[cfg(test)]
mod tests {
    // use super::File;
    use super::Tree;
    use crate::object::ObjectType; 
    #[test]
    // fn test_file_from() {
        // let file_str = "040000 tree 586adda20ea6368204ec255813d7540e0f50eae3    cmd";
        // let file = File::from(file_str.as_bytes()).unwrap();
        // assert_eq!(file.mode, 40000);
        // assert_eq!(file.name, "cmd");
        // assert_eq!(file.typ, ObjectType::Tree);
        // assert_eq!(&file.hash, "586adda20ea6368204ec255813d7540e0f50eae3");
    // }
    // #[test]
    // fn test_file_fmt() {
        // let file_str = "040000 tree 586adda20ea6368204ec255813d7540e0f50eae3    cmd";
        // let file = File::from(file_str.as_bytes()).unwrap();
        // let res = format!("{}", file);
        // assert_eq!(res, file_str);
    // }
    #[test]
    fn test_tree_from() {
        let tree_str = "040000 tree 586adda20ea6368204ec255813d7540e0f50eae3    cmd
100644 blob 621c8b5649992e727e38edf8a70ea56b38ddae1b    main.rs
040000 tree 3b74be79d3c861a3e114c608debbd7bdc4518ba6    object";
        let tree = Tree::from(tree_str.as_bytes()).unwrap();
        assert_eq!(tree.files.len(), 3);
        assert_eq!(tree.files[0].name, String::from("cmd"));
        assert_eq!(tree.files[1].mode, 100644);
    }
    #[test]
    fn test_tree_fmt() {
        let tree_str = "040000 tree 586adda20ea6368204ec255813d7540e0f50eae3    cmd
100644 blob 621c8b5649992e727e38edf8a70ea56b38ddae1b    main.rs
040000 tree 3b74be79d3c861a3e114c608debbd7bdc4518ba6    object";
        let tree = Tree::from(tree_str.as_bytes()).unwrap();
        let res = format!("{}", tree);
        assert_eq!(res, tree_str);

    }
}
