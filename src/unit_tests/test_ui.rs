use super::super::ui::*;

#[test]
fn test_handle_menu_action() {
    assert_eq!(handle_menu_action(0), "Kernels action triggered");
    assert_eq!(handle_menu_action(1), "Snapshots action triggered");
    assert_eq!(handle_menu_action(999), "Unknown action");
}

#[test]
fn test_menu_new_and_into_select_view() {
    let items = vec!["Item1", "Item2"];
    let menu = Menu::new(items.clone());
    assert_eq!(menu.items, items);

    let select_view = menu.into_select_view();
    assert_eq!(select_view.len(), items.len());
}

#[test]
fn test_on_menu_select() {
    assert_eq!(
        on_menu_select(&"0".to_string()),
        0,
        "Should return 0 for Kernels action"
    );
    assert_eq!(
        on_menu_select(&"1".to_string()),
        1,
        "Should return 1 for Snapshots action"
    );
    assert_eq!(
        on_menu_select(&"2".to_string()),
        2,
        "Should return 2 for unknown actions"
    );
}
