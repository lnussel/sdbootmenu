use super::super::cli::*;
use clap::error::ErrorKind;
use clap::CommandFactory;
use clap::Parser;
use std::path::Path;

#[test]
fn test_non_empty_string_with_non_empty_input() {
    let result = non_empty_string("test");
    assert_eq!(result, Ok("test".to_string()));
}

#[test]
fn test_non_empty_string_with_empty_input() {
    let result = non_empty_string("");
    assert!(result.is_err());
    assert_eq!(result, Err("Value cannot be empty"));
}

#[test]
fn test_parse_kernels_command() {
    let args = Args::try_parse_from(vec!["test", "kernels"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::Kernels {})));
}

#[test]
fn test_parse_snapshots_command() {
    let args = Args::try_parse_from(vec!["test", "snapshots"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::Snapshots {})));
}

#[test]
fn test_parse_add_kernel_command_with_version() {
    let args = Args::try_parse_from(vec!["test", "add-kernel", "5.10"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    match args.cmd {
        Some(Commands::AddKernel { kernel_version }) => assert_eq!(kernel_version, "5.10"),
        _ => panic!("Expected AddKernel command"),
    }
}

#[test]
fn test_parse_with_esp_path() {
    let args = Args::try_parse_from(vec!["test", "--esp-path", "/boot/efi"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.esp_path, Some("/boot/efi".to_string()));
}

#[test]
fn test_parse_with_invalid_esp_path() {
    let args = Args::try_parse_from(vec!["test", "--esp-path", ""]);
    assert!(args.is_err());
    let error = args.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::ValueValidation);
}

#[test]
fn test_parse_with_no_variables_flag() {
    let args = Args::try_parse_from(vec!["test", "--no-variables"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(args.no_variables);
}

#[test]
fn test_parse_with_verbosity() {
    let args = Args::try_parse_from(vec!["test", "-vvv"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.verbosity, 3);
}

#[test]
fn test_parse_set_default_snapshot_command() {
    let args = Args::try_parse_from(vec!["test", "set-default-snapshot"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::SetDefaultSnapshot {})));
}

#[test]
fn test_parse_is_bootable_command() {
    let args = Args::try_parse_from(vec!["test", "is-bootable"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::IsBootable {})));
}

#[test]
fn test_parse_with_architecture() {
    let args = Args::try_parse_from(vec!["test", "--arch", "x86_64"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.arch, Some("x86_64".to_string()));
}

#[test]
fn test_parse_with_entry_token() {
    let args = Args::try_parse_from(vec!["test", "--entry-token", "token123"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.entry_token, Some("token123".to_string()));
}

#[test]
fn test_parse_with_image() {
    let args = Args::try_parse_from(vec!["test", "--image", "vmlinuz-linux"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.image, Some("vmlinuz-linux".to_string()));
}

#[test]
fn test_parse_with_regenerate_initrd_flag() {
    let args = Args::try_parse_from(vec!["test", "--regenerate-initrd"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(args.regenerate_initrd);
}

#[test]
fn test_parse_remove_kernel_command_with_version() {
    let args = Args::try_parse_from(vec!["test", "remove-kernel", "5.10"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    match args.cmd {
        Some(Commands::RemoveKernel { kernel_version }) => assert_eq!(kernel_version, "5.10"),
        _ => panic!("Expected RemoveKernel command"),
    }
}

#[test]
fn test_parse_list_entries_command() {
    let args = Args::try_parse_from(vec!["test", "list-entries"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::ListEntries {})));
}

#[test]
fn test_parse_install_command() {
    let args = Args::try_parse_from(vec!["test", "install"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::Install {})));
}

#[test]
fn test_parse_update_command() {
    let args = Args::try_parse_from(vec!["test", "update"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::Update {})));
}

#[test]
fn test_parse_force_update_command() {
    let args = Args::try_parse_from(vec!["test", "force-update"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::ForceUpdate {})));
}

#[test]
fn test_parse_update_predictions_command() {
    let args = Args::try_parse_from(vec!["test", "update-predictions"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert!(matches!(args.cmd, Some(Commands::UpdatePredictions {})));
}

#[test]
fn test_default_values_for_optional_arguments() {
    let args = Args::try_parse_from(vec!["test"]).unwrap();
    assert!(args.esp_path.is_none());
    assert!(args.arch.is_none());
    assert!(args.entry_token.is_none());
    assert!(args.image.is_none());
    assert_eq!(args.verbosity, 0);
}

#[test]
fn test_help_command() {
    let mut app = Args::command();
    let mut help_output = Vec::new();
    app.write_help(&mut help_output).unwrap();
    let help_string = String::from_utf8(help_output).unwrap();
    assert!(help_string.contains("Usage:"));
}

#[test]
fn test_version_command() {
    let app = Args::command();
    assert_eq!(app.get_version().unwrap(), "0.1.0"); // Replace "1.0" with your actual version
}

#[test]
fn test_invalid_numeric_input_for_snapshot() {
    let result = Args::try_parse_from(vec!["test", "--snapshot", "abc"]);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.kind(), ErrorKind::ValueValidation); // Use the correct ErrorKind variant
}

#[test]
fn test_verbosity_levels() {
    let args = Args::try_parse_from(vec!["test", "-vv"]);
    assert!(args.is_ok());
    let args = args.unwrap();
    assert_eq!(args.verbosity, 2);
}

#[test]
fn test_combining_flags() {
    let args = Args::try_parse_from(vec!["test", "--no-variables", "--regenerate-initrd"]).unwrap();
    assert!(args.no_variables, "Expected no_variables flag to be true");
    assert!(
        args.regenerate_initrd,
        "Expected regenerate_initrd flag to be true"
    );
}

#[test]
fn test_default_snapshot_when_not_specified() {
    let args = Args::try_parse_from(vec!["test"]).unwrap();
    assert!(
        args.snapshot.is_none(),
        "Expected no default value for snapshot"
    );
}

#[test]
fn test_ensure_root_permissions_override() {
    let override_path = Path::new("/dummy/path");
    let result = ensure_root_permissions(Some(override_path));
    assert!(result.is_ok());
}
