use super::super::*;
use std::env::consts::ARCH;
use std::fs;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_command_kernels() {
    let result = command_kernels().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_snapshots() {
    let result = command_snapshots().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_entries() {
    let result = command_entries().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_bootloader_only_systemd_boot_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    assert!(command_bootloader(Some(0), "x64", Some(temp_dir.path())).unwrap());
}

#[test]
fn test_command_bootloader_systemd_boot_and_grub2_exist() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let grub2_efi_path = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));

    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for GRUB2 EFI file");

    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&grub2_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    assert!(!command_bootloader(Some(0), "x64", Some(temp_dir.path())).unwrap());
}

#[test]
fn test_command_bootloader_only_grub2_exist() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));

    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for GRUB2 EFI file");

    File::create(&grub2_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    assert!(!command_bootloader(Some(0), "x64", Some(temp_dir.path())).unwrap());
}

#[test]
fn test_bootloader_name_neither_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    assert!(command_bootloader(Some(0), "x64", Some(temp_dir.path())).is_err());
}

#[test]
fn test_command_add_kernel() {
    let result = command_add_kernel("5.8.0-53-generic").unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_add_all_kernels() {
    let result = command_add_all_kernels().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_mkinitrd() {
    let result = command_mkinitrd().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_remove_kernel() {
    let result = command_remove_kernel("5.8.0-53-generic").unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_remove_all_kernels() {
    let result = command_remove_all_kernels().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_list_kernels() {
    let result = command_list_kernels().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_list_entries() {
    let result = command_list_entries().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_list_snapshots() {
    let result = command_list_snapshots().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_set_default_snapshot() {
    let result = command_set_default_snapshot().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_is_bootable() {
    let result = command_is_bootable().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_command_is_installed_true() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    let systemd_boot_path = snapshot_dir.join("EFI").join("systemd");
    let is_installed_file_path = temp_dir.path().join("installed_by_sdbootutil");
    fs::create_dir_all(&is_installed_file_path.parent().unwrap())
        .expect("Failed to create installed path");
    fs::create_dir_all(&systemd_boot_path).expect("Failed to create systemd-boot path");
    let systemd_boot_test_file = systemd_boot_path.join("systemd-bootx64.efi");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    File::create(&is_installed_file_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    let is_installed = command_is_installed(
        Some(0),
        "x64",
        "",
        "",
        "",
        Some(systemd_boot_test_file),
        Some(temp_dir.path()),
    )
    .unwrap();
    assert_eq!(is_installed, true, "Expected is_installed to return true")
}

#[test]
fn test_command_is_installed_false_bootloader() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    let systemd_boot_path = snapshot_dir.join("EFI").join("systemd");
    let is_installed_file_path = temp_dir.path().join("installed_by_sdbootutil");
    fs::create_dir_all(&is_installed_file_path.parent().unwrap())
        .expect("Failed to create installed path");
    fs::create_dir_all(&systemd_boot_path).expect("Failed to create systemd-boot path");
    let systemd_boot_test_file = systemd_boot_path.join("systemd-bootx64.efi");
    File::create(&systemd_boot_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&is_installed_file_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    let is_installed = command_is_installed(
        Some(0),
        "x64",
        "",
        "",
        "",
        Some(systemd_boot_test_file),
        Some(temp_dir.path()),
    )
    .unwrap();
    assert_eq!(is_installed, false, "Expected is_installed to return false")
}

#[test]
fn test_command_is_installed_false_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    let systemd_boot_path = snapshot_dir.join("EFI").join("systemd");
    let is_installed_file_path = temp_dir.path().join("installed_by_sdbootutil");
    fs::create_dir_all(&is_installed_file_path.parent().unwrap())
        .expect("Failed to create installed path");
    fs::create_dir_all(&systemd_boot_path).expect("Failed to create systemd-boot path");
    let systemd_boot_test_file = systemd_boot_path.join("systemd-bootx64.efi");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");

    let is_installed = command_is_installed(
        Some(0),
        "x64",
        "",
        "",
        "",
        Some(systemd_boot_test_file),
        Some(temp_dir.path()),
    )
    .unwrap();
    assert_eq!(is_installed, false, "Expected is_installed to return false")
}

#[test]
fn test_command_install_shim_sdboot() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";
    let mock_bootloader_content = b"mock bootloader content";

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(mock_bootloader_content)
        .unwrap();

    let temp_shimdir = snapshot_dir.join("usr/share/efi/x64");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mock_mok_manager_content = mock_mok_manager_content;
    let mock_shim_content = mock_shim_content;

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_install(
        Some(0),
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        None,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let full_boot_root = override_root.path().join("boot/efi");
    assert!(full_boot_root.join("loader/entries.srel").exists());
    let copied_content_entries = std::fs::read(full_boot_root.join("loader/entries.srel")).unwrap();
    assert_eq!(copied_content_entries, b"type1");
    assert!(full_boot_root.join("loader/loader.conf").exists());
    let copied_content_loader = std::fs::read(full_boot_root.join("loader/loader.conf")).unwrap();
    assert_eq!(copied_content_loader, b"#timeout 3\n#console-mode keep\n");

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_mok_manager_efi = temp_boot_dst.join("MokManager.efi");
    let copied_shim_efi = temp_boot_dst.join("shim.efi");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(copied_mok_manager_efi.exists());
    let copied_content = std::fs::read(&copied_mok_manager_efi).unwrap();
    assert_eq!(copied_content, mock_mok_manager_content);

    assert!(copied_shim_efi.exists());
    let copied_content = std::fs::read(&copied_shim_efi).unwrap();
    assert_eq!(copied_content, mock_shim_content);

    assert!(copied_bootloader.exists());
    let copied_content = std::fs::read(&copied_bootloader).unwrap();
    assert_eq!(copied_content, mock_bootloader_content);

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(boot_csv.exists());
    let copied_boot_csv_content = std::fs::read(&boot_csv).unwrap();
    assert_eq!(
        String::from_utf8(copied_boot_csv_content).unwrap(),
        format!("{},openSUSE Boot Manager\n", "shim.efi")
    );

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(install_flag.exists());
    let install_flag_content = std::fs::read(&install_flag).unwrap();
    assert_eq!(
        String::from_utf8(install_flag_content).unwrap(),
        "opensuse-tumbleweed"
    );

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "opensuse-tumbleweed"
    );
}

#[test]
fn test_command_install_shim_grub2() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";
    let mock_bootloader_content = b"mock bootloader content";

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));

    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for GRUB2 EFI file");
    File::create(&grub2_efi_path)
        .unwrap()
        .write_all(mock_bootloader_content)
        .unwrap();

    let bli_mod_src = snapshot_dir.join("grub2moddir/bli.mod");
    fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
    fs::write(&bli_mod_src, "bli module content").unwrap();

    let temp_shimdir = snapshot_dir.join("usr/share/efi/x64");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mock_mok_manager_content = mock_mok_manager_content;
    let mock_shim_content = mock_shim_content;

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_install(
        Some(0),
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        None,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let full_boot_root = override_root.path().join("boot/efi");
    let grub_cfg_path = full_boot_root.join("EFI/opensuse").join("grub.cfg");
    assert!(grub_cfg_path.exists());
    assert_eq!(
        fs::read_to_string(grub_cfg_path).unwrap(),
        "timeout=8\nfunction load_video {\n  true\n}\ninsmod bli\nblscfg\n"
    );

    let efi_boot_grub_cfg_path = full_boot_root.join("EFI/BOOT/grub.cfg");
    assert!(efi_boot_grub_cfg_path.exists());
    assert_eq!(
        fs::read_to_string(efi_boot_grub_cfg_path).unwrap(),
        "timeout=8\nfunction load_video {\n  true\n}\ninsmod bli\nblscfg\n"
    );

    let mod_dir = full_boot_root
        .join("EFI/opensuse")
        .join(format!("{}-efi", ARCH));
    let bli_mod_dst = mod_dir.join("bli.mod");
    assert!(bli_mod_dst.exists());
    assert_eq!(
        fs::read_to_string(bli_mod_dst).unwrap(),
        "bli module content"
    );

    let temp_boot_root = override_root.path().join("boot/efi");
    let temp_boot_dst = temp_boot_root.as_path().join("EFI/opensuse");
    let copied_mok_manager_efi = temp_boot_dst.join("MokManager.efi");
    let copied_shim_efi = temp_boot_dst.join("shim.efi");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.exists());

    assert!(copied_mok_manager_efi.exists());
    let copied_content = std::fs::read(&copied_mok_manager_efi).unwrap();
    assert_eq!(copied_content, mock_mok_manager_content);

    assert!(copied_shim_efi.exists());
    let copied_content = std::fs::read(&copied_shim_efi).unwrap();
    assert_eq!(copied_content, mock_shim_content);

    assert!(copied_bootloader.exists());
    let copied_content = std::fs::read(&copied_bootloader).unwrap();
    assert_eq!(copied_content, mock_bootloader_content);

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(boot_csv.exists());
    let copied_boot_csv_content = std::fs::read(&boot_csv).unwrap();
    assert_eq!(
        String::from_utf8(copied_boot_csv_content).unwrap(),
        format!("{},openSUSE Boot Manager\n", "shim.efi")
    );

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(install_flag.exists());
    let install_flag_content = std::fs::read(&install_flag).unwrap();
    assert_eq!(
        String::from_utf8(install_flag_content).unwrap(),
        "opensuse-tumbleweed"
    );

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "opensuse-tumbleweed"
    );
}

#[test]
fn test_command_install_no_no_shim_grub2_no_snapshots() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path();

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_efi_path,
    )
    .expect("Failed to copy systemd-boot test file");
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"GNU GRUB  version %s\x002.12\x00prefixESC at any time exits.")
        .unwrap();

        let bli_mod_src = snapshot_dir.join("grub2moddir/bli.mod");
        fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
        fs::write(&bli_mod_src, "bli module content").unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let full_boot_root = override_root.path().join("boot/efi");
    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    std::fs::write(&install_flag, "opensuse-tumbleweed").unwrap();

    let result = command_install(
        None,
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        None,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);

    let full_boot_root = override_root.path().join("boot/efi");
    let grub_cfg_path = full_boot_root.join("EFI/opensuse").join("grub.cfg");
    assert!(!grub_cfg_path.exists());

    let efi_boot_grub_cfg_path = full_boot_root.join("EFI/BOOT/grub.cfg");
    assert!(!efi_boot_grub_cfg_path.exists());

    let mod_dir = full_boot_root.join("EFI/opensuse").join(format!("{}-efi", ARCH));
    let bli_mod_dst = mod_dir.join("bli.mod");
    assert!(!bli_mod_dst.exists());

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(!random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(!entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(copied_bootloader.exists());

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(!boot_csv.exists());

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(!kernel_dir.is_dir());

    assert!(install_flag.exists());

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(!entry_token_path.exists());
}

#[test]
fn test_command_install_no_shim_sdboot() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    let shim_test_file = snapshot_dir.join("usr/share/efi/x86_64/shim.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"#### LoaderInfo: systemd-boot 256.4+suse.17.gbe772961ad ####")
        .unwrap();

    let temp_shimdir = snapshot_dir.join("usr/share/efi/x86_64");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let full_boot_root = override_root.path().join("boot/efi");
    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    std::fs::write(&install_flag, "opensuse-tumbleweed").unwrap();

    let result = command_install(
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        None,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);

    assert!(!full_boot_root.join("loader/entries.srel").exists());
    assert!(!full_boot_root.join("loader/loader.conf").exists());

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(!random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(!entries_dir.is_dir());
    
    let copied_mok_manager_efi = temp_boot_dst.join("MokManager.efi");
    let copied_shim_efi = temp_boot_dst.join("shim.efi");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(!copied_mok_manager_efi.exists());

    assert!(!copied_shim_efi.exists());

    assert!(copied_bootloader.exists());

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(!boot_csv.exists());

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(!kernel_dir.is_dir());

    assert!(install_flag.exists());

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(!entry_token_path.exists());
}

#[test]
fn test_command_needs_update_no_systemd_boot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let systemd_boot_efi_file = temp_dir
        .path()
        .join("boot/efi/EFI/systemd/systemd-bootx64.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_efi_file,
    )
    .expect("Failed to copy systemd-boot efi file");

    let needs_update = command_needs_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        Some(temp_dir.path()),
    )
    .unwrap();
    assert!(!needs_update);
}

#[test]
fn test_command_needs_update_no_grub2() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let grub2 = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    let grub2_efi_file = temp_dir.path().join("boot/efi/EFI/systemd/grub.efi");
    fs::create_dir_all(grub2.parent().unwrap())
        .expect("Failed to create directory for grub2 EFI file");
    fs::create_dir_all(grub2_efi_file.parent().unwrap())
        .expect("Failed to create directory for grub2 EFI file");
    fs::copy(PathBuf::from("src/unit_tests/test_files/grub2.efi"), &grub2)
        .expect("Failed to copy grub2 test file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_efi_file,
    )
    .expect("Failed to copy grub2 efi file");

    let needs_update = command_needs_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        Some(temp_dir.path()),
    )
    .unwrap();
    assert!(!needs_update);
}

#[test]
fn test_command_needs_update_shim_systemd_boot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    let shim_test_file = snapshot_dir.join("usr/share/efi/x86_64/shim.efi");
    let grub_efi_file = temp_dir.path().join("boot/efi/EFI/systemd/grub.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    fs::create_dir_all(grub_efi_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&grub_efi_file)
        .unwrap()
        .write_all(b"#### LoaderInfo: systemd-boot 253.4+suse.17.gbe772961ad ####")
        .unwrap();

    let needs_update = command_needs_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        Some(temp_dir.path()),
    )
    .unwrap();
    assert!(needs_update);
}

#[test]
fn test_command_needs_update_no_shim_systemd_boot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let systemd_boot_efi_file = temp_dir
        .path()
        .join("boot/efi/EFI/systemd/systemd-bootx64.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    ).expect("Failed to copy systemd-boot test file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_efi_file,
    )
    .expect("Failed to copy systemd-boot test file");

    let needs_update = command_needs_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        Some(temp_dir.path()),
    )
    .unwrap();
    assert!(!needs_update);
}

#[test]
fn test_command_needs_update_shim_grub2() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let grub2 = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    let grub2_efi_file = temp_dir.path().join("boot/efi/EFI/systemd/grub.efi");
    fs::create_dir_all(grub2.parent().unwrap())
        .expect("Failed to create directory for grub2 EFI file");
    fs::create_dir_all(grub2_efi_file.parent().unwrap())
        .expect("Failed to create directory for grub2 EFI file");
    fs::copy(PathBuf::from("src/unit_tests/test_files/grub2.efi"), &grub2)
        .expect("Failed to copy grub2 test file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_efi_file,
    )
    .expect("Failed to copy grub2 efi file");
    let shim_test_file = temp_dir.path().join("usr/share/efi/x86_64/shim.efi");
    let shim_efi_file = temp_dir.path().join("boot/efi/EFI/systemd/grub.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    fs::create_dir_all(shim_efi_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&shim_efi_file)
        .unwrap()
        .write_all(b"GNU GRUB  version %s\x002.10\x00prefixESC at any time exits.")
        .unwrap();

    let needs_update = command_needs_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        Some(temp_dir.path()),
    )
    .unwrap();
    assert!(needs_update);
}

#[test]
fn test_command_needs_update_no_shim_grub2() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let grub2 = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    let grub2_efi_file = temp_dir.path().join("boot/efi/EFI/systemd/grub.efi");
    fs::create_dir_all(grub2.parent().unwrap())
        .expect("Failed to create directory for grub2 EFI file");
    fs::create_dir_all(grub2_efi_file.parent().unwrap())
        .expect("Failed to create directory for grub2 EFI file");
    fs::copy(PathBuf::from("src/unit_tests/test_files/grub2.efi"), &grub2)
        .expect("Failed to copy grub2 test file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_efi_file,
    )
    .expect("Failed to copy grub2 efi file");
    let shim_test_file = temp_dir.path().join("usr/share/efi/x86_64/shim.efi");
    let shim_efi_file = temp_dir.path().join("boot/efi/EFI/systemd/grub.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    fs::create_dir_all(shim_efi_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&shim_efi_file)
        .unwrap()
        .write_all(b"GNU GRUB  version %s\x002.13\x00prefixESC at any time exits.")
        .unwrap();

    let needs_update = command_needs_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        Some(temp_dir.path()),
    )
    .unwrap();
    assert!(!needs_update);
}

#[test]
fn test_command_update_shim_sdboot() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    let shim_test_file = snapshot_dir.join("usr/share/efi/x86_64/shim.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"#### LoaderInfo: systemd-boot 253.4+suse.17.gbe772961ad ####")
        .unwrap();

    let temp_shimdir = snapshot_dir.join("usr/share/efi/x86_64");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let full_boot_root = override_root.path().join("boot/efi");
    assert!(full_boot_root.join("loader/entries.srel").exists());
    let copied_content_entries = std::fs::read(full_boot_root.join("loader/entries.srel")).unwrap();
    assert_eq!(copied_content_entries, b"type1");
    assert!(full_boot_root.join("loader/loader.conf").exists());
    let copied_content_loader = std::fs::read(full_boot_root.join("loader/loader.conf")).unwrap();
    assert_eq!(copied_content_loader, b"#timeout 3\n#console-mode keep\n");

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_mok_manager_efi = temp_boot_dst.join("MokManager.efi");
    let copied_shim_efi = temp_boot_dst.join("shim.efi");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(copied_mok_manager_efi.exists());
    let copied_content = std::fs::read(&copied_mok_manager_efi).unwrap();
    assert_eq!(copied_content, mock_mok_manager_content);

    assert!(copied_shim_efi.exists());
    let copied_content = std::fs::read(&copied_shim_efi).unwrap();
    assert_eq!(copied_content, mock_shim_content);

    assert!(copied_bootloader.exists());
    let copied_content = std::fs::read(&copied_bootloader).unwrap();
    assert_ne!(copied_content, b"#### LoaderInfo: systemd-boot 253.4+suse.17.gbe772961ad ####");

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(boot_csv.exists());
    let copied_boot_csv_content = std::fs::read(&boot_csv).unwrap();
    assert_eq!(
        String::from_utf8(copied_boot_csv_content).unwrap(),
        format!("{},openSUSE Boot Manager\n", "shim.efi")
    );

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(install_flag.exists());
    let install_flag_content = std::fs::read(&install_flag).unwrap();
    assert_eq!(
        String::from_utf8(install_flag_content).unwrap(),
        "opensuse-tumbleweed"
    );

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "opensuse-tumbleweed"
    );
}

#[test]
fn test_command_update_no_shim_sdboot() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/systemd-bootx64.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"#### LoaderInfo: systemd-boot 253.4+suse.17.gbe772961ad ####")
        .unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let full_boot_root = override_root.path().join("boot/efi");
    assert!(full_boot_root.join("loader/entries.srel").exists());
    let copied_content_entries = std::fs::read(full_boot_root.join("loader/entries.srel")).unwrap();
    assert_eq!(copied_content_entries, b"type1");
    assert!(full_boot_root.join("loader/loader.conf").exists());
    let copied_content_loader = std::fs::read(full_boot_root.join("loader/loader.conf")).unwrap();
    assert_eq!(copied_content_loader, b"#timeout 3\n#console-mode keep\n");

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_bootloader = temp_boot_dst.join("systemd-bootx64.efi");

    assert!(copied_bootloader.exists());
    let copied_content = std::fs::read(&copied_bootloader).unwrap();
    assert_ne!(copied_content, b"#### LoaderInfo: systemd-boot 253.4+suse.17.gbe772961ad ####");

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(boot_csv.exists());
    let copied_boot_csv_content = std::fs::read(&boot_csv).unwrap();
    assert_eq!(
        String::from_utf8(copied_boot_csv_content).unwrap(),
        format!("{},openSUSE Boot Manager\n", "systemd-bootx64.efi")
    );

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(install_flag.exists());
    let install_flag_content = std::fs::read(&install_flag).unwrap();
    assert_eq!(
        String::from_utf8(install_flag_content).unwrap(),
        "opensuse-tumbleweed"
    );

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "opensuse-tumbleweed"
    );
}

#[test]
fn test_command_update_shim_grub2() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_efi_path,
    )
    .expect("Failed to copy systemd-boot test file");
    let shim_test_file = snapshot_dir.join("usr/share/efi/x86_64/shim.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"GNU GRUB  version %s\x002.10\x00prefixESC at any time exits.")
        .unwrap();

        let bli_mod_src = snapshot_dir.join("grub2moddir/bli.mod");
        fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
        fs::write(&bli_mod_src, "bli module content").unwrap();

    let temp_shimdir = snapshot_dir.join("usr/share/efi/x86_64");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let full_boot_root = override_root.path().join("boot/efi");
    let grub_cfg_path = full_boot_root.join("EFI/opensuse").join("grub.cfg");
    assert!(grub_cfg_path.exists());

    let efi_boot_grub_cfg_path = full_boot_root.join("EFI/BOOT/grub.cfg");
    assert!(efi_boot_grub_cfg_path.exists());

    let mod_dir = full_boot_root.join("EFI/opensuse").join(format!("{}-efi", ARCH));
    let bli_mod_dst = mod_dir.join("bli.mod");
    assert!(bli_mod_dst.exists());

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_mok_manager_efi = temp_boot_dst.join("MokManager.efi");
    let copied_shim_efi = temp_boot_dst.join("shim.efi");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(copied_mok_manager_efi.exists());
    let copied_content = std::fs::read(&copied_mok_manager_efi).unwrap();
    assert_eq!(copied_content, mock_mok_manager_content);

    assert!(copied_shim_efi.exists());
    let copied_content = std::fs::read(&copied_shim_efi).unwrap();
    assert_eq!(copied_content, mock_shim_content);

    assert!(copied_bootloader.exists());

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(boot_csv.exists());
    let copied_boot_csv_content = std::fs::read(&boot_csv).unwrap();
    assert_eq!(
        String::from_utf8(copied_boot_csv_content).unwrap(),
        format!("{},openSUSE Boot Manager\n", "shim.efi")
    );

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(install_flag.exists());
    let install_flag_content = std::fs::read(&install_flag).unwrap();
    assert_eq!(
        String::from_utf8(install_flag_content).unwrap(),
        "opensuse-tumbleweed"
    );

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "opensuse-tumbleweed"
    );
}

#[test]
fn test_command_update_no_shim_grub2_no_snapshots() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path();

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_efi_path,
    )
    .expect("Failed to copy systemd-boot test file");
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"GNU GRUB  version %s\x002.10\x00prefixESC at any time exits.")
        .unwrap();

        let bli_mod_src = snapshot_dir.join("grub2moddir/bli.mod");
        fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
        fs::write(&bli_mod_src, "bli module content").unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_update(
        None,
        None,
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let full_boot_root = override_root.path().join("boot/efi");
    let grub_cfg_path = full_boot_root.join("EFI/opensuse").join("grub.cfg");
    assert!(grub_cfg_path.exists());

    let efi_boot_grub_cfg_path = full_boot_root.join("EFI/BOOT/grub.cfg");
    assert!(efi_boot_grub_cfg_path.exists());

    let mod_dir = full_boot_root.join("EFI/opensuse").join(format!("{}-efi", ARCH));
    let bli_mod_dst = mod_dir.join("bli.mod");
    assert!(bli_mod_dst.exists());

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(copied_bootloader.exists());

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(boot_csv.exists());
    let copied_boot_csv_content = std::fs::read(&boot_csv).unwrap();
    assert_eq!(
        String::from_utf8(copied_boot_csv_content).unwrap(),
        format!("{},openSUSE Boot Manager\n", "grub.efi")
    );

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(install_flag.exists());
    let install_flag_content = std::fs::read(&install_flag).unwrap();
    assert_eq!(
        String::from_utf8(install_flag_content).unwrap(),
        "opensuse-tumbleweed"
    );

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "opensuse-tumbleweed"
    );
}

#[test]
fn test_command_update_no_update_no_shim_grub2_no_snapshots() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path();

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_efi_path,
    )
    .expect("Failed to copy systemd-boot test file");
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"GNU GRUB  version %s\x002.12\x00prefixESC at any time exits.")
        .unwrap();

        let bli_mod_src = snapshot_dir.join("grub2moddir/bli.mod");
        fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
        fs::write(&bli_mod_src, "bli module content").unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_update(
        None,
        None,
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);

    let full_boot_root = override_root.path().join("boot/efi");
    let grub_cfg_path = full_boot_root.join("EFI/opensuse").join("grub.cfg");
    assert!(!grub_cfg_path.exists());

    let efi_boot_grub_cfg_path = full_boot_root.join("EFI/BOOT/grub.cfg");
    assert!(!efi_boot_grub_cfg_path.exists());

    let mod_dir = full_boot_root.join("EFI/opensuse").join(format!("{}-efi", ARCH));
    let bli_mod_dst = mod_dir.join("bli.mod");
    assert!(!bli_mod_dst.exists());

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(!random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(!entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(copied_bootloader.exists());

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(!boot_csv.exists());

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(!kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(!install_flag.exists());

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(!entry_token_path.exists());
}

#[test]
fn test_command_update_no_update_shim_sdboot() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    let shim_test_file = snapshot_dir.join("usr/share/efi/x86_64/shim.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"#### LoaderInfo: systemd-boot 256.4+suse.17.gbe772961ad ####")
        .unwrap();

    let temp_shimdir = snapshot_dir.join("usr/share/efi/x86_64");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_update(
        Some(0),
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);

    let full_boot_root = override_root.path().join("boot/efi");
    assert!(!full_boot_root.join("loader/entries.srel").exists());
    assert!(!full_boot_root.join("loader/loader.conf").exists());

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(!random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(!entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_mok_manager_efi = temp_boot_dst.join("MokManager.efi");
    let copied_shim_efi = temp_boot_dst.join("shim.efi");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(!copied_mok_manager_efi.exists());

    assert!(!copied_shim_efi.exists());

    assert!(copied_bootloader.exists());

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(!boot_csv.exists());

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(!kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(!install_flag.exists());

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(!entry_token_path.exists());
}

#[test]
fn test_command_force_no_update_no_shim_sdboot() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    let shim_test_file = snapshot_dir.join("usr/share/efi/x86_64/shim.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"#### LoaderInfo: systemd-boot 256.4+suse.17.gbe772961ad ####")
        .unwrap();

    let temp_shimdir = snapshot_dir.join("usr/share/efi/x86_64");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = command_force_update(
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        None,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);

    let full_boot_root = override_root.path().join("boot/efi");
    assert!(!full_boot_root.join("loader/entries.srel").exists());
    assert!(!full_boot_root.join("loader/loader.conf").exists());

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(!random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(!entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_mok_manager_efi = temp_boot_dst.join("MokManager.efi");
    let copied_shim_efi = temp_boot_dst.join("shim.efi");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(!copied_mok_manager_efi.exists());

    assert!(!copied_shim_efi.exists());

    assert!(copied_bootloader.exists());

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(!boot_csv.exists());

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(!kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(!install_flag.exists());

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(!entry_token_path.exists());
}

#[test]
fn test_command_force_update_no_shim_grub2_no_snapshots() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path();

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_efi_path,
    )
    .expect("Failed to copy systemd-boot test file");
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"GNU GRUB  version %s\x002.12\x00prefixESC at any time exits.")
        .unwrap();

        let bli_mod_src = snapshot_dir.join("grub2moddir/bli.mod");
        fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
        fs::write(&bli_mod_src, "bli module content").unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let full_boot_root = override_root.path().join("boot/efi");
    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    std::fs::write(&install_flag, "opensuse-tumbleweed").unwrap();

    let result = command_force_update(
        None,
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        None,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    let full_boot_root = override_root.path().join("boot/efi");
    let grub_cfg_path = full_boot_root.join("EFI/opensuse").join("grub.cfg");
    assert!(grub_cfg_path.exists());

    let efi_boot_grub_cfg_path = full_boot_root.join("EFI/BOOT/grub.cfg");
    assert!(efi_boot_grub_cfg_path.exists());

    let mod_dir = full_boot_root.join("EFI/opensuse").join(format!("{}-efi", ARCH));
    let bli_mod_dst = mod_dir.join("bli.mod");
    assert!(bli_mod_dst.exists());

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.is_dir());

    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(copied_bootloader.exists());

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(boot_csv.exists());
    let copied_boot_csv_content = std::fs::read(&boot_csv).unwrap();
    assert_eq!(
        String::from_utf8(copied_boot_csv_content).unwrap(),
        format!("{},openSUSE Boot Manager\n", "grub.efi")
    );

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(install_flag.exists());
    let install_flag_content = std::fs::read(&install_flag).unwrap();
    assert_eq!(
        String::from_utf8(install_flag_content).unwrap(),
        "opensuse-tumbleweed"
    );

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "opensuse-tumbleweed"
    );
}

#[test]
fn test_command_force_update_shim_sdboot() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";

    let systemd_boot_test_file = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    let systemd_boot_efi_file = override_root
        .path()
        .join("boot/efi/EFI/opensuse/grub.efi");
    fs::create_dir_all(systemd_boot_test_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::create_dir_all(systemd_boot_efi_file.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");
    let shim_test_file = snapshot_dir.join("usr/share/efi/x86_64/shim.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    File::create(&systemd_boot_efi_file)
        .unwrap()
        .write_all(b"#### LoaderInfo: systemd-boot 256.4+suse.17.gbe772961ad ####")
        .unwrap();

    let temp_shimdir = snapshot_dir.join("usr/share/efi/x86_64");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let full_boot_root = override_root.path().join("boot/efi");
    let temp_boot_dst = full_boot_root.as_path().join("EFI/opensuse");
    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    std::fs::write(&install_flag, "opensuse-tumbleweed").unwrap();

    let result = command_force_update(
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        None,
        Some(override_root.path()),
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    assert!(full_boot_root.join("loader/entries.srel").exists());
    let copied_content_entries = std::fs::read(full_boot_root.join("loader/entries.srel")).unwrap();
    assert_eq!(copied_content_entries, b"type1");
    assert!(full_boot_root.join("loader/loader.conf").exists());
    let copied_content_loader = std::fs::read(full_boot_root.join("loader/loader.conf")).unwrap();
    assert_eq!(copied_content_loader, b"#timeout 3\n#console-mode keep\n");

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.is_dir());
    
    let copied_mok_manager_efi = temp_boot_dst.join("MokManager.efi");
    let copied_shim_efi = temp_boot_dst.join("shim.efi");
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    assert!(copied_mok_manager_efi.exists());
    let copied_content = std::fs::read(&copied_mok_manager_efi).unwrap();
    assert_eq!(copied_content, mock_mok_manager_content);

    assert!(copied_shim_efi.exists());
    let copied_content = std::fs::read(&copied_shim_efi).unwrap();
    assert_eq!(copied_content, mock_shim_content);

    assert!(copied_bootloader.exists());
    let copied_content = std::fs::read(&copied_bootloader).unwrap();
    assert_ne!(copied_content, b"#### LoaderInfo: systemd-boot 253.4+suse.17.gbe772961ad ####");

    let boot_csv = temp_boot_dst.join("boot.csv");
    assert!(boot_csv.exists());
    let copied_boot_csv_content = std::fs::read(&boot_csv).unwrap();
    assert_eq!(
        String::from_utf8(copied_boot_csv_content).unwrap(),
        format!("{},openSUSE Boot Manager\n", "shim.efi")
    );

    let kernel_dir = full_boot_root.join("opensuse-tumbleweed");
    assert!(kernel_dir.is_dir());

    let install_flag = temp_boot_dst.join("installed_by_sdbootutil");
    assert!(install_flag.exists());
    let install_flag_content = std::fs::read(&install_flag).unwrap();
    assert_eq!(
        String::from_utf8(install_flag_content).unwrap(),
        "opensuse-tumbleweed"
    );

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "opensuse-tumbleweed"
    );
}

#[test]
fn test_command_update_predictions() {
    let result = command_update_predictions().unwrap();
    assert_eq!(result, true);
}

#[test]
fn test_process_args_and_get_system_info() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proc_mounts_path = temp_dir.path().join("proc/mounts");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    fs::create_dir_all(proc_mounts_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&proc_mounts_path).unwrap();

    writeln!(mounts_file, "btrfs / btrfs rw,relatime 0 0").unwrap();
    writeln!(mounts_file, "ext4 /etc ext4 rw,relatime 0 0").unwrap();

    let (
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
        cmds,
    ) = process_args_and_get_system_info(Some(temp_dir.path())).unwrap();

    assert_eq!(root_snapshot, Some(0));
    assert_eq!(root_prefix, Some("/.snapshots".to_string()));
    assert_eq!(root_subvol, Some(temp_dir.path().display().to_string()));
    assert_eq!(root_uuid, "123456789".to_string());
    assert_eq!(root_device, "sda1".to_string());
    assert_eq!(firmware_arch, "x64".to_string());
    assert_eq!(snapshot, Some(0));
    assert_eq!(arg_entry_token, "entry_token".to_string());
    assert_eq!(boot_root, temp_dir.path().display().to_string());
    assert_eq!(boot_dst, "/EFI/systemd".to_string());
    assert_eq!(image, "vmlinuz".to_string());
    assert_eq!(no_variables, false);
    assert_eq!(regenerate_initrd, false);
    assert_eq!(no_random_seed, false);
    assert_eq!(all, false);
    assert_eq!(shimdir, format!("/usr/share/efi/{}", ARCH));
    assert!(cmds.is_none());
}
