use clap::{ArgAction, Parser, Subcommand};
use std::path::Path;

/// Validates that a given string is not empty.
///
/// This function is intended to be used as a value parser for command line arguments,
/// ensuring that certain arguments are provided with non-empty values.
///
/// # Arguments
///
/// * `s` - A string slice to validate.
///
/// # Returns
///
/// If the string is non-empty, it returns `Ok` with the original string converted to a `String`.
/// If the string is empty, it returns an `Err` with a message indicating that the value cannot be empty.
pub(crate) fn non_empty_string(s: &str) -> Result<String, &'static str> {
    if s.is_empty() {
        Err("Value cannot be empty")
    } else {
        Ok(s.to_string())
    }
}

/// Tool to manage systemd-boot in a btrfs based, snapper managed system
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Represents command-line arguments parsed for the application.
///
/// This struct is derived from `clap::Parser`, which automates the process of parsing
/// command line arguments based on the struct's fields and their attributes.
///
/// # Fields
///
/// * `snapshot` - Optional u64 value to manually specify a snapshot.
/// * `esp_path` - Optional string to manually specify the path to the ESP.
/// * `arch` - Optional string to manually set the architecture.
/// * `entry_token` - Optional string to override the entry token.
/// * `image` - Optional string to specify the Linux kernel file name.
/// * `no_variables` - Boolean flag to disable updating UEFI variables.
/// * `regenerate_initrd` - Boolean flag to force regeneration of initrd.
/// * `verbosity` - u8 value to control the verbosity of the output.
/// * `cmd` - Optional subcommand to specify the action to be performed.
pub struct Args {
    /// Manually specify snapshot
    #[arg(short, long)]
    pub(crate) snapshot: Option<u64>,

    /// Manually specify path to ESP
    #[arg(short = 'p', long = "esp-path", value_parser = non_empty_string)]
    pub(crate) esp_path: Option<String>,

    /// Manually set architecture
    #[arg(short, long, value_parser = non_empty_string)]
    pub(crate) arch: Option<String>,

    /// Override entry token
    #[arg(short = 't', long = "entry-token", value_parser = non_empty_string)]
    pub(crate) entry_token: Option<String>,

    /// Specify Linux kernel file name
    #[arg(short, long, value_parser = non_empty_string)]
    pub(crate) image: Option<String>,

    /// Do not update UEFI variables
    #[arg(short = 'n', long = "no-variables")]
    pub(crate) no_variables: bool,

    /// Always regenerate initrd
    #[arg(short = 'r', long = "regenerate-initrd")]
    pub(crate) regenerate_initrd: bool,

    /// Do not use random seed
    #[arg(short = 'S', long = "no-random-seed")]
    pub(crate) no_random_seed: bool,

    /// More verbose output
    #[arg(short, long, action = ArgAction::Count)]
    pub(crate) verbosity: u8,

    /// Show all entries
    #[arg(short = 'A', long = "all")]
    pub(crate) all: bool,

    #[command(subcommand)]
    pub(crate) cmd: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
/// Enumerates the available subcommands and their associated actions.
///
/// Each variant corresponds to a different subcommand that the user can invoke,
/// along with any parameters specific to that subcommand.
///
/// # Variants
///
/// * `Kernels` - Opens the kernel menu.
/// * `Snapshots` - Opens the snapshots menu.
/// * `Entries` - Opens the entry menu.
/// * `Bootloader` - Prints the detected bootloader.
/// * `AddKernel` - Creates a boot entry for a specified kernel version.
/// * `AddAllKernels` - Creates boot entries for all kernels in a snapshot.
/// * `Mkinitrd` - Regenerates initrd for all kernels in a snapshot.
/// * `RemoveKernel` - Removes a boot entry for a specified kernel version.
/// * `RemoveAllKernels` - Removes all boot entries related to a snapshot.
/// * `ListKernels` - Lists all kernels related to a snapshot.
/// * `ListEntries` - Lists all boot entries related to a snapshot.
/// * `ListSnapshots` - Lists all snapshots.
/// * `SetDefaultSnapshot` - Sets a snapshot as the default for the next boot.
/// * `IsBootable` - Checks whether a snapshot is potentially bootable.
/// * `Install` - Installs systemd-boot and shim into the ESP.
/// * `NeedsUpdate` - Checks if the bootloader in the ESP needs updating.
/// * `Update` - Updates the bootloader if it is outdated.
/// * `ForceUpdate` - Forces an update of the bootloader.
/// * `UpdatePredictions` - Updates TPM2 predictions.
pub enum Commands {
    /// [UI] Open kernel menu
    Kernels {},

    /// [UI] Open snapshots menu
    Snapshots {},

    /// [UI] Open entry menu
    Entries {},

    /// Print the detected bootloader
    Bootloader {},

    /// Create boot entry for specified kernel
    #[command(name = "add-kernel")]
    AddKernel { kernel_version: String },

    /// Create boot entries for all kernels in SNAPSHOT
    #[command(name = "add-all-kernels")]
    AddAllKernels {},

    /// Create boot entries for all kernels  in SNAPSHOT,
    /// sets --regenerate-initrd
    Mkinitrd {},

    /// Remove boot entry for specified kernel in SNAPSHOT
    #[command(name = "remove-kernel")]
    RemoveKernel { kernel_version: String },

    /// Remove boot entries for all kernels in SNAPSHOT
    #[command(name = "remove-all-kernels")]
    RemoveAllKernels {},

    /// List all kernels related to SNAPSHOT
    #[command(name = "list-kernels")]
    ListKernels {},

    /// List all entries related to SNAPSHOT
    #[command(name = "list-entries")]
    ListEntries {},

    /// List all snapshots
    #[command(name = "list-snapshots")]
    ListSnapshots {},

    /// Make SNAPSHOT the default for next boot and install all kernels if needed
    #[command(name = "set-default-snapshot")]
    SetDefaultSnapshot {},

    /// Check whether SNAPSHOT has any kernels registered, ie is potentially bootable
    #[command(name = "is-bootable")]
    IsBootable {},

    /// Checks if systemd-boot has been installed using this tool
    IsInstalled {},

    /// Install systemd-boot and shim into ESP
    Install {},

    /// Check whether the bootloader in ESP needs updating
    #[command(name = "needs-update")]
    NeedsUpdate {},

    /// Update the bootloader if it's old
    Update {},

    /// Update the bootloader in any case
    #[command(name = "force-update")]
    ForceUpdate {},

    /// Update TPM2 predictions
    #[command(name = "update-predictions")]
    UpdatePredictions {},
}

/// Parses command-line arguments and returns an `Args` struct representing the parsed values.
///
/// This function relies on `clap::Parser` to automatically parse the arguments
/// based on the `Args` struct definition and return an instance of `Args`.
///
/// # Returns
///
/// An `Args` struct containing the parsed command-line arguments.
pub fn parse_args() -> Args {
    Args::parse()
}

pub(crate) fn ensure_root_permissions(
    override_prefix: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    if override_prefix.is_some() {
        return Ok(());
    }
    match elevate::check() {
        elevate::RunningAs::Root | elevate::RunningAs::Suid => Ok(()),
        _ => {
            elevate::escalate_if_needed()?;
            Ok(())
        }
    }
}
