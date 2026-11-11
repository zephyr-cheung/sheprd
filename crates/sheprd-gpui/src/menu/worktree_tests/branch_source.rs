use super::*;
use crate::repo_items::BranchSource;

/// The local daemon's own folder is read here, by this machine's Git.
#[gpui::test]
fn a_local_pickers_branches_are_read_here(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    open_dialog(&view, cx, true, Tab::Branches);
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            let Ok(BranchSource::Local(input)) = view.branch_source() else {
                panic!("the local fixture reads its own folder");
            };
            assert_eq!(
                input.repo_key.as_deref(),
                Some(sidebar::layout_tests::REPO_KEY)
            );
        });
    });
}

/// A saved SSH host's checkout is read by that host's own Git: this machine
/// cannot see `C:\…`, which is what the daemon there reports.
#[gpui::test]
fn a_remote_pickers_branches_are_read_on_its_host(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            view.endpoints.push(crate::endpoint::Endpoint::new(
                "ssh:work".into(),
                "Work".into(),
                sheprd_client::ConnectTarget::Ssh {
                    target: "me@work".into(),
                    session: "default".into(),
                },
                true,
            ));
            view.selected_endpoint = 1;
        });
    });
    open_dialog(&view, cx, true, Tab::Branches);
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            let Ok(BranchSource::Ssh { host, input }) = view.branch_source() else {
                panic!("a saved SSH host reads its own repository");
            };
            assert_eq!(host, "me@work");
            assert_eq!(
                input.repo_key.as_deref(),
                Some("/fixture/agent-launcher/.git")
            );

            // A forwarded socket names no host to ask, so nothing may read it.
            view.endpoints[1].connection.target =
                sheprd_client::ConnectTarget::Socket("/forwarded.sock".into());
            assert!(view.branch_source().is_err());
        });
    });
}
