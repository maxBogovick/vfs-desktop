use std::fs::File;
use std::path::{Path, PathBuf};
use crate::core::FileSystemEntry;
use std::io::Write;

#[allow(dead_code)]
pub fn is_archive(path: &str) -> bool {
    let path_obj = Path::new(path);
    if let Some(ext) = path_obj.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        return match ext_str.as_str() {
            "zip" | "tar" | "gz" | "tgz" => true,
            _ => false,
        };
    }
    false
}

pub fn list_archive_contents(path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let path_obj = Path::new(path);
    if let Some(ext) = path_obj.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            "zip" => list_zip(path),
            "tar" => list_tar(path),
            "gz" | "tgz" => list_tar_gz(path),
            _ => Err(format!("Unsupported archive format: {}", ext_str)),
        }
    } else {
        Err("No extension found".to_string())
    }
}

pub fn extract_archive(archive_path: &str, destination_path: &str) -> Result<(), String> {
    let path_obj = Path::new(archive_path);
    if let Some(ext) = path_obj.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            "zip" => extract_zip(archive_path, destination_path),
            "tar" => extract_tar(archive_path, destination_path),
            "gz" | "tgz" => extract_tar_gz(archive_path, destination_path),
            _ => Err(format!("Unsupported archive format: {}", ext_str)),
        }
    } else {
        Err("No extension found".to_string())
    }
}

pub fn create_archive(source_paths: Vec<String>, destination_path: String) -> Result<(), String> {
    if source_paths.is_empty() {
        return Err("No source files specified".to_string());
    }

    let dest_path = Path::new(&destination_path);
    let ext = dest_path.extension()
        .and_then(|e| e.to_str())
        .ok_or("Destination file has no extension")?
        .to_lowercase();

    // Check for double extension for tar.gz
    let is_tar_gz = destination_path.ends_with(".tar.gz") || destination_path.ends_with(".tgz");

    if is_tar_gz {
        create_tar_gz(&source_paths, &destination_path)
    } else {
        match ext.as_str() {
            "zip" => create_zip(&source_paths, &destination_path),
            "tar" => create_tar(&source_paths, &destination_path),
            _ => Err(format!("Unsupported archive format: {}", ext)),
        }
    }
}

// === ZIP Implementation ===

fn list_zip(path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut entries = Vec::new();

    for i in 0..archive.len() {
        let file = match archive.by_index(i) {
            Ok(file) => file,
            Err(_) => continue,
        };
        
        let name = file.name().replace("\\", "/"); 
        let is_dir = file.is_dir() || name.ends_with('/');
        let size = file.size();
        let modified = Some(0);

        entries.push(FileSystemEntry {
            path: format!("{}/{}", path, name),
            name,
            is_dir,
            is_file: !is_dir,
            size: Some(size),
            modified,
            created: None,
            accessed: None,
        });
    }

    Ok(entries)
}

fn extract_zip(archive_path: &str, destination_path: &str) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    
    std::fs::create_dir_all(destination_path).map_err(|e| e.to_string())?;
    archive.extract(destination_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn create_zip(source_paths: &[String], destination_path: &str) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for src_path in source_paths {
        let path = Path::new(src_path);
        if !path.exists() {
            continue;
        }

        let file_name = path.file_name() 
            .ok_or("Invalid file name")? 
            .to_string_lossy();

        if path.is_dir() {
            // Recursively add directory
            let walker = walkdir::WalkDir::new(path);
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                // Calculate relative path from the source item's parent directory
                // If adding /a/b/folder, and folder contains item.txt, 
                // we want "folder/item.txt" inside zip
                let relative_path = entry_path.strip_prefix(path.parent().unwrap_or(path))
                    .map_err(|e| e.to_string())?;
                
                let name_str = relative_path.to_string_lossy().replace("\\", "/");

                if entry_path.is_dir() {
                    zip.add_directory(&name_str, options).map_err(|e| e.to_string())?;
                } else {
                    zip.start_file(&name_str, options).map_err(|e| e.to_string())?;
                    let mut f = File::open(entry_path).map_err(|e| e.to_string())?;
                    std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                }
            }
        } else {
            // Add single file
            zip.start_file(&file_name, options).map_err(|e| e.to_string())?;
            let mut f = File::open(path).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

// === TAR Implementation ===

fn list_tar(path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(file);
    let mut entries = Vec::new();

    for file in archive.entries().map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let header = file.header();
        let path_str = file.path().map_err(|e| e.to_string())?.to_string_lossy().to_string();
        let is_dir = header.entry_type().is_dir();
        let size = header.size().unwrap_or(0);
        let modified = header.mtime().unwrap_or(0);

        entries.push(FileSystemEntry {
            path: format!("{}/{}", path, path_str),
            name: path_str,
            is_dir,
            is_file: !is_dir,
            size: Some(size),
            modified: Some(modified),
            created: None,
            accessed: None,
        });
    }
    Ok(entries)
}

fn extract_tar(archive_path: &str, destination_path: &str) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(destination_path).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(file);
    archive.unpack(destination_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn create_tar(source_paths: &[String], destination_path: &str) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|e| e.to_string())?;
    let mut archive = tar::Builder::new(file);

    for src_path in source_paths {
        let path = Path::new(src_path);
        if !path.exists() { continue; }

        let file_name = path.file_name() 
            .ok_or("Invalid file name")?;

        if path.is_dir() {
            archive.append_dir_all(&file_name, path).map_err(|e| e.to_string())?;
        } else {
            let mut f = File::open(path).map_err(|e| e.to_string())?;
            archive.append_file(&file_name, &mut f).map_err(|e| e.to_string())?;
        }
    }

    archive.finish().map_err(|e| e.to_string())?;
    Ok(())
}

// === TAR.GZ Implementation ===

fn list_tar_gz(path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let tar = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(tar);
    let mut entries = Vec::new();

    for file in archive.entries().map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let header = file.header();
        let path_str = file.path().map_err(|e| e.to_string())?.to_string_lossy().to_string();
        let is_dir = header.entry_type().is_dir();
        let size = header.size().unwrap_or(0);
        let modified = header.mtime().unwrap_or(0);

        entries.push(FileSystemEntry {
            path: format!("{}/{}", path, path_str),
            name: path_str,
            is_dir,
            is_file: !is_dir,
            size: Some(size),
            modified: Some(modified),
            created: None,
            accessed: None,
        });
    }
    Ok(entries)
}

fn extract_tar_gz(archive_path: &str, destination_path: &str) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(destination_path).map_err(|e| e.to_string())?;
    let tar = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(destination_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn create_tar_gz(source_paths: &[String], destination_path: &str) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|e| e.to_string())?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut archive = tar::Builder::new(enc);

    for src_path in source_paths {
        let path = Path::new(src_path);
        if !path.exists() { continue; }

        let file_name = path.file_name() 
            .ok_or("Invalid file name")?;

        if path.is_dir() {
            archive.append_dir_all(&file_name, path).map_err(|e| e.to_string())?;
        } else {
            let mut f = File::open(path).map_err(|e| e.to_string())?;
            archive.append_file(&file_name, &mut f).map_err(|e| e.to_string())?;
        }
    }

    archive.finish().map_err(|e| e.to_string())?;
    Ok(())
}