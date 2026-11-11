//! Bounded Git work on the saved SSH host that owns a picker target.

use super::Item;
use sheprd_client::shell_quote;
use std::time::Instant;

pub(super) fn directory(path: &str) -> crate::Result<std::borrow::Cow<'_, str>> {
    let path = sheprd_client::ssh::plain_remote_dir(path);
    if !sheprd_client::ssh::is_absolute_dir(&path) {
        return Err(sheprd_client::Error::InvalidGitDir.into());
    }
    Ok(path)
}

/// The local branches of the host's repository, in the lines
/// [`super::branches::parse`] reads. `--git-dir` names the repository the
/// daemon reported, so no working directory has to be found on the host.
pub(super) fn branches_script(path: &str, format: &str) -> crate::Result<String> {
    Ok(format!(
        // i18n: data
        "git -c core.fsmonitor=false --git-dir {} for-each-ref --sort=-committerdate --format={} refs/heads\n",
        shell_quote(&directory(path)?),
        shell_quote(format),
    ))
}

pub(super) fn fetch_script(path: &str, item: &Item) -> crate::Result<String> {
    // Let Git choose the host's configured key or SSH wrapper. Only supply
    // OpenSSH defaults when neither the environment nor Git config names one.
    // The remote command has no TTY; disable graphical SSH prompts as well.
    Ok(format!(
        // i18n: data
        r#"set -eu
export GIT_TERMINAL_PROMPT=0 GCM_INTERACTIVE=never SSH_ASKPASS_REQUIRE=never
directory={}
if [ "${{GIT_SSH_COMMAND+x}}" != x ] && [ "${{GIT_SSH+x}}" != x ]; then
    if git -c core.fsmonitor=false -C "$directory" config --get core.sshCommand >/dev/null; then
        :
    else
        status=$?
        [ "$status" -eq 1 ] || exit "$status"
        export GIT_SSH_COMMAND='ssh -o BatchMode=yes -o StrictHostKeyChecking=yes'
    fi
fi
exec git -c core.fsmonitor=false -C "$directory" fetch --no-tags --quiet {} {}
"#,
        shell_quote(&directory(path)?),
        shell_quote(item.remote.name()),
        shell_quote(&item.fetch_refspec()),
    ))
}

pub(super) fn run(
    host: &str,
    script: &str,
    deadline: Instant,
    cancelled: &impl Fn() -> bool,
) -> crate::Result<()> {
    let mut command = sheprd_client::ssh::script_command(host, script)?;
    let (ok, _) = crate::pull_request::run(&mut command, deadline, cancelled)?;
    if !ok {
        return Err(sheprd_vcs::Error::from(crate::git::Error::Failed {
            operation: "prepare the remote pull request or issue checkout", // i18n: data
            // Stands where Git's output would: the host's own output is not read.
            details: crate::i18n::fl!("repo-items-remote-failed"),
        })
        .into());
    }
    Ok(())
}

/// [`run`] keeping what the script printed, for a read whose output is the
/// answer rather than a ref it left behind.
pub(super) fn output(
    host: &str,
    script: &str,
    deadline: Instant,
    cancelled: &impl Fn() -> bool,
) -> crate::Result<String> {
    let mut command = sheprd_client::ssh::script_command(host, script)?;
    let (ok, output) = crate::pull_request::run(&mut command, deadline, cancelled)?;
    if !ok {
        return Err(sheprd_vcs::Error::from(crate::git::Error::Failed {
            operation: "read the remote repository", // i18n: data
            // Stands where Git's output would: the host's own output is not read.
            details: crate::i18n::fl!("repo-items-remote-failed"),
        })
        .into());
    }
    Ok(output)
}
