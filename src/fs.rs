use super::io::get_findmnt_output;

use super::io::{create_efi_boot_entry, log_info};
use libbtrfs::subvolume;

use super::utils;
use rand::{thread_rng, RngCore};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env::consts::ARCH;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::{self, Read, Write};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A structure representing an item subject to rollback actions.
///
/// This structure is used to encapsulate the path of a file that may need to be
/// restored to a previous state or cleaned up as part of a rollback process.
///
/// # Fields
///
/// * `original_path`: The `PathBuf` representing the path to the original file.
pub(crate) struct RollbackItem {
    original_path: PathBuf,
}

impl RollbackItem {
    /// Creates a new `RollbackItem` with the specified original file path.
    ///
    /// # Arguments
    ///
    /// * `original_path` - A `PathBuf` indicating the path to the original file that might be subject to rollback.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `RollbackItem`.
    pub(crate) fn new(original_path: PathBuf) -> Self {
        RollbackItem { original_path }
    }
    /// Performs cleanup actions for the rollback item.
    ///
    /// If a backup file (with a `.bak` extension) exists, this function will attempt to restore the original file from the backup.
    /// If no backup is found but the original file exists, the original file will be removed.
    /// If neither the original file nor its backup exists, an informational message will be logged.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if cleanup actions are completed successfully, or an `IoError` if any file operations fail.
    pub(crate) fn cleanup(&self) -> std::io::Result<()> {
        let backup_path = self.original_path.with_extension("bak");
        if backup_path.exists() {
            fs::rename(&backup_path, &self.original_path).expect("Failed to restore from backup");
            let message = format!("restored {}", self.original_path.display());
            log_info(&message, 1);
        } else {
            if self.original_path.exists() {
                fs::remove_file(&self.original_path).expect("Failed to remove original file");
            } else {
                let message = format!(
                    "The following file doesn't exist and couldn't be removed: '{}'",
                    self.original_path.display()
                );
                log_info(&message, 1);
            }
        }
        Ok(())
    }
}

/// Creates a new temporary directory using the `tempfile` crate.
///
/// This function is typically used to create a temporary workspace for operations that require filesystem changes
/// which should not affect the permanent storage.
///
/// # Returns
///
/// Returns a tuple containing the `TempDir` object representing the temporary directory and its `PathBuf`.
/// The `TempDir` object ensures that the directory is deleted when it goes out of scope.
pub(crate) fn create_temp_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create a temporary directory");
    let temp_dir_path = temp_dir.path().to_path_buf();
    (temp_dir, temp_dir_path)
}

/// Cleans up a list of `RollbackItem`s.
///
/// Iterates over each `RollbackItem` in the provided slice and performs cleanup actions.
/// If any cleanup action fails, an error message will be logged.
///
/// # Arguments
///
/// * `rollback_items` - A slice of `RollbackItem`s to be cleaned up.
pub(crate) fn cleanup_rollback_items(rollback_items: &[RollbackItem]) {
    for item in rollback_items {
        if let Err(e) = item.cleanup() {
            let message = format!("Error cleaning up item: {}", e);
            log_info(&message, 0);
        }
    }
}

/// Resets the state of a list of `RollbackItem`s by removing any associated backup files.
///
/// For each item in the list, if a backup file exists, it will be removed. After processing all items,
/// the list will be cleared, indicating that no rollback items remain to be processed.
///
/// # Arguments
///
/// * `rollback_items` - A mutable reference to a `Vec<RollbackItem>` representing the list of items to be reset.
pub(crate) fn reset_rollback_items(rollback_items: &mut Vec<RollbackItem>) {
    for item in rollback_items.iter() {
        let backup_path = item.original_path.with_extension("bak");
        if backup_path.exists() {
            if let Err(e) = fs::remove_file(&backup_path) {
                let message = format!(
                    "Failed to remove backup file {}: {}",
                    backup_path.display(),
                    e
                );
                log_info(&message, 0);
            } else {
                let message = format!("Removed backup file {}", backup_path.display());
                log_info(&message, 1)
            }
        }
    }
    rollback_items.clear();
}

/// Determines if the system is operating in a transactional mode by checking the filesystem type of `/etc`.
///
/// Transactional environments typically mount `/etc` with `overlayfs`, allowing changes to be atomic and reversible.
///
/// # Arguments
///
/// * `prefix` - An optional string slice that provides a path prefix, allowing checks in a modified filesystem structure, such as during testing or in a chroot environment.
///
/// # Returns
///
/// - `Ok(true)` if `/etc` is mounted with `overlayfs`, indicating a transactional system.
/// - `Ok(false)` if `/etc` is not mounted with `overlayfs`.
/// - `Err(String)` if an error occurs during the check, with an explanatory message.
///
/// # Errors
///
/// An error is returned if reading the mount information fails or if the expected mount information cannot be parsed.
pub(crate) fn is_transactional(prefix: Option<&str>) -> Result<bool, String> {
    let mounts_file_path = match prefix {
        Some(prefix) => PathBuf::from(prefix)
            .join("proc/mounts")
            .to_string_lossy()
            .into_owned(),
        None => "/proc/mounts".to_string(),
    };
    let mounts_file = File::open(mounts_file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(mounts_file);

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 2 {
            let mount_point = parts[1];
            let fs_type = parts[2];

            if mount_point == "/etc" {
                return Ok(fs_type == "overlayfs");
            }
        }
    }

    Ok(false)
}

/// Retrieves detailed information about the root Btrfs snapshot.
///
/// This function is designed to parse the system's root directory to extract information critical for managing Btrfs snapshots.
/// It identifies the prefix path leading up to the snapshot, the snapshot ID, and the full path of the snapshot itself.
///
/// # Arguments
///
/// * `override_prefix` - An optional override path. If provided, the function returns predefined values without performing actual parsing.
///
/// # Returns
///
/// A `Result` containing a tuple of:
/// - A `String` representing the prefix path leading up to the `.snapshots` directory.
/// - A `u64` representing the numeric ID of the snapshot.
/// - A `String` representing the full path of the snapshot.
///
/// # Errors
///
/// Returns an `Err` with a boxed error if:
/// - The snapshot path does not conform to the expected structure.
/// - Parsing of any component (prefix, snapshot ID, full path) fails.
pub(crate) fn get_root_snapshot_info(
    override_prefix: Option<&Path>,
) -> Result<(String, u64, String), Box<dyn std::error::Error>> {
    if override_prefix.is_some() {
        return Ok((
            "/.snapshots".to_string(),
            0,
            override_prefix.unwrap().to_string_lossy().to_string(),
        ));
    }
    let full_path = subvolume::get_subvol_path("/")?;
    let parts: Vec<&str> = full_path.split("/.snapshots/").collect();
    let prefix = parts.get(0).ok_or("Prefix not found")?.to_string();
    let snapshot_part = parts.get(1).ok_or("Snapshot part not found")?;
    let snapshot_id_str = snapshot_part
        .split('/')
        .next()
        .ok_or("Snapshot ID not found")?;
    let snapshot_id = snapshot_id_str.parse::<u64>()?;

    Ok((prefix, snapshot_id, full_path))
}

/// Determines if a specified Btrfs subvolume is set to read-only mode.
///
/// This function checks the read-only status of a given Btrfs subvolume.
///
/// # Parameters
///
/// - `subvol`: An `Option<String>` representing the path to the subvolume. If `None`, the function
///   assumes there's no subvolume to check and returns `Ok(false)`.
///
/// # Returns
///
/// - `Ok(true)`: If the subvolume is found and is set to read-only.
/// - `Ok(false)`: If the subvolume is not set to read-only or if `None` is passed as the subvolume path.
/// - `Err(e)`: If there's an error checking the read-only status of the subvolume, wrapped in a `Box<dyn Error>`.
///   The error includes a custom message indicating the failure to retrieve the read-only status, along with
///   the original error message.
pub(crate) fn is_subvol_ro(subvol: Option<String>) -> Result<bool, Box<dyn std::error::Error>> {
    match subvol {
        Some(subvol) => {
            let is_ro = subvolume::get_readonly(&subvol).map_err(|e| {
                Box::<dyn std::error::Error>::from(format!(
                    "Failed to get readonly status for subvolume '{}': {}",
                    subvol, e
                ))
            })?;
            Ok(is_ro)
        }
        None => Ok(false),
    }
}

/// Finds the path to the systemd-boot EFI file based on a given snapshot and firmware architecture,
/// with an optional prefix to override the default path for testing or other purposes.
///
/// # Arguments
///
/// * `snapshot` - A numeric identifier for the snapshot directory.
/// * `firmware_arch` - The architecture of the firmware, used to construct the EFI file name.
/// * `prefix_override` - An optional path to override the default "/.snapshots" prefix.
///
/// # Returns
///
/// Returns the `PathBuf` pointing to the systemd-boot EFI file.
pub(crate) fn find_sdboot(
    snapshot: Option<u64>,
    firmware_arch: &str,
    prefix_override: Option<&Path>,
) -> PathBuf {
    let base_prefix = match prefix_override {
        Some(override_path) => override_path.to_path_buf(),
        None => Path::new("/").to_path_buf(),
    };
    let prefix = match snapshot {
        Some(snap) => base_prefix
            .join(".snapshots")
            .join(snap.to_string())
            .join("snapshot"),
        None => base_prefix,
    };

    let mut sdboot_path = prefix.join(format!(
        "usr/lib/systemd-boot/systemd-boot{}.efi",
        firmware_arch
    ));

    if !sdboot_path.exists() {
        sdboot_path = prefix.join(format!(
            "usr/lib/systemd/boot/efi/systemd-boot{}.efi",
            firmware_arch
        ));
    }

    sdboot_path
}

/// Finds the path to the GRUB2 EFI file based on a given snapshot.
///
/// This function constructs a path within a specified snapshot directory to locate the GRUB2 EFI file.
/// It tries the primary expected location first and falls back to a secondary location if the EFI file is not found.
/// An optional override prefix can be provided for testing purposes.
///
/// # Arguments
///
/// * `snapshot` - A numeric identifier for the snapshot directory.
/// * `override_prefix` - An optional override prefix for the search, used primarily for testing.
///
/// # Returns
///
/// Returns the `PathBuf` pointing to the GRUB2 EFI file, whether it's in the primary or fallback location.
pub(crate) fn find_grub2(snapshot: Option<u64>, prefix_override: Option<&Path>) -> PathBuf {
    let base_prefix = match prefix_override {
        Some(override_path) => override_path.to_path_buf(),
        None => Path::new("/").to_path_buf(),
    };
    let prefix = match snapshot {
        Some(snap) => base_prefix
            .join(".snapshots")
            .join(snap.to_string())
            .join("snapshot"),
        None => base_prefix,
    };
    let mut grub2_path = prefix.join(format!("usr/share/efi/{}/grub.efi", ARCH));

    if !grub2_path.exists() {
        grub2_path = prefix.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    }
    grub2_path
}

/// Determines if the systemd-boot bootloader is installed for a given snapshot and firmware architecture.
///
/// This function checks for the presence of a systemd-boot EFI file and the absence of a GRUB2 EFI file
/// to determine if systemd-boot is the installed bootloader.
///
/// # Arguments
///
/// * `snapshot` - A numeric identifier for the snapshot directory.
/// * `firmware_arch` - The architecture of the firmware, used to check for the systemd-boot EFI file.
///
/// # Returns
///
/// Returns `true` if the systemd-boot EFI file exists and the GRUB2 EFI file does not, indicating systemd-boot is installed.
pub(crate) fn is_sdboot(
    snapshot: Option<u64>,
    firmware_arch: &str,
    override_prefix: Option<&Path>,
) -> bool {
    let sdboot = find_sdboot(snapshot, firmware_arch, override_prefix);
    let grub2 = find_grub2(snapshot, override_prefix);

    sdboot.exists() && !grub2.exists()
}

/// Determines if the GRUB2 bootloader is installed for a given snapshot.
///
/// This function checks for the presence of a GRUB2 EFI file to determine if GRUB2 is the installed bootloader.
///
/// # Arguments
///
/// * `snapshot` - A numeric identifier for the snapshot directory.
///
/// # Returns
///
/// Returns `true` if the GRUB2 EFI file exists, indicating GRUB2 is installed.
pub(crate) fn is_grub2(snapshot: Option<u64>, override_prefix: Option<&Path>) -> bool {
    find_grub2(snapshot, override_prefix).exists()
}

/// Determines the boot destination path based on the installed bootloader for a given snapshot.
///
/// This function checks whether the systemd-boot or GRUB2 bootloader is installed for
/// the specified snapshot and firmware architecture. It returns the appropriate boot destination path
/// based on the bootloader detected. The function supports overriding the default search prefix through an optional parameter.
///
/// # Arguments
///
/// * `snapshot` - A numeric identifier for the snapshot directory. This is used to locate the snapshot-specific bootloader files.
/// * `firmware_arch` - The architecture of the firmware, such as "x64" or "arm64".
/// This is used to refine the search for the bootloader files.
/// * `override_prefix` - An optional path override. If provided, this path will be used as the base directory
/// for searching bootloader files, instead of the default path.
///
/// # Returns
///
/// Returns `Ok("/EFI/systemd")` if systemd-boot is detected as the installed bootloader, `Ok("/EFI/opensuse")` if GRUB2 is detected,
/// or an `Err` with a message indicating that the bootloader is unsupported or could not be determined.
pub(crate) fn determine_boot_dst(
    snapshot: Option<u64>,
    firmware_arch: &str,
    override_prefix: Option<&Path>,
) -> Result<&'static str, &'static str> {
    if is_sdboot(snapshot, firmware_arch, override_prefix) {
        Ok("/EFI/systemd")
    } else if is_grub2(snapshot, override_prefix) {
        Ok("/EFI/opensuse")
    } else {
        Err("Unsupported bootloader or unable to determine bootloader")
    }
}

/// Finds the installed bootloader (systemd-boot or GRUB2) for a given snapshot and firmware architecture.
///
/// This function attempts to determine which bootloader is installed by checking for the presence of systemd-boot and GRUB2 EFI files.
/// It favors systemd-boot unless only GRUB2 is found.
///
/// # Arguments
///
/// * `snapshot` - A numeric identifier for the snapshot directory.
/// * `firmware_arch` - The architecture of the firmware, used in the search for the systemd-boot EFI file.
///
/// # Returns
///
/// Returns a `Result` containing a `PathBuf` to the detected bootloader EFI file on success,
/// or an error string if no bootloader is detected.
pub(crate) fn find_bootloader(
    snapshot: Option<u64>,
    firmware_arch: &str,
    override_prefix: Option<&Path>,
) -> Result<PathBuf, &'static str> {
    if is_sdboot(snapshot, firmware_arch, override_prefix) {
        Ok(find_sdboot(snapshot, firmware_arch, override_prefix))
    } else if is_grub2(snapshot, override_prefix) {
        Ok(find_grub2(snapshot, override_prefix))
    } else {
        Err("Bootloader not detected")
    }
}

/// Extracts a version string from binary content based on start and end patterns.
///
/// This function searches the given binary content for a sequence that starts with the specified
/// `start_pattern` and ends with the `end_pattern`. It extracts and returns the bytes found between these two patterns
/// as a UTF-8 string. The function is useful for parsing version information from binary files, such as firmware images
/// or compiled executables, where version strings are embedded within the binary data.
///
/// # Arguments
///
/// * `content` - A slice of bytes representing the binary content to be searched.
/// * `start_pattern` - A slice of bytes representing the pattern that marks the beginning of the version string.
/// * `end_pattern` - A slice of bytes representing the pattern that marks the end of the version string.
///
/// # Returns
///
/// Returns an `Option<String>`:
/// - `Some(String)` containing the extracted version string if both start and end patterns are found
/// and the content between them is valid UTF-8.
/// - `None` if either pattern is not found, or if the content between the patterns is not valid UTF-8.
pub(crate) fn find_version(
    content: &[u8],
    start_pattern: &[u8],
    end_pattern: &[u8],
) -> Option<String> {
    if let Some(start_pos) = content
        .windows(start_pattern.len())
        .position(|window| window == start_pattern)
    {
        let version_start_pos = start_pos + start_pattern.len();
        if let Some(end_pos) = content[version_start_pos..]
            .windows(end_pattern.len())
            .position(|window| window == end_pattern)
        {
            let version_bytes = &content[version_start_pos..version_start_pos + end_pos];
            return std::str::from_utf8(version_bytes).ok().map(str::to_string);
        }
    }
    None
}

/// Determines the version of the installed bootloader by analyzing the binary content of the bootloader file.
///
/// This function attempts to find and read the bootloader file specified by the `filename` argument, or by constructing
/// a path based on provided parameters. It then searches the file's content for known version string patterns specific to
/// either systemd-boot or GRUB2 bootloaders. The function is designed to work with binary files where version information
/// is embedded within the data.
///
/// # Arguments
///
/// * `snapshot` - A numeric identifier for the snapshot directory, used when constructing the default path to the bootloader file.
/// * `firmware_arch` - The architecture of the firmware (e.g., "x64"), used in path construction and potentially in selecting
/// version patterns.
/// * `shimdir` - Directory containing the bootloader shim, which is part of the path if the default bootloader file is used.
/// * `boot_root` - The root directory for boot files, forming the base of the constructed path to the bootloader file.
/// * `boot_dst` - The destination directory for boot files, relative to `boot_root`, further specifying the constructed path.
/// * `filename` - An optional specific filename to directly check for the bootloader version. If provided, other path parameters
/// are ignored.
/// * `override_prefix` - An optional path override that, if provided, replaces the `boot_root` in the constructed path to
/// the bootloader file.
///
/// # Returns
///
/// Returns a `Result` with:
/// - `Ok(String)` containing the extracted version string if a known version pattern is found within the bootloader file's content.
/// - `Err(String)` with an appropriate error message if the file does not exist, cannot be read, or if no known version pattern is found.
pub(crate) fn bootloader_version(
    snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    filename: Option<PathBuf>,
    override_prefix: Option<&Path>,
) -> Result<String, String> {
    let base_prefix = match override_prefix {
        Some(override_path) => override_path.to_path_buf(),
        None => Path::new("/").to_path_buf(),
    };
    let prefix = match snapshot {
        Some(snap) => base_prefix
            .join(".snapshots")
            .join(snap.to_string())
            .join("snapshot"),
        None => base_prefix.clone(),
    };
    let fn_path = match filename {
        Some(f) => f,
        None => {
            if PathBuf::from(format!("{}{}/shim.efi", prefix.display(), shimdir)).exists() {
                PathBuf::from(format!(
                    "{}{}{}/grub.efi",
                    base_prefix.display(),
                    boot_root,
                    boot_dst
                ))
            } else {
                let bootloader = find_bootloader(snapshot, firmware_arch, override_prefix)?;
                PathBuf::from(format!(
                    "{}{}{}/{}",
                    base_prefix.display(),
                    boot_root,
                    boot_dst,
                    bootloader.file_name().unwrap().to_str().unwrap()
                ))
            }
        }
    };
    if !fn_path.exists() {
        let err = format!("File does not exist: {}", fn_path.display());
        return Err(err);
    }

    let content = fs::read(&fn_path).map_err(|_| "Failed to read file")?;

    let patterns = [
        (&b"LoaderInfo: systemd-boot "[..], &b" ####"[..]),
        (&b"GNU GRUB  version %s\x00"[..], &b"\x00"[..]),
    ];
    for (start, end) in &patterns {
        if let Some(version) = find_version(&content, start, end) {
            return Ok(version);
        }
    }
    Err("Version not found".to_string())
}

/// Determines if the installed bootloader needs an update by comparing its version with the system version.
///
/// This function retrieves the currently deployed bootloader version and compares it with the system's bootloader version.
/// It determines whether an update is necessary based on the comparison result.
///
/// # Arguments
///
/// * `snapshot` - Optional snapshot identifier used for specifying a particular system snapshot for version comparison.
/// * `firmware_arch` - The architecture of the firmware, such as "x64" or "arm64", used to determine the appropriate bootloader file.
/// * `shimdir` - Directory containing bootloader shim files, used in constructing the path to the bootloader file if a default filename is not provided.
/// * `boot_root` - The root directory where boot files are located, used in constructing the path to the bootloader file.
/// * `boot_dst` - Destination directory for boot files relative to `boot_root`, used in constructing the path.
/// * `override_prefix` - An optional path override that replaces `boot_root` in the constructed path to the bootloader file.
///
/// # Returns
///
/// Returns `Ok(true)` if the installed bootloader is older than the system's bootloader version and needs an update.
/// Returns `Ok(false)` if the installed bootloader is up-to-date with the system's version.
/// Returns `Err(String)` with an error message if the operation fails, such as when the bootloader version cannot be determined.
pub fn bootloader_needs_update(
    snapshot: Option<u64>,
    root_snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    override_prefix: Option<&Path>,
) -> Result<bool, String> {
    let deployed_version = bootloader_version(
        root_snapshot,
        firmware_arch,
        shimdir,
        boot_root,
        boot_dst,
        None,
        override_prefix,
    )?;
    log_info(&format!("deployed version {}", deployed_version), 1);

    let system_bootloader = find_bootloader(snapshot, firmware_arch, override_prefix)
        .map_err(|e| format!("Couldn't find current bootloader: {}", e))?;

    let system_version = bootloader_version(
        snapshot,
        firmware_arch,
        shimdir,
        boot_root,
        boot_dst,
        Some(system_bootloader),
        override_prefix,
    )?;
    log_info(&format!("system version {}", system_version), 1);

    if utils::compare_versions(&deployed_version, &system_version) {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Identifies the installed bootloader type for a given system setup.
///
/// This function determines which bootloader is currently installed by checking the presence and configuration
/// of systemd-boot and GRUB2. It returns the name of the detected bootloader as a string. The function uses
/// the `is_sdboot` and `is_grub2` helper functions to ascertain the bootloader type based on the snapshot,
/// firmware architecture, and optionally provided path prefix.
///
/// # Arguments
///
/// * `snapshot` - An optional snapshot identifier. If provided, the function checks the snapshot-specific
///   bootloader configuration instead of the system's current configuration.
/// * `firmware_arch` - The architecture of the firmware (e.g., "x64", "aa64"). This parameter is used to
///   locate the EFI files specific to the architecture.
/// * `override_prefix` - An optional path prefix that overrides the default path used to locate bootloader
///   files. This can be useful for testing or when working with chroot environments.
///
/// # Returns
///
/// A `Result` containing:
/// - `Ok(String)` with the bootloader name ("systemd-boot" or "grub2") if a bootloader is detected.
/// - `Err(&'static str)` with an error message ("Bootloader not detected") if neither systemd-boot nor GRUB2
///   configurations are found.
pub(crate) fn bootloader_name(
    snapshot: Option<u64>,
    firmware_arch: &str,
    override_prefix: Option<&Path>,
) -> Result<String, &'static str> {
    if is_sdboot(snapshot, firmware_arch, override_prefix) {
        Ok("systemd-boot".to_string())
    } else if is_grub2(snapshot, override_prefix) {
        Ok("grub2".to_string())
    } else {
        Err("Bootloader not detected")
    }
}

/// Checks whether systemd-boot is installed and marked by `sdbootutil`.
///
/// This function verifies if systemd-boot is installed by checking two criteria: the successful detection of the
/// bootloader version and the presence of a flag file indicating that `sdbootutil` was used for the installation.
/// It constructs the path to the flag file based on given parameters and checks for its existence.
///
/// # Arguments
///
/// * `snapshot` - A numeric identifier for the snapshot directory, used in determining the bootloader version.
/// * `firmware_arch` - The architecture of the firmware, such as "x64" or "arm64".
/// * `shimdir` - The directory containing the bootloader shim, part of the path if the default bootloader file is used.
/// * `boot_root` - The root directory for boot files, used in constructing the path to the flag file.
/// * `boot_dst` - The destination directory for boot files, relative to `boot_root`, used in constructing the path.
/// * `filename` - An optional specific filename to check for the bootloader version. If provided, other path parameters are ignored.
/// * `override_prefix` - An optional path override that replaces `boot_root` in the constructed path to the flag file.
///
/// # Returns
///
/// Returns `Ok(true)` if both the bootloader version is successfully detected and the installation flag file exists,
/// indicating systemd-boot was installed using `sdbootutil`. Returns `Ok(false)` otherwise.
/// Returns `Err(String)` with an error message if any operation (like reading the bootloader file) fails.
pub(crate) fn is_installed(
    snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    filename: Option<PathBuf>,
    override_prefix: Option<&Path>,
) -> Result<bool, String> {
    let prefix = override_prefix.unwrap_or(Path::new(""));
    let bootloader_version_successful = bootloader_version(
        snapshot,
        firmware_arch,
        shimdir,
        boot_root,
        boot_dst,
        filename,
        override_prefix,
    )
    .is_ok();
    let flag_path = format!(
        "{}{}{}/installed_by_sdbootutil",
        prefix.display(),
        boot_root,
        boot_dst
    );
    let installed_flag_path = Path::new(&flag_path);
    let installed_flag_exists = installed_flag_path.exists();

    Ok(bootloader_version_successful && installed_flag_exists)
}

/// Retrieves the path to the shim directory based on the system architecture.
///
/// This function constructs a path to where the EFI shim files are stored, which varies
/// depending on the system's architecture. It uses a global `ARCH` constant to determine
/// the architecture-specific subdirectory.
///
/// # Returns
///
/// Returns a `String` representing the full path to the shim directory.
pub(crate) fn get_shimdir() -> String {
    format!("/usr/share/efi/{}", ARCH)
}

/// Updates the system's random seed stored in the EFI boot loader directory.
///
/// This function generates a new random seed and stores it in the `loader/random-seed` file
/// within the specified boot root directory. It supports disabling this operation with the
/// `arg_no_random_seed` flag and allows overriding the root directory path with `override_prefix`.
///
/// # Arguments
///
/// * `boot_root` - A string slice specifying the boot root directory where the `loader` directory resides.
/// * `arg_no_random_seed` - A boolean flag that, if `true`, skips the random seed update process.
/// * `override_prefix` - An optional reference to a `Path` that overrides the root directory path.
///
/// # Returns
///
/// Returns `Ok(())` on successful update or an `Err` with a `String` message if the operation fails.
///
/// # Errors
///
/// Returns an error if file operations (create, read, write) fail or if random seed generation encounters an issue.
pub(crate) fn update_random_seed(
    boot_root: &str,
    arg_no_random_seed: bool,
    override_prefix: Option<&Path>,
) -> Result<(), String> {
    if arg_no_random_seed {
        return Ok(());
    }

    let prefix = match override_prefix {
        Some(ov_prefix) => PathBuf::from(ov_prefix),
        None => PathBuf::from("/"),
    };
    let full_boot_root = prefix.join(boot_root.strip_prefix("/").unwrap_or(boot_root));
    let mut rng = thread_rng();
    let mut new_seed = [0u8; 32];
    rng.fill_bytes(&mut new_seed);

    let random_seed_path = full_boot_root.join("loader/random-seed");
    fs::create_dir_all(
        random_seed_path
            .parent()
            .ok_or_else(|| "Failed to find the parent directory of dst".to_string())?,
    )
    .map_err(|e| e.to_string())?;

    if random_seed_path.exists() {
        let mut file = File::open(&random_seed_path).map_err(|e| {
            format!(
                "Failed to open random seed file {}: {}",
                random_seed_path.display(),
                e
            )
        })?;
        let mut old_seed = Vec::new();
        file.read_to_end(&mut old_seed).map_err(|e| {
            format!(
                "Failed to read random seed file {}: {}",
                random_seed_path.display(),
                e
            )
        })?;

        if old_seed.len() == 32 {
            new_seed
                .iter_mut()
                .zip(old_seed.iter())
                .for_each(|(n, o)| *n ^= o);
        }
    }

    let mut hasher = Sha256::new();
    hasher.update(new_seed);
    let hashed_seed = hasher.finalize();

    let new_seed_path = full_boot_root.join("loader/random-seed.new");
    fs::write(&new_seed_path, &hashed_seed).map_err(|e| {
        format!(
            "Failed to write new random seed file {}: {}",
            new_seed_path.display(),
            e
        )
    })?;

    fs::rename(&new_seed_path, &random_seed_path).map_err(|e| {
        format!(
            "Failed to rename new random seed file {}: {}",
            new_seed_path.display(),
            e
        )
    })?;

    Ok(())
}

/// Reads the partition number from a specified file.
///
/// This function opens the file at `partition_file_path` and reads the first line to extract
/// the partition number. It's primarily used to determine the partition number from system files
/// in `/sys` or `/proc`.
///
/// # Arguments
///
/// * `partition_file_path` - A reference to a `Path` that specifies the file from which to read the partition number.
///
/// # Returns
///
/// Returns `Ok(u32)` with the partition number on success, or an `Err` with a `String` message if the operation fails.
///
/// # Errors
///
/// Returns an error if opening or reading the file fails, or if the content cannot be parsed into a `u32`.
pub(crate) fn read_partition_number(partition_file_path: &Path) -> Result<u32, String> {
    let file = File::open(partition_file_path).map_err(|e| {
        format!(
            "Failed to open partition file {}: {}",
            partition_file_path.display(),
            e
        )
    })?;

    let mut buf_reader = io::BufReader::new(file);
    let mut partno_str = String::new();
    buf_reader.read_line(&mut partno_str).map_err(|e| {
        format!(
            "Failed to read partition number from {}: {}",
            partition_file_path.display(),
            e
        )
    })?;

    partno_str.trim().parse::<u32>().map_err(|e| {
        format!(
            "Invalid partition number in {}: {}",
            partition_file_path.display(),
            e
        )
    })
}

/// Determines the drive and partition number from a given block device path.
///
/// This function parses system information to extract the drive path and partition number
/// from a specified block device identifier (e.g., `sda1`). It supports an `override_path`
/// for alternative system root directories.
///
/// # Arguments
///
/// * `block_device` - A string slice representing the block device identifier.
/// * `override_path` - An optional reference to a `Path` that specifies an alternative root directory.
///
/// # Returns
///
/// Returns `Ok((PathBuf, u32))` containing the drive path and partition number, or an `Err`
/// with a `String` message if the operation fails.
///
/// # Errors
///
/// Returns an error if system file operations fail or if the drive or partition information cannot be determined.
pub(crate) fn get_drive_and_partition_from_block_device(
    block_device: &str,
    override_path: Option<&Path>,
) -> Result<(PathBuf, u32), String> {
    let base_path = override_path.unwrap_or(Path::new("/"));
    let block_device_name = block_device.trim_start_matches("/dev/");
    let sys_block_path = base_path.join("sys/class/block").join(block_device_name);

    let abs_path = fs::read_link(&sys_block_path).map_err(|e| {
        format!(
            "Failed to read link for {}: {}",
            sys_block_path.display(),
            e
        )
    })?;

    let drive_path = abs_path
        .parent()
        .ok_or_else(|| format!("Failed to get drive path from {}", abs_path.display()))?
        .to_path_buf();

    let drive = Path::new("/dev").join(
        drive_path
            .file_name()
            .ok_or_else(|| format!("Failed to get drive name from {}", drive_path.display()))?,
    );

    let partition_file_path = sys_block_path.join("partition");
    let partition_number = read_partition_number(&partition_file_path)?;

    Ok((drive, partition_number))
}

/// Copies EFI shim files from the system directory to the boot loader directory.
///
/// This function copies specific EFI shim files (`MokManager.efi` and `shim.efi`) from the shim directory
/// within a system prefix to the destination directory in the boot loader path. It ensures the necessary
/// directories exist and handles path overrides with `override_prefix`.
///
/// # Arguments
///
/// * `snapshot_prefix` - A string slice indicating the system directory prefix.
/// * `shimdir` - A string slice specifying the subdirectory within the snapshot prefix where shim files are located.
/// * `boot_root` - A string slice specifying the root boot directory.
/// * `boot_dst` - A string slice indicating the destination directory within the boot root where files will be copied.
/// * `bootloader` - A reference to a `PathBuf` specifying the bootloader file path.
/// * `override_prefix` - An optional reference to a `Path` for overriding the root directory path.
///
/// # Returns
///
/// Returns `Ok(())` on successful copy, or an `Err` with a `String` message if the operation fails.
///
/// # Errors
///
/// Returns an error if directory creation or file copy operations fail.
pub(crate) fn copy_shim_files(
    snapshot_prefix: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    bootloader: &PathBuf,
    override_prefix: Option<&Path>,
) -> Result<(), String> {
    let base_path = override_prefix.unwrap_or(Path::new("/"));
    let full_boot_root = base_path.join(boot_root.strip_prefix("/").unwrap_or(boot_root));
    let full_boot_dst = full_boot_root.join(boot_dst.strip_prefix("/").unwrap_or(boot_dst));
    let full_snapshot_prefix =
        base_path.join(snapshot_prefix.strip_prefix("/").unwrap_or(snapshot_prefix));
    let full_shimdir = full_snapshot_prefix.join(shimdir.strip_prefix("/").unwrap_or(shimdir));

    let shim_files = ["MokManager.efi", "shim.efi"];

    for shim_file in &shim_files {
        let src = full_shimdir.join(shim_file);
        let dst = full_boot_dst.join(shim_file);
        fs::create_dir_all(
            dst.parent()
                .ok_or_else(|| "Failed to find the parent directory of dst".to_string())?,
        )
        .map_err(|e| e.to_string())?;
        fs::copy(&src, &dst).map_err(|e| format!("Failed to copy {}: {}", src.display(), e))?;
    }

    let dst_bootloader = full_boot_dst.join("grub.efi");
    fs::create_dir_all(
        dst_bootloader
            .parent()
            .ok_or_else(|| "Failed to find the parent directory of dst_bootloader".to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::copy(bootloader, &dst_bootloader)
        .map_err(|e| format!("Failed to copy bootloader: {}", e))?;

    Ok(())
}

/// Copies the bootloader file to both the specified boot destination and the EFI/BOOT directory.
///
/// This function copies the bootloader file to the destination directory within the boot root, and also
/// to the standard EFI/BOOT directory, renaming it according to the EFI specification and system architecture.
/// It supports path overrides with `override_prefix`.
///
/// # Arguments
///
/// * `bootloader` - A reference to a `PathBuf` specifying the bootloader file path.
/// * `boot_root` - A string slice specifying the root boot directory.
/// * `boot_dst` - A string slice indicating the destination directory within the boot root for the bootloader.
/// * `firmware_arch` - A string slice representing the system's firmware architecture (e.g., `x64`, `aa64`).
/// * `override_prefix` - An optional reference to a `Path` for overriding the root directory path.
///
/// # Returns
///
/// Returns `Ok(PathBuf)` with the path to the copied bootloader file in the boot destination, or an `Err`
/// with a `String` message if the operation fails.
///
/// # Errors
///
/// Returns an error if directory creation or file copy operations fail.
pub(crate) fn copy_bootloader(
    bootloader: &PathBuf,
    boot_root: &str,
    boot_dst: &str,
    firmware_arch: &str,
    override_prefix: Option<&Path>,
) -> Result<PathBuf, String> {
    let base_path = override_prefix.unwrap_or(Path::new("/"));
    let full_boot_root = base_path.join(boot_root.strip_prefix("/").unwrap_or(boot_root));
    let full_boot_dst = full_boot_root.join(boot_dst.strip_prefix("/").unwrap_or(boot_dst));

    let bootloader_file_name = bootloader
        .file_name()
        .ok_or("Failed to get bootloader file name")?;
    let efi_bootloader_name = format!("BOOT{}.EFI", firmware_arch.to_uppercase());

    let dst = full_boot_dst.join(bootloader_file_name);
    fs::create_dir_all(
        dst.parent()
            .ok_or_else(|| "Failed to find the parent directory of dst".to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::copy(bootloader, &dst).map_err(|e| format!("Failed to copy bootloader: {}", e))?;

    let efi_dst = full_boot_root
        .join("EFI")
        .join("BOOT")
        .join(efi_bootloader_name);
    fs::create_dir_all(
        efi_dst
            .parent()
            .ok_or_else(|| "Failed to find the parent directory of efi_dst".to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::copy(bootloader, efi_dst).map_err(|e| format!("Failed to copy EFI bootloader: {}", e))?;

    Ok(Path::new(boot_dst).join(bootloader_file_name))
}

/// Updates the configuration for the systemd-boot bootloader.
///
/// This function ensures the existence of the `loader/entries` directory and `loader/loader.conf`
/// file within the specified boot root directory. It supports path overrides with `override_prefix`.
///
/// # Arguments
///
/// * `boot_root` - A string slice specifying the root boot directory.
/// * `override_prefix` - An optional reference to a `Path` for overriding the root directory path.
///
/// # Returns
///
/// Returns `Ok(())` on successful update, or an `Err` with a `String` message if the operation fails.
///
/// # Errors
///
/// Returns an error if directory creation or file operations fail.
pub(crate) fn update_sdboot_configuration(
    boot_root: &str,
    override_prefix: Option<&Path>,
) -> Result<(), String> {
    let base_path = override_prefix.unwrap_or_else(|| Path::new("/"));
    let full_boot_root = base_path.join(boot_root.strip_prefix("/").unwrap_or(boot_root));

    let entries_rel_path = full_boot_root.join("loader/entries.srel");
    fs::create_dir_all(
        entries_rel_path
            .parent()
            .ok_or_else(|| "Failed to find the parent directory of entries_rel_path".to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if !entries_rel_path.exists() {
        fs::write(&entries_rel_path, "type1").map_err(|e| e.to_string())?;
    }

    let loader_conf_path = full_boot_root.join("loader/loader.conf");
    if !loader_conf_path.exists() {
        let mut loader_conf = File::create(&loader_conf_path).map_err(|e| e.to_string())?;
        writeln!(loader_conf, "#timeout 3\n#console-mode keep").map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Updates the GRUB2 bootloader configuration.
///
/// This function copies the `grub.cfg` file from the system directory to both the specified boot destination
/// and the standard EFI/BOOT directory. It also ensures the GRUB2 module directory exists and copies the `bli.mod`
/// file from the system to the module directory. It supports path overrides with `override_prefix`.
///
/// # Arguments
///
/// * `snapshot_prefix` - A string slice indicating the system directory prefix.
/// * `boot_root` - A string slice specifying the root boot directory.
/// * `boot_dst` - A string slice indicating the destination directory within the boot root for GRUB2 configuration files.
/// * `override_prefix` - An optional reference to a `Path` for overriding the root directory path.
///
/// # Returns
///
/// Returns `Ok(())` on successful update, or an `Err` with a `String` message if the operation fails.
///
/// # Errors
///
/// Returns an error if directory creation, file copy, or file renaming operations fail.
pub(crate) fn update_grub2_configuration(
    snapshot_prefix: &str,
    boot_root: &str,
    boot_dst: &str,
    override_prefix: Option<&Path>,
) -> Result<(), String> {
    let base_path = override_prefix.unwrap_or_else(|| Path::new("/"));
    let full_boot_root = base_path.join(boot_root.strip_prefix("/").unwrap_or(boot_root));
    let full_boot_dst = full_boot_root.join(boot_dst.strip_prefix("/").unwrap_or(boot_dst));
    let full_snapshot_prefix =
        base_path.join(snapshot_prefix.strip_prefix("/").unwrap_or(snapshot_prefix));

    let grub_cfg_path = full_boot_dst.join("grub.cfg");
    fs::create_dir_all(
        grub_cfg_path
            .parent()
            .ok_or_else(|| "Failed to find the parent directory of grub_cfg".to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if !grub_cfg_path.exists() {
        let mut grub_cfg = File::create(&grub_cfg_path).map_err(|e| e.to_string())?;
        writeln!(
            grub_cfg,
            "timeout=8\nfunction load_video {{\n  true\n}}\ninsmod bli\nblscfg"
        )
        .map_err(|e| e.to_string())?;
    }

    let efi_boot_grub_cfg_path = full_boot_root.join("EFI/BOOT/grub.cfg");
    fs::create_dir_all(efi_boot_grub_cfg_path.parent().ok_or_else(|| {
        "Failed to find the parent directory of efi_boot_grub_cfg_path".to_string()
    })?)
    .map_err(|e| e.to_string())?;
    fs::copy(&grub_cfg_path, &efi_boot_grub_cfg_path).map_err(|e| e.to_string())?;

    let mod_dir = full_boot_dst.join(format!("{}-efi", ARCH));
    fs::create_dir_all(&mod_dir).map_err(|e| e.to_string())?;

    let bli_mod_src = full_snapshot_prefix.join("grub2moddir/bli.mod");
    let bli_mod_dst = mod_dir.join("bli.mod");
    fs::copy(bli_mod_src, bli_mod_dst)
        .map_err(|e| format!("error copying grub2moddir: {}", e.to_string()))?;

    Ok(())
}

/// Installs the bootloader and updates the boot configuration.
///
/// This function handles the installation and configuration of the bootloader, including copying shim files or the bootloader itself,
/// updating the random seed, and setting up the EFI boot entry. It supports snapshot-based installation, architecture-specific
/// considerations, and various flags to control the installation process. Path overrides can be provided with `override_prefix`.
///
/// # Arguments
///
/// * `snapshot` - An optional snapshot ID used for snapshot-based installation.
/// * `firmware_arch` - A string slice representing the system's firmware architecture.
/// * `shimdir` - A string slice specifying the directory containing shim files for secure boot.
/// * `boot_root` - A string slice specifying the root boot directory.
/// * `boot_dst` - A string slice indicating the destination directory within the boot root for bootloader files.
/// * `entry_token` - A string representing the entry token for the bootloader.
/// * `arg_no_variables` - A boolean flag indicating whether EFI variables should be updated.
/// * `arg_no_random_seed` - A boolean flag indicating whether the random seed should be updated.
/// * `override_prefix` - An optional reference to a `Path` for overriding the root directory path.
///
/// # Returns
///
/// Returns `Ok(())` on successful installation and configuration, or an `Err` with a `String` message if the operation fails.
///
/// # Errors
///
/// Returns an error if any step in the installation or configuration process fails, including file operations, bootloader copying, and EFI boot entry creation.
pub(crate) fn install_bootloader(
    snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    entry_token: String,
    arg_no_variables: bool,
    arg_no_random_seed: bool,
    override_prefix: Option<&Path>,
) -> Result<(), String> {
    let prefix = match override_prefix {
        Some(ov_prefix) => PathBuf::from(ov_prefix),
        None => PathBuf::from("/"),
    };
    let snapshot_prefix = match snapshot {
        Some(snap) => format!("/.snapshots/{}/snapshot", snap),
        None => "/".to_string(),
    };
    let full_snapshot_prefix = prefix.join(
        snapshot_prefix
            .strip_prefix("/")
            .unwrap_or(&snapshot_prefix),
    );
    let full_boot_root = prefix.join(boot_root.strip_prefix("/").unwrap_or(boot_root));
    let full_boot_dst = full_boot_root.join(boot_dst.strip_prefix("/").unwrap_or(boot_dst));

    let bootloader =
        find_bootloader(snapshot, firmware_arch, override_prefix).map_err(|e| e.to_string())?;
    let bldr_name =
        bootloader_name(snapshot, firmware_arch, override_prefix).map_err(|e| e.to_string())?;

    fs::create_dir_all(full_boot_root.join("loader/entries")).map_err(|e| e.to_string())?;

    let entry = if full_snapshot_prefix
        .join(shimdir.strip_prefix("/").unwrap_or(&shimdir))
        .join("shim.efi")
        .exists()
    {
        log_info(
            &format!(
                "Installing {} with shim into {:?}",
                bldr_name, full_boot_root
            ),
            1,
        );
        copy_shim_files(
            &snapshot_prefix,
            shimdir,
            boot_root,
            boot_dst,
            &bootloader,
            override_prefix,
        )?;
        Path::new(boot_dst).join("shim.efi")
    } else {
        log_info(&format!("Installing {} into {:?}", bldr_name, boot_root), 1);
        copy_bootloader(
            &bootloader,
            boot_root,
            boot_dst,
            firmware_arch,
            override_prefix,
        )?
    };

    let boot_csv = full_boot_dst.join("boot.csv");
    let mut boot_csv_file = BufWriter::new(File::create(&boot_csv).map_err(|e| e.to_string())?);
    writeln!(
        boot_csv_file,
        "{},openSUSE Boot Manager",
        entry.file_name().unwrap().to_string_lossy()
    )
    .map_err(|e| e.to_string())?;

    fs::create_dir_all(full_boot_root.join(entry_token.clone())).map_err(|e| e.to_string())?;
    fs::write(
        full_boot_dst.join("installed_by_sdbootutil"),
        entry_token.clone(),
    )
    .map_err(|e| e.to_string())?;

    let entry_token_path = prefix.join(Path::new("etc/kernel/entry-token"));
    if !entry_token_path.exists() {
        fs::create_dir_all(entry_token_path.parent().ok_or_else(|| {
            "Failed to find the parent directory of entry_token_path".to_string()
        })?)
        .map_err(|e| e.to_string())?;
        fs::write(entry_token_path, entry_token).map_err(|e| e.to_string())?;
    }
    update_random_seed(boot_root, arg_no_random_seed, override_prefix)
        .map_err(|e| format!("Failed to update random seed: {}", e))?;
    if is_sdboot(snapshot, firmware_arch, override_prefix) {
        update_sdboot_configuration(boot_root, override_prefix)?;
    } else if is_grub2(snapshot, override_prefix) {
        update_grub2_configuration(&snapshot_prefix, boot_root, boot_dst, override_prefix)?;
    }

    let (_, blkpart) = get_findmnt_output(boot_root, override_prefix)?;
    let (drive, partno) = get_drive_and_partition_from_block_device(&blkpart, override_prefix)?;
    if !arg_no_variables {
        create_efi_boot_entry(&drive, partno, &entry, override_prefix)?;
    }

    Ok(())
}

/// Checks if the root filesystem is Btrfs and contains a `.snapshots` directory.
///
/// This function reads the system's mount information to determine if the root (`/`) is mounted
/// with the Btrfs filesystem and verifies the existence of the `/.snapshots` directory. It supports
/// an optional `prefix` argument to specify an alternate root directory for testing or non-standard
/// environments.
///
/// # Arguments
///
/// * `prefix` - An optional string slice representing an alternative root directory to check.
///
/// # Returns
///
/// Returns `Ok(true)` if the root is Btrfs and contains a `/.snapshots` directory, `Ok(false)` otherwise,
/// or an `Err` with a `String` message if an error occurs during execution.
///
/// # Errors
///
/// Returns an error if reading from the mounts file fails or if the path construction is invalid.
pub(crate) fn is_snapshotted(prefix: Option<&str>) -> Result<bool, String> {
    let mounts_file_path = match prefix {
        Some(prefix) => PathBuf::from(prefix)
            .join("proc/mounts")
            .to_string_lossy()
            .into_owned(),
        None => "/proc/mounts".to_string(),
    };
    let mounts_file = fs::File::open(mounts_file_path).expect("Could not open /proc/mounts");
    let reader = BufReader::new(mounts_file);
    let snapshots_dir_path = match prefix {
        Some(prefix) => PathBuf::from(prefix)
            .join(".snapshots")
            .to_string_lossy()
            .into_owned(),
        None => "/.snapshots".to_string(),
    };
    let snapshots_dir = Path::new(&snapshots_dir_path);

    for line in reader.lines() {
        let line = line.expect("Error reading line");
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 2 {
            let mount_point = parts[1];
            let fs_type = parts[2];

            if mount_point == "/" {
                return Ok(fs_type == "btrfs" && snapshots_dir.is_dir());
            }
        }
    }
    Ok(false)
}

/// Reads and parses the OS release information from standard locations.
///
/// This function attempts to read the OS release information from either `/usr/lib/os-release`
/// or `/etc/os-release`, preferring the first if both exist. It can also adjust the base path
/// using an optional subvolume path and an optional override prefix, making it suitable for
/// use in environments where the root filesystem is not mounted at `/`.
///
/// # Arguments
///
/// * `subvol` - An optional path to a subvolume that should be used as the root for reading the OS release information.
/// * `override_prefix` - An optional path prefix that overrides the base path for reading the OS release files.
///
/// # Returns
///
/// A `Result` containing a tuple with the following optional elements, or an error message if the files cannot be read:
///
/// * `ID` - The OS identifier, e.g., "ubuntu".
/// * `VERSION_ID` - The OS version identifier, e.g., "20.04".
/// * `PRETTY_NAME` - A pretty name for the OS, e.g., "Ubuntu 20.04 LTS".
/// * `IMAGE_ID` - An optional identifier for the OS image used, if available.
///
/// # Errors
///
/// Returns an error if both `/usr/lib/os-release` and `/etc/os-release` are not found or cannot be opened.
pub(crate) fn read_os_release(
    subvol: Option<&Path>,
    override_prefix: Option<&Path>,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    String,
> {
    let prefix = override_prefix.unwrap_or(Path::new("/"));
    let paths = subvol.map_or(
        vec![
            prefix.join("usr/lib/os-release"),
            prefix.join("etc/os-release"),
        ],
        |subvol_path| {
            let relative_subvol = subvol_path.strip_prefix("/").unwrap_or(subvol_path);
            vec![
                prefix.join(relative_subvol).join("usr/lib/os-release"),
                prefix.join(relative_subvol).join("etc/os-release"),
            ]
        },
    );

    for path in &paths {
        if path.exists() {
            let file = File::open(path).map_err(|e| format!("Couldn't open file: {}", e))?;
            let reader = BufReader::new(file);
            let mut info = HashMap::new();

            for line in reader.lines() {
                let line = line.map_err(|e| format!("Couldn't read line: {}", e))?;
                if let Some((key, value)) = line.split_once('=') {
                    info.insert(
                        key.trim_start_matches("os_release_").to_string(),
                        value.trim_matches('"').to_string(),
                    );
                }
            }
            let os_release_id = info.get("ID").cloned();
            let os_release_version_id = info.get("VERSION_ID").cloned();
            let os_release_pretty_name = info.get("PRETTY_NAME").cloned();
            let os_release_image_id = info.get("IMAGE_ID").cloned();

            return Ok((
                os_release_id,
                os_release_version_id,
                os_release_pretty_name,
                os_release_image_id,
            ));
        }
    }

    Err("OS release file not found".to_string())
}

/// Reads the machine ID from the system or specified subvolume.
///
/// This function reads the machine ID from `/etc/machine-id`. In a transactional update scenario,
/// it might also read from `/var/lib/overlay/<snapshot>/etc/machine-id` if the system is determined
/// to be transactional. It supports reading from an alternate root via an optional subvolume path
/// and an override prefix, making it adaptable for different environment setups.
///
/// # Arguments
///
/// * `subvol` - An optional path to a subvolume that should be used as the root for reading the machine ID.
/// * `snapshot` - An optional snapshot identifier, used in transactional environments to specify the snapshot
///   layer from which to read the machine ID.
/// * `override_prefix` - An optional path prefix that overrides the base path for reading the machine ID file.
///
/// # Returns
///
/// A `Result` containing the machine ID as a `String`, or an error message if the file does not exist,
/// cannot be opened, or is empty.
///
/// # Errors
///
/// Returns an error if the machine ID file does not exist, cannot be opened, or is empty.
pub(crate) fn read_machine_id(
    subvol: Option<&Path>,
    snapshot: Option<u64>,
    override_prefix: Option<&Path>,
) -> Result<String, String> {
    let prefix = override_prefix.unwrap_or(Path::new("/"));
    let mut paths = Vec::new();

    if is_transactional(override_prefix.map(|p| p.to_str().unwrap_or("")))
        .map_err(|e| format!("Couldn't get transactional status: {}", e))?
    {
        if let Some(snap) = snapshot {
            paths.push(prefix.join(format!("var/lib/overlay/{}/etc/machine-id", snap)));
        }
    }

    if let Some(subvol_path) = subvol {
        let relative_subvol = subvol_path.strip_prefix("/").unwrap_or(subvol_path);
        paths.push(prefix.join(relative_subvol).join("etc/machine-id"));
    } else {
        paths.push(prefix.join("etc/machine-id"));
    }

    for path in paths {
        if path.exists()
            && path
                .metadata()
                .map_err(|e| format!("Machine ID file has invalid metadata: {}", e))?
                .len()
                > 0
        {
            let file = File::open(path).map_err(|e| format!("Couldn't open file: {}", e))?;
            let mut reader = BufReader::new(file);
            let mut machine_id = String::new();
            reader
                .read_line(&mut machine_id)
                .map_err(|e| format!("Couldn't read line: {}", e))?;
            return Ok(machine_id.trim().to_string());
        }
    }

    Err("Machine ID file not found".to_string())
}

/// Determines the system's entry token based on various system files and an optional user input.
///
/// This function reads the OS release information and the machine ID from the system or specified subvolume,
/// and determines the entry token to use for the bootloader based on the provided `arg_entry_token` parameter.
/// The token can be set explicitly to use the machine ID, the OS ID (from `/etc/os-release`), an OS image ID,
/// or a custom token. If `arg_entry_token` is set to "auto" or is not provided, the function attempts to read
/// the entry token from `/etc/kernel/entry-token`; if the file does not exist, it tries to derive
/// the token from available system information, starting with the machine ID, then OS image ID, and finally the OS ID.
///
/// # Arguments
///
/// * `subvol` - An optional path to the subvolume from which to read the OS release information and machine ID.
///   If not provided, the function reads from the system's root.
/// * `snapshot` - An optional snapshot identifier used when determining the machine ID in a transactional environment.
/// * `arg_entry_token` - An optional argument that specifies how to determine the entry token. It can be "auto",
///   "machine-id", "os-id", "os-image", or a custom token prefixed with "literal:" for direct usage.
///   If "auto" or not provided, the function uses the default mechanism described above.
/// * `override_prefix` - An optional path prefix that overrides the base path for reading system files. This is useful
///   for testing or when operating in a chroot environment.
///
/// # Returns
///
/// A `Result` containing:
///
/// * A tuple with the following elements:
///   - `entry_token`: The determined entry token.
///   - `machine_id`: The machine ID of the system.
///   - `os_release_id`: The OS ID from `/etc/os-release`, if available.
///   - `os_release_version_id`: The OS version ID from `/etc/os-release`, if available.
///   - `os_release_pretty_name`: The pretty name of the OS from `/etc/os-release`, if available.
/// * An `Err` with a string describing the error if the function fails to read required files or if the specified
///   `arg_entry_token` requires data that is not available.
pub(crate) fn settle_system_tokens(
    subvol: Option<&Path>,
    snapshot: Option<u64>,
    arg_entry_token: Option<&str>,
    override_prefix: Option<&Path>,
) -> Result<
    (
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    String,
> {
    let prefix = override_prefix.unwrap_or(Path::new("/"));
    let (os_release_id, os_release_version_id, os_release_pretty_name, os_release_image_id) =
        read_os_release(subvol, override_prefix)?;
    let machine_id_result = read_machine_id(subvol, snapshot, override_prefix);

    let entry_token = match arg_entry_token {
        Some("auto") | None => {
            if let Ok(token) = fs::read_to_string(prefix.join("etc/kernel/entry-token")) {
                token.trim_end_matches('\n').to_string()
            } else {
                [
                    machine_id_result.as_deref().ok(),
                    os_release_image_id.as_deref(),
                    os_release_id.as_deref(),
                ]
                .iter()
                .find_map(|&token| token)
                .map(String::from)
                .ok_or_else(|| "Can't auto detect".to_string())?
            }
        }
        Some("machine-id") => machine_id_result.clone()?,
        Some("os-id") => os_release_id
            .clone()
            .ok_or_else(|| "Missing ID".to_string())?,
        Some("os-image") => os_release_image_id.ok_or_else(|| "Missing IMAGE_ID".to_string())?,
        Some(token) if token.starts_with("literal:") => token.replacen("literal:", "", 1),
        Some(token) => {
            return Err(format!("Unexpected parameter for --entry-token: {}", token).to_string())
        }
    };

    Ok((
        entry_token,
        machine_id_result.unwrap_or_default(),
        os_release_id,
        os_release_version_id,
        os_release_pretty_name,
    ))
}
