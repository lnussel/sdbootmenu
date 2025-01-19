pub mod cli;
pub mod fs;
pub mod io;
pub mod ui;
pub mod utils;

use fs::{
    bootloader_name, bootloader_needs_update, install_bootloader, is_installed,
    settle_system_tokens,
};
use io::{log_info, set_systemd_log_level};
use std::path::{Path, PathBuf};

/// Executes the `Kernels` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `0`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_kernels().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_kernels() -> Result<bool, String> {
    let message = "Kernels command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `Snapshots` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `1`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_snapshots().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_snapshots() -> Result<bool, String> {
    let message = "Snapshots command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `Entries` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `2`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_entries().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_entries() -> Result<bool, String> {
    let message = "Entries command called";
    log_info(message, 1);
    Ok(true)
}

/// Identifies and logs the detected bootloader type, then returns a boolean indicating
/// if the detected bootloader is systemd-boot.
///
/// This function attempts to determine the installed bootloader by invoking `fs::bootloader_name`
/// with the provided snapshot, firmware architecture, and an optional path prefix. It logs
/// the detected bootloader type. If the detected bootloader is systemd-boot, it returns `true`;
/// otherwise, it returns `false`. In case of an error during bootloader detection, it returns
/// an error message.
///
/// # Arguments
///
/// * `snapshot` - An optional snapshot identifier. If provided, the function checks for the
///   bootloader configuration specific to this snapshot.
/// * `firmware_arch` - The architecture of the firmware (e.g., "x64", "aa64"). This parameter
///   is crucial for locating architecture-specific EFI files.
/// * `override_prefix` - An optional path prefix for locating bootloader files, useful in
///   scenarios like testing or operating within chroot environments.
///
/// # Returns
///
/// * `Ok(true)` if the detected bootloader is systemd-boot.
/// * `Ok(false)` if another bootloader is detected.
/// * `Err(String)` with an error message if the detection process fails.
///
/// # Examples
///
/// ```
/// let result = sdbootutil::command_bootloader(Some(0), "x64", None);
/// assert!(result.is_err(), "Expected an error from command_bootloader");
/// ```
pub fn command_bootloader(
    snapshot: Option<u64>,
    firmware_arch: &str,
    override_prefix: Option<&Path>,
) -> Result<bool, String> {
    match fs::bootloader_name(snapshot, firmware_arch, override_prefix) {
        Ok(bootloader) => {
            log_info(&format!("Detected bootloader: {}", bootloader), 0);
            if bootloader == "systemd-boot" {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => {
            let message = format!("Error detecting bootloader: {}", e);
            Err(message)
        }
    }
}

/// Executes the `AddKernel` command with a specified kernel version.
///
/// Logs the action including the kernel version and returns a status code.
///
/// # Arguments
///
/// * `kernel_version` - A string slice that specifies the kernel version to remove.
///
/// # Returns
///
/// Always returns `4`, which could represent a specific status related to the command's outcome.
///
/// # Example
///
/// ```
/// let status = sdbootutil::command_add_kernel("5.8.0-53-generic").unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_add_kernel(kernel_version: &str) -> Result<bool, String> {
    let message = format!("AddKernel command called with version {}", kernel_version);
    log_info(&message, 1);
    Ok(true)
}

/// Executes the `AddAllKernels` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `5`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_add_all_kernels().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_add_all_kernels() -> Result<bool, String> {
    let message = "AddAllKernels command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `Mkinitrd` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `6`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_mkinitrd().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_mkinitrd() -> Result<bool, String> {
    let message = "Mkinitrd command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `RemoveKernel` command with a specified kernel version.
///
/// Logs the action including the kernel version and returns a status code.
///
/// # Arguments
///
/// * `kernel_version` - A string slice that specifies the kernel version to remove.
///
/// # Returns
///
/// Always returns `7`, which could represent a specific status related to the command's outcome.
///
/// # Example
///
/// ```
/// let status = sdbootutil::command_remove_kernel("5.8.0-53-generic").unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_remove_kernel(kernel_version: &str) -> Result<bool, String> {
    let message = format!(
        "RemoveKernel command called with version {}",
        kernel_version
    );
    log_info(&message, 1);
    Ok(true)
}

/// Executes the `RemoveAllKernels` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `8`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_remove_all_kernels().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_remove_all_kernels() -> Result<bool, String> {
    let message = "RemoveAllKernels command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `ListKernels` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `9`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_list_kernels().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_list_kernels() -> Result<bool, String> {
    let message = "ListKernels command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `ListEntries` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `10`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_list_entries().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_list_entries() -> Result<bool, String> {
    let message = "ListEntries command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `ListSnapshots` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `11`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_list_snapshots().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_list_snapshots() -> Result<bool, String> {
    let message = "ListSnapshots command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `SetDefaultSnapshot` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `12`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_set_default_snapshot().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_set_default_snapshot() -> Result<bool, String> {
    let message = "SetDefaultSnapshot command called";
    log_info(message, 1);
    Ok(true)
}

/// Executes the `IsBootable` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `13`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_is_bootable().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_is_bootable() -> Result<bool, String> {
    let message = "IsBootable command called";
    log_info(message, 1);
    Ok(true)
}

/// Checks if systemd-boot is installed by `sdbootutil`.
///
/// This function verifies the installation status of systemd-boot by checking for the presence of a bootloader version file
/// and an installation flag file. It determines whether systemd-boot was installed using `sdbootutil` based on these criteria.
///
/// # Arguments
/// - `snapshot`: The snapshot identifier used to locate the snapshot-specific bootloader files.
/// - `firmware_arch`: The architecture of the firmware, such as "x86_64" or "arm64", used to refine the search for bootloader files.
/// - `shimdir`: The directory containing the bootloader shim, if any. This is used in constructing the path to check for a shim EFI file.
/// - `boot_root`: The root directory for boot files. This path is used as the base for constructing the paths to the bootloader version file and the installation flag file.
/// - `boot_dst`: The destination directory for boot files, relative to `boot_root`. This is further used in constructing the path to the installation flag file.
/// - `filename`: An optional filename for the bootloader version file. If provided, this file will be checked directly; otherwise, the function will attempt to determine the appropriate file based on other parameters.
/// - `override_prefix`: An optional path override that, if provided, will be used as the base directory for searching the bootloader files instead of the default path.
///
/// # Returns
/// - Returns `true` if both the bootloader version check is successful and the installation flag file is found,
/// indicating that systemd-boot was likely installed using `sdbootutil`.
/// - Returns `false` otherwise.
///
/// # Examples
/// ```
/// use sdbootutil::command_is_installed;
///
/// let installed = command_is_installed(
///     Some(0),
///     "x64",
///     "/usr/share/efi/x86_64",
///     "/boot/efi",
///     "EFI/systemd",
///     None,
///     None,
/// ).is_ok();
///
/// assert!(!installed, "Expected systemd-boot to not be detected as installed");
/// ```
pub fn command_is_installed(
    snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    filename: Option<PathBuf>,
    override_prefix: Option<&Path>,
) -> Result<bool, String> {
    let bootloader_name =
        bootloader_name(snapshot, firmware_arch, override_prefix).map_err(|e| {
            format!(
                "Is Installed - Can't determine possible bootloader name: {}",
                e
            )
        })?;
    let result = is_installed(
        snapshot,
        firmware_arch,
        shimdir,
        boot_root,
        boot_dst,
        filename,
        override_prefix,
    );
    match result {
        Ok(bool) => {
            if bool {
                let message = format!(
                    "Bootloader '{}' was installed using this tool",
                    bootloader_name
                );
                log_info(&message, 0);
                Ok(true)
            } else {
                let message = format!(
                    "Bootloader '{}' was not installed using this tool",
                    bootloader_name
                );
                log_info(&message, 0);
                Ok(false)
            }
        }
        Err(e) => {
            let message = format!("Can't determine if bootloader is installed: {}", e);
            Err(message)
        }
    }
}

/// Executes the `install` command to set up the bootloader.
///
/// This command facilitates the installation of a bootloader, supporting configurations for various architectures and boot environments.
/// It utilizes several internal functions to perform tasks such as copying necessary files and updating configurations.
///
/// # Arguments
///
/// * `snapshot` - An optional snapshot identifier for systems utilizing Btrfs snapshots.
/// * `firmware_arch` - The architecture of the system's firmware.
/// * `shimdir` - The directory containing bootloader shim files.
/// * `boot_root` - The root directory of the boot partition.
/// * `boot_dst` - The destination directory within the boot partition for bootloader files.
/// * `entry_token` - A unique token associated with the bootloader entry.
/// * `arg_no_variables` - Flag indicating whether EFI variables should be skipped.
/// * `arg_no_random_seed` - Flag indicating whether the random seed should be skipped.
/// * `override_prefix` - Optional path prefix for operations in a different filesystem root, such as a chroot environment.
///
/// # Returns
///
/// - `Ok(true)` upon successful execution of the install command.
/// - `Err(String)` with an error message if the installation fails.
///
/// # Errors
///
/// Errors may occur due to issues with copying files, updating configurations, or interacting with the EFI system partition.
///
/// # Examples
///
/// ```
/// let result = sdbootutil::command_install(
///     Some(0),
///     "x64",
///     "/usr/share/efi/x86_64",
///     "/boot/efi",
///     "EFI/systemd",
///     "opensuse-tumbleweed".to_string(),
///     false,
///     true,
///     None,
///     None
/// );
/// assert!(result.is_err(), "Expected an error from command_install");
/// ```
pub fn command_install(
    snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    entry_token: String,
    arg_no_variables: bool,
    arg_no_random_seed: bool,
    filename: Option<PathBuf>,
    override_prefix: Option<&Path>,
) -> Result<bool, String> {
    let bootloader_name = bootloader_name(snapshot, firmware_arch, override_prefix)
        .map_err(|e| format!("Install - Can't determine bootloader name: {}", e))?;
    let result_installed = is_installed(
        snapshot,
        firmware_arch,
        shimdir,
        boot_root,
        boot_dst,
        filename,
        override_prefix,
    );
    match result_installed {
        Ok(bool) => {
            if bool {
                let message = format!(
                    "Bootloader '{}' is already installed",
                    bootloader_name
                );
                log_info(&message, 0);
                Ok(false)
            } else {
                let message = format!(
                    "Bootloader '{}' was not installed using this tool",
                    bootloader_name
                );
                log_info(&message, 1);
                let result = install_bootloader(
                    snapshot,
                    firmware_arch,
                    shimdir,
                    boot_root,
                    boot_dst,
                    entry_token,
                    arg_no_variables,
                    arg_no_random_seed,
                    override_prefix,
                );
                match result {
                    Ok(()) => {
                        let message = format!(
                            "Bootloader '{}' successfully installed",
                            bootloader_name
                        );
                        log_info(&message, 0);
                        Ok(true)
                    }
                    Err(e) => {
                        let message = format!("Bootloader could not be installed: {}", e);
                        Err(message)
                    }
                }
            }
        }
        Err(e) => {
            let message = format!("Can't determine if bootloader is installed: {}", e);
            Err(message)
        }
    }

    //THIS STILL NEEDS TO UPDATE PREDICTIONS!!
}

/// Executes the `NeedsUpdate` command to check if the bootloader needs an update.
///
/// This command invokes the `bootloader_needs_update` function to determine if the installed bootloader is outdated
/// compared to the system's bootloader version. It logs the result of the operation, indicating whether an update is necessary.
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
/// Returns `Ok(true)` if the bootloader needs an update.
/// Returns `Ok(false)` if the bootloader is up-to-date.
/// Returns `Err(String)` with an error message if the operation fails, such as when the bootloader version cannot be determined.
///
/// # Examples
///
/// ```
/// let result = sdbootutil::command_needs_update(
///     Some(0),
///     Some(0),
///     "x64",
///     "/usr/share/efi/x86_64",
///     "/boot/efi",
///     "EFI/systemd",
///     None,
/// );
/// assert!(result.is_err(), "Expected an error from command_needs_update");
/// ```
pub fn command_needs_update(
    snapshot: Option<u64>,
    root_snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    override_prefix: Option<&Path>,
) -> Result<bool, String> {
    let bootloader_name = bootloader_name(snapshot, firmware_arch, override_prefix)
        .map_err(|e| format!("Needs Update - Can't determine bootloader name: {}", e))?;
    let result = bootloader_needs_update(
        snapshot,
        root_snapshot,
        firmware_arch,
        shimdir,
        boot_root,
        boot_dst,
        override_prefix,
    );
    match result {
        Ok(bool) => {
            if bool {
                let message = format!("Bootloader '{}' needs to be updated", bootloader_name);
                log_info(&message, 0);
                Ok(true)
            } else {
                let message = format!("Bootloader '{}' is up-to-date", bootloader_name);
                log_info(&message, 0);
                Ok(false)
            }
        }
        Err(e) => {
            let message = format!("Can't determine if bootloader needs update: {}", e);
            Err(message)
        }
    }
}

/// Attempts to update the system bootloader if it's determined to be outdated.
///
/// This function checks if the bootloader requires an update based on the current system
/// configuration and the specified snapshot. If an update is necessary, it will proceed
/// to install the new bootloader configuration.
///
/// # Arguments
///
/// * `snapshot` - An optional snapshot identifier for systems utilizing Btrfs snapshots.
/// * `root_snapshot` - The current root snapshot, used for determining the bootloader configuration.
/// * `firmware_arch` - The system's firmware architecture.
/// * `shimdir` - The directory containing EFI shim binaries.
/// * `boot_root` - The mount point of the boot partition.
/// * `boot_dst` - The directory within the boot partition where bootloader files are located.
/// * `entry_token` - A unique identifier for the bootloader entry.
/// * `arg_no_variables` - Flag indicating whether to skip updating EFI variables.
/// * `arg_no_random_seed` - Flag indicating whether to skip updating the random seed file.
/// * `override_prefix` - Optional path used to override the system root, useful in chroot environments.
///
/// # Returns
///
/// - `Ok(true)` if the bootloader was successfully updated.
/// - `Ok(false)` if the bootloader is already up-to-date and does not require an update.
/// - `Err(String)` with an error message if the update process fails.
///
/// # Errors
///
/// Errors can arise from the inability to determine the bootloader status, failure in the installation process,
/// or issues related to accessing necessary files or directories.
///
/// # Examples
///
/// ```
/// let result = sdbootutil::command_update(
///     Some(0),
///     Some(0),
///     "x64",
///     "/usr/share/efi/x86_64",
///     "/boot/efi",
///     "EFI/systemd",
///     "opensuse-tumbleweed".to_string(),
///     false,
///     true,
///     None,
/// );
/// assert!(result.is_err(), "Expected an error from command_install");
/// ```
pub fn command_update(
    snapshot: Option<u64>,
    root_snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    entry_token: String,
    arg_no_variables: bool,
    arg_no_random_seed: bool,
    override_prefix: Option<&Path>,
) -> Result<bool, String> {
    let bootloader_name = bootloader_name(snapshot, firmware_arch, override_prefix)
        .map_err(|e| format!("Update - Can't determine bootloader name: {}", e))?;
    let result_needs_update = bootloader_needs_update(
        snapshot,
        root_snapshot,
        firmware_arch,
        shimdir,
        boot_root,
        boot_dst,
        override_prefix,
    );
    match result_needs_update {
        Ok(bool) => {
            if bool {
                let message = format!("Bootloader '{}' needs to be updated", bootloader_name);
                log_info(&message, 1);
                let result = install_bootloader(
                    snapshot,
                    firmware_arch,
                    shimdir,
                    boot_root,
                    boot_dst,
                    entry_token,
                    arg_no_variables,
                    arg_no_random_seed,
                    override_prefix,
                );
                match result {
                    Ok(()) => {
                        let message =
                            format!("Bootloader '{}' successfully updated", bootloader_name);
                        log_info(&message, 0);
                        Ok(true)
                    }
                    Err(e) => {
                        let message = format!("Bootloader could not be updated: {}", e);
                        Err(message)
                    }
                }
            } else {
                let message = format!(
                    "Bootloader '{}' is already up-to-date, no update needed",
                    bootloader_name
                );
                log_info(&message, 0);
                Ok(false)
            }
        }
        Err(e) => {
            let message = format!("Can't determine if bootloader needs update: {}", e);
            Err(message)
        }
    }
}

/// Forces an update of the system bootloader, regardless of its current version.
///
/// This function is used in scenarios where a forced reinstallation of the bootloader
/// is required, such as after a manual system modification or when recovering from a
/// corrupted bootloader configuration.
///
/// # Arguments
///
/// * `snapshot` - An optional snapshot identifier for systems utilizing Btrfs snapshots.
/// * `firmware_arch` - The system's firmware architecture.
/// * `shimdir` - The directory containing EFI shim binaries.
/// * `boot_root` - The mount point of the boot partition.
/// * `boot_dst` - The directory within the boot partition where bootloader files are located.
/// * `entry_token` - A unique identifier for the bootloader entry.
/// * `arg_no_variables` - Flag indicating whether to skip updating EFI variables.
/// * `arg_no_random_seed` - Flag indicating whether to skip updating the random seed file.
/// * `filename` - An optional parameter specifying a custom bootloader filename, used in advanced configurations.
/// * `override_prefix` - Optional path used to override the system root, useful in chroot environments.
///
/// # Returns
///
/// - `Ok(true)` if the bootloader was successfully force-updated.
/// - `Ok(false)` if the bootloader is not installed using this tool and cannot be force-updated.
/// - `Err(String)` with an error message if the force update process fails.
///
/// # Errors
///
/// Errors may occur if the tool cannot verify the current bootloader installation status,
/// if the bootloader installation fails, or if there are issues with accessing necessary files or directories.
///
/// # Examples
///
/// ```
/// let result = sdbootutil::command_force_update(
///     Some(0),
///     "x64",
///     "/usr/share/efi/x86_64",
///     "/boot/efi",
///     "EFI/systemd",
///     "opensuse-tumbleweed".to_string(),
///     false,
///     true,
///     None,
///     None
/// );
/// assert!(result.is_err(), "Expected an error from command_force_update");
/// ```
pub fn command_force_update(
    snapshot: Option<u64>,
    firmware_arch: &str,
    shimdir: &str,
    boot_root: &str,
    boot_dst: &str,
    entry_token: String,
    arg_no_variables: bool,
    arg_no_random_seed: bool,
    filename: Option<PathBuf>,
    override_prefix: Option<&Path>,
) -> Result<bool, String> {
    let bootloader_name = bootloader_name(snapshot, firmware_arch, override_prefix)
        .map_err(|e| format!("Force Update - Can't determine bootloader name: {}", e))?;
    let result_installed = is_installed(
        snapshot,
        firmware_arch,
        shimdir,
        boot_root,
        boot_dst,
        filename,
        override_prefix,
    );
    match result_installed {
        Ok(bool) => {
            if bool {
                let message = format!(
                    "Bootloader '{}' was installed using this tool",
                    bootloader_name
                );
                log_info(&message, 1);
                let result = install_bootloader(
                    snapshot,
                    firmware_arch,
                    shimdir,
                    boot_root,
                    boot_dst,
                    entry_token,
                    arg_no_variables,
                    arg_no_random_seed,
                    override_prefix,
                );
                match result {
                    Ok(()) => {
                        let message = format!(
                            "Bootloader '{}' successfully force-updated",
                            bootloader_name
                        );
                        log_info(&message, 0);
                        Ok(true)
                    }
                    Err(e) => {
                        let message = format!("Bootloader could not be force-updated: {}", e);
                        Err(message)
                    }
                }
            } else {
                let message = format!(
                    "Bootloader '{}' isn't installed and can't be force-updated",
                    bootloader_name
                );
                log_info(&message, 1);
                Ok(false)
            }
        }
        Err(e) => {
            let message = format!("Can't determine if bootloader is installed: {}", e);
            Err(message)
        }
    }
}

/// Executes the `UpdatePredictions` command.
///
/// This function logs the action and returns a predefined status code.
///
/// # Returns
///
/// Always returns `18`, indicating a specific status after execution.
///
/// # Examples
///
/// ```
/// let status = sdbootutil::command_update_predictions().unwrap();
/// assert_eq!(status, true);
/// ```
pub fn command_update_predictions() -> Result<bool, String> {
    let message = "UpdatePredictions command called";
    log_info(message, 1);
    Ok(true)
}

/// Processes command-line arguments and gathers essential system information required for bootloader management.
///
/// This function consolidates various system details crucial for configuring and managing the bootloader.
/// It covers aspects such as filesystem type, snapshot information, boot configurations, and user-specified options.
///
/// # Arguments
///
/// * `override_prefix` - Optional `Path` reference to override the default system root,
/// useful in scenarios like chroot environments or testing.
///
/// # Returns
///
/// - `Ok((Option<u64>, Option<String>, Option<String>, String, String, String, Option<u64>, String,
/// String, String, String, bool, bool, bool, bool, String, Option<cli::Commands>))`
/// containing detailed system information and user options.
/// - `Err(String)` with an error message if any part of the information gathering or argument processing fails.
///
/// # Errors
///
/// Errors can arise from insufficient permissions, failure to read system configurations, or invalid command-line arguments.
///
/// # Example
///
/// ```no_run
/// let system_info = sdbootutil::process_args_and_get_system_info(None)
///     .expect("Failed to gather system information");
///
/// let (
///     root_snapshot,
///     root_prefix,
///     root_subvol,
///     root_uuid,
///     root_device,
///     firmware_arch,
///     snapshot,
///     entry_token,
///     boot_root,
///     boot_dst,
///     image,
///     no_variables,
///     regenerate_initrd,
///     no_random_seed,
///     all,
///     shimdir,
///     cmd
/// ) = system_info;
/// ```
/// Note that this function should be used with care, as it relies on obtaining root privileges
/// and accessing various system paths and configuration details.
pub fn process_args_and_get_system_info(
    override_prefix: Option<&Path>,
) -> Result<
    (
        Option<u64>,
        Option<String>,
        Option<String>,
        String,
        String,
        String,
        Option<u64>,
        String,
        String,
        String,
        String,
        bool,
        bool,
        bool,
        bool,
        String,
        Option<cli::Commands>,
    ),
    String,
> {
    if let Err(e) = cli::ensure_root_permissions(override_prefix) {
        let message = format!("Failed to get root privileges: {}", e);
        return Err(message);
    }

    let args = cli::parse_args();

    set_systemd_log_level(args.verbosity, override_prefix);

    if let Some(ref path) = args.esp_path {
        std::env::set_var("SYSTEMD_ESP_PATH", path);
    }

    let prefix_str = override_prefix.and_then(|pref| pref.to_str());
    let has_snapshots = fs::is_snapshotted(prefix_str)
        .map_err(|e| format!("Couldn't find out if snapshotted: {}", e))?;
    let (default_firmware_arch, default_entry_token, boot_root) =
        io::get_bootctl_info(override_prefix)
            .map_err(|e| format!("Couldn't get bootctl info: {}", e))?;
    let (root_uuid, root_device) = io::get_findmnt_output("/", override_prefix)
        .map_err(|e| format!("Couldn't get root filesystem info: {}", e))?;
    let (root_snapshot, root_prefix, root_subvol) = if has_snapshots {
        fs::get_root_snapshot_info(override_prefix)
            .map(|(prefix, snapshot_id, full_path)| {
                (Some(snapshot_id), Some(prefix), Some(full_path))
            })
            .map_err(|e| format!("Failed to get root snapshot info: {}", e))?
    } else {
        (None, None, None)
    };
    let shimdir = fs::get_shimdir();

    let firmware_arch = args.arch.unwrap_or(default_firmware_arch);
    let image = match args.image {
        Some(ref img) => img.clone(),
        None => match firmware_arch.as_str() {
            "x64" => "vmlinuz".to_string(),
            "aa64" => "Image".to_string(),
            _ => {
                return Err(format!(
                    "Unsupported architecture '{}'. Supported are: x64, aa64",
                    firmware_arch
                ))
            }
        },
    };
    let snapshot = args.snapshot.or(root_snapshot);
    let arg_entry_token = args.entry_token.unwrap_or(default_entry_token);
    let no_variables = args.no_variables;
    let regenerate_initrd = args.regenerate_initrd;
    let no_random_seed = args.no_random_seed;
    let all = args.all;
    let boot_dst = match fs::determine_boot_dst(root_snapshot, &firmware_arch, override_prefix) {
        Ok(dst) => dst.to_string(),
        Err(e) => {
            let message = format!("Failed to determine boot_dst: {}", e);
            return Err(message);
        }
    };

    if let Some(ref esp_path) = args.esp_path {
        if esp_path != &boot_root {
            return Err("ESP paths don't match".to_string());
        }
    }

    Ok((
        root_snapshot,
        root_prefix,
        root_subvol,
        root_uuid,
        root_device,
        firmware_arch,
        snapshot,
        arg_entry_token,
        boot_root,
        boot_dst,
        image,
        no_variables,
        regenerate_initrd,
        no_random_seed,
        all,
        shimdir,
        args.cmd,
    ))
}

/// only for demonstration purposes
pub fn test_functions() {
    let (entry_token, machine_id, os_release_id, os_release_version_id, os_release_pretty_name) =
        match settle_system_tokens(None, None, None, None) {
            Ok(info) => info,
            Err(_e) => (
                "".to_string(),
                "".to_string(),
                Some("".to_string()),
                Some("".to_string()),
                Some("".to_string()),
            ),
        };
    let message = format!("entry_token: {}, machine_id: {}, os_release_id: {}, os_release_version_id: {}, os_release_pretty_name: {}", entry_token, machine_id, os_release_id.unwrap_or_default(), os_release_version_id.unwrap_or_default(), os_release_pretty_name.unwrap_or_default());
    log_info(&message, 1);
    let (_temp_dir, _tmpdir_path) = fs::create_temp_dir();
    let mut rollback_items = vec![
        fs::RollbackItem::new(PathBuf::from("/path/to/file1")),
        fs::RollbackItem::new(PathBuf::from("/path/to/file2")),
    ];
    fs::cleanup_rollback_items(&rollback_items);
    fs::reset_rollback_items(&mut rollback_items);

    if fs::is_subvol_ro(None).expect("Failed to check if filesystem is ro") {
        log_info("It is ro", 1)
    } else {
        log_info("Subvol is not ro", 1)
    }
}

#[cfg(test)]
mod unit_tests;
