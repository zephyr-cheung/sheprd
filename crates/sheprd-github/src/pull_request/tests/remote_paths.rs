use super::*;

/// An SSH host judges its own paths: a Windows daemon reports `C:\…`, and a
/// workspace it named no Git directory for is read at its own folder.
#[test]
fn an_ssh_hosts_paths_are_judged_by_that_host() {
    let mut workspace = shell_workspace();
    workspace.branch = Some("main".into());
    workspace.worktree = Some(sheprd_client::protocol::ClientShellWorktree {
        key: "/remote/project/.git".into(),
        label: "project".into(),
        is_linked_worktree: true,
    });
    let host = Origin::Ssh("work".into());
    let input = super::super::workspace_input(&workspace, &host).unwrap();
    assert_eq!(input.repo_key.as_deref(), Some("/remote/project/.git"));
    assert!(input.checkout.is_none());

    for key in [
        r"C:\project\.git",
        "C:/project/.git",
        r"\\?\C:\project\.git",
        r"\\server\share\project\.git",
    ] {
        workspace.worktree.as_mut().unwrap().key = key.into();
        let input = super::super::workspace_input(&workspace, &host).unwrap();
        assert_eq!(input.repo_key.as_deref(), Some(key), "{key}");
    }
    for key in ["relative", "/remote\nproject/.git", ""] {
        workspace.worktree.as_mut().unwrap().key = key.into();
        assert!(
            matches!(
                super::super::workspace_input(&workspace, &host),
                Err(Error::PullRequest(super::super::Error::AbsolutePath))
            ),
            "{key}"
        );
    }

    // No Git directory from the daemon: the host's own folder is read instead,
    // which is what a Windows daemon leaves for a main checkout.
    workspace.worktree = None;
    workspace.new_workspace_cwd = r"C:\Users\me\app".into();
    let input = super::super::workspace_input(&workspace, &host).unwrap();
    assert!(input.repo_key.is_none());
    assert_eq!(input.checkout.as_deref(), Some(r"C:\Users\me\app"));
    assert_eq!(input.branch, "main");

    // Neither named: there is nothing to read.
    workspace.new_workspace_cwd = String::new();
    assert!(matches!(
        super::super::workspace_input(&workspace, &host),
        Err(Error::PullRequest(super::super::Error::Metadata))
    ));
    // Nor is a folder the host would not call absolute.
    workspace.new_workspace_cwd = "app".into();
    assert!(matches!(
        super::super::workspace_input(&workspace, &host),
        Err(Error::PullRequest(super::super::Error::AbsolutePath))
    ));
}
