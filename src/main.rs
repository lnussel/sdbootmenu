use sdbootutil as lib;
use sdbootutil::cli::Commands;

fn main() -> Result<(), String> {
    let (
        root_snapshot,
        _root_prefix,
        _root_subvol,
        _root_uuid,
        _root_device,
        firmware_arch,
        snapshot,
        entry_token,
        boot_root,
        boot_dst,
        _image,
        no_variables,
        _regenerate_initrd,
        no_random_seed,
        _all,
        shimdir,
        cmd,
    ) = lib::process_args_and_get_system_info(None)
        .map_err(|e| format!("An error occurred while fetching system information: {}", e))?;

    lib::test_functions();

    let result = match cmd {
        Some(Commands::Kernels {}) => lib::command_kernels(),
        Some(Commands::Snapshots {}) => lib::command_snapshots(),
        Some(Commands::Entries {}) => lib::command_entries(),
        Some(Commands::Bootloader {}) => lib::command_bootloader(snapshot, &firmware_arch, None),
        Some(Commands::AddKernel { kernel_version }) => lib::command_add_kernel(&kernel_version),
        Some(Commands::AddAllKernels {}) => lib::command_add_all_kernels(),
        Some(Commands::Mkinitrd {}) => lib::command_mkinitrd(),
        Some(Commands::RemoveKernel { kernel_version }) => {
            lib::command_remove_kernel(&kernel_version)
        }
        Some(Commands::RemoveAllKernels {}) => lib::command_remove_all_kernels(),
        Some(Commands::ListKernels {}) => lib::command_list_kernels(),
        Some(Commands::ListEntries {}) => lib::command_list_entries(),
        Some(Commands::ListSnapshots {}) => lib::command_list_snapshots(),
        Some(Commands::SetDefaultSnapshot {}) => lib::command_set_default_snapshot(),
        Some(Commands::IsBootable {}) => lib::command_is_bootable(),
        Some(Commands::IsInstalled {}) => lib::command_is_installed(
            snapshot,
            &firmware_arch,
            &shimdir,
            &boot_root,
            &boot_dst,
            None,
            None,
        ),
        Some(Commands::Install {}) => lib::command_install(
            snapshot,
            &firmware_arch,
            &shimdir,
            &boot_root,
            &boot_dst,
            entry_token,
            no_variables,
            no_random_seed,
            None,
            None,
        ),
        Some(Commands::NeedsUpdate {}) => lib::command_needs_update(
            snapshot,
            root_snapshot,
            &firmware_arch,
            &shimdir,
            &boot_root,
            &boot_dst,
            None,
        ),
        Some(Commands::Update {}) => lib::command_update(
            snapshot,
            root_snapshot,
            &firmware_arch,
            &shimdir,
            &boot_root,
            &boot_dst,
            entry_token,
            no_variables,
            no_random_seed,
            None,
        ),
        Some(Commands::ForceUpdate {}) => lib::command_force_update(
            snapshot,
            &firmware_arch,
            &shimdir,
            &boot_root,
            &boot_dst,
            entry_token,
            no_variables,
            no_random_seed,
            None,
            None,
        ),
        Some(Commands::UpdatePredictions {}) => lib::command_update_predictions(),
        None => lib::ui::show_main_menu(),
    };
    match result {
        Ok(_value) => Ok(()),
        Err(e) => {
            let message = format!("Command failed: {}", e);
            Err(message)
        }
    }
}
