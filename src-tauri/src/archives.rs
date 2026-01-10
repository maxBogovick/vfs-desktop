use std::fs::File;
use std::path::{Path, PathBuf};
use crate::core::FileSystemEntry;
use std::io::{Write, Read, Seek, Cursor};
use crate::api_service::API;

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
    list_archive_contents_with_fs(path, None)
}

pub fn list_archive_contents_with_fs(path: &str, panel_fs: Option<&str>) -> Result<Vec<FileSystemEntry>, String> {
    let is_real = panel_fs.is_none() || panel_fs == Some("real");
    
    if is_real {
        let path_obj = Path::new(path);
        if let Some(ext) = path_obj.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return match ext_str.as_str() {
                "zip" => list_zip(path),
                "tar" => list_tar(path),
                "gz" | "tgz" => list_tar_gz(path),
                _ => Err(format!("Unsupported archive format: {}", ext_str)),
            };
        } else {
            return Err("No extension found".to_string());
        }
    }

    // Generic implementation (In-Memory)
    let archive_bytes = API.files.read_file_bytes(path, panel_fs)
        .map_err(|e| format!("Failed to read archive: {}", e))?;
    
    let cursor = Cursor::new(archive_bytes);
    
    let path_obj = Path::new(path);
    if let Some(ext) = path_obj.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            "zip" => list_zip_generic(cursor, path),
            "tar" => list_tar_generic(cursor, path),
            "gz" | "tgz" => list_tar_gz_generic(cursor, path),
            _ => Err(format!("Unsupported archive format for virtual fs: {}", ext_str)),
        }
    } else {
        Err("No extension found".to_string())
    }
}

pub fn extract_archive(archive_path: &str, destination_path: &str) -> Result<(), String> {
    extract_archive_with_fs(archive_path, destination_path, None, None)
}

pub fn extract_archive_with_fs(archive_path: &str, destination_path: &str, source_fs: Option<&str>, dest_fs: Option<&str>) -> Result<(), String> {
    let is_real_source = source_fs.is_none() || source_fs == Some("real");
    let is_real_dest = dest_fs.is_none() || dest_fs == Some("real");

    if is_real_source && is_real_dest {
        // Use optimized real-fs implementation
        let path_obj = Path::new(archive_path);
        if let Some(ext) = path_obj.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return match ext_str.as_str() {
                "zip" => extract_zip(archive_path, destination_path),
                "tar" => extract_tar(archive_path, destination_path),
                "gz" | "tgz" => extract_tar_gz(archive_path, destination_path),
                _ => Err(format!("Unsupported archive format: {}", ext_str)),
            };
        } else {
            return Err("No extension found".to_string());
        }
    }

    // Generic implementation (In-Memory)
    let archive_bytes = API.files.read_file_bytes(archive_path, source_fs)
        .map_err(|e| format!("Failed to read archive: {}", e))?;
    
    let cursor = Cursor::new(archive_bytes);
    
    let path_obj = Path::new(archive_path);
    if let Some(ext) = path_obj.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            "zip" => extract_zip_generic(cursor, destination_path, dest_fs),
            "tar" => extract_tar_generic(cursor, destination_path, dest_fs),
            "gz" | "tgz" => extract_tar_gz_generic(cursor, destination_path, dest_fs),
            _ => Err(format!("Unsupported archive format for virtual fs: {}", ext_str)),
        }
    } else {
        Err("No extension found".to_string())
    }
}

pub fn create_archive(source_paths: Vec<String>, destination_path: String) -> Result<(), String> {
    create_archive_with_fs(source_paths, destination_path, None, None)
}

pub fn create_archive_with_fs(source_paths: Vec<String>, destination_path: String, source_fs: Option<&str>, dest_fs: Option<&str>) -> Result<(), String> {
    if source_paths.is_empty() {
        return Err("No source files specified".to_string());
    }

    let is_real_source = source_fs.is_none() || source_fs == Some("real");
    let is_real_dest = dest_fs.is_none() || dest_fs == Some("real");

    let dest_path = Path::new(&destination_path);
    let ext = dest_path.extension()
        .and_then(|e| e.to_str())
        .ok_or("Destination file has no extension")?;
    let is_tar_gz = destination_path.ends_with(".tar.gz") || destination_path.ends_with(".tgz");

    if is_real_source && is_real_dest {
        if is_tar_gz {
            create_tar_gz(&source_paths, &destination_path)
        } else {
            match ext.to_lowercase().as_str() {
                "zip" => create_zip(&source_paths, &destination_path),
                "tar" => create_tar(&source_paths, &destination_path),
                _ => Err(format!("Unsupported archive format: {}", ext)),
            }
        }
    } else {
        // Generic implementation
        let buffer = Cursor::new(Vec::new());
        let result_buffer = if is_tar_gz {
            create_tar_gz_generic(buffer, &source_paths, source_fs)?
        } else {
            match ext.to_lowercase().as_str() {
                "zip" => create_zip_generic(buffer, &source_paths, source_fs)?,
                "tar" => create_tar_generic(buffer, &source_paths, source_fs)?,
                _ => return Err(format!("Unsupported archive format: {}", ext)),
            }
        };

        let bytes = result_buffer.into_inner();
        API.files.write_file_bytes(&destination_path, &bytes, dest_fs)
            .map_err(|e| format!("Failed to write archive: {}", e))?;
        Ok(())
    }
}

// === ZIP Implementation ===

fn list_zip(path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    list_zip_generic(file, path)
}

fn list_zip_generic<R: Read + Seek>(reader: R, archive_path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;
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
            path: format!("{}/{}", archive_path, name),
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

fn extract_zip_generic<R: Read + Seek>(reader: R, destination_path: &str, dest_fs: Option<&str>) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;
    
    // Ensure destination directory exists (if empty archive or just root)
    if let Err(_) = API.files.create_folder(Path::new(destination_path).parent().unwrap_or(Path::new("")).to_string_lossy().as_ref(), Path::new(destination_path).file_name().unwrap_or_default().to_string_lossy().as_ref(), dest_fs) {
         // Ignore if root already exists or is top level, but generally we try to create
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().replace("\\", "/");
        
        // Construct full path logic is tricky with VFS string paths.
        // Assume destination_path is a directory.
        let dest_path = if destination_path.ends_with('/') {
            format!("{}{}", destination_path, name)
        } else {
            format!("{}/{}", destination_path, name)
        };

        if file.is_dir() || name.ends_with('/') {
            // Create directory
            // We need to split path to parent and name for create_folder
            let p = Path::new(&dest_path);
            if let Some(parent) = p.parent() {
                if let Some(fname) = p.file_name() {
                    let _ = API.files.create_folder(&parent.to_string_lossy(), &fname.to_string_lossy(), dest_fs);
                }
            }
        } else {
            // Read content
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            
            // Ensure parent exists
            let p = Path::new(&dest_path);
            if let Some(parent) = p.parent() {
                 // Try to create parent structure if missing? 
                 // Virtual FS might handle this or fail. 
                 // Real implementation uses create_dir_all.
                 // Here we assume folders appear in zip before files or VFS handles missing parents (it usually doesn't).
                 // TODO: recursively create parents if needed. 
            }

            API.files.write_file_bytes(&dest_path, &buffer, dest_fs)
                .map_err(|e| format!("Failed to write file {}: {}", name, e))?;
        }
    }
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

fn create_zip_generic<W: Write + Seek>(writer: W, source_paths: &[String], source_fs: Option<&str>) -> Result<W, String> {
    let mut zip = zip::ZipWriter::new(writer);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for src_path in source_paths {
        // Recursive helper
        add_to_zip_recursive(&mut zip, src_path, src_path, options, source_fs)?;
    }

    let writer = zip.finish().map_err(|e| e.to_string())?;
    Ok(writer)
}

fn add_to_zip_recursive<W: Write + Seek>(
    zip: &mut zip::ZipWriter<W>, 
    current_path: &str, 
    root_path: &str,
    options: zip::write::SimpleFileOptions,
    source_fs: Option<&str>
) -> Result<(), String> {
    let info = API.files.get_file_info(current_path, source_fs)
        .map_err(|e| e.to_string())?;

    let root_parent = Path::new(root_path).parent().unwrap_or(Path::new(""));
    let relative_path = Path::new(current_path).strip_prefix(root_parent)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .replace("\\", "/");

    if info.is_dir {
        zip.add_directory(&relative_path, options).map_err(|e| e.to_string())?;
        let children = API.files.list_directory(current_path, source_fs)
            .map_err(|e| e.to_string())?;
        for child in children {
            add_to_zip_recursive(zip, &child.path, root_path, options, source_fs)?;
        }
    } else {
        zip.start_file(&relative_path, options).map_err(|e| e.to_string())?;
        let content = API.files.read_file_bytes(current_path, source_fs)
            .map_err(|e| e.to_string())?;
        zip.write_all(&content).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// === TAR Implementation ===

fn list_tar(path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    list_tar_generic(file, path)
}

fn list_tar_generic<R: Read>(reader: R, archive_path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let mut archive = tar::Archive::new(reader);
    let mut entries = Vec::new();

    for file in archive.entries().map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let header = file.header();
        let path_str = file.path().map_err(|e| e.to_string())?.to_string_lossy().to_string();
        let is_dir = header.entry_type().is_dir();
        let size = header.size().unwrap_or(0);
        let modified = header.mtime().unwrap_or(0);

        entries.push(FileSystemEntry {
            path: format!("{}/{}", archive_path, path_str),
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

fn extract_tar_generic<R: Read>(reader: R, destination_path: &str, dest_fs: Option<&str>) -> Result<(), String> {
    let mut archive = tar::Archive::new(reader);
    
    for file in archive.entries().map_err(|e| e.to_string())? {
        let mut file = file.map_err(|e| e.to_string())?;
        let path = file.path().map_err(|e| e.to_string())?.to_string_lossy().to_string();
        
        let dest_path = if destination_path.ends_with('/') {
            format!("{}{}", destination_path, path)
        } else {
            format!("{}/{}", destination_path, path)
        };

        if file.header().entry_type().is_dir() {
             let p = Path::new(&dest_path);
             if let Some(parent) = p.parent() {
                 if let Some(fname) = p.file_name() {
                     let _ = API.files.create_folder(&parent.to_string_lossy(), &fname.to_string_lossy(), dest_fs);
                 }
             }
        } else {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            API.files.write_file_bytes(&dest_path, &buffer, dest_fs)
                .map_err(|e| format!("Failed to write file: {}", e))?;
        }
    }
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

fn create_tar_generic<W: Write>(writer: W, source_paths: &[String], source_fs: Option<&str>) -> Result<W, String> {
    let mut archive = tar::Builder::new(writer);

    for src_path in source_paths {
        add_to_tar_recursive(&mut archive, src_path, src_path, source_fs)?;
    }

    let writer = archive.into_inner().map_err(|e| e.to_string())?;
    Ok(writer)
}

fn add_to_tar_recursive<W: Write>(
    archive: &mut tar::Builder<W>,
    current_path: &str,
    root_path: &str,
    source_fs: Option<&str>
) -> Result<(), String> {
    let info = API.files.get_file_info(current_path, source_fs)
        .map_err(|e| e.to_string())?;

    let root_parent = Path::new(root_path).parent().unwrap_or(Path::new(""));
    let relative_path = Path::new(current_path).strip_prefix(root_parent)
        .map_err(|e| e.to_string())?;

    if info.is_dir {
        archive.append_dir(&relative_path, ".").map_err(|e| e.to_string())?;
        let children = API.files.list_directory(current_path, source_fs)
            .map_err(|e| e.to_string())?;
        for child in children {
            add_to_tar_recursive(archive, &child.path, root_path, source_fs)?;
        }
    } else {
        let content = API.files.read_file_bytes(current_path, source_fs)
            .map_err(|e| e.to_string())?;
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        
        archive.append_data(&mut header, &relative_path, &content[..])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// === TAR.GZ Implementation ===

fn list_tar_gz(path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    list_tar_gz_generic(file, path)
}

fn list_tar_gz_generic<R: Read>(reader: R, archive_path: &str) -> Result<Vec<FileSystemEntry>, String> {
    let tar = flate2::read::GzDecoder::new(reader);
    list_tar_generic(tar, archive_path)
}

fn extract_tar_gz(archive_path: &str, destination_path: &str) -> Result<(), String> {
    let file = File::open(archive_path).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(destination_path).map_err(|e| e.to_string())?;
    let tar = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(destination_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn extract_tar_gz_generic<R: Read>(reader: R, destination_path: &str, dest_fs: Option<&str>) -> Result<(), String> {
    let tar = flate2::read::GzDecoder::new(reader);
    extract_tar_generic(tar, destination_path, dest_fs)
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

fn create_tar_gz_generic<W: Write>(writer: W, source_paths: &[String], source_fs: Option<&str>) -> Result<W, String> {
    let enc = flate2::write::GzEncoder::new(writer, flate2::Compression::default());
    let enc_finished = create_tar_generic(enc, source_paths, source_fs)?;
    let writer = enc_finished.finish().map_err(|e| e.to_string())?;
    Ok(writer)
}
