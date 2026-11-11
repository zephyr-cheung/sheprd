use super::*;

/// A Windows host names no folder for the workspace itself, and every reader
/// of the snapshot — Issues, Review, the sidebar's avatar — needs one.
#[test]
fn a_received_workspace_without_a_folder_takes_its_panes() {
    let mut raw = (*snapshot()).clone();
    let workspace = &mut raw.workspaces[0];
    workspace.new_workspace_cwd = String::new();
    let id = workspace.workspace_id.clone();
    let folder = raw
        .panes
        .iter()
        .find(|pane| pane.workspace_id == id && pane.cwd.is_some())
        .and_then(|pane| pane.cwd.clone())
        .expect("the fixture runs a pane in its first workspace");
    let mut state = LiveState::default();
    state.apply(ClientEvent::Snapshot(Arc::new(raw)));
    let shown = state.snapshot.as_deref().unwrap();
    assert_eq!(shown.workspaces[0].new_workspace_cwd, folder);
}

/// A snapshot whose workspaces all name their folder is passed through, not
/// rebuilt, so the surface that pairs with it keeps its own reference.
#[test]
fn a_snapshot_that_names_every_folder_is_kept_as_it_arrived() {
    let raw = snapshot();
    let mut state = LiveState::default();
    state.apply(ClientEvent::Snapshot(raw.clone()));
    assert!(Arc::ptr_eq(state.snapshot.as_ref().unwrap(), &raw));
}
