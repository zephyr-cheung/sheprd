#![allow(clippy::unwrap_used)]
use super::*;
use crate::pull_request::{Error, Input};
use std::time::Duration;

#[test]
fn both_hosts_resolve_forks_and_ignore_non_github_or_duplicate_upstreams() {
    let origin = || Ok(("contributor".into(), "project".into()));
    assert_eq!(
        base(origin(), Some("git@github.com:maintainer/project.git")).unwrap(),
        Base {
            owner: "maintainer".into(),
            repo: "project".into(),
            remote: Remote::Upstream,
            fork_owner: Some("contributor".into()),
        }
    );
    for upstream in [
        None,
        Some("https://gitlab.com/maintainer/project"),
        Some("https://github.com/CONTRIBUTOR/Project"),
    ] {
        assert_eq!(
            base(origin(), upstream).unwrap(),
            Base {
                owner: "contributor".into(),
                repo: "project".into(),
                remote: Remote::Origin,
                fork_owner: None,
            }
        );
    }
    assert_eq!(
        base(
            Err(Error::Origin.into()),
            Some("https://github.com/maintainer/project")
        )
        .unwrap()
        .remote,
        Remote::Upstream
    );
    assert!(matches!(
        base(Err(Error::Origin.into()), None),
        Err(crate::Error::PullRequest(Error::Origin))
    ));
}

/// The directory is a path on the host, in the spelling the host uses: the
/// daemon's Git directory when it names one, else the workspace's own folder,
/// which a Windows daemon reports and `git -C` also reads.
#[test]
fn a_hosts_git_directory_or_workspace_folder_is_read() {
    let mut input = Input {
        checkout: None,
        repo_key: None,
        branch: "main".into(),
    };
    assert!(matches!(
        remote_directory(&input),
        Err(crate::Error::PullRequest(Error::Metadata))
    ));
    input.repo_key = Some(r"C:/repo/.git".into());
    input.checkout = Some(r"C:/repo".into());
    assert_eq!(remote_directory(&input).unwrap(), r"C:/repo/.git");
    input.repo_key = None;
    assert_eq!(remote_directory(&input).unwrap(), r"C:/repo");
    input.repo_key = Some(r"\\?\C:\repo\.git".into());
    assert_eq!(remote_directory(&input).unwrap(), r"\\?\C:\repo\.git");
}

#[test]
fn remote_repository_refuses_missing_metadata_invalid_paths_and_expired_deadlines() {
    let mut input = Input {
        checkout: None,
        repo_key: None,
        branch: "main".into(),
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    assert!(matches!(
        remote_repository("work", &input, deadline, &|| false),
        Err(crate::Error::PullRequest(Error::Metadata))
    ));
    // A workspace directory is read as it came, so a relative one is refused
    // before SSH runs.
    input.checkout = Some("relative".into());
    assert!(matches!(
        remote_repository("work", &input, deadline, &|| false),
        Err(crate::Error::Client(sheprd_client::Error::InvalidGitDir))
    ));
    input.checkout = Some("/srv/app".into());
    for path in ["relative", "/repo\nother"] {
        input.repo_key = Some(path.into());
        assert!(matches!(
            remote_repository("work", &input, deadline, &|| false),
            Err(crate::Error::Client(sheprd_client::Error::InvalidGitDir))
        ));
    }
    input.repo_key = Some("/repo/.git".into());
    input.checkout = None;
    // Windows can report the same Instant on consecutive reads. Both an
    // expired deadline and a zero remaining budget must stop before SSH.
    for expired in [Instant::now() - Duration::from_secs(1), Instant::now()] {
        assert!(matches!(
            remote_repository("work", &input, expired, &|| false),
            Err(crate::Error::PullRequest(Error::Timeout))
        ));
    }
}
