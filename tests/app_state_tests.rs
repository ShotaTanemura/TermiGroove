use std::path::PathBuf;

use termigroove::selection::SelectionModel;

#[test]
fn add_appends_and_sets_status_and_cursor() {
    let mut m = SelectionModel::default();
    m.add_file(PathBuf::from("/tmp/a.wav"));
    assert_eq!(m.items.len(), 1);
    assert_eq!(m.right_idx, 0);
    assert!(m.status.starts_with("Added "));
}

#[test]
fn toggle_remove_via_add_file_removes_and_updates_cursor() {
    let mut m = SelectionModel::default();
    m.add_file(PathBuf::from("/tmp/a.wav"));
    m.add_file(PathBuf::from("/tmp/b.wav"));
    // toggle a.wav by adding it again
    m.add_file(PathBuf::from("/tmp/a.wav"));
    assert_eq!(m.items.len(), 1);
    assert_eq!(m.items[0].file_name().unwrap(), "b.wav");
    assert!(m.status.starts_with("Removed "));
    // cursor should clamp to last
    assert_eq!(m.right_idx, 0);
}

#[test]
fn remove_at_cursor_repositions_cursor() {
    let mut m = SelectionModel::default();
    m.add_file(PathBuf::from("/tmp/a.wav"));
    m.add_file(PathBuf::from("/tmp/b.wav"));
    m.add_file(PathBuf::from("/tmp/c.wav"));
    m.right_idx = 2;
    m.remove_at_cursor(); // remove c
    assert_eq!(m.items.len(), 2);
    assert_eq!(m.right_idx, 1); // now points to b
}

#[test]
fn move_up_down_bounds() {
    let mut m = SelectionModel::default();
    m.add_file(PathBuf::from("/tmp/a.wav"));
    m.add_file(PathBuf::from("/tmp/b.wav"));
    m.right_idx = 0;
    m.move_up();
    assert_eq!(m.right_idx, 0);
    m.move_down();
    assert_eq!(m.right_idx, 1);
    m.move_down();
    assert_eq!(m.right_idx, 1);
}

#[test]
fn empty_list_noops_on_nav_and_remove() {
    let mut m = SelectionModel::default();
    // initial
    assert_eq!(m.items.len(), 0);
    assert_eq!(m.right_idx, 0);
    let before_status = m.status.clone();

    // No-ops should not panic and should not alter index/status
    m.move_up();
    m.move_down();
    m.remove_at_cursor();

    assert_eq!(m.items.len(), 0);
    assert_eq!(m.right_idx, 0);
    assert_eq!(m.status, before_status);
}
