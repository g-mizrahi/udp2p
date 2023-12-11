//! This library contains functions to manipulate files.
//! Its goal is to provide all utilities to extract data from files and prepare it to be exported to the REST server and sent over the network.

use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use std::{fmt, fs, io};

/// The nodes of the Merkle tree can be of three types :
/// - `chunk` are the leaf nodes and contain at most 1024 bytes
/// - `directory` represent the directories in the file system, they only hold children and no data
/// - `bigfile` represent files bigger than the chunk size, they have children and no data.
#[derive(Debug)]
pub enum MktFsNodeType {
    DIRECTORY,
    CHUNK,
    BIGFILE,
}

/// Holds a node of the Merkle tree representing the file system.
#[derive(Debug)]
pub struct MktFsNode {
    path: String,                     // mandatory
    ntype: MktFsNodeType,             // mandatory
    children: Option<Vec<MktFsNode>>, // optional (chunk no child)
    data: Option<Vec<u8>>,            // optional (dir no data)
    hash: [u8; 32],                   // mandatory
}

impl fmt::Display for MktFsNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Node[{:?}]({})", self.ntype, self.path);
        match &self.data {
            Some(d) => writeln!(f, "\tData : {:?}", d.iter().take(10).collect::<Vec<&u8>>()),
            _ => writeln!(f, "\tData : No data"),
        };
        writeln!(f, "\tHash : {:?}", &self.hash.iter().take(10).collect::<Vec<&u8>>());
        match &self.children {
            Some(c) => {
                write!(f, "\tChildren :");
                for s in c.iter() {
                    write!(f, "\n{}", s);
                }
            }
            _ => {
                write!(f, "\tChildren : No child");
            }
        };
        return Ok(());
    }
}

impl MktFsNode {
    fn try_from_bytes(
        path: impl Into<String> + Copy,
        data: impl Into<Vec<u8>>,
        chunk_size: usize,
        max_children: usize,
    ) -> Result<MktFsNode> {
        // If there is data, the chunk_size cannot be 0
        if chunk_size == 0 {
            bail!("Cannot build a node with chunk_size 0.");
        }

        let data = data.into();

        // If the data is small enough to fit in a single chunk
        if data.len() <= chunk_size {
            return (Ok(MktFsNode {
                path: path.into(),
                ntype: MktFsNodeType::CHUNK,
                children: None,
                hash: hash_bytes_prefix(&data, 0),
                data: Some(data),
            }));
        }
        // If it cannot fit we have to split the data in children nodes
        else {
            if max_children == 0 {
                bail!("Cannot build a big file node with 0 child.");
            }

            // Compute the adequate number of children for optimal packing
            let n_chunks = data.len().div_ceil(chunk_size);
            let mut n_layers = n_chunks.ilog(max_children); // ilog is rounded down
            if n_chunks == max_children.pow(n_layers) {
                n_layers -= 1;
            }

            // Generate children nodes
            let children = data
                .chunks(chunk_size * max_children.pow(n_layers))
                .map(|d| MktFsNode::try_from_bytes(path, d, chunk_size, max_children).unwrap())
                .collect::<Vec<MktFsNode>>();
            // Compute hash
            let mut hasher = Sha256::new();
            for c in children.iter() {
                // In practice, the DIRECTORY is never reached
                match c.ntype {
                    MktFsNodeType::CHUNK => hasher.update(hash_bytes_prefix(&c.hash, 0)),
                    MktFsNodeType::BIGFILE => hasher.update(hash_bytes_prefix(&c.hash, 1)),
                    MktFsNodeType::DIRECTORY => hasher.update(hash_bytes_prefix(&c.hash, 2)),
                }
            }
            let mut hash = <[u8; 32]>::default();
            hash.copy_from_slice(hasher.finalize().as_slice());

            // Generate root node
            return (Ok(MktFsNode {
                path: path.into(),
                ntype: MktFsNodeType::BIGFILE,
                children: Some(children),
                hash: hash,
                data: None,
            }));
        }
    }
}

// fn read_file(path: &str) -> Result<Vec<u8>, io::Error> {
//     let content = fs::read(path)?;
//     Ok(content)
// }

// fn split_file(file_bytes: Vec<u8>, chunk_size: usize) -> Vec<Vec<u8>> {
//     return file_bytes
//         .chunks(chunk_size)
//         .map(|x| x.to_owned())
//         .collect::<Vec<Vec<u8>>>();
// }

fn hash_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let mut hash = <[u8; 32]>::default();
    hash.copy_from_slice(hasher.finalize().as_slice());
    return hash;
}

fn hash_bytes_prefix(bytes: &[u8], prefix: u8) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([prefix]);
    hasher.update(bytes);
    let mut hash = <[u8; 32]>::default();
    hash.copy_from_slice(hasher.finalize().as_slice());
    return hash;
}

fn hash_bytes_array(bytes_array: Vec<Vec<u8>>) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for bytes in bytes_array.iter() {
        hasher.update(hash_bytes(bytes));
    }
    let mut hash = <[u8; 32]>::default();
    hash.copy_from_slice(hasher.finalize().as_slice());
    return hash;
}

#[cfg(test)]
mod tests {

    use super::*;
    use hex;

    // #[test]
    // fn lib_file_read_file() {
    //     let path = "Cargo.toml";
    //     let content = read_file(&path).unwrap();
    //     // println!("File content : {:?}", content);
    // }

    // #[test]
    // fn lib_file_split_file() {
    //     let path = "Cargo.toml";
    //     let content = read_file(&path).unwrap();
    //     let chunks = split_file(content, 8usize);
    //     // for chunk in chunks.iter() {
    //     //     println!("Chunk : {:?}", chunk);
    //     // }
    // }

    #[test]
    fn lib_file_hash_bytes() {
        let hash = hash_bytes(b"hello");
        assert_eq!(
            hash,
            hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
                .unwrap()[0..32]
        );
    }

    #[test]
    fn lib_file_hash_array() {
        let hash = hash_bytes_array(Vec::from([b"hello".to_vec(), b"world".to_vec()]));
        assert_eq!(
            hash,
            hex::decode("7305db9b2abccd706c256db3d97e5ff48d677cfe4d3a5904afb7da0e3950e1e2")
                .unwrap()[0..32]
        );
    }

    #[test]
    fn lib_file_node_from_small_chunk() {
        let data = b"abc";
        let node = MktFsNode::try_from_bytes("/root", data, 4, 2).unwrap();
        println!("{node}");
    }

    #[test]
    fn lib_file_node_from_big_chunk() {
        //           | 4 | 4 | 4 | 4 |2|
        //           |       |       | |
        //           |               | |
        //           |                 |
        let data = b"abcdefghijklmabcde";
        let node = MktFsNode::try_from_bytes("/root", data, 4, 2).unwrap();
        println!("{node}");
    }
}
