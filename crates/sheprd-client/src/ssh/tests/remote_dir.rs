use super::*;

/// A remote host judges its own paths: a Windows checkout is absolute there
/// and would be rejected by this machine's own rules.
#[test]
fn a_directory_is_absolute_by_its_own_hosts_rules() {
    for absolute in [
        "/srv/app",
        "/",
        r"C:\app",
        "C:/app",
        r"C:\",
        r"\\?\C:\app",
        r"\\server\share\app",
        r"\\?\UNC\server\share",
    ] {
        assert!(is_absolute_dir(absolute), "{absolute}");
    }
    for relative in ["", ".", "app", "./app", "C:", "C:app", r"\app", "~/.local"] {
        assert!(!is_absolute_dir(relative), "{relative}");
    }
    assert!(!is_absolute_dir(&format!("/{}", "a".repeat(4096))));
    assert!(!is_absolute_dir("/srv/\u{1}"));
}

/// Windows' sshd hands the shell `\\?\C:\app` as `\?\C:\app`, which Git
/// refuses; the plain spelling is the one it accepts.
#[test]
fn an_extended_length_prefix_is_dropped_for_the_host() {
    assert_eq!(plain_remote_dir(r"\\?\C:\app"), r"C:\app");
    assert_eq!(plain_remote_dir(r"\\?\UNC\server\share"), r"\\server\share");
    assert_eq!(plain_remote_dir(r"C:\app"), r"C:\app");
    assert_eq!(plain_remote_dir("/srv/app"), "/srv/app");
    assert_eq!(plain_remote_dir(r"\\server\share"), r"\\server\share");
}
