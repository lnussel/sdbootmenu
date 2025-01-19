use super::super::utils::*;

#[test]
fn test_compare_versions() {
    assert!(compare_versions("1.0", "2.0"));
    assert!(!compare_versions("2.0", "1.0"));
    assert!(compare_versions("1.0", "1.1"));
    assert!(!compare_versions("1.2", "1.1"));
    assert!(compare_versions("1.2.0", "1.2.1"));
    assert!(!compare_versions("1.2.1", "1.2.0"));
    assert!(!compare_versions("1.2.1", "1.2.1"));
    assert!(compare_versions("1.2.1", "1.2.1.1"));
    assert!(!compare_versions("1.2.1.1", "1.2.1"));
    assert!(compare_versions("0.9.9", "1.0.0"));
    assert!(!compare_versions("1.0.0", "0.9.9"));
}

#[test]
fn test_compare_versions_complex() {
    assert!(compare_versions(
        "255.3+suse.16.g12345678",
        "255.4+suse.17.gbe772961ad"
    ));
    assert!(!compare_versions(
        "255.5+suse.18.gabcd1234",
        "255.4+suse.17.gbe772961ad"
    ));
    assert!(compare_versions(
        "255.4+suse.16.g12345678",
        "255.4+suse.17.gbe772961ad"
    ));
    assert!(!compare_versions(
        "255.4+suse.17.gbe772961ad",
        "255.4+suse.17.gbe772961ad"
    ));
}

#[test]
fn test_compare_versions_mixed() {
    assert!(compare_versions("1.0.0", "2.0.0"));
    assert!(!compare_versions("2.0.0", "1.0.0"));

    assert!(compare_versions("1.0.0-alpha", "1.0.0-alpha.1"));
    assert!(!compare_versions("1.0.0-alpha.1", "1.0.0-alpha"));

    assert!(compare_versions("1.0.0+build.1", "1.0.0+build.2"));

    assert!(compare_versions(
        "1.0.0-alpha+build.1",
        "1.0.0-beta+build.2"
    ));

    assert!(compare_versions(
        "255.3+suse.16.g12345678",
        "255.4+suse.17.gbe772961ad"
    ));
    assert!(!compare_versions(
        "255.4+suse.17.gbe772961ad",
        "255.4+suse.16.g12345678"
    ));

    assert!(compare_versions(
        "0.9.9-alpha+001",
        "1.0.0-beta+exp.sha.5114f85"
    ));
    assert!(compare_versions("1.0.0-rc.1+build.1", "1.0.0-rc.1+build.2"));
    assert!(!compare_versions(
        "1.0.0-rc.1+build.2",
        "1.0.0-rc.1+build.1"
    ));

    assert!(compare_versions(
        "1.0.0-dev.foo.bar+123",
        "1.0.0-dev.foo.baz+124"
    ));
}
