use super::*;
use sheprd_client::protocol::ClientShellPane;

fn pane(workspace: &str, cwd: Option<&str>, foreground: Option<&str>) -> ClientShellPane {
    ClientShellPane {
        pane_id: format!("{workspace}:p1"),
        workspace_id: workspace.into(),
        tab_id: "t0".into(),
        label: None,
        cwd: cwd.map(Into::into),
        foreground_cwd: foreground.map(Into::into),
        focused: true,
        right_click_passthrough: false,
    }
}

/// A Windows daemon names no folder for the workspace itself, so the folder
/// its pane runs in is the only one there is.
#[test]
fn a_workspace_without_a_folder_of_its_own_takes_its_panes() {
    let mut snapshot = snapshot(1);
    snapshot.workspaces[0].new_workspace_cwd = String::new();
    snapshot
        .panes
        .push(pane("w0", Some(r"C:\Users\me\app"), None));
    let workspace = &snapshot.workspaces[0];
    assert_eq!(
        workspace_folder(&snapshot, workspace),
        Some(r"C:\Users\me\app")
    );
    assert_eq!(of_snapshot(&snapshot).workspaces[0].cwd, r"C:\Users\me\app");
}

/// The pane's own folder wins over the one a program moved into.
#[test]
fn a_panes_folder_is_read_before_the_one_a_program_moved_into() {
    let mut snapshot = snapshot(1);
    snapshot.workspaces[0].new_workspace_cwd = String::new();
    snapshot
        .panes
        .push(pane("w0", Some("/srv/app"), Some("/srv/app/src")));
    let workspace = &snapshot.workspaces[0];
    assert_eq!(workspace_folder(&snapshot, workspace), Some("/srv/app"));
}

/// Some daemons report `.` rather than nothing.
#[test]
fn a_dot_names_no_folder() {
    let mut snapshot = snapshot(1);
    snapshot.workspaces[0].new_workspace_cwd = " . ".into();
    snapshot.panes.push(pane("w0", Some("/srv/app"), None));
    let workspace = &snapshot.workspaces[0];
    assert_eq!(workspace_folder(&snapshot, workspace), Some("/srv/app"));
}

/// Another workspace's panes are not this one's.
#[test]
fn only_this_workspaces_panes_count() {
    let mut snapshot = snapshot(2);
    snapshot.workspaces[1].new_workspace_cwd = String::new();
    snapshot.panes.push(pane("w0", Some("/srv/other"), None));
    let workspace = &snapshot.workspaces[1];
    assert_eq!(workspace_folder(&snapshot, workspace), None);
    // Without a folder the session keeps what the daemon said, as before.
    assert_eq!(of_snapshot(&snapshot).workspaces[1].cwd, "");
}
