use sha2::{Digest, Sha256};
use std::{fs, io};

fn read_file(path: &str) -> Result<Vec<u8>, io::Error> {
    let content = fs::read(path)?;
    Ok(content)
}

fn split_file(file_bytes: Vec<u8>, chunk_size: usize) -> Vec<Vec<u8>> {
    return file_bytes
        .chunks(chunk_size)
        .map(|x| x.to_owned())
        .collect::<Vec<Vec<u8>>>();
}

fn hash_bytes(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let mut hash = <[u8; 32]>::default();
    hash.copy_from_slice(hasher.finalize().as_slice());
    return hash;
}

fn hash_bytes_array(bytes_array: Vec<impl AsRef<[u8]>>) -> [u8; 32] {
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

    #[test]
    fn lib_file_read_file() -> Result<(), io::Error> {
        let path = "Cargo.toml";
        let content = read_file(&path)?;
        // println!("{:?}", content);
        Ok(())
    }

    #[test]
    fn lib_file_split_file() -> Result<(), io::Error> {
        let path = "Cargo.toml";
        let content = read_file(&path)?;
        let chunks = split_file(content, 8usize);
        // for chunk in chunks.iter() {
        //     println!("{:?}", chunk);
        // }
        Ok(())
    }

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
        let hash = hash_bytes_array(Vec::from([b"hello", b"world"]));
        assert_eq!(
            hash,
            hex::decode("7305db9b2abccd706c256db3d97e5ff48d677cfe4d3a5904afb7da0e3950e1e2")
                .unwrap()[0..32]
        );
    }
}
