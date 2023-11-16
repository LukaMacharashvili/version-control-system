use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io;
use std::io::{Read, Write};
use std::io::{Result, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::thread;

use super::load_ignores;

pub fn write_to_data_file(path: &str, data: &str, exists: bool) -> Result<()> {
    if exists {
        let mut file = OpenOptions::new()
            .append(true)
            .open(path.to_owned() + "/data.bin")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    } else {
        let mut file = File::create(path.to_owned() + "/data.bin")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }
}

pub fn read_part_of_file(file_path: &str, start: u64, length: usize) -> Result<String> {
    let mut file = File::open(file_path)?;
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = vec![0; length];
    file.read_exact(&mut buffer)?;

    let contents = String::from_utf8(buffer).unwrap();

    Ok(contents)
}

pub fn traverse_directory(path: Option<&Path>, ignores: Option<&Vec<String>>) -> Vec<PathBuf> {
    let path = match path {
        Some(path) => path,
        None => Path::new("."),
    };
    let mut result = Vec::new();
    let binding_ignores = Vec::new();
    let ignores = match ignores {
        Some(ignores) => ignores,
        None => &binding_ignores,
    };

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let entry_name = entry_path.file_name().unwrap().to_str().unwrap();
                if ignores.contains(&entry_name.to_owned()) {
                    continue;
                }
                if entry_path.is_file() {
                    result.push(entry_path.clone());
                } else if entry_path.is_dir() {
                    if entry_path.file_name().unwrap().to_str().unwrap() == ".history" {
                        continue;
                    }
                    let subdirectory_files = traverse_directory(Some(&entry_path), Some(ignores));
                    result.extend(subdirectory_files);
                }
            }
        }
    }

    result
}

pub fn delete_contents_of_directory(path: &str, ignores: Option<&Vec<String>>) -> io::Result<()> {
    let binding_ignores = Vec::new();
    let ignores = match ignores {
        Some(ignores) => ignores,
        None => &binding_ignores,
    };
    let files = traverse_directory(Some(Path::new(path)), Some(ignores));
    let files_to_ignore = load_ignores();
    let mut join_handles = Vec::new();

    for entry_path in files {
        let files_to_ignore = files_to_ignore.clone();
        let handle = thread::spawn(move || {
            let entry_name = entry_path.file_name().unwrap().to_str().unwrap();
            if files_to_ignore.contains(&entry_name.to_owned()) {
                return;
            }
            if entry_path.is_file() {
                fs::remove_file(entry_path).unwrap();
            } else if entry_path.is_dir() {
                fs::remove_dir_all(entry_path).unwrap();
            }
        });

        join_handles.push(handle);
    }

    for handle in join_handles {
        handle.join().unwrap();
    }

    Ok(())
}
