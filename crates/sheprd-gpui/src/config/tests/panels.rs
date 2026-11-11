use super::*;
use crate::sidebar::ActivityPanel;

#[test]
fn every_section_is_listed_unless_the_config_names_them() -> anyhow::Result<()> {
    for config in [Config::default(), Config::parse(DEFAULT_CONFIG)?] {
        assert_eq!(config.sidebar_style.panels, None);
    }
    let named =
        Config::parse("[sidebar]\npanels = [\"workspaces\", \"pull-requests\", \"issues\"]")?;
    assert_eq!(
        named.sidebar_style.panels,
        Some(vec![
            ActivityPanel::Workspaces,
            ActivityPanel::PullRequests,
            ActivityPanel::Issues
        ])
    );
    // Names are the bar's own, and a section this build does not have is a
    // config error rather than a silently empty bar.
    assert!(matches!(
        Config::parse("[sidebar]\npanels = [\"workspaces\", \"dashboard\"]"),
        Err(crate::Error::Settings(Error::UnknownPanel(name)))
            if name == "dashboard"
    ));
    assert!(Config::parse("[sidebar]\npanels = \"workspaces\"").is_err());
    Ok(())
}
