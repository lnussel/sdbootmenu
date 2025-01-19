use super::log_info;
use cursive::event::Event;
use cursive::traits::*;
use cursive::views::{Dialog, LinearLayout, SelectView, TextView};

pub struct Menu {
    pub items: Vec<String>,
}

/// Handles menu actions based on the selected index.
///
/// # Arguments
///
/// * `idx` - The index of the selected menu item.
///
/// # Returns
///
/// A message indicating the action triggered by the selected menu item.
pub fn handle_menu_action(idx: usize) -> &'static str {
    match idx {
        0 => "Kernels action triggered",
        1 => "Snapshots action triggered",
        _ => "Unknown action",
    }
}

/// Responds to menu selection events by executing an action based on the selected item's index.
///
/// This function is typically used as a callback in menu-driven applications. It parses the
/// selected item's index, determines the appropriate action to take via `handle_menu_action`,
/// and logs the resulting message.
///
/// # Arguments
///
/// * `idx` - A string reference to the selected menu item's index. This string is parsed into a `usize`.
/// * `printer` - An implementation of the `MessagePrinter` trait used to log the action taken.
///
/// # Panics
///
/// This function will panic if `idx` cannot be parsed into a `usize`.
pub(crate) fn on_menu_select(idx: &String) -> u32 {
    let idx = idx.parse::<usize>().unwrap_or_default();
    let message = handle_menu_action(idx);
    log_info(&message, 1);
    idx.try_into().unwrap()
}

impl Menu {
    /// Creates a new `Menu` with the given list of items.
    ///
    /// # Arguments
    ///
    /// * `items` - A vector of strings representing the menu items.
    ///
    /// # Returns
    ///
    /// A `Menu` instance containing the provided items.
    pub fn new(items: Vec<&str>) -> Self {
        Menu {
            items: items.into_iter().map(String::from).collect(),
        }
    }

    /// Converts the `Menu` into a `cursive::views::SelectView` for use in a `cursive` TUI application.
    ///
    /// Each menu item is added to the `SelectView` along with its corresponding index.
    ///
    /// # Returns
    ///
    /// A `cursive::views::SelectView<String>` populated with the menu's items.
    pub fn into_select_view(self) -> SelectView<String> {
        let mut select_view = SelectView::new().h_align(cursive::align::HAlign::Center);

        for (idx, item) in self.items.iter().enumerate() {
            select_view.add_item(item.clone(), idx.to_string());
        }

        select_view
    }
}

pub fn show_main_menu() -> Result<bool, String> {
    let menu_items = vec!["Kernels", "Snapshots", "Entries", "Install/Update"];
    let menu = Menu::new(menu_items);
    let mut siv = cursive::default();

    let title = TextView::new("Systemd-boot").center();
    let select_view = menu.into_select_view().on_submit(move |_, idx| {
        on_menu_select(idx);
    });

    let dialog = Dialog::around(select_view.scrollable()).title("Main Menu");
    let layout = LinearLayout::vertical().child(title).child(dialog);

    siv.add_layer(layout);
    siv.add_global_callback(Event::Char('q'), |s| s.quit());
    siv.run();
    Ok(true)
}
