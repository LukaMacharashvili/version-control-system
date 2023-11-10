use std::fs;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub fn traverse_directory(path: Option<&Path>) -> Vec<PathBuf> {
    let path = match path {
        Some(path) => path,
        None => Path::new("."),
    };
    let mut result = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    result.push(entry_path.clone());
                } else if entry_path.is_dir() {
                    if entry_path.file_name().unwrap().to_str().unwrap() == ".history" {
                        continue;
                    }
                    let subdirectory_files = traverse_directory(Some(&entry_path));
                    result.extend(subdirectory_files);
                }
            }
        }
    }

    result
}

pub fn delete_contents_of_directory(path: &str) -> io::Result<()> {
    // Exclude .history directory
    let files = traverse_directory(Some(Path::new(path)));

    for file in files {
        fs::remove_file(file)?;
    }

    Ok(())
}

pub fn compare_strings(a: &str, b: &str) -> bool {
    a == b
}

pub fn read_part_of_file(file_path: &str, start: u64, length: usize) -> io::Result<String> {
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = vec![0; length];
    file.read_exact(&mut buffer)?;

    let contents = String::from_utf8(buffer).unwrap();

    Ok(contents)
}
