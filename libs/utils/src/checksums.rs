use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diff {
    Missing,
    Mismatch,
    Extra,
}

/// Calculates SHA-256 hash of data
pub fn calculate_sha256(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

/// SHA256SUMS file representation
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckSums(HashMap<PathBuf, String>);

impl CheckSums {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Add file content with hash calculation, normalizing path
    ///
    /// This method calculates the SHA-256 hash of the provided content
    /// and stores it with the normalized path (forward slashes).
    pub fn raw_add<P: AsRef<Path>>(&mut self, content: &[u8], path: P) {
        let hash = calculate_sha256(content);
        let normalized = PathBuf::from(path.as_ref().to_string_lossy().replace("\\", "/"));
        self.0.insert(normalized, hash);
    }

    /// Add file by reading, hashing, and normalizing path
    pub fn add<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        let content =
            std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        self.raw_add(&content, path);
        Ok(())
    }

    /// Parse SHA256SUMS content: `<hash>  <filepath>`
    pub fn parse(content: &[u8]) -> Result<Self, String> {
        let content = std::str::from_utf8(content).map_err(|_| "SHA256SUMS is not valid UTF-8")?;
        let mut checksums = Self::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, "  ").collect();
            if parts.len() != 2 {
                return Err(format!(
                    "Invalid format at line {}: expected '<hash>  <filepath>'",
                    line_num + 1
                ));
            }

            let hash = parts[0].trim();
            let filepath = parts[1].trim();

            if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!(
                    "Invalid hash at line {}: expected 64 hex characters",
                    line_num + 1
                ));
            }

            let normalized = PathBuf::from(filepath.replace("\\", "/"));
            checksums.0.insert(normalized, hash.to_lowercase());
        }

        if checksums.is_empty() {
            return Err("Empty or invalid SHA256SUMS".to_string());
        }

        Ok(checksums)
    }

    /// Write checksums to file
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = self.to_plain_text();
        std::fs::write(path.as_ref(), content)
            .map_err(|e| format!("Failed to write {}: {}", path.as_ref().display(), e))
    }

    /// Convert to SHA256SUMS string format
    pub fn to_plain_text(&self) -> String {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by_key(|(path, _)| path.to_string_lossy().to_string());

        let mut output = String::new();
        for (path, hash) in entries {
            let path_str = path.to_string_lossy().replace("\\", "/");
            output.push_str(&format!("{}  {}\n", hash, path_str));
        }
        output
    }

    /// Compare with another CheckSums
    pub fn compare(&self, other: &Self) -> Vec<(Diff, PathBuf)> {
        let mut diffs = Vec::new();

        for (path, hash) in &self.0 {
            match other.0.get(path) {
                None => diffs.push((Diff::Missing, path.clone())),
                Some(other_hash) if hash != other_hash => {
                    diffs.push((Diff::Mismatch, path.clone()))
                }
                _ => {}
            }
        }

        for path in other.0.keys() {
            if !self.0.contains_key(path) {
                diffs.push((Diff::Extra, path.clone()));
            }
        }

        diffs
    }

    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&String> {
        self.0.get(path.as_ref())
    }

    pub fn contains<P: AsRef<Path>>(&self, path: P) -> bool {
        self.0.contains_key(path.as_ref())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sha256() {
        assert_eq!(
            calculate_sha256(b"hello world"),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_checksums_parse() {
        let content = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2  file.txt";
        let checksums = CheckSums::parse(content.as_bytes()).unwrap();
        assert_eq!(checksums.len(), 1);
        assert!(checksums.contains("file.txt"));
    }

    #[test]
    fn test_checksums_compare() {
        let mut c1 = CheckSums::new();
        c1.0.insert(PathBuf::from("a.txt"), "hash1".to_string());

        let mut c2 = CheckSums::new();
        c2.0.insert(PathBuf::from("a.txt"), "hash2".to_string());

        let diffs = c1.compare(&c2);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].0, Diff::Mismatch);
        assert_eq!(diffs[0].1, PathBuf::from("a.txt"));
    }
}
