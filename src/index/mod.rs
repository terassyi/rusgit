use std::fmt;
use chrono::{DateTime, TimeZone, Utc};
use sha1::{Sha1, Digest};

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
pub struct Index {
    pub entries: Vec<Entry>,
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
        let mode = hex_to_num(&data[24..28]);
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
        write!(f, "{} {} 0\t{}", num_to_mode(self.mode), hex::encode(&self.hash), self.name)
        // write!(f, "{} {} 0       {}", num_to_mode(self.mode), hex::encode(&self.hash), self.name)
    }

}
impl Index {
    pub fn new(entries: Vec<Entry>) -> Index {
        Index {
            entries,
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
        Some(Index::new(entries))
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let hdr = [*b"DIRC", [0x00, 0x00, 0x00, 0x02], (self.entries.len() as u32).to_be_bytes()].concat();
        let entries = self.entries.iter()
                        .flat_map(|d| d.as_bytes())
                        .collect::<Vec<u8>>();
        let content = [hdr, entries].concat();
        let hash = Vec::from(Sha1::digest(&content).as_slice());
        [content, hash].concat()
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries.iter()
            .try_for_each(|e| write!(f, "{}\n", e))
    }
}

fn hex_to_num(data: &[u8]) -> u32 {
    data.iter().rev().fold((0u32, 1u32), |(sum, offset), &d| {
        (sum + (d as u32 * offset), offset << 8)
    }).0
}

fn num_to_mode(mode: u32) -> String {
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

#[cfg(test)]
mod tests {
    use super::Entry;
    use super::Index;
    #[test]
    fn test_hex_to_num() {
        assert_eq!(super::hex_to_num(&[0x00, 0x00, 0x60, 0x63]), 0x6063);
    }
    #[test]
    fn test_num_to_mode() {
        assert_eq!(super::num_to_mode(33188), String::from("100644"));
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
        assert_eq!(super::num_to_mode(entry.mode), "100644");
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
        let entry_str = "100644 eb305475bb51923200531ac7fef46d4b987b253c 0       .gitignore";
        let res = format!("{}", entry);
        assert_eq!(&res, entry_str);
    }
    const index_data: [u8; 1114] = [
        0x44, 0x49, 0x52, 0x43, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x0c, 0x60, 0x5e, 0xf0, 0xa5,
        0x08, 0x7a, 0x51, 0xf3, 0x60, 0x5e, 0xf0, 0xa5, 0x08, 0x7a, 0x51, 0xf3, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x18, 0xb3, 0xb9, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x00, 0x10, 0xeb, 0x30, 0x54, 0x75, 0xbb, 0x51, 0x92, 0x32, 0x00, 0x53, 0x1a, 0xc7,
        0xfe, 0xf4, 0x6d, 0x4b, 0x98, 0x7b, 0x25, 0x3c, 0x00, 0x0a, 0x2e, 0x67, 0x69, 0x74, 0x69, 0x67,
        0x6e, 0x6f, 0x72, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x63, 0x13, 0xd4,
        0x11, 0x42, 0x4d, 0xac, 0x60, 0x63, 0x13, 0xd4, 0x11, 0x42, 0x4d, 0xac, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x18, 0xb4, 0x01, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x1d, 0xfb, 0x5c, 0x81, 0x55, 0x5c, 0x8b, 0x0e, 0xc5, 0x3d, 0x7e, 0x1d, 0xab, 0xcd,
        0x8f, 0x53, 0x8c, 0x5c, 0x9b, 0x8a, 0x57, 0x5c, 0x00, 0x0a, 0x43, 0x61, 0x72, 0x67, 0x6f, 0x2e,
        0x6c, 0x6f, 0x63, 0x6b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x63, 0x13, 0xce,
        0x0e, 0xdd, 0xf2, 0xa0, 0x60, 0x63, 0x13, 0xce, 0x0e, 0xdd, 0xf2, 0xa0, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x18, 0xb3, 0xba, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x01, 0x2f, 0xc7, 0x6a, 0x26, 0xce, 0xdf, 0xa6, 0x11, 0x0a, 0xc5, 0x6e, 0x3d, 0xd7,
        0xef, 0x5f, 0xb5, 0xb4, 0x48, 0xa9, 0x04, 0xe3, 0x00, 0x0a, 0x43, 0x61, 0x72, 0x67, 0x6f, 0x2e,
        0x74, 0x6f, 0x6d, 0x6c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x62, 0x91, 0xa9,
        0x2c, 0xf2, 0xf2, 0x61, 0x60, 0x62, 0x91, 0xa9, 0x2c, 0xf2, 0xf2, 0x61, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x19, 0x89, 0x61, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x09, 0x4c, 0x62, 0xb8, 0xb9, 0x08, 0xd2, 0x8a, 0x3f, 0x9e, 0x3e, 0x85, 0xdd, 0x82,
        0x7a, 0xf2, 0x0e, 0xae, 0xc2, 0x37, 0x06, 0x0e, 0x00, 0x13, 0x73, 0x72, 0x63, 0x2f, 0x63, 0x6d,
        0x64, 0x2f, 0x63, 0x61, 0x74, 0x5f, 0x66, 0x69, 0x6c, 0x65, 0x2e, 0x72, 0x73, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x60, 0x63, 0x51, 0xd6, 0x37, 0x10, 0x5a, 0xb1, 0x60, 0x63, 0x51, 0xd6,
        0x37, 0x10, 0x5a, 0xb1, 0x01, 0x00, 0x00, 0x04, 0x03, 0x1c, 0x79, 0x08, 0x00, 0x00, 0x81, 0xa4,
        0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x02, 0x0e, 0x05, 0xca, 0x50, 0x36,
        0x78, 0xdb, 0x12, 0xb0, 0x33, 0xfb, 0x1b, 0xbb, 0xf7, 0x3c, 0xff, 0x74, 0x46, 0x43, 0xdd, 0x4a,
        0x00, 0x16, 0x73, 0x72, 0x63, 0x2f, 0x63, 0x6d, 0x64, 0x2f, 0x68, 0x61, 0x73, 0x68, 0x5f, 0x6f,
        0x62, 0x6a, 0x65, 0x63, 0x74, 0x2e, 0x72, 0x73, 0x00, 0x00, 0x00, 0x00, 0x60, 0x5f, 0x16, 0x33,
        0x15, 0x7a, 0x97, 0x93, 0x60, 0x5f, 0x16, 0x33, 0x15, 0x7a, 0x97, 0x93, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x19, 0x0d, 0x55, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x02, 0x0b, 0x0e, 0xdb, 0x71, 0x73, 0x6c, 0xc9, 0xb8, 0xcb, 0xbe, 0xe1, 0xb5, 0xa0,
        0xaa, 0xe9, 0x7e, 0x4f, 0x2a, 0x2f, 0x5c, 0x1f, 0x00, 0x0f, 0x73, 0x72, 0x63, 0x2f, 0x63, 0x6d,
        0x64, 0x2f, 0x69, 0x6e, 0x69, 0x74, 0x2e, 0x72, 0x73, 0x00, 0x00, 0x00, 0x60, 0x63, 0x3d, 0xcc,
        0x2a, 0x03, 0x88, 0xde, 0x60, 0x63, 0x3d, 0xcc, 0x2a, 0x03, 0x88, 0xde, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x19, 0x23, 0x4a, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x00, 0xf8, 0xed, 0x56, 0x00, 0x57, 0x1c, 0x5a, 0x4b, 0x7a, 0xdd, 0x67, 0xe4, 0xe4,
        0xa9, 0x1e, 0xaa, 0x48, 0x0c, 0x9c, 0xdb, 0x8d, 0x00, 0x0e, 0x73, 0x72, 0x63, 0x2f, 0x63, 0x6d,
        0x64, 0x2f, 0x6d, 0x6f, 0x64, 0x2e, 0x72, 0x73, 0x00, 0x00, 0x00, 0x00, 0x60, 0x70, 0x1f, 0x8b,
        0x16, 0x43, 0x72, 0xf9, 0x60, 0x70, 0x1f, 0x8b, 0x16, 0x43, 0x72, 0xf9, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x18, 0xb3, 0xbc, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x0c, 0x60, 0xe5, 0x35, 0x66, 0x81, 0x48, 0xe4, 0x3b, 0x4c, 0x64, 0x0a, 0xb5, 0xed,
        0x6e, 0xd3, 0xd3, 0xd9, 0x16, 0x3d, 0x73, 0x7a, 0x00, 0x0b, 0x73, 0x72, 0x63, 0x2f, 0x6d, 0x61,
        0x69, 0x6e, 0x2e, 0x72, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x63, 0x38, 0xed,
        0x06, 0x17, 0x51, 0x14, 0x60, 0x63, 0x38, 0xed, 0x06, 0x17, 0x51, 0x14, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x1a, 0x39, 0x13, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x04, 0x15, 0x2d, 0x02, 0xba, 0x98, 0x66, 0x96, 0x1b, 0xb9, 0x85, 0x9c, 0xdb, 0xe4,
        0x5a, 0xf1, 0x04, 0x93, 0xf5, 0x74, 0xa9, 0x14, 0x00, 0x12, 0x73, 0x72, 0x63, 0x2f, 0x6f, 0x62,
        0x6a, 0x65, 0x63, 0x74, 0x2f, 0x62, 0x6c, 0x6f, 0x62, 0x2e, 0x72, 0x73, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x60, 0x63, 0x39, 0x00, 0x0b, 0x78, 0x37, 0x46, 0x60, 0x63, 0x39, 0x00,
        0x0b, 0x78, 0x37, 0x46, 0x01, 0x00, 0x00, 0x04, 0x03, 0x1a, 0x39, 0x1c, 0x00, 0x00, 0x81, 0xa4,
        0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x14, 0xdf, 0x49, 0x42, 0xc7, 0xef,
        0x13, 0xc1, 0x66, 0x5f, 0x32, 0x50, 0xd5, 0xd6, 0xb9, 0x02, 0x06, 0xce, 0xf4, 0xb4, 0x9b, 0x3d,
        0x00, 0x14, 0x73, 0x72, 0x63, 0x2f, 0x6f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x2f, 0x63, 0x6f, 0x6d,
        0x6d, 0x69, 0x74, 0x2e, 0x72, 0x73, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x60, 0x63, 0x4d, 0xdd,
        0x31, 0xb0, 0x7d, 0xbe, 0x60, 0x63, 0x4d, 0xdd, 0x31, 0xb0, 0x7d, 0xbe, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x19, 0xb2, 0xbb, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x0d, 0x50, 0x67, 0xb6, 0x32, 0x47, 0x69, 0x08, 0x2c, 0xd4, 0x40, 0x52, 0x89, 0xa2,
        0x0e, 0xd3, 0xf5, 0xf3, 0x51, 0x80, 0x73, 0xdc, 0x00, 0x11, 0x73, 0x72, 0x63, 0x2f, 0x6f, 0x62,
        0x6a, 0x65, 0x63, 0x74, 0x2f, 0x6d, 0x6f, 0x64, 0x2e, 0x72, 0x73, 0x00, 0x60, 0x63, 0x3a, 0x26,
        0x10, 0xb7, 0x5c, 0x4c, 0x60, 0x63, 0x3a, 0x26, 0x10, 0xb7, 0x5c, 0x4c, 0x01, 0x00, 0x00, 0x04,
        0x03, 0x1a, 0x39, 0x1e, 0x00, 0x00, 0x81, 0xa4, 0x00, 0x00, 0x01, 0xf5, 0x00, 0x00, 0x00, 0x14,
        0x00, 0x00, 0x12, 0x7c, 0xb3, 0xf2, 0xaa, 0x86, 0x3f, 0x4e, 0xe2, 0xe6, 0x2f, 0xad, 0x03, 0xf3,
        0xb3, 0x9e, 0xeb, 0x8a, 0x08, 0x9b, 0xec, 0xff, 0x00, 0x12, 0x73, 0x72, 0x63, 0x2f, 0x6f, 0x62,
        0x6a, 0x65, 0x63, 0x74, 0x2f, 0x74, 0x72, 0x65, 0x65, 0x2e, 0x72, 0x73, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x54, 0x52, 0x45, 0x45, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x2d, 0x31, 0x20,
        0x31, 0x0a, 0x73, 0x72, 0x63, 0x00, 0x2d, 0x31, 0x20, 0x32, 0x0a, 0x63, 0x6d, 0x64, 0x00, 0x34,
        0x20, 0x30, 0x0a, 0x89, 0x3e, 0xed, 0x7b, 0xa4, 0xe3, 0xd4, 0x04, 0xc2, 0xa8, 0x3d, 0xdf, 0x3a,
        0xca, 0x42, 0xe9, 0xe0, 0x19, 0xe7, 0x30, 0x6f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x00, 0x34, 0x20,
        0x30, 0x0a, 0x7a, 0xec, 0xc2, 0x29, 0xb9, 0x5b, 0x4c, 0xb4, 0x02, 0x0c, 0x4b, 0x23, 0x71, 0x6a,
        0x5b, 0x97, 0xad, 0x64, 0xc5, 0x07, 0x64, 0x9f, 0x7d, 0xd4, 0x5d, 0xdd, 0xbc, 0xf5, 0x3d, 0x68,
        0xbc, 0x16, 0xff, 0xad, 0xc3, 0xb0, 0x64, 0x32, 0xe0, 0x46,
    ];
    #[test]
    fn test_index_from() {
        let index = Index::from(&index_data).unwrap();
        let entry_size = 12;
        assert_eq!(index.entries.len(), entry_size);
        assert_eq!(index.entries[0].name, ".gitignore");
        assert_eq!(index.entries[11].name, "src/object/tree.rs");
    }
}
