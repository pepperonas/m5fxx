//! Virtual SD Card storage with sandboxing preventing directory traversal attacks.

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SdCardStorage {
    root_path: PathBuf,
}

impl SdCardStorage {
    pub fn new<P: AsRef<Path>>(root: P) -> io::Result<Self> {
        let root_path = root.as_ref().to_path_buf();
        if !root_path.exists() {
            fs::create_dir_all(&root_path)?;
        }
        let canonical_root = root_path.canonicalize()?;
        Ok(Self {
            root_path: canonical_root,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root_path
    }

    /// Resolves a sandboxed relative virtual path within the root directory.
    /// Traversal attempts (e.g. "../" or absolute root escapes) return PermissionDenied.
    fn resolve_path(&self, rel_path: &str) -> io::Result<PathBuf> {
        // Strip leading slashes to treat as relative to SD root
        let clean = rel_path.trim_start_matches('/').trim_start_matches('\\');

        // Check for suspicious components
        for comp in Path::new(clean).components() {
            if let std::path::Component::ParentDir = comp {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Access denied: path traversal not permitted on SD card",
                ));
            }
        }

        let target = self.root_path.join(clean);

        // If target already exists, verify its canonical path stays inside root
        if target.exists() {
            let canonical_target = target.canonicalize()?;
            if !canonical_target.starts_with(&self.root_path) {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Access denied: sandbox escape detected",
                ));
            }
            Ok(canonical_target)
        } else {
            // For new files, verify the existing ancestor stays inside root
            let mut ancestor = target.parent();
            while let Some(anc) = ancestor {
                if anc.exists() {
                    let canonical_anc = anc.canonicalize()?;
                    if !canonical_anc.starts_with(&self.root_path) {
                        return Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "Access denied: sandbox escape detected in ancestor",
                        ));
                    }
                    break;
                }
                ancestor = anc.parent();
            }
            Ok(target)
        }
    }

    pub fn write_file(&self, path: &str, data: &[u8]) -> io::Result<()> {
        let full_path = self.resolve_path(path)?;
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(full_path)?;
        file.write_all(data)?;
        file.flush()?;
        Ok(())
    }

    pub fn read_file(&self, path: &str) -> io::Result<Vec<u8>> {
        let full_path = self.resolve_path(path)?;
        let mut file = File::open(full_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_to_string(&self, path: &str) -> io::Result<String> {
        let bytes = self.read_file(path)?;
        String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn file_exists(&self, path: &str) -> bool {
        match self.resolve_path(path) {
            Ok(p) => p.exists(),
            Err(_) => false,
        }
    }

    pub fn list_dir(&self, path: &str) -> io::Result<Vec<String>> {
        let full_path = self.resolve_path(path)?;
        let entries = fs::read_dir(full_path)?;
        let mut files = Vec::new();
        for entry in entries {
            let entry = entry?;
            if let Ok(name) = entry.file_name().into_string() {
                files.push(name);
            }
        }
        files.sort();
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sd_card_sandbox_and_traversal_prevention() {
        let temp_dir = std::env::temp_dir().join("m5fxx_test_sd");
        let sd = SdCardStorage::new(&temp_dir).expect("Create SD storage");

        // Allowed write and read
        sd.write_file("test.txt", b"hello m5fxx")
            .expect("write test.txt");
        assert!(sd.file_exists("test.txt"));
        let content = sd.read_to_string("test.txt").expect("read test.txt");
        assert_eq!(content, "hello m5fxx");

        // Subdirectory write
        sd.write_file("data/logs.csv", b"1,2,3")
            .expect("write subfile");
        assert!(sd.file_exists("data/logs.csv"));

        // Traversal attempt must fail
        assert!(sd.write_file("../escape.txt", b"fail").is_err());
        assert!(sd.write_file("data/../../escape.txt", b"fail").is_err());
        assert!(sd.read_file("../../etc/passwd").is_err());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
