#![crate_name = "sims_far"]

use std::fs::File;
use std::io::SeekFrom::Start;
use std::io::{Read, Seek};
use std::str::from_utf8;

/// The FAR format (.far files) are used to bundle (archive) multiple files together. All numeric
/// values in the header and manifest are stored in little-endian order(least significant byte
/// first).
pub struct Far {
    /// The signature is an eight-byte string, consisting literally of "FAR!byAZ" (without the
    /// quotes).
    pub signature: String,
    /// The version is always one.
    pub version: u32,
    /// The manifest offset is the byte offset from the beginning of the file to the manifest.
    /// The contents of the archived files are simply concatenated together without any other
    /// structure or padding.Caveat: all of the files observed have been a multiple of four in
    /// length, so it's possible that the files may be padded to a two-byte or four-byte boundary
    /// and the case has simply never been encountered.
    pub manifest_offset: u32,
    /// The manifest contains a count of the number of archived files, followed by an entry for
    /// each file. In all of the examples examined the order of the entries matches the order of
    /// the archived files, but whether this is a firm requirement or not is unknown.
    pub manifest: Manifest,
}

impl Far {
    /// Create a new instance of Far and parse it
    pub fn new(path: &str) -> Far {
        return parse_far(path);
    }
}

/// The manifest contains a count of the number of archived files, followed by an entry for each
/// file. In all of the examples examined the order of the entries matches the order of the archived
/// files, but whether this is a firm requirement or not is unknown.
pub struct Manifest {
    /// The number of files in the far file.
    pub number_of_files: u32,
    /// A list of Manifest Entries in the far file.
    pub manifest_entries: Vec<ManifestEntry>,
}

/// A manifest entry containing the first file length, second file length, file offset, file name
/// length, and file name.
pub struct ManifestEntry {
    file_path: String,
    /// The file length is stored twice. Perhaps this is because some variant of FAR files supports
    /// compressed data and the fields would hold the compressed and uncompressed sizes, but this is
    /// pure speculation. The safest thing to do is to leave the fields identical.
    pub file_length1: u32,
    /// The file length is stored twice. Perhaps this is because some variant of FAR files supports
    /// compressed data and the fields would hold the compressed and uncompressed sizes, but this is
    /// pure speculation. The safest thing to do is to leave the fields identical.
    pub file_length2: u32,
    /// The file offset is the byte offset from the beginning of the FAR file to the archived file.
    pub file_offset: u32,
    /// The filename length is the number of bytes in the filename. Filenames are stored without a
    /// terminating null. For example, the filename "foo" would have a filename length of three and
    /// the entry would be nineteen bytes long in total.
    pub file_name_length: u32,
    /// The name of the file. This can include directories.
    pub file_name: String,
}

impl ManifestEntry {
    pub fn get_bytes(&self) -> Vec<u8> {
        let mut f = File::open(self.file_path.as_str()).unwrap();
        let mut buf: Vec<u8> = vec![0x00; self.file_length1 as usize];
        f.seek(Start(self.file_offset as u64)).unwrap();
        f.read_exact(&mut *buf).unwrap();
        return buf;
    }
}

fn parse_far(path: &str) -> Far {
    let mut far = Far {
        signature: "".to_string(),
        version: 0,
        manifest_offset: 0,
        manifest: Manifest {
            number_of_files: 0,
            manifest_entries: vec![],
        },
    };

    let mut f = File::open(path).unwrap();

    // read signature
    let mut buf: [u8; 8] = [0x00; 8];
    f.read_exact(&mut buf).unwrap();
    far.signature = from_utf8(&buf).unwrap().to_string();

    // read version
    let mut buf: [u8; 4] = [0x00; 4];
    f.read_exact(&mut buf).unwrap();
    far.version = u32::from_le_bytes(buf.try_into().unwrap());

    // read manifest offset
    f.read_exact(&mut buf).unwrap();
    far.manifest_offset = u32::from_le_bytes(buf.try_into().unwrap());

    // read manifest
    f.seek(Start(far.manifest_offset as u64)).unwrap();
    f.read_exact(&mut buf).unwrap();
    far.manifest.number_of_files = u32::from_le_bytes(buf.try_into().unwrap());

    // read manifest entries
    for _ in 0..far.manifest.number_of_files {
        far.manifest
            .manifest_entries
            .push(parse_manifest_entry(&mut f, path));
    }

    return far;
}

fn parse_manifest_entry(f: &mut File, uigraphics_path: &str) -> ManifestEntry {
    let mut me = ManifestEntry {
        file_path: uigraphics_path.to_string(),
        file_length1: 0,
        file_length2: 0,
        file_offset: 0,
        file_name_length: 0,
        file_name: "".to_string(),
    };
    let mut buf: [u8; 4] = [0x00; 4];

    // read file length 1
    f.read_exact(&mut buf).unwrap();
    me.file_length1 = u32::from_le_bytes(buf.try_into().unwrap());

    // read file length 2
    f.read_exact(&mut buf).unwrap();
    me.file_length2 = u32::from_le_bytes(buf.try_into().unwrap());

    // read file offset
    f.read_exact(&mut buf).unwrap();
    me.file_offset = u32::from_le_bytes(buf.try_into().unwrap());

    // read file name length
    f.read_exact(&mut buf).unwrap();
    me.file_name_length = u32::from_le_bytes(buf.try_into().unwrap());

    // read file name
    let mut buf: Vec<u8> = vec![0x00; me.file_name_length as usize];
    f.read_exact(&mut buf).unwrap();
    me.file_name = from_utf8(&buf).unwrap().to_string();

    return me;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let path = r"test.far";
        let far = Far::new(path);
        assert_eq!(far.signature, "FAR!byAZ");
        assert_eq!(far.version, 1);
        assert_eq!(far.manifest_offset, 160);
        assert_eq!(far.manifest.number_of_files, 1);
        assert_eq!(far.manifest.manifest_entries[0].file_length1, 144);
        assert_eq!(far.manifest.manifest_entries[0].file_length2, 144);
        assert_eq!(far.manifest.manifest_entries[0].file_offset, 16);
        assert_eq!(far.manifest.manifest_entries[0].file_name_length, 8);
        assert_eq!(far.manifest.manifest_entries[0].file_name, "test.bmp");
    }

    #[test]
    fn test_get_bytes() {
        let path = r"test.far";
        let far = Far::new(path);
        assert_eq!(far.manifest.manifest_entries[0].get_bytes().len(), 144);
    }
}
