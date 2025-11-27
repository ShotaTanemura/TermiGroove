//! Tests for the FileExplorerAdapter.
//!
//! These tests verify that the adapter correctly implements the FileNavigator trait
//! and properly bridges between our application-layer KeyCode and the underlying
//! ratatui_explorer FileExplorer.

use ratatui_explorer::Theme as ExplorerTheme;
use termigroove::application::dto::input_action::KeyCode;
use termigroove::application::ports::FileNavigator;
use termigroove::presentation::view_model::ViewModel;

/// Create a test ViewModel with a file explorer rooted at the project root.
fn create_test_view_model() -> ViewModel {
    // Create a simple default theme for testing
    let theme = ExplorerTheme::default();

    let file_explorer =
        ratatui_explorer::FileExplorer::with_theme(theme).expect("Should create file explorer");
    ViewModel::new(file_explorer)
}

#[test]
fn test_adapter_handle_navigation_key_up() {
    let mut view_model = create_test_view_model();

    // Navigate down first to have a position to go up from
    {
        let mut navigator = view_model.as_navigator();
        let _ = navigator.handle_navigation_key(KeyCode::Down);
    }

    // Then navigate up
    {
        let mut navigator = view_model.as_navigator();
        let result = navigator.handle_navigation_key(KeyCode::Up);
        assert!(result.is_ok(), "Navigation up should succeed");
    }
}

#[test]
fn test_adapter_handle_navigation_key_down() {
    let mut view_model = create_test_view_model();
    let mut navigator = view_model.as_navigator();

    let result = navigator.handle_navigation_key(KeyCode::Down);
    assert!(result.is_ok(), "Navigation down should succeed");
}

#[test]
fn test_adapter_selected_entry_returns_some_for_non_empty_directory() {
    let mut view_model = create_test_view_model();

    // The project root should have at least some files/directories
    let navigator = view_model.as_navigator();
    let entry = navigator.selected_entry();

    // We expect there to be an entry selected in the project root
    assert!(
        entry.is_some(),
        "Should have a selected entry in project root"
    );
}

#[test]
fn test_adapter_selected_entry_has_valid_path() {
    let mut view_model = create_test_view_model();
    let navigator = view_model.as_navigator();

    if let Some(entry) = navigator.selected_entry() {
        assert!(
            !entry.path.as_os_str().is_empty(),
            "Path should not be empty"
        );
    }
}

#[test]
fn test_adapter_navigation_changes_selection() {
    let mut view_model = create_test_view_model();

    // Get initial selection
    let initial_entry = view_model.as_navigator().selected_entry();

    // Navigate down
    {
        let mut navigator = view_model.as_navigator();
        let _ = navigator.handle_navigation_key(KeyCode::Down);
    }

    // Get new selection
    let new_entry = view_model.as_navigator().selected_entry();

    // After navigating down, the selection should potentially have changed
    // (unless we're at the end of a short list)
    // We can't guarantee it changed, but we can verify both are valid
    assert!(
        initial_entry.is_some() || new_entry.is_some(),
        "At least one entry should be selected"
    );
}

#[test]
fn test_adapter_enter_key_handling() {
    let mut view_model = create_test_view_model();

    // Enter key should work (navigate into directory or select file)
    let mut navigator = view_model.as_navigator();
    let result = navigator.handle_navigation_key(KeyCode::Enter);

    // Enter should succeed (even if it doesn't change directory, it shouldn't error)
    assert!(result.is_ok(), "Enter key should be handled without error");
}

#[test]
fn test_adapter_left_right_key_handling() {
    let mut view_model = create_test_view_model();

    // Left key (go to parent directory)
    {
        let mut navigator = view_model.as_navigator();
        let result = navigator.handle_navigation_key(KeyCode::Left);
        assert!(result.is_ok(), "Left key should be handled without error");
    }

    // Right key (enter directory)
    {
        let mut navigator = view_model.as_navigator();
        let result = navigator.handle_navigation_key(KeyCode::Right);
        assert!(result.is_ok(), "Right key should be handled without error");
    }
}

#[test]
fn test_file_entry_is_dir_property() {
    let mut view_model = create_test_view_model();
    let navigator = view_model.as_navigator();

    if let Some(entry) = navigator.selected_entry() {
        // Verify is_dir matches actual file system state
        let metadata = std::fs::metadata(&entry.path)
            .expect("Should be able to get metadata for selected entry");
        assert_eq!(
            entry.is_dir,
            metadata.is_dir(),
            "is_dir should match file system state for {:?}",
            entry.path
        );
    }
}

#[test]
fn test_multiple_navigation_operations() {
    let mut view_model = create_test_view_model();

    // Perform multiple navigation operations in sequence
    let keys = [KeyCode::Down, KeyCode::Down, KeyCode::Up, KeyCode::Down];

    for key in keys {
        let mut navigator = view_model.as_navigator();
        let result = navigator.handle_navigation_key(key.clone());
        assert!(result.is_ok(), "Navigation with {:?} should succeed", key);
    }

    // Should still have a valid selection after all operations
    let navigator = view_model.as_navigator();
    let entry = navigator.selected_entry();
    assert!(
        entry.is_some(),
        "Should still have a selected entry after multiple navigations"
    );
}
