use super::super::{Target, fetch};
use super::*;
use crate::pull_request::{Input, Origin as Host};

#[test]
fn cancelled_remote_preparation_never_runs_ssh_or_reads_a_local_checkout() {
    let target = Target {
        host: Host::Ssh("unused.invalid".into()),
        input: Input {
            checkout: Some("/must-not-be-read-locally".into()),
            repo_key: Some("/remote/repository/.git".into()),
            branch: "main".into(),
        },
    };
    let item = &parse(&response(), &origin(), Kind::PullRequest).unwrap()[0];
    assert!(matches!(
        fetch::fetch_branch(&target, item, &|| true),
        Err(Error::GitHub(sheprd_github::Error::PullRequest(
            sheprd_github::pull_request::Error::Cancelled
        )))
    ));

    // Neither a Git directory nor a workspace folder: nothing to read, and no
    // host is asked.
    let target = Target {
        input: Input {
            checkout: None,
            repo_key: None,
            branch: "main".into(),
        },
        ..target
    };
    assert!(matches!(
        fetch::fetch_branch(&target, item, &|| false),
        Err(Error::GitHub(sheprd_github::Error::PullRequest(
            sheprd_github::pull_request::Error::Metadata
        )))
    ));
}
