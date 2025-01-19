use super::cli;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Logs an informational message to the console based on the specified verbosity level.
///
/// This function compares the application's current verbosity level with the provided `log_verbosity`
/// parameter. If the current verbosity level is equal to or higher than `log_verbosity`, the message
/// is printed to the standard output. This allows for granular control over which messages are displayed
/// under different verbosity settings.
///
/// # Arguments
///
/// * `message` - A string slice representing the message to be logged.
/// * `log_verbosity` - The verbosity level required for this message to be logged. A lower value
///   indicates higher importance; the message is logged if the application's verbosity level is
///   this value or higher.
///
pub(crate) fn log_info(message: &str, log_verbosity: u8) {
    let verbosity = cli::parse_args().verbosity;
    if verbosity >= log_verbosity {
        println!("{}", message);
    }
}

/// Executes a specified command with arguments and returns its output as a String.
///
/// # Arguments
///
/// * `command` - A string slice that holds the command to execute.
/// * `args` - A slice of string slices that holds the arguments to the command.
///
/// # Returns
///
/// The command's output as a `String` if successful, or an `Error` if the command fails.
pub(crate) fn get_command_output(
    command: &str,
    args: &[&str],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut command = Command::new(command);

    command.args(args);

    let output = command.output()?;
    let output_str = String::from_utf8(output.stdout)?.trim().to_string();

    Ok(output_str)
}

/// Gathers bootloader information using the `bootctl` utility, including firmware architecture, entry token, and boot root.
///
/// This function runs `bootctl --no-pager` and parses its output to extract crucial bootloader configuration details.
/// It's useful for scripts or programs that need to interact with the system bootloader programmatically.
///
/// # Arguments
///
/// * `override_prefix` - An optional `Path` reference that overrides the system root for testing or operation in a chroot environment.
///
/// # Returns
///
/// - `Ok((String, String, String))` containing the firmware architecture, bootloader entry token, and boot root path, respectively.
/// - `Err(String)` with an error message if the command fails or the expected output is not found.
///
/// # Errors
///
/// Errors can occur due to failed command execution or missing information in the command output.
pub(crate) fn get_bootctl_info(
    override_prefix: Option<&Path>,
) -> Result<(String, String, String), String> {
    if override_prefix.is_some() {
        return Ok((
            "x64".to_string(),
            "entry_token".to_string(),
            override_prefix.unwrap().to_string_lossy().to_string(),
        ));
    }
    let output = get_command_output("bootctl", &["--no-pager"])
        .map_err(|e| format!("Bootctl call failed: {}", e))?;

    #[derive(Default)]
    struct BootctlInfo {
        firmware_arch: Option<String>,
        entry_token: Option<String>,
        boot_root: Option<String>,
    }

    let result = output
        .lines()
        .fold(BootctlInfo::default(), |mut acc, line| {
            if acc.firmware_arch.is_none() && line.contains("Firmware Arch: ") {
                acc.firmware_arch = line.split("Firmware Arch: ").nth(1).map(str::to_string);
            } else if acc.entry_token.is_none() && line.contains("token: ") {
                acc.entry_token = line.split("token: ").nth(1).map(str::to_string);
            } else if acc.boot_root.is_none() && line.contains("$BOOT: ") {
                let start_index = line.find("$BOOT: ").unwrap() + "$BOOT: ".len();
                let end_index = line[start_index..]
                    .find(' ')
                    .map_or(line.len(), |i| start_index + i);
                acc.boot_root = Some(line[start_index..end_index].to_string());
            }
            acc
        });

    let firmware_arch = result
        .firmware_arch
        .ok_or_else(|| "Firmware Arch not found".to_string())?;
    let entry_token = result
        .entry_token
        .ok_or_else(|| "Entry token not found".to_string())?;
    let boot_root = result
        .boot_root
        .ok_or_else(|| "Boot root not found".to_string())?;

    Ok((firmware_arch, entry_token, boot_root))
}

/// Retrieves the UUID and source device for a given mount point using `findmnt`.
///
/// This function is particularly useful for scripts or systems that need to work with UUIDs and device paths,
/// ensuring operations are performed on the correct filesystems.
///
/// # Arguments
///
/// * `mount_point` - A string slice that specifies the filesystem mount point to query, such as "/" for the root filesystem.
/// * `override_prefix` - An optional `Path` reference for use in an alternative filesystem hierarchy, such as within a chroot environment.
///
/// # Returns
///
/// - `Ok((String, String))` with the UUID and source device of the specified mount point.
/// - `Err(String)` with a description of the error if the operation fails.
///
/// # Errors
///
/// Errors may arise from command execution failure or parsing issues with the command's output.
pub(crate) fn get_findmnt_output(
    mount_point: &str,
    override_prefix: Option<&Path>,
) -> Result<(String, String), String> {
    if override_prefix.is_some() {
        return Ok(("123456789".to_string(), "sda1".to_string()));
    }
    let output = get_command_output(
        "findmnt",
        &[mount_point, "-v", "-r", "-n", "-o", "UUID,SOURCE"],
    )
    .map_err(|e| format!("findmnt call failed: {}", e))?;

    let mut parts = output.split_whitespace();
    let mount_uuid = parts.next().ok_or("UUID not found")?.to_string();
    let mount_device = parts.next().ok_or("Device not found")?.to_string();

    Ok((mount_uuid, mount_device))
}

/// Creates an EFI boot entry using `efibootmgr`.
///
/// This function attempts to create a new EFI boot entry for the system. It constructs
/// the necessary arguments for `efibootmgr` based on the provided drive, partition number,
/// and the path to the bootloader entry. If an `override_prefix` is provided or an entry
/// already exists, the function skips the boot entry creation and returns success immediately.
///
/// # Arguments
///
/// * `drive` - A reference to a `PathBuf` that specifies the drive (e.g., `/dev/sda`).
/// * `partno` - The partition number as a `u32`.
/// * `entry` - A reference to a `PathBuf` that specifies the path to the bootloader entry.
/// * `override_prefix` - An optional reference to a `Path` that, if provided, overrides the default behavior.
///
/// # Returns
///
/// Returns `Ok(())` on success or an `Err` containing a `String` error message if the operation fails.
///
/// # Errors
///
/// Returns an error if the `efibootmgr` command fails or if any provided path arguments are invalid.
pub(crate) fn create_efi_boot_entry(
    drive: &PathBuf,
    partno: u32,
    entry: &PathBuf,
    override_prefix: Option<&Path>,
) -> Result<(), String> {
    if override_prefix.is_some() {
        return Ok(());
    }

    let efibootmgr_output = get_command_output("efibootmgr", &[]).map_err(|e| e.to_string())?;

    if efibootmgr_output.contains("openSUSE Boot Manager") {
        log_info("EFI entry for openSUSE already exists, skipping...", 2);
        return Ok(());
    }

    let disk_arg = format!("--disk={}", drive.to_string_lossy());
    let part_arg = format!("--part={}", partno);
    let loader_arg = format!("--loader={}", entry.to_string_lossy());

    let _output = get_command_output(
        "efibootmgr",
        &[
            "-q",
            "--create",
            &disk_arg,
            &part_arg,
            "--label=openSUSE Boot Manager",
            &loader_arg,
        ],
    )
    .map_err(|e| e.to_string())?;
    log_info("Created EFI boot entry for openSUSE", 2);

    Ok(())
}

/// Sets the systemd log level based on the provided verbosity level.
///
/// This function sets the `SYSTEMD_LOG_LEVEL` environment variable to control logging verbosity.
/// It checks if the environment variable is already set before applying any changes. If an override prefix
/// is provided, the function will return early without changing the log level. The log level is set to "debug"
/// for verbosity levels greater than 1, otherwise it is set to "info" for any positive verbosity level.
///
/// # Arguments
///
/// * `verbosity` - A `u8` that specifies the verbosity level:
///   - `0`: Do nothing.
///   - `1`: Set log level to "info".
///   - `>1`: Set log level to "debug".
/// * `override_prefix` - An optional reference to a `Path` that, if provided, bypasses setting the log level.
pub(crate) fn set_systemd_log_level(verbosity: u8, override_prefix: Option<&Path>) {
    if override_prefix.is_some() {
        return;
    }
    if env::var("SYSTEMD_LOG_LEVEL").is_err() && verbosity >= 1 {
        let log_level = if verbosity > 1 { "debug" } else { "info" };
        env::set_var("SYSTEMD_LOG_LEVEL", log_level);
    }
}
