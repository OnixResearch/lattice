use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Act {
    Save,
    Open,
}

struct TestContributor {
    items: Vec<MenuContribution<Act>>,
}

impl MenuContributor<Act> for TestContributor {
    fn menu_items(&self) -> Vec<MenuContribution<Act>> {
        self.items.clone()
    }
}

fn contrib(key: char, label: &str, action: LeaderAction<Act>) -> MenuContribution<Act> {
    MenuContribution {
        key,
        label: label.into(),
        action,
        placement: MenuPlacement::Root,
        priority: PRIORITY_BUILTIN,
        source: "test".into(),
    }
}

struct ContributionSpec<'a> {
    key: char,
    label: &'a str,
    action: LeaderAction<Act>,
    placement: MenuPlacement,
    priority: u16,
    source: &'a str,
}

fn contrib_in(spec: ContributionSpec<'_>) -> MenuContribution<Act> {
    MenuContribution {
        key: spec.key,
        label: spec.label.into(),
        action: spec.action,
        placement: spec.placement,
        priority: spec.priority,
        source: spec.source.into(),
    }
}

fn empty_hidden() -> HiddenSet {
    std::collections::HashSet::new()
}

#[test]
fn single_contributor() {
    let c = TestContributor {
        items: vec![
            contrib('a', "alpha", LeaderAction::Command("/alpha".into())),
            contrib('b', "beta", LeaderAction::Command("/beta".into())),
        ],
    };
    let (menu, conflicts) = build(&[&c], &empty_hidden());
    assert!(conflicts.is_empty());
    assert_eq!(menu.root_def().items.len(), 2);
    assert_eq!(menu.root_def().items[0].key, 'a');
    assert_eq!(menu.root_def().items[1].key, 'b');
}

#[test]
fn higher_priority_wins() {
    let lo = TestContributor {
        items: vec![contrib_in(ContributionSpec {
            key: 'x',
            label: "lo",
            action: LeaderAction::Action(Act::Save),
            placement: MenuPlacement::Root,
            priority: PRIORITY_BUILTIN,
            source: "builtin",
        })],
    };
    let hi = TestContributor {
        items: vec![contrib_in(ContributionSpec {
            key: 'x',
            label: "hi",
            action: LeaderAction::Action(Act::Open),
            placement: MenuPlacement::Root,
            priority: PRIORITY_PLUGIN,
            source: "plugin",
        })],
    };
    let (menu, conflicts) = build(&[&lo, &hi], &empty_hidden());
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].winner, "plugin");
    assert_eq!(menu.root_def().items[0].label, "hi");
}

#[test]
fn hidden_excluded() {
    let c = TestContributor {
        items: vec![
            contrib('a', "keep", LeaderAction::Action(Act::Save)),
            contrib('b', "hide", LeaderAction::Action(Act::Open)),
        ],
    };
    let mut hidden = std::collections::HashSet::new();
    hidden.insert(('b', MenuPlacement::Root));
    let (menu, _) = build(&[&c], &hidden);
    assert_eq!(menu.root_def().items.len(), 1);
    assert_eq!(menu.root_def().items[0].key, 'a');
}

#[test]
fn submenu_auto_creation() {
    let c = TestContributor {
        items: vec![
            contrib_in(ContributionSpec {
                key: 'p',
                label: "plugins",
                action: LeaderAction::Submenu("plugins".into()),
                placement: MenuPlacement::Root,
                priority: PRIORITY_BUILTIN,
                source: "test",
            }),
            contrib_in(ContributionSpec {
                key: 'c',
                label: "calendar",
                action: LeaderAction::Command("/cal".into()),
                placement: MenuPlacement::Submenu("plugins".into()),
                priority: PRIORITY_PLUGIN,
                source: "calendar",
            }),
        ],
    };
    let (menu, _) = build(&[&c], &empty_hidden());
    assert_eq!(menu.root_def().items.len(), 1);
    let subs = menu.submenu_defs();
    let plugins = subs.iter().find(|s| s.label == "plugins").unwrap();
    assert_eq!(plugins.items.len(), 1);
    assert_eq!(plugins.items[0].key, 'c');
}

#[test]
fn same_key_different_placement_no_conflict() {
    let c = TestContributor {
        items: vec![
            contrib_in(ContributionSpec {
                key: 'a',
                label: "root-a",
                action: LeaderAction::Action(Act::Save),
                placement: MenuPlacement::Root,
                priority: PRIORITY_BUILTIN,
                source: "test",
            }),
            contrib_in(ContributionSpec {
                key: 'a',
                label: "sub-a",
                action: LeaderAction::Action(Act::Open),
                placement: MenuPlacement::Submenu("foo".into()),
                priority: PRIORITY_BUILTIN,
                source: "test",
            }),
        ],
    };
    let (menu, conflicts) = build(&[&c], &empty_hidden());
    assert!(conflicts.is_empty());
    assert_eq!(menu.root_def().items.len(), 1);
}

#[test]
fn empty_build() {
    let (menu, conflicts) = build::<Act>(&[], &empty_hidden());
    assert!(conflicts.is_empty());
    assert!(menu.root_def().items.is_empty());
}
