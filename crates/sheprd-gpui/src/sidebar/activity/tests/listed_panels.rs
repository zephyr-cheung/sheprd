use super::super::panel_selector;
use super::*;
use crate::sidebar::layout_tests::{SidebarFixture, fixture_window, full_draw};
use gpui::{AppContext as _, Entity, VisualTestContext, px, size};

fn fixture(
    cx: &mut gpui::TestAppContext,
    panels: Option<Vec<ActivityPanel>>,
) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        crate::bind_keys(cx);
        let view = cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            view.config.sidebar_style.activity_bar = true;
            view.config.sidebar_style.panels = panels;
            view
        });
        cx.observe(&view, |_, _, cx| cx.notify()).detach();
        SidebarFixture(view)
    });
    cx.simulate_resize(size(px(1000.), px(700.)));
    let view = cx.update(|_, cx| fixture.read(cx).0.clone());
    draw(cx);
    (view, cx)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        full_draw(window, cx).clear(cx);
    });
}

#[gpui::test]
fn the_bar_offers_only_the_sections_the_config_lists(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = fixture(
        cx,
        Some(vec![ActivityPanel::Workspaces, ActivityPanel::PullRequests]),
    );
    assert!(
        cx.debug_bounds(panel_selector(ActivityPanel::Workspaces))
            .is_some()
    );
    assert!(
        cx.debug_bounds(panel_selector(ActivityPanel::PullRequests))
            .is_some()
    );
    for unlisted in [
        ActivityPanel::Overview,
        ActivityPanel::Issues,
        ActivityPanel::Tasks,
        ActivityPanel::Security,
        ActivityPanel::Automations,
        ActivityPanel::Ports,
        ActivityPanel::Usage,
    ] {
        assert!(
            cx.debug_bounds(panel_selector(unlisted)).is_none(),
            "{unlisted:?}"
        );
    }
}

/// The terminals' own section is the way back from any other, so a list that
/// forgets it keeps it. A window stored on a section the list drops falls back
/// to it rather than showing a section the bar cannot leave.
#[gpui::test]
fn workspaces_stays_and_a_stored_unlisted_section_yields(cx: &mut gpui::TestAppContext) {
    let (view, cx) = fixture(cx, Some(vec![ActivityPanel::Issues]));
    assert!(
        cx.debug_bounds(panel_selector(ActivityPanel::Workspaces))
            .is_some()
    );
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            assert!(view.activity_panel_listed(ActivityPanel::Workspaces));
            assert!(!view.activity_panel_listed(ActivityPanel::Usage));
            view.activity.panel = ActivityPanel::Usage;
            assert_eq!(view.activity_panel(), ActivityPanel::Workspaces);
            view.activity.panel = ActivityPanel::Issues;
            assert_eq!(view.activity_panel(), ActivityPanel::Issues);
        });
    });
}

/// No list at all is the whole bar, as every build had it.
#[gpui::test]
fn an_unset_list_offers_every_section(cx: &mut gpui::TestAppContext) {
    let (view, cx) = fixture(cx, None);
    for panel in [
        ActivityPanel::Workspaces,
        ActivityPanel::Overview,
        ActivityPanel::PullRequests,
        ActivityPanel::Issues,
        ActivityPanel::Security,
        ActivityPanel::Ports,
        ActivityPanel::Usage,
    ] {
        assert!(
            cx.debug_bounds(panel_selector(panel)).is_some(),
            "{panel:?}"
        );
    }
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            assert!(view.activity_panel_listed(ActivityPanel::Usage));
        });
    });
}
