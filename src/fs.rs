use super::io::{get_command_output, log_info, print_error};
use super::*;
use libbtrfs::subvolume;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// A structure representing an item subject to rollback actions.
///
/// This structure is used to encapsulate the path of a file that may need to be
/// restored to a previous state or cleaned up as part of a rollback process.
///
/// # Fields
///
/// * `original_path`: The `PathBuf` representing the path to the original file.
pub struct RollbackItem {
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
    pub fn new(original_path: PathBuf) -> Self {
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
    pub fn cleanup(&self) -> std::io::Result<()> {
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
pub fn create_temp_dir() -> (TempDir, PathBuf) {
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
pub fn cleanup_rollback_items(rollback_items: &[RollbackItem]) {
    for item in rollback_items {
        if let Err(e) = item.cleanup() {
            let message = format!("Error cleaning up item: {}", e);
            print_error(&message);
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
pub fn reset_rollback_items(rollback_items: &mut Vec<RollbackItem>) {
    for item in rollback_items.iter() {
        let backup_path = item.original_path.with_extension("bak");
        if backup_path.exists() {
            if let Err(e) = fs::remove_file(&backup_path) {
                let message = format!(
                    "Failed to remove backup file {}: {}",
                    backup_path.display(),
                    e
                );
                print_error(&message);
            } else {
                let message = format!("Removed backup file {}", backup_path.display());
                log_info(&message, 1)
            }
        }
    }
    rollback_items.clear();
}

/// Checks if the filesystem type of `/etc` is `overlayfs`.
///
/// # Returns
///
/// `Ok(true)` if the filesystem type is `overlayfs`, `Ok(false)` otherwise, or an `Error` if the command execution fails.
pub fn is_transactional() -> Result<bool, Box<dyn Error>> {
    let filesystem_type = get_command_output("stat", &["-f", "-c", "%T", "/etc"])?;
    Ok(filesystem_type == "overlayfs")
}

/// Retrieves detailed information about the root Btrfs snapshot.
///
/// This function extracts and returns the prefix path, the snapshot ID, and the full snapshot path from the system's root directory. It's designed to parse the snapshot path to identify these components, crucial for Btrfs snapshot management.
///
/// # Returns
///
/// A Result containing a tuple of:
/// - The prefix path as a String.
/// - The snapshot ID as a u64.
/// - The full snapshot path as a String.
///
/// # Errors
///
/// Returns an error if the snapshot path does not conform to the expected structure or if any parsing fails.
pub fn get_root_snapshot_info() -> Result<(String, u64, String), Box<dyn std::error::Error>> {
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
