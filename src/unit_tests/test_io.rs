use super::super::io::*;
use std::path::{Path, PathBuf};

#[test]
fn test_non_existent_command() {
    let result = get_command_output("command_that_does_not_exist", &["arg1"]);

    assert!(
        result.is_err(),
        "Expected an error when executing a non-existent command"
    );
}

#[test]
fn test_command_output() {
    let command_output = get_command_output("echo", &["This is a test"]).unwrap();
    assert_eq!(
        command_output, "This is a test",
        "Expected 'This is a test' as command output"
    );
}

#[test]
fn test_get_findmnt_output_with_override() {
    let override_path = Path::new("/dummy/path");
    let (uuid, source) = get_findmnt_output("/mount/point", Some(override_path)).unwrap();
    assert_eq!(uuid, "123456789");
    assert_eq!(source, "sda1");
}

#[test]
fn test_create_efi_boot_entry_with_override() {
    let override_path = Path::new("/dummy/path");
    let result = create_efi_boot_entry(
        &PathBuf::from("/mount/point"),
        0,
        &PathBuf::from("/mount/point"),
        Some(override_path),
    );
    assert!(result.is_ok());
}

#[test]
fn test_get_bootctl_info_with_override() {
    let override_path = Path::new("/dummy/path");
    let (firmware_arch, entry_token, boot_root) = get_bootctl_info(Some(override_path)).unwrap();
    assert_eq!(firmware_arch, "x64");
    assert_eq!(entry_token, "entry_token");
    assert_eq!(boot_root, override_path.to_string_lossy());
}
