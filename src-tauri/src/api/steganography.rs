use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::api::security::{derive_master_key, encrypt_blob, decrypt_blob, generate_salt};
use crate::api::vault_error::{VaultError, VaultResult};

const STEGO_MAGIC: &[u8; 11] = b"VFDIR_STEGO";
const STEGO_VERSION: u8 = 1;
const SALT_LEN: usize = 16;
const FOOTER_SIZE: usize = 11 + 1 + SALT_LEN + 8; // Magic + Version + Salt + PayloadSize

/// Embeds an arbitrary file or directory into a host file.
///
/// # Arguments
/// * `host_path` - Path to the cover file (video, image, etc.)
/// * `source_path` - Path to the file or directory to hide
/// * `output_path` - Path where the resulting file will be saved
/// * `password` - Password to encrypt the steganography layer
pub fn embed_path(
    host_path: &Path,
    source_path: &Path,
    output_path: &Path,
    password: &str,
) -> VaultResult<()> {
    // 1. Prepare the payload (Tar + Gzip + Encrypt)
    let salt = generate_salt();
    let session = derive_master_key(password, &salt)?;
    
    // Create archive of the source path
    let tar_gz = create_tarball_generic(source_path)?;
    
    // Encrypt
    let encrypted_payload = encrypt_blob(&tar_gz, &session)?;
    let final_payload_size = encrypted_payload.len() as u64;

    // 2. Open input and output files
    let mut input = File::open(host_path).map_err(|e| VaultError::Io(e))?;
    let mut output = File::create(output_path).map_err(|e| VaultError::Io(e))?;

    // 3. Copy host file content
    io::copy(&mut input, &mut output).map_err(|e| VaultError::Io(e))?;

    // 4. Append payload
    output.write_all(&encrypted_payload).map_err(|e| VaultError::Io(e))?;

    // 5. Append Footer
    // Structure: [Magic 11] [Version 1] [Salt 16] [PayloadSize 8]
    output.write_all(STEGO_MAGIC).map_err(|e| VaultError::Io(e))?;
    output.write_u8(STEGO_VERSION).map_err(|e| VaultError::Io(e))?;
    output.write_all(&salt).map_err(|e| VaultError::Io(e))?;
    output.write_u64::<LittleEndian>(final_payload_size).map_err(|e| VaultError::Io(e))?;

    Ok(())
}

/// Embeds the vault into a host file (specialized wrapper for embed_path)
pub fn embed_vault(
    host_path: &Path,
    vault_root: &Path,
    output_path: &Path,
    password: &str,
) -> VaultResult<()> {
    // For vault, we want to archive specific files inside the root, not the root itself directly
    // to preserve structure expected by extract_vault. 
    // Actually, create_tarball_generic archives the *contents* if it's a dir, or the file itself?
    // The previous implementation added specific files. 
    // Let's keep the custom tarball creation for vault to ensure structure is exactly as expected.
    
    let salt = generate_salt();
    let session = derive_master_key(password, &salt)?;
    
    // Create archive specialized for vault structure
    let tar_gz = create_tarball_vault(vault_root)?;
    
    // Encrypt
    let encrypted_payload = encrypt_blob(&tar_gz, &session)?;
    let final_payload_size = encrypted_payload.len() as u64;

    // Open input and output files
    let mut input = File::open(host_path).map_err(|e| VaultError::Io(e))?;
    let mut output = File::create(output_path).map_err(|e| VaultError::Io(e))?;

    // Copy host file content
    io::copy(&mut input, &mut output).map_err(|e| VaultError::Io(e))?;

    // Append payload
    output.write_all(&encrypted_payload).map_err(|e| VaultError::Io(e))?;

    // Append Footer
    output.write_all(STEGO_MAGIC).map_err(|e| VaultError::Io(e))?;
    output.write_u8(STEGO_VERSION).map_err(|e| VaultError::Io(e))?;
    output.write_all(&salt).map_err(|e| VaultError::Io(e))?;
    output.write_u64::<LittleEndian>(final_payload_size).map_err(|e| VaultError::Io(e))?;

    Ok(())
}

/// Extracts the vault from a container file.
pub fn extract_vault(
    container_path: &Path,
    output_root: &Path,
    password: &str,
) -> VaultResult<()> {
    let mut file = File::open(container_path).map_err(|e| VaultError::Io(e))?;
    let file_len = file.metadata().map_err(|e| VaultError::Io(e))?.len();

    if file_len < FOOTER_SIZE as u64 {
        return Err(VaultError::InvalidData); // File too small
    }

    // 1. Read Footer
    file.seek(SeekFrom::End(-(FOOTER_SIZE as i64))).map_err(|e| VaultError::Io(e))?;
    
    let mut magic = [0u8; 11];
    file.read_exact(&mut magic).map_err(|e| VaultError::Io(e))?;
    
    if &magic != STEGO_MAGIC {
        return Err(VaultError::InvalidData); // Not a stego container
    }

    let version = file.read_u8().map_err(|e| VaultError::Io(e))?;
    if version != STEGO_VERSION {
        return Err(VaultError::InvalidData); // Unknown version
    }

    let mut salt = [0u8; SALT_LEN];
    file.read_exact(&mut salt).map_err(|e| VaultError::Io(e))?;

    let payload_size = file.read_u64::<LittleEndian>().map_err(|e| VaultError::Io(e))?;

    // 2. Read Payload
    let payload_start = file_len - (FOOTER_SIZE as u64) - payload_size;
    if payload_start > file_len { // Underflow check
        return Err(VaultError::InvalidData);
    }

    file.seek(SeekFrom::Start(payload_start)).map_err(|e| VaultError::Io(e))?;
    let mut payload = vec![0u8; payload_size as usize];
    file.read_exact(&mut payload).map_err(|e| VaultError::Io(e))?;

    // 3. Decrypt
    let session = derive_master_key(password, &salt)?;
    let decrypted_tar_gz = decrypt_blob(&payload, &session)?;

    // 4. Unpack
    unpack_tarball(&decrypted_tar_gz, output_root)?;

    Ok(())
}

/// Updates an existing steganography container with new content from source_path.
/// Safely handles the case where we want to overwrite the container.
pub fn update_container(
    container_path: &Path,
    source_path: &Path,
    password: &str,
) -> VaultResult<()> {
    // 1. Create a temporary file for the new container
    let temp_output = container_path.with_extension("tmp");
    
    // 2. Embed content into the temp file using the original container as host
    embed_path(container_path, source_path, &temp_output, password)?;
    
    // 3. Replace original container with temp file atomically-ish
    std::fs::rename(&temp_output, container_path).map_err(|e| VaultError::Io(e))?;
    
    Ok(())
}

fn create_tarball_vault(root_path: &Path) -> VaultResult<Vec<u8>> {
    let enc = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Add vault.meta
    let meta_path = root_path.join("vault.meta");
    if meta_path.exists() {
        tar.append_path_with_name(&meta_path, "vault.meta").map_err(|e| VaultError::Io(e))?;
    }

    // Add vault.bin
    let bin_path = root_path.join("vault.bin");
    if bin_path.exists() {
        tar.append_path_with_name(&bin_path, "vault.bin").map_err(|e| VaultError::Io(e))?;
    }

    // Add vault_data directory
    let data_path = root_path.join("vault_data");
    if data_path.exists() {
        tar.append_dir_all("vault_data", &data_path).map_err(|e| VaultError::Io(e))?;
    }

    tar.finish().map_err(|e| VaultError::Io(e))?;
    
    let enc = tar.into_inner().map_err(|e| VaultError::Io(e))?;
    enc.finish().map_err(|e| VaultError::Io(e))
}

fn create_tarball_generic(source_path: &Path) -> VaultResult<Vec<u8>> {
    let enc = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar = tar::Builder::new(enc);

    if source_path.is_dir() {
        tar.append_dir_all(".", source_path).map_err(|e| VaultError::Io(e))?;
    } else {
        let name = source_path.file_name()
            .ok_or(VaultError::InvalidPath)?;
        tar.append_path_with_name(source_path, name).map_err(|e| VaultError::Io(e))?;
    }

    tar.finish().map_err(|e| VaultError::Io(e))?;
    
    let enc = tar.into_inner().map_err(|e| VaultError::Io(e))?;
    enc.finish().map_err(|e| VaultError::Io(e))
}

fn unpack_tarball(data: &[u8], output_path: &Path) -> VaultResult<()> {
    let dec = GzDecoder::new(data);
    let mut tar = tar::Archive::new(dec);

    tar.unpack(output_path).map_err(|e| VaultError::Io(e))
}
