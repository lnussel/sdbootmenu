use super::super::fs::*;
use std::env::consts::ARCH;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_backup_file(temp_dir_path: &PathBuf, filename: &str) -> std::io::Result<PathBuf> {
    let backup_path = temp_dir_path.join(format!("{}.bak", filename));
    let mut backup_file = File::create(&backup_path)?;
    writeln!(backup_file, "Backup content")?;
    Ok(backup_path)
}

fn create_original_file(temp_dir_path: &PathBuf, filename: &str) -> std::io::Result<PathBuf> {
    let original_path = temp_dir_path.join(filename);
    let mut original_file = File::create(&original_path)?;
    writeln!(original_file, "Original content")?;
    Ok(original_path)
}

#[test]
fn test_restore_from_backup() {
    let (_temp_dir, temp_dir_path) = create_temp_dir();

    let original_file_path = temp_dir_path.join("testfile");
    create_backup_file(&temp_dir_path, "testfile").unwrap();

    let rollback_item = RollbackItem::new(original_file_path.clone());

    rollback_item.cleanup().unwrap();

    assert!(
        original_file_path.exists(),
        "Original file should have been restored from backup"
    );
}

#[test]
fn test_remove_original_no_backup() {
    let (_temp_dir, temp_dir_path) = create_temp_dir();

    let original_file_path = create_original_file(&temp_dir_path, "testfile").unwrap();

    let rollback_item = RollbackItem::new(original_file_path.clone());

    rollback_item.cleanup().unwrap();

    assert!(
        !original_file_path.exists(),
        "Original file should have been removed"
    );
}

#[test]
fn test_no_file_no_backup() {
    let (_temp_dir, temp_dir_path) = create_temp_dir();

    let non_existent_file_path = temp_dir_path.join("nonexistentfile");
    let rollback_item = RollbackItem::new(non_existent_file_path.clone());

    let result = rollback_item.cleanup();

    assert!(
        result.is_ok(),
        "Cleanup should not error when no files exist"
    );
}

#[test]
fn test_cleanup_success() {
    let (_temp_dir, temp_dir_path) = create_temp_dir();

    let original_file_path = create_original_file(&temp_dir_path, "testfile").unwrap();
    create_backup_file(&temp_dir_path, "testfile").unwrap();

    let rollback_item = RollbackItem::new(original_file_path.clone());
    let rollback_items = vec![rollback_item];
    cleanup_rollback_items(&rollback_items);

    assert!(
        original_file_path.exists(),
        "Original file should exist after cleanup"
    );
    assert!(
        !original_file_path.with_extension("bak").exists(),
        "Backup file should not exist after cleanup"
    );
}

#[test]
fn test_cleanup_success_content() {
    let (_temp_dir, temp_dir_path) = create_temp_dir();

    let backup_content = "Backup content";

    let original_file_path = create_original_file(&temp_dir_path, "testfile").unwrap();
    let backup_file_path = create_backup_file(&temp_dir_path, "testfile").unwrap();

    let mut backup_file = File::create(backup_file_path).unwrap();
    writeln!(backup_file, "{}", backup_content).unwrap();

    let rollback_item = RollbackItem::new(original_file_path.clone());
    let rollback_items = vec![rollback_item];
    cleanup_rollback_items(&rollback_items);

    assert!(
        original_file_path.exists(),
        "Original file should exist after cleanup"
    );
    assert!(
        !original_file_path.with_extension("bak").exists(),
        "Backup file should not exist after cleanup"
    );

    let mut restored_content = String::new();
    let mut restored_file = File::open(original_file_path).unwrap();
    restored_file.read_to_string(&mut restored_content).unwrap();

    assert_eq!(
        restored_content.trim(),
        backup_content,
        "Restored file content should match backup content"
    );
}

#[test]
fn test_reset_rollback_items() {
    let (_temp_dir, temp_dir_path) = create_temp_dir();

    let file_names = vec!["testfile1", "testfile2"];
    let mut rollback_items = Vec::new();

    for file_name in &file_names {
        let original_file_path = create_original_file(&temp_dir_path, file_name).unwrap();
        create_backup_file(&temp_dir_path, file_name).unwrap();

        rollback_items.push(RollbackItem::new(original_file_path.clone()));
    }

    reset_rollback_items(&mut rollback_items);

    assert!(
        rollback_items.is_empty(),
        "Rollback items should be cleared after reset"
    );

    for file_name in &file_names {
        let original_file_path = temp_dir_path.join(file_name);
        assert!(
            original_file_path.exists(),
            "Original file should still exist after reset"
        );

        let backup_file_path = original_file_path.with_extension("bak");
        assert!(
            !backup_file_path.exists(),
            "Backup file should be removed after reset"
        );
    }
}

#[test]
fn test_reset_rollback_items_no_backups() {
    let (_temp_dir, temp_dir_path) = create_temp_dir();

    let file_names = vec!["testfile1", "testfile2"];
    let mut rollback_items = Vec::new();

    for file_name in &file_names {
        let original_file_path = create_original_file(&temp_dir_path, file_name).unwrap();

        rollback_items.push(RollbackItem::new(original_file_path.clone()));
    }

    reset_rollback_items(&mut rollback_items);

    assert!(
        rollback_items.is_empty(),
        "Rollback items should be cleared after reset even if no backups exist"
    );

    for file_name in &file_names {
        let original_file_path = temp_dir_path.join(file_name);
        assert!(
            original_file_path.exists(),
            "Original file should still exist after reset without backups"
        );
    }
}

#[test]
fn test_is_transactional_with_overlayfs() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proc_mounts_path = temp_dir.path().join("proc/mounts");

    fs::create_dir_all(proc_mounts_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&proc_mounts_path).unwrap();

    writeln!(mounts_file, "overlayfs /etc overlayfs rw,relatime,lowerdir=/path/to/lower,upperdir=/path/to/upper,workdir=/path/to/work 0 0").unwrap();

    let result = is_transactional(Some(temp_dir.path().to_str().unwrap()));
    assert!(
        result.unwrap(),
        "Expected /etc to be transactional (overlayfs)"
    );
}

#[test]
fn test_is_transactional_without_overlayfs() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proc_mounts_path = temp_dir.path().join("proc/mounts");

    fs::create_dir_all(proc_mounts_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&proc_mounts_path).unwrap();

    writeln!(mounts_file, "ext4 /etc ext4 rw,relatime 0 0").unwrap();

    let result = is_transactional(Some(temp_dir.path().to_str().unwrap()));
    assert!(
        !result.unwrap(),
        "Expected /etc not to be transactional (not overlayfs)"
    );
}

#[test]
fn test_get_root_snapshot_info() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proc_mounts_path = temp_dir.path().join("proc/mounts");

    fs::create_dir_all(proc_mounts_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&proc_mounts_path).unwrap();

    writeln!(mounts_file, "ext4 /etc ext4 rw,relatime 0 0").unwrap();

    let (prefix, snapshot_id, full_path) = get_root_snapshot_info(Some(temp_dir.path())).unwrap();
    assert_eq!(prefix, "/.snapshots");
    assert_eq!(snapshot_id, 0);
    assert_eq!(full_path, temp_dir.path().to_string_lossy());
}

#[test]
fn test_is_subvol_ro_empty() {
    let result = is_subvol_ro(None).unwrap();

    assert_eq!(result, false);
}

#[test]
fn test_find_sdboot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    fs::create_dir_all(&snapshot_dir.join("usr/lib/systemd-boot"))
        .expect("Failed to create systemd-boot path");
    fs::create_dir_all(&snapshot_dir.join("usr/lib/systemd/boot/efi"))
        .expect("Failed to create systemd-boot EFI fallback path");

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::write(&sdboot_efi_path, "").expect("Failed to create dummy systemd-boot EFI file");

    let found_path = find_sdboot(Some(0), "x64", Some(temp_dir.path()));

    assert_eq!(found_path, sdboot_efi_path);
}

#[test]
fn test_find_sdboot_fallback() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("1")
        .join("snapshot");

    fs::create_dir_all(&snapshot_dir.join("usr/lib/systemd-boot"))
        .expect("Failed to create systemd-boot path");
    let fallback_dir = snapshot_dir.join("usr/lib/systemd/boot/efi");
    fs::create_dir_all(&fallback_dir).expect("Failed to create systemd-boot EFI fallback path");

    let fallback_efi_path = fallback_dir.join("systemd-bootx64.efi");
    File::create(&fallback_efi_path)
        .expect("Failed to create dummy systemd-boot EFI file in fallback location");

    let found_path = find_sdboot(Some(1), "x64", Some(temp_dir.path()));

    assert_eq!(
        found_path, fallback_efi_path,
        "The found path did not match the expected fallback path"
    );
}

#[test]
fn test_find_grub2_primary_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    fs::create_dir_all(&snapshot_dir.join(format!("usr/share/efi/{}/", ARCH)))
        .expect("Failed to create GRUB2 path");

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/efi/{}/grub.efi", ARCH));
    fs::write(&grub2_efi_path, "").expect("Failed to create dummy GRUB2 EFI file");

    let found_path = find_grub2(Some(0), Some(temp_dir.path()));

    assert_eq!(found_path, grub2_efi_path);
}

#[test]
fn test_find_grub2_fallback_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    fs::create_dir_all(&snapshot_dir.join(format!("usr/share/grub2/{}-efi/", ARCH)))
        .expect("Failed to create GRUB2 fallback path");

    let grub2_efi_fallback_path =
        snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    fs::write(&grub2_efi_fallback_path, "")
        .expect("Failed to create dummy GRUB2 EFI file in fallback location");

    let found_path = find_grub2(Some(0), Some(temp_dir.path()));

    assert_eq!(found_path, grub2_efi_fallback_path);
}

#[test]
fn test_find_sdboot_no_snapshot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::create_dir_all(temp_dir.path().join("usr/lib/systemd-boot"))
        .expect("Failed to create systemd-boot path");
    fs::create_dir_all(temp_dir.path().join("usr/lib/systemd/boot/efi"))
        .expect("Failed to create systemd-boot EFI fallback path");

    let sdboot_efi_path = temp_dir
        .path()
        .join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::write(&sdboot_efi_path, "").expect("Failed to create dummy systemd-boot EFI file");

    let found_path = find_sdboot(None, "x64", Some(temp_dir.path()));

    assert_eq!(found_path, sdboot_efi_path);
}

#[test]
fn test_find_sdboot_fallback_no_snapshot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    fs::create_dir_all(temp_dir.path().join("usr/lib/systemd-boot"))
        .expect("Failed to create systemd-boot path");
    let fallback_dir = temp_dir.path().join("usr/lib/systemd/boot/efi");
    fs::create_dir_all(&fallback_dir).expect("Failed to create systemd-boot EFI fallback path");

    let fallback_efi_path = fallback_dir.join("systemd-bootx64.efi");
    File::create(&fallback_efi_path)
        .expect("Failed to create dummy systemd-boot EFI file in fallback location");

    let found_path = find_sdboot(None, "x64", Some(temp_dir.path()));

    assert_eq!(
        found_path, fallback_efi_path,
        "The found path did not match the expected fallback path"
    );
}

#[test]
fn test_find_grub2_primary_path_no_snapshot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::create_dir_all(temp_dir.path().join(format!("usr/share/efi/{}/", ARCH)))
        .expect("Failed to create GRUB2 path");

    let grub2_efi_path = temp_dir
        .path()
        .join(format!("usr/share/efi/{}/grub.efi", ARCH));
    fs::write(&grub2_efi_path, "").expect("Failed to create dummy GRUB2 EFI file");

    let found_path = find_grub2(None, Some(temp_dir.path()));

    assert_eq!(found_path, grub2_efi_path);
}

#[test]
fn test_find_grub2_fallback_path_no_snapshot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    fs::create_dir_all(
        temp_dir
            .path()
            .join(format!("usr/share/grub2/{}-efi/", ARCH)),
    )
    .expect("Failed to create GRUB2 fallback path");

    let grub2_efi_fallback_path = temp_dir
        .path()
        .join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    fs::write(&grub2_efi_fallback_path, "")
        .expect("Failed to create dummy GRUB2 EFI file in fallback location");

    let found_path = find_grub2(None, Some(temp_dir.path()));

    assert_eq!(found_path, grub2_efi_fallback_path);
}

#[test]
fn test_is_sdboot_only_systemd_boot_exists() {
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

    assert!(is_sdboot(Some(0), "x64", Some(temp_dir.path())));
}

#[test]
fn test_is_sdboot_systemd_boot_and_grub2_exist() {
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

    assert!(!is_sdboot(Some(0), "x64", Some(temp_dir.path())));
}

#[test]
fn test_is_sdboot_only_grub2_exist() {
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

    assert!(!is_sdboot(Some(0), "x64", Some(temp_dir.path())));
}

#[test]
fn test_is_sdboot_neither_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    assert!(!is_sdboot(Some(0), "x64", Some(temp_dir.path())));
}

#[test]
fn test_is_grub2_exists_primary() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let grub2_efi_path = snapshot_dir.join(format!("usr/share/efi/{}/grub.efi", ARCH));
    fs::create_dir_all(grub2_efi_path.parent().unwrap())
        .expect("Failed to create directory for GRUB2 EFI file");
    File::create(&grub2_efi_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    assert!(is_grub2(Some(0), Some(temp_dir.path())));
}

#[test]
fn test_is_grub2_not_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    assert!(!is_grub2(Some(0), Some(temp_dir.path())));
}

#[test]
fn test_is_grub2_exists_fallback() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let grub2_efi_fallback_path =
        snapshot_dir.join(format!("usr/share/grub2/{}-efi/grub.efi", ARCH));
    fs::create_dir_all(grub2_efi_fallback_path.parent().unwrap())
        .expect("Failed to create directory for GRUB2 EFI file in fallback location");
    File::create(&grub2_efi_fallback_path)
        .unwrap()
        .write_all(b"")
        .unwrap();

    assert!(is_grub2(Some(0), Some(temp_dir.path())));
}

#[test]
fn test_determine_boot_dst_only_systemd_boot_exists() {
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

    assert_eq!(
        determine_boot_dst(Some(0), "x64", Some(temp_dir.path())).unwrap(),
        "/EFI/systemd",
        "Failed to get boot_dst"
    );
}

#[test]
fn test_determine_boot_dst_systemd_boot_and_grub2_exist() {
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

    assert_eq!(
        determine_boot_dst(Some(0), "x64", Some(temp_dir.path())).unwrap(),
        "/EFI/opensuse",
        "Failed to get boot_dst"
    );
}

#[test]
fn test_determine_boot_dst_only_grub2_exist() {
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

    assert_eq!(
        determine_boot_dst(Some(0), "x64", Some(temp_dir.path())).unwrap(),
        "/EFI/opensuse",
        "Failed to get boot_dst"
    );
}

#[test]
fn test_determine_boot_dst_neither_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let result = determine_boot_dst(Some(0), "x64", Some(temp_dir.path()));
    assert!(
        result.is_err(),
        "Expected an error for file without version pattern"
    );
    assert_eq!(
        result.unwrap_err(),
        "Unsupported bootloader or unable to determine bootloader"
    );
}

#[test]
fn test_find_bootloader_sdboot_present() {
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

    assert_eq!(
        find_bootloader(Some(0), "x64", Some(temp_dir.path())).unwrap(),
        sdboot_efi_path
    );
}

#[test]
fn test_find_bootloader_grub2_present() {
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

    assert_eq!(
        find_bootloader(Some(0), "x64", Some(temp_dir.path())).unwrap(),
        grub2_efi_path
    );
}

#[test]
fn test_find_bootloader_none_present() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    assert!(find_bootloader(Some(0), "x64", Some(temp_dir.path())).is_err());
}

#[test]
fn test_find_bootloader_with_both_systemd_and_grub2() {
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

    let result = find_bootloader(Some(0), "x64", Some(temp_dir.path()));

    assert!(matches!(result, Ok(ref path) if path == &grub2_efi_path));
}

#[test]
fn test_find_version() {
    let content = b"Some text before version: START 1.2.3 END some text after version";
    let start_pattern = b"START ";
    let end_pattern = b" END";

    let version = find_version(content, start_pattern, end_pattern);

    assert_eq!(
        version,
        Some("1.2.3".to_string()),
        "The version found does not match the expected value"
    );
}

#[test]
fn test_find_version_no_match() {
    let content = b"The quick brown fox jumps over the lazy dog";
    let start_pattern = b"version:";
    let end_pattern = b";";

    let result = find_version(content, start_pattern, end_pattern);
    assert!(result.is_none(), "Expected None, but got Some");
}

#[test]
fn test_find_version_with_special_characters() {
    let content = b"Here is the version: \x012.34\x00; and some more text";
    let start_pattern = b"version: \x01";
    let end_pattern = b"\x00;";

    let result = find_version(content, start_pattern, end_pattern)
        .expect("Expected a version string, but none was found");

    assert_eq!(
        result, "2.34",
        "The extracted version string does not match the expected value"
    );
}

#[test]
fn test_bootloader_version_custom_systemd_boot() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let systemd_boot_path = snapshot_dir.join("EFI").join("systemd");
    fs::create_dir_all(&systemd_boot_path).expect("Failed to create systemd-boot path");
    let systemd_boot_test_file = systemd_boot_path.join("systemd-bootx64.efi");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &systemd_boot_test_file,
    )
    .expect("Failed to copy systemd-boot test file");

    let version_sdboot =
        bootloader_version(Some(0), "", "", "", "", Some(systemd_boot_test_file), None).unwrap();
    assert_eq!(
        version_sdboot, "255.4+suse.17.gbe772961ad",
        "Failed to detect systemd-boot version"
    );
}

#[test]
fn test_bootloader_version_custom_grub2() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

    let grub2_path = snapshot_dir.join("EFI").join("opensuse");
    fs::create_dir_all(&grub2_path).expect("Failed to create GRUB2 path");
    let grub2_test_file = grub2_path.join("grubx64.efi");
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/grub2.efi"),
        &grub2_test_file,
    )
    .expect("Failed to copy GRUB2 test file");

    let version_grub2 =
        bootloader_version(Some(0), "", "", "", "", Some(grub2_test_file), None).unwrap();
    assert_eq!(version_grub2, "2.12", "Failed to detect GRUB2 version");
}

#[test]
fn test_bootloader_version_systemd_boot() {
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

    let version_sdboot = bootloader_version(
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        None,
        Some(temp_dir.path()),
    )
    .unwrap();
    assert_eq!(
        version_sdboot, "255.4+suse.17.gbe772961ad",
        "Failed to detect systemd-boot version"
    );
}

#[test]
fn test_bootloader_version_grub2() {
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

    let version_grub2 = bootloader_version(
        Some(0),
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        None,
        Some(temp_dir.path()),
    )
    .unwrap();
    assert_eq!(version_grub2, "2.12", "Failed to detect grub2 version");
}

#[test]
fn test_bootloader_version_shim_no_snapshots() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let shim_test_file = temp_dir.path().join("usr/share/efi/x86_64/shim.efi");
    let grub_efi_file = temp_dir.path().join("boot/efi/EFI/systemd/grub.efi");
    fs::create_dir_all(shim_test_file.parent().unwrap())
        .expect("Failed to create directory for shim EFI file");
    fs::create_dir_all(grub_efi_file.parent().unwrap())
        .expect("Failed to create directory for grub EFI file");
    File::create(&shim_test_file)
        .unwrap()
        .write_all(b"")
        .unwrap();
    fs::copy(
        PathBuf::from("src/unit_tests/test_files/systemd_boot.efi"),
        &grub_efi_file,
    )
    .expect("Failed to copy shim efi file");

    let version_sdboot = bootloader_version(
        None,
        "x64",
        "/usr/share/efi/x86_64",
        "/boot/efi",
        "/EFI/systemd",
        None,
        Some(temp_dir.path()),
    )
    .unwrap();
    assert_eq!(
        version_sdboot, "255.4+suse.17.gbe772961ad",
        "Failed to detect shim (systemd-boot) version"
    );
}

#[test]
fn test_bootloader_version_file_not_found() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let non_existent_file = temp_dir.path().join("non_existent_file.efi");

    let result = bootloader_version(
        Some(0),
        "x64",
        "/usr/lib",
        temp_dir.path().to_str().unwrap(),
        "EFI/nonexistent",
        Some(non_existent_file),
        None,
    );

    assert!(result.is_err(), "Expected an error for non-existent file");
    assert!(
        result.unwrap_err().starts_with("File does not exist:"),
        "Error message did not start with expected text"
    );
}

#[test]
fn test_bootloader_version_no_version_pattern() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("empty_version_pattern.efi");

    let mut file = fs::File::create(&file_path).expect("Failed to create test file");
    writeln!(
        file,
        "This file does not contain a recognizable version pattern."
    )
    .expect("Failed to write to test file");

    let result = bootloader_version(
        Some(0),
        "x64",
        "/usr/lib",
        temp_dir.path().to_str().unwrap(),
        "EFI/nonexistent",
        Some(file_path),
        None,
    );

    assert!(
        result.is_err(),
        "Expected an error for file without version pattern"
    );
    assert_eq!(result.unwrap_err(), "Version not found");
}

#[test]
fn test_bootloader_needs_update_no_systemd_boot() {
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

    let needs_update = bootloader_needs_update(
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
fn test_bootloader_needs_update_no_grub2() {
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

    let needs_update = bootloader_needs_update(
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
fn test_bootloader_needs_update_shim_systemd_boot() {
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

    let needs_update = bootloader_needs_update(
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
fn test_bootloader_needs_update_no_shim_systemd_boot() {
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
    .expect("Failed to copy systemd-boot test file");

    let needs_update = bootloader_needs_update(
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
fn test_bootloader_needs_update_shim_grub2() {
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

    let needs_update = bootloader_needs_update(
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
fn test_bootloader_needs_update_no_shim_grub2() {
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

    let needs_update = bootloader_needs_update(
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
fn test_bootloader_name_only_systemd_boot_exists() {
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

    assert_eq!(
        bootloader_name(Some(0), "x64", Some(temp_dir.path())).unwrap(),
        "systemd-boot".to_string(),
        "Expected 'systemd-boot'"
    );
}

#[test]
fn test_bootloader_name_systemd_boot_and_grub2_exist() {
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

    assert_eq!(
        bootloader_name(Some(0), "x64", Some(temp_dir.path())).unwrap(),
        "grub2".to_string(),
        "Expected 'grub2'"
    );
}

#[test]
fn test_bootloader_name_only_grub2_exist() {
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

    assert_eq!(
        bootloader_name(Some(0), "x64", Some(temp_dir.path())).unwrap(),
        "grub2".to_string(),
        "Expected 'grub2'"
    );
}

#[test]
fn test_bootloader_name_neither_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    assert!(bootloader_name(Some(0), "x64", Some(temp_dir.path())).is_err());
}

#[test]
fn test_is_installed_true() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

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

    let is_installed = is_installed(
        Some(0),
        "",
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
fn test_is_installed_false_bootloader() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

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

    let is_installed = is_installed(
        Some(0),
        "",
        "",
        "",
        "",
        Some(systemd_boot_test_file),
        Some(temp_dir.path()),
    )
    .unwrap();
    assert_eq!(is_installed, false, "Expected is_installed to return true")
}

#[test]
fn test_is_installed_false_flag() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let snapshot_dir = temp_dir
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");

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

    let is_installed = is_installed(
        Some(0),
        "",
        "",
        "",
        "",
        Some(systemd_boot_test_file),
        Some(temp_dir.path()),
    )
    .unwrap();
    assert_eq!(is_installed, false, "Expected is_installed to return true")
}

#[test]
fn test_update_random_seed_no_override_no_existing_seed() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let boot_root = temp_dir.path();

    update_random_seed(boot_root.to_str().unwrap(), false, None)
        .expect("Failed to update reandom seed");

    let random_seed_path = boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());
}

#[test]
fn test_update_random_seed_with_override_no_existing_seed() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let override_dir = TempDir::new().expect("Failed to create temp dir");
    let boot_root = temp_dir.path();

    update_random_seed(
        boot_root.to_str().unwrap(),
        false,
        Some(override_dir.path()),
    )
    .expect("Failed to update reandom seed");

    let random_seed_path = override_dir
        .path()
        .join(boot_root.strip_prefix("/").unwrap_or(boot_root))
        .join("loader/random-seed");
    assert!(random_seed_path.exists());
}

#[test]
fn test_update_random_seed_no_override_with_existing_seed() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let boot_root = temp_dir.path();

    let random_seed_path = boot_root.join("loader/random-seed");
    fs::create_dir_all(random_seed_path.parent().unwrap()).expect("Failed to create directory");
    fs::write(&random_seed_path, &[0u8; 32]).unwrap();

    update_random_seed(boot_root.to_str().unwrap(), false, None)
        .expect("Failed to update reandom seed");

    let mut new_seed = Vec::new();
    File::open(&random_seed_path)
        .unwrap()
        .read_to_end(&mut new_seed)
        .expect("Failed to open file");
    assert_ne!(new_seed, vec![0u8; 32]);
}

#[test]
fn test_update_random_seed_with_override_with_existing_seed() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let override_dir = TempDir::new().expect("Failed to create temp dir");
    let boot_root = temp_dir.path();

    let random_seed_path = override_dir
        .path()
        .join(boot_root.strip_prefix("/").unwrap_or(boot_root))
        .join("loader/random-seed");
    fs::create_dir_all(random_seed_path.parent().unwrap()).expect("Failed to create directory");
    fs::write(&random_seed_path, &[0u8; 32]).expect("Failed to write to file");

    update_random_seed(
        boot_root.to_str().unwrap(),
        false,
        Some(override_dir.path()),
    )
    .expect("Failed to update reandom seed");

    let mut new_seed = Vec::new();
    File::open(&random_seed_path)
        .unwrap()
        .read_to_end(&mut new_seed)
        .expect("Failed to open file");
    assert_ne!(new_seed, vec![0u8; 32]);
}

#[test]
fn test_update_random_seed_no_action() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let boot_root = temp_dir.path();

    update_random_seed(boot_root.to_str().unwrap(), true, None)
        .expect("Failed update reandom seed");

    let random_seed_path = boot_root.join("loader/random-seed");
    assert!(!random_seed_path.exists());
}

#[test]
fn test_read_partition_number_valid() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file_path = temp_dir.path().join("partition_number");

    {
        let mut temp_file = File::create(&temp_file_path).expect("Failed to create temp file");
        writeln!(temp_file, "42").expect("Failed to write to temp file");
    }

    let partition_number =
        read_partition_number(&temp_file_path).expect("Failed to read partition number");
    assert_eq!(partition_number, 42);
}

#[test]
fn test_read_partition_number_invalid_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file_path = temp_dir.path().join("invalid_content");

    {
        let mut temp_file = File::create(&temp_file_path).expect("Failed to create temp file");
        writeln!(temp_file, "not_a_number").expect("Failed to write to temp file");
    }

    assert!(
        read_partition_number(&temp_file_path).is_err(),
        "Expected an error for invalid partition number content"
    );
}

#[test]
fn test_read_partition_number_missing_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let missing_file_path = temp_dir.path().join("missing_file");

    assert!(
        read_partition_number(&missing_file_path).is_err(),
        "Expected an error for missing file"
    );
}

#[test]
fn test_get_drive_and_partition_from_block_device() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let sys_class_block_dir = temp_dir.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = temp_dir.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let (drive, partition_number) =
        get_drive_and_partition_from_block_device("sda1", Some(temp_dir.path()))
            .expect("Function failed");

    assert_eq!(drive, PathBuf::from("/dev/sda"));
    assert_eq!(partition_number, 1);
}

#[test]
fn test_get_drive_and_partition_from_block_device_missing_partition_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let sys_class_block_dir = temp_dir.path().join("sys/class/block/sda1");
    fs::create_dir_all(&sys_class_block_dir)
        .expect("Failed to create mock sys/class/block directory");

    let drive_link = temp_dir.path().join("sda");
    fs::write(&drive_link, "").expect("Failed to create mock drive file");

    #[cfg(target_os = "linux")]
    std::os::unix::fs::symlink(&drive_link, &sys_class_block_dir.join("device"))
        .expect("Failed to create symbolic link for device");

    assert!(
        get_drive_and_partition_from_block_device("sda1", Some(temp_dir.path())).is_err(),
        "Expected an error due to missing partition file"
    );
}

#[test]
fn test_copy_shim_files() {
    let temp_snapshot_prefix = TempDir::new().unwrap();
    let temp_shimdir = temp_snapshot_prefix.path().join("EFI/BOOT");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";
    let mock_bootloader_content = b"mock bootloader content";

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let temp_boot_root = TempDir::new().unwrap();
    let temp_boot_dst = temp_boot_root.path().join("EFI/BOOT");

    let mock_bootloader_file = TempDir::new().unwrap().path().join("grub.efi");
    std::fs::create_dir_all(&mock_bootloader_file.parent().unwrap()).unwrap();
    std::fs::write(&mock_bootloader_file, mock_bootloader_content).unwrap();

    let result = copy_shim_files(
        temp_snapshot_prefix.path().to_str().unwrap(),
        "EFI/BOOT",
        temp_boot_root.path().to_str().unwrap(),
        "EFI/BOOT",
        &mock_bootloader_file,
        None,
    );

    assert!(result.is_ok());

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
}

#[test]
fn test_copy_shim_files_sdboot() {
    let temp_snapshot_prefix = TempDir::new().unwrap();
    let temp_shimdir = temp_snapshot_prefix.path().join("EFI/BOOT");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";
    let mock_bootloader_content = b"mock bootloader content";

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let temp_boot_root = TempDir::new().unwrap();
    let temp_boot_dst = temp_boot_root.path().join("EFI/BOOT");

    let mock_bootloader_file = TempDir::new().unwrap().path().join("systemd-boot.efi");
    std::fs::create_dir_all(&mock_bootloader_file.parent().unwrap()).unwrap();
    std::fs::write(&mock_bootloader_file, mock_bootloader_content).unwrap();

    let result = copy_shim_files(
        temp_snapshot_prefix.path().to_str().unwrap(),
        "EFI/BOOT",
        temp_boot_root.path().to_str().unwrap(),
        "EFI/BOOT",
        &mock_bootloader_file,
        None,
    );

    assert!(result.is_ok());

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
}

#[test]
fn test_copy_shim_files_with_missing_source() {
    let temp_snapshot_prefix = TempDir::new().unwrap();

    let temp_boot_root = TempDir::new().unwrap();

    let mock_bootloader_file = temp_boot_root.path().join("grub.efi");
    std::fs::create_dir_all(&mock_bootloader_file.parent().unwrap()).unwrap();
    File::create(&mock_bootloader_file).unwrap();

    let result = copy_shim_files(
        temp_snapshot_prefix.path().to_str().unwrap(),
        "EFI/BOOT",
        temp_boot_root.path().to_str().unwrap(),
        "EFI/BOOT",
        &mock_bootloader_file,
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_copy_shim_files_with_invalid_paths() {
    let invalid_path = "/path/does/not/exist";

    let mock_bootloader_file = TempDir::new().unwrap().path().join("grub.efi");
    std::fs::create_dir_all(&mock_bootloader_file.parent().unwrap()).unwrap();
    File::create(&mock_bootloader_file).unwrap();

    let result = copy_shim_files(
        invalid_path,
        invalid_path,
        invalid_path,
        invalid_path,
        &mock_bootloader_file,
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_copy_shim_files_with_override_prefix() {
    let temp_snapshot_prefix = TempDir::new().unwrap();
    let temp_shimdir = temp_snapshot_prefix.path().join("EFI/BOOT");
    fs::create_dir_all(&temp_shimdir).unwrap();

    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";
    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    fs::write(&shim_efi, mock_shim_content).unwrap();

    let temp_boot_root = TempDir::new().unwrap();

    let mock_bootloader_file = TempDir::new().unwrap().path().join("grub.efi");
    std::fs::create_dir_all(&mock_bootloader_file.parent().unwrap()).unwrap();
    fs::write(&mock_bootloader_file, b"mock bootloader content").unwrap();

    let override_prefix = TempDir::new().unwrap();

    let result = copy_shim_files(
        temp_snapshot_prefix.path().to_str().unwrap(),
        "EFI/BOOT",
        temp_boot_root.path().to_str().unwrap(),
        "EFI/BOOT",
        &mock_bootloader_file,
        Some(override_prefix.path()),
    );

    assert!(result.is_err());
}

#[test]
fn test_copy_shim_files_with_override() {
    let temp_snapshot_prefix = TempDir::new().unwrap();
    let temp_shimdir = temp_snapshot_prefix.path().join("EFI/BOOT");
    std::fs::create_dir_all(&temp_shimdir).unwrap();

    let mock_mok_manager_content = b"mock MokManager.efi content";
    let mock_shim_content = b"mock shim.efi content";
    let mock_bootloader_content = b"mock bootloader content";

    let mok_manager_efi = temp_shimdir.join("MokManager.efi");
    let shim_efi = temp_shimdir.join("shim.efi");
    std::fs::write(&mok_manager_efi, mock_mok_manager_content).unwrap();
    std::fs::write(&shim_efi, mock_shim_content).unwrap();

    let temp_boot_root = temp_snapshot_prefix.path().join("bootroot");
    let temp_boot_dst = temp_boot_root.as_path().join("EFI/BOOT");

    let mock_bootloader_file = TempDir::new().unwrap().path().join("grub.efi");
    std::fs::create_dir_all(&mock_bootloader_file.parent().unwrap()).unwrap();
    std::fs::write(&mock_bootloader_file, mock_bootloader_content).unwrap();

    let result = copy_shim_files(
        "/",
        "EFI/BOOT",
        "bootroot",
        "EFI/BOOT",
        &mock_bootloader_file,
        Some(temp_snapshot_prefix.path()),
    );

    assert!(result.is_ok());

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
}

#[test]
fn test_copy_bootloader_success() {
    let temp_dir = TempDir::new().unwrap();
    let bootloader_file = temp_dir.path().join("mock_bootloader.efi");
    let boot_root = temp_dir.path().join("boot");
    let boot_dst = Path::new("EFI/Custom");

    let mock_bootloader_content = b"mock bootloader content";

    fs::write(&bootloader_file, mock_bootloader_content).unwrap();

    let result = copy_bootloader(
        &bootloader_file,
        &boot_root.to_str().unwrap(),
        boot_dst.to_str().unwrap(),
        "x64",
        None,
    );

    assert!(result.is_ok());

    assert!(boot_root
        .join(boot_dst)
        .join("mock_bootloader.efi")
        .exists());

    assert!(boot_root.join("EFI/BOOT/BOOTX64.EFI").exists());
    let copied_content = std::fs::read(boot_root.join("EFI/BOOT/BOOTX64.EFI")).unwrap();
    assert_eq!(copied_content, mock_bootloader_content);
}

#[test]
fn test_copy_bootloader_missing_source() {
    let temp_dir = TempDir::new().unwrap();
    let missing_bootloader_file = temp_dir.path().join("non_existent_bootloader.efi");
    let boot_root = temp_dir.path().join("boot");
    let boot_dst = Path::new("EFI/Custom");

    let result = copy_bootloader(
        &missing_bootloader_file,
        &boot_root.to_str().unwrap(),
        boot_dst.to_str().unwrap(),
        "x64",
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_copy_bootloader_with_override_prefix() {
    let override_root = TempDir::new().unwrap();
    let boot_root = Path::new("boot");
    let boot_dst = Path::new("EFI/Custom");

    let mock_bootloader_content = b"mock bootloader content";

    let bootloader_file = override_root.path().join("mock_bootloader.efi");
    fs::write(&bootloader_file, mock_bootloader_content).unwrap();

    let result = copy_bootloader(
        &bootloader_file,
        boot_root.to_str().unwrap(),
        boot_dst.to_str().unwrap(),
        "x64",
        Some(override_root.path()),
    );

    assert!(result.is_ok());

    assert!(override_root
        .path()
        .join(boot_root)
        .join(boot_dst)
        .join("mock_bootloader.efi")
        .exists());

    assert!(override_root
        .path()
        .join(boot_root)
        .join("EFI/BOOT/BOOTX64.EFI")
        .exists());
    let copied_content = std::fs::read(
        override_root
            .path()
            .join(boot_root)
            .join("EFI/BOOT/BOOTX64.EFI"),
    )
    .unwrap();
    assert_eq!(copied_content, mock_bootloader_content);
}

#[test]
fn test_create_sdboot_configuration_files() {
    let temp_dir = TempDir::new().unwrap();
    let boot_root = temp_dir.path();

    let result = update_sdboot_configuration(boot_root.to_str().unwrap(), None);
    assert!(result.is_ok());

    let entries_rel_path = boot_root.join("loader/entries.srel");
    assert!(entries_rel_path.exists());
    let copied_content_entries = std::fs::read(entries_rel_path).unwrap();
    assert_eq!(copied_content_entries, b"type1");

    let loader_conf_path = boot_root.join("loader/loader.conf");
    assert!(loader_conf_path.exists());
    let copied_content_loader = std::fs::read(loader_conf_path).unwrap();
    assert_eq!(copied_content_loader, b"#timeout 3\n#console-mode keep\n");
}

#[test]
fn test_existing_sdboot_configuration_files_unchanged() {
    let temp_dir = TempDir::new().unwrap();
    let boot_root = temp_dir.path();
    let entries_rel_path = boot_root.join("loader/entries.srel");
    let loader_conf_path = boot_root.join("loader/loader.conf");

    fs::create_dir_all(entries_rel_path.parent().unwrap()).unwrap();
    fs::write(&entries_rel_path, "existing type").unwrap();
    fs::create_dir_all(loader_conf_path.parent().unwrap()).unwrap();
    fs::write(&loader_conf_path, "existing configuration").unwrap();

    let result = update_sdboot_configuration(boot_root.to_str().unwrap(), None);
    assert!(result.is_ok());

    assert_eq!(
        fs::read_to_string(entries_rel_path).unwrap(),
        "existing type"
    );
    assert_eq!(
        fs::read_to_string(loader_conf_path).unwrap(),
        "existing configuration"
    );
}

#[test]
fn test_sdboot_configuration_with_override_prefix() {
    let override_root = TempDir::new().unwrap();
    let boot_root = Path::new("boot");

    let result =
        update_sdboot_configuration(boot_root.to_str().unwrap(), Some(override_root.path()));
    assert!(result.is_ok());

    let full_boot_root = override_root.path().join(boot_root);
    assert!(full_boot_root.join("loader/entries.srel").exists());
    let copied_content_entries = std::fs::read(full_boot_root.join("loader/entries.srel")).unwrap();
    assert_eq!(copied_content_entries, b"type1");
    assert!(full_boot_root.join("loader/loader.conf").exists());
    let copied_content_loader = std::fs::read(full_boot_root.join("loader/loader.conf")).unwrap();
    assert_eq!(copied_content_loader, b"#timeout 3\n#console-mode keep\n");
}

#[test]
fn test_grub2_configuration_file_creation() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_prefix = temp_dir.path().join("snapshot");
    let boot_root = temp_dir.path().join("boot");
    let boot_dst = Path::new("EFI/BOOT");

    let bli_mod_src = snapshot_prefix.join("grub2moddir/bli.mod");
    fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
    fs::write(&bli_mod_src, "bli module content").unwrap();

    let result = update_grub2_configuration(
        &snapshot_prefix.to_str().unwrap(),
        &boot_root.to_str().unwrap(),
        boot_dst.to_str().unwrap(),
        None,
    );
    assert!(result.is_ok());

    let grub_cfg_path = boot_root.join(boot_dst).join("grub.cfg");
    assert!(grub_cfg_path.exists());

    let efi_boot_grub_cfg_path = boot_root.join("EFI/BOOT/grub.cfg");
    assert!(efi_boot_grub_cfg_path.exists());

    let mod_dir = boot_root.join(boot_dst).join(format!("{}-efi", ARCH));
    let bli_mod_dst = mod_dir.join("bli.mod");
    assert!(bli_mod_dst.exists());
}

#[test]
fn test_existing_grub2_configuration_unchanged() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_prefix = temp_dir.path().join("snapshot");
    let boot_root = temp_dir.path().join("boot");
    let boot_dst = Path::new("EFI/BOOT2");

    let grub_cfg_path = boot_root.join(boot_dst).join("grub.cfg");
    fs::create_dir_all(grub_cfg_path.parent().unwrap()).unwrap();
    fs::write(&grub_cfg_path, "existing grub configuration").unwrap();

    let efi_boot_grub_cfg_path = boot_root.join("EFI/BOOT/grub.cfg");
    fs::create_dir_all(efi_boot_grub_cfg_path.parent().unwrap()).unwrap();
    fs::write(
        &efi_boot_grub_cfg_path,
        "existing EFI boot grub configuration",
    )
    .unwrap();

    let bli_mod_src = snapshot_prefix.join("grub2moddir/bli.mod");
    fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
    fs::write(&bli_mod_src, "bli module content").unwrap();

    let result = update_grub2_configuration(
        &snapshot_prefix.to_str().unwrap(),
        &boot_root.to_str().unwrap(),
        boot_dst.to_str().unwrap(),
        None,
    );
    assert!(result.is_ok());

    assert_eq!(
        fs::read_to_string(grub_cfg_path).unwrap(),
        "existing grub configuration"
    );
    assert_eq!(
        fs::read_to_string(efi_boot_grub_cfg_path).unwrap(),
        "existing grub configuration"
    );
}

#[test]
fn test_grub2_configuration_with_override_prefix() {
    let override_root = TempDir::new().unwrap();
    let snapshot_prefix = Path::new("snapshot");
    let boot_root = Path::new("boot");
    let boot_dst = Path::new("EFI/BOOT2");

    let bli_mod_src = override_root
        .path()
        .join(snapshot_prefix)
        .join("grub2moddir/bli.mod");
    fs::create_dir_all(bli_mod_src.parent().unwrap()).unwrap();
    fs::write(&bli_mod_src, "bli module content").unwrap();

    let result = update_grub2_configuration(
        snapshot_prefix.to_str().unwrap(),
        boot_root.to_str().unwrap(),
        boot_dst.to_str().unwrap(),
        Some(override_root.path()),
    );
    assert!(result.is_ok());

    let full_boot_root = override_root.path().join(boot_root);
    let grub_cfg_path = full_boot_root.join(boot_dst).join("grub.cfg");
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

    let mod_dir = full_boot_root.join(boot_dst).join(format!("{}-efi", ARCH));
    let bli_mod_dst = mod_dir.join("bli.mod");
    assert!(bli_mod_dst.exists());
    assert_eq!(
        fs::read_to_string(bli_mod_dst).unwrap(),
        "bli module content"
    );
}

#[test]
fn test_install_bootloader_shim_sdboot() {
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

    let result = install_bootloader(
        Some(0),
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());

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
fn test_install_bootloader_shim_sdboot_no_snapshots() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root.path();
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

    let result = install_bootloader(
        None,
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());

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
fn test_install_bootloader_sdboot() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
    let mock_bootloader_content = b"mock bootloader content";

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(mock_bootloader_content)
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

    let result = install_bootloader(
        Some(0),
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());

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
    assert_eq!(copied_content, mock_bootloader_content);

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
fn test_install_bootloader_sdboot_no_snapshots() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root.path();
    let mock_bootloader_content = b"mock bootloader content";

    let sdboot_efi_path = snapshot_dir.join("usr/lib/systemd-boot/systemd-bootx64.efi");
    fs::create_dir_all(sdboot_efi_path.parent().unwrap())
        .expect("Failed to create directory for systemd-boot EFI file");
    File::create(&sdboot_efi_path)
        .unwrap()
        .write_all(mock_bootloader_content)
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

    let result = install_bootloader(
        None,
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());

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
    assert_eq!(copied_content, mock_bootloader_content);

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
fn test_install_bootloader_shim_grub2() {
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

    let result = install_bootloader(
        Some(0),
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());

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
fn test_install_bootloader_grub2() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
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

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = install_bootloader(
        Some(0),
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());

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
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.exists());

    assert!(copied_bootloader.exists());
    let copied_content = std::fs::read(&copied_bootloader).unwrap();
    assert_eq!(copied_content, mock_bootloader_content);

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
fn test_install_bootloader_grub2_no_snapshots() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root.path();
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

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let result = install_bootloader(
        None,
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());

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
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.exists());

    assert!(copied_bootloader.exists());
    let copied_content = std::fs::read(&copied_bootloader).unwrap();
    assert_eq!(copied_content, mock_bootloader_content);

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
fn test_install_bootloader_grub2_existing_entry_token() {
    let override_root = TempDir::new().unwrap();
    let snapshot_dir = override_root
        .path()
        .join(".snapshots")
        .join("0")
        .join("snapshot");
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

    let sys_class_block_dir = override_root.path().join("sys/class/block/sda1");

    fs::create_dir_all(&sys_class_block_dir.parent().unwrap())
        .expect("Failed to create mock sys/class/block directory");
    let device_dir = override_root.path().join("sys/devices/pci0000:00/0000:00:02.1/0000:04:00.0/0000:05:0d.0/0000:18:00.0/ata8/host7/target7:0:0/7:0:0:0/block/sda/sda1");
    fs::create_dir_all(&device_dir).expect("Failed to create mock device directory");

    std::os::unix::fs::symlink(&device_dir, &sys_class_block_dir)
        .expect("Failed to create symbolic link for device");

    let partition_file = sys_class_block_dir.join("partition");
    fs::write(&partition_file, "1").expect("Failed to write mock partition number");

    let entry_token_path = override_root.path().join("etc/kernel/entry-token");
    fs::create_dir_all(entry_token_path.parent().unwrap())
        .expect("Failed to create entry token directory");
    fs::write(entry_token_path.clone(), "This is a test").unwrap();

    let result = install_bootloader(
        Some(0),
        "x64",
        "usr/share/efi/x64",
        "boot/efi",
        "EFI/opensuse",
        "opensuse-tumbleweed".to_string(),
        false,
        false,
        Some(override_root.path()),
    );
    assert!(result.is_ok());

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
    let copied_bootloader = temp_boot_dst.join("grub.efi");

    let random_seed_path = full_boot_root.join("loader/random-seed");
    assert!(random_seed_path.exists());

    let entries_dir = full_boot_root.join("loader/entries");
    assert!(entries_dir.exists());

    assert!(copied_bootloader.exists());
    let copied_content = std::fs::read(&copied_bootloader).unwrap();
    assert_eq!(copied_content, mock_bootloader_content);

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

    assert!(entry_token_path.exists());
    let entry_token_path_content = std::fs::read(&entry_token_path).unwrap();
    assert_eq!(
        String::from_utf8(entry_token_path_content).unwrap(),
        "This is a test"
    );
}

#[test]
fn test_get_shimdir() {
    let expected_path = format!("/usr/share/efi/{}", ARCH);
    assert_eq!(get_shimdir(), expected_path);
}

#[test]
fn test_is_snapshotted_with_btrfs_and_snapshots() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proc_mounts_path = temp_dir.path().join("proc/mounts");

    fs::create_dir_all(proc_mounts_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&proc_mounts_path).unwrap();

    writeln!(mounts_file, "btrfs / btrfs rw,relatime 0 0").unwrap();

    let snapshots_dir_path = temp_dir.path().join(".snapshots");
    fs::create_dir(&snapshots_dir_path).unwrap();

    let result = is_snapshotted(Some(temp_dir.path().to_str().unwrap()));
    assert!(
        result.unwrap(),
        "Expected system to be snapshotted with btrfs and .snapshots directory"
    );
}

#[test]
fn test_is_snapshotted_with_btrfs_without_snapshots() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proc_mounts_path = temp_dir.path().join("proc/mounts");

    fs::create_dir_all(proc_mounts_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&proc_mounts_path).unwrap();

    writeln!(mounts_file, "btrfs / btrfs rw,relatime 0 0").unwrap();

    let result = is_snapshotted(Some(temp_dir.path().to_str().unwrap()));
    assert!(
        !result.unwrap(),
        "Expected system not to be snapshotted with btrfs and without .snapshots directory"
    );
}

#[test]
fn test_is_snapshotted_without_btrfs_with_snapshots() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proc_mounts_path = temp_dir.path().join("proc/mounts");

    fs::create_dir_all(proc_mounts_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&proc_mounts_path).unwrap();

    writeln!(mounts_file, "ext4 / ext4 rw,relatime 0 0").unwrap();
    let snapshots_dir_path = temp_dir.path().join(".snapshots");
    fs::create_dir(&snapshots_dir_path).unwrap();

    let result = is_snapshotted(Some(temp_dir.path().to_str().unwrap()));
    assert!(
        !result.unwrap(),
        "Expected system not to be snapshotted without btrfs and with .snapshots directory"
    );
}

#[test]
fn test_is_snapshotted_without_btrfs_or_snapshots() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let proc_mounts_path = temp_dir.path().join("proc/mounts");

    fs::create_dir_all(proc_mounts_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&proc_mounts_path).unwrap();

    writeln!(mounts_file, "ext4 / ext4 rw,relatime 0 0").unwrap();

    let result = is_snapshotted(Some(temp_dir.path().to_str().unwrap()));
    assert!(
        !result.unwrap(),
        "Expected system not to be snapshotted without btrfs or .snapshots directory"
    );
}

#[test]
fn test_read_os_release() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "ID=custom-linux").unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();

    let result = read_os_release(None, Some(temp_path)).unwrap();

    assert_eq!(result.0, Some("custom-linux".to_string()));
    assert_eq!(result.1, Some("1.0".to_string()));
    assert_eq!(result.2, Some("Custom Linux 1.0".to_string()));

    assert_eq!(result.3, None);
}

#[test]
fn test_read_os_release_with_subvol() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root_fs_temp_path = temp_dir.path();
    let subvol_path = root_fs_temp_path.join(".snapshots/1/snapshot");
    std::fs::create_dir_all(&subvol_path).unwrap();
    let subvol_usr_lib_os_release = subvol_path.join("usr/lib/os-release");
    let subvol_etc_os_release = subvol_path.join("etc/os-release");

    std::fs::create_dir_all(subvol_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(subvol_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&subvol_usr_lib_os_release).unwrap();
    writeln!(file, "ID=subvol-linux").unwrap();
    writeln!(file, "VERSION_ID=2.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Subvol Linux 2.0\"").unwrap();
    writeln!(file, "IMAGE_ID=\"subvol-image\"").unwrap();

    let result = read_os_release(
        Some(&PathBuf::from("/.snapshots/1/snapshot")),
        Some(root_fs_temp_path),
    )
    .unwrap();

    assert_eq!(result.0, Some("subvol-linux".to_string()));
    assert_eq!(result.1, Some("2.0".to_string()));
    assert_eq!(result.2, Some("Subvol Linux 2.0".to_string()));
    assert_eq!(result.3, Some("subvol-image".to_string()));
}

#[test]
fn test_read_os_release_missing_in_subvol() {
    let root_fs_temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root_fs_temp_path = root_fs_temp_dir.path();
    let subvol_path = root_fs_temp_path.join(".snapshots/1/snapshot");
    std::fs::create_dir_all(&subvol_path).unwrap();

    let root_usr_lib_os_release = root_fs_temp_path.join("usr/lib/os-release");
    let root_etc_os_release = root_fs_temp_path.join("etc/os-release");

    std::fs::create_dir_all(root_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(root_etc_os_release.parent().unwrap()).unwrap();
    let result = read_os_release(
        Some(&PathBuf::from("/.snapshots/1/snapshot")),
        Some(root_fs_temp_path),
    );

    assert!(result.is_err(), "Expected read_os_release function to fail")
}

#[test]
fn test_read_machine_id_default_location() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    let machine_id_path = temp_path.join("etc/machine-id");

    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();

    let mut file = File::create(&machine_id_path).unwrap();
    writeln!(file, "123456789abcdef").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result = read_machine_id(None, None, Some(temp_path)).unwrap();

    assert_eq!(result, "123456789abcdef");
}

#[test]
fn test_read_machine_id_in_subvol() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    let subvol_path = Path::new(".snapshots/1/snapshot");

    let machine_id_path = temp_path.join(subvol_path).join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();
    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let mut file = File::create(&machine_id_path).unwrap();
    writeln!(file, "987654321fedcba").unwrap();

    let result = read_machine_id(Some(&subvol_path), Some(1), Some(temp_path)).unwrap();

    assert_eq!(result, "987654321fedcba");
}

#[test]
fn test_read_machine_id_overlayfs_location() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    let overlay_path = temp_path.join("var/lib/overlay/1/etc/machine-id");
    fs::create_dir_all(overlay_path.parent().unwrap()).unwrap();

    let mut file = File::create(&overlay_path).unwrap();
    writeln!(file, "overlayfs12345").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "overlayfs /etc overlayfs rw,relatime,lowerdir=/var/lib/overlay/1/etc,upperdir=/etc/upper,workdir=/etc/work 0 0").unwrap();

    let result = read_machine_id(None, Some(1), Some(temp_path)).unwrap();

    assert_eq!(result, "overlayfs12345");
}

#[test]
fn test_read_machine_id_missing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    let result = read_machine_id(None, None, Some(temp_path));

    assert!(result.is_err(), "Expected read_machine_id function to fail");
}

#[test]
fn test_settle_system_tokens_with_entry_token_file() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "ID=custom-linux").unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();

    let entry_token_path = temp_dir.path().join("etc/kernel/entry-token");
    fs::create_dir_all(entry_token_path.parent().unwrap()).unwrap();
    fs::write(&entry_token_path, "custom-entry-token\n").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let machine_id_path = temp_path.join("etc/machine-id");

    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();

    let mut file = File::create(&machine_id_path).unwrap();
    writeln!(file, "123456789abcdef").unwrap();

    let result = settle_system_tokens(None, None, None, Some(temp_dir.path())).unwrap();

    assert_eq!(result.0, "custom-entry-token");
}

#[test]
fn test_settle_system_tokens_fallback_to_machine_id() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "ID=custom-linux").unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();
    fs::write(&machine_id_path, "machine-id-value").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result = settle_system_tokens(None, None, None, Some(temp_dir.path())).unwrap();

    assert_eq!(result.0, "machine-id-value");
}

#[test]
fn test_settle_system_tokens_fallback_to_id_empty_machine_id() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "ID=custom-linux").unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result = settle_system_tokens(None, None, None, Some(temp_dir.path()));

    assert_eq!(result.unwrap().0, "custom-linux");
}

#[test]
fn test_settle_system_tokens_fallback_to_image_id_empty_machine_id() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "IMAGE_ID=custom-linuxos").unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result = settle_system_tokens(None, None, None, Some(temp_dir.path()));

    assert_eq!(result.unwrap().0, "custom-linuxos");
}

#[test]
fn test_settle_system_tokens_fallback_to_empty_machine_id() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result = settle_system_tokens(None, None, None, Some(temp_dir.path()));

    assert!(result.is_err());
}

#[test]
fn test_settle_system_tokens_literal() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result = settle_system_tokens(
        None,
        None,
        Some("literal:custom-linux:literal"),
        Some(temp_dir.path()),
    );

    assert_eq!(result.unwrap().0, "custom-linux:literal");
}

#[test]
fn test_settle_system_tokens_unknown_arg_entry_token() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result = settle_system_tokens(
        None,
        None,
        Some("custom-linux:literal"),
        Some(temp_dir.path()),
    );

    assert!(result.is_err());
}

#[test]
fn test_settle_system_tokens_set_to_machine_id() {
    let temp_dir = TempDir::new().unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();
    fs::write(&machine_id_path, "machine-id-value").unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "ID=custom-linux").unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result =
        settle_system_tokens(None, None, Some("machine-id"), Some(temp_dir.path())).unwrap();

    assert_eq!(result.0, "machine-id-value");
}

#[test]
fn test_settle_system_tokens_set_to_empty_machine_id() {
    let temp_dir = TempDir::new().unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();
    let temp_path = temp_dir.path();
    let custom_usr_lib_os_release = temp_path.join("usr/lib/os-release");
    let custom_etc_os_release = temp_path.join("etc/os-release");

    std::fs::create_dir_all(custom_usr_lib_os_release.parent().unwrap()).unwrap();
    std::fs::create_dir_all(custom_etc_os_release.parent().unwrap()).unwrap();

    let mut file = File::create(&custom_usr_lib_os_release).unwrap();
    writeln!(file, "ID=custom-linux").unwrap();
    writeln!(file, "VERSION_ID=1.0").unwrap();
    writeln!(file, "PRETTY_NAME=\"Custom Linux 1.0\"").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();

    let result = settle_system_tokens(None, None, Some("machine-id"), Some(temp_dir.path()));

    assert!(result.is_err());
}

#[test]
fn test_settle_system_tokens_set_to_os_id() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let os_release_path = temp_dir.path().join("etc/os-release");
    fs::create_dir_all(os_release_path.parent().unwrap()).unwrap();
    fs::write(&os_release_path, "ID=custom-os\n").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();
    fs::write(&machine_id_path, "machine-id-value").unwrap();

    let result = settle_system_tokens(None, None, Some("os-id"), Some(temp_dir.path())).unwrap();

    assert_eq!(result.0, "custom-os");
}

#[test]
fn test_settle_system_tokens_set_to_os_id_empty_machine_id() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let os_release_path = temp_dir.path().join("etc/os-release");
    fs::create_dir_all(os_release_path.parent().unwrap()).unwrap();
    fs::write(&os_release_path, "ID=custom-os\n").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();

    let result = settle_system_tokens(None, None, Some("os-id"), Some(temp_dir.path())).unwrap();

    assert_eq!(result.0, "custom-os");
    assert_eq!(result.1, "");
}

#[test]
fn test_settle_system_tokens_set_to_os_image() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();
    let os_release_path = temp_dir.path().join("etc/os-release");
    fs::create_dir_all(os_release_path.parent().unwrap()).unwrap();
    fs::write(&os_release_path, "IMAGE_ID=custom-image\n").unwrap();

    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();
    fs::write(&machine_id_path, "machine-id-value").unwrap();

    let result = settle_system_tokens(None, None, Some("os-image"), Some(temp_dir.path())).unwrap();

    assert_eq!(result.0, "custom-image");
}

#[test]
fn test_settle_system_tokens_when_os_id_missing() {
    let temp_dir = TempDir::new().unwrap();
    let os_release_path = temp_dir.path().join("etc/os-release");
    fs::create_dir_all(os_release_path.parent().unwrap()).unwrap();
    fs::write(&os_release_path, "NAME=Custom OS\n").unwrap();

    let temp_path = temp_dir.path();
    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();
    fs::write(&machine_id_path, "machine-id-value").unwrap();

    let result = settle_system_tokens(None, None, Some("os-id"), Some(temp_dir.path()));

    assert!(
        result.is_err(),
        "Expected an error due to missing ID in os-release"
    );
}

#[test]
fn test_settle_system_tokens_when_os_image_id_missing() {
    let temp_dir = TempDir::new().unwrap();
    let os_release_path = temp_dir.path().join("etc/os-release");
    fs::create_dir_all(os_release_path.parent().unwrap()).unwrap();
    fs::write(&os_release_path, "ID=custom-os\n").unwrap();

    let temp_path = temp_dir.path();
    let transactional_status_path = temp_path.join("proc/mounts");
    fs::create_dir_all(transactional_status_path.parent().unwrap()).unwrap();
    let mut mounts_file = File::create(&transactional_status_path).unwrap();
    writeln!(mounts_file, "").unwrap();
    let machine_id_path = temp_dir.path().join("etc/machine-id");
    fs::create_dir_all(machine_id_path.parent().unwrap()).unwrap();
    fs::write(&machine_id_path, "machine-id-value").unwrap();

    let result = settle_system_tokens(None, None, Some("os-image"), Some(temp_dir.path()));

    assert!(
        result.is_err(),
        "Expected an error due to missing IMAGE_ID in os-release"
    );
}
