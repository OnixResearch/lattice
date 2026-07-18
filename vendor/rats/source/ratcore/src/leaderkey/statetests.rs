use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Act {
    Save,
    Open,
}

fn make_menu() -> LeaderMenu<Act> {
    let root = LeaderMenuDef {
        label: "Leader".into(),
        items: vec![
            LeaderMenuItem {
                key: 's',
                label: "save".into(),
                action: LeaderAction::Action(Act::Save),
            },
            LeaderMenuItem {
                key: 'o',
                label: "open".into(),
                action: LeaderAction::Action(Act::Open),
            },
            LeaderMenuItem {
                key: 'x',
                label: "extra".into(),
                action: LeaderAction::Submenu("extra".into()),
            },
        ],
    };
    let submenus = vec![LeaderMenuDef {
        label: "extra".into(),
        items: vec![LeaderMenuItem {
            key: 'a',
            label: "alpha".into(),
            action: LeaderAction::Command("/alpha".into()),
        }],
    }];
    debug_assert!(!root.items.is_empty());
    debug_assert!(!submenus.is_empty());
    LeaderMenu::from_parts(root, submenus)
}

#[test]
fn opens_and_closes() {
    let mut m = make_menu();
    assert!(!m.visible);
    m.open();
    assert!(m.visible);
    m.close();
    assert!(!m.visible);
}

#[test]
fn escape_closes_root() {
    let mut m = make_menu();
    m.open();
    let r = m.handle_input(MenuInput::Escape);
    assert!(r.is_none());
    assert!(!m.visible);
}

#[test]
fn unknown_key_dismisses() {
    let mut m = make_menu();
    m.open();
    let r = m.handle_char('z');
    assert!(r.is_none());
    assert!(!m.visible);
}

#[test]
fn other_input_dismisses() {
    let mut m = make_menu();
    m.open();
    let r = m.handle_input(MenuInput::Other);
    assert!(r.is_none());
    assert!(!m.visible);
}

#[test]
fn direct_action_returns_and_closes() {
    let mut m = make_menu();
    m.open();
    let r = m.handle_char('s');
    assert_eq!(r, Some(LeaderAction::Action(Act::Save)));
    assert!(!m.visible);
}

#[test]
fn submenu_navigation() {
    let mut m = make_menu();
    m.open();

    let r = m.handle_char('x');
    assert!(r.is_none());
    assert!(m.visible);
    assert_eq!(m.depth(), 1);

    let r = m.handle_char('a');
    assert_eq!(r, Some(LeaderAction::Command("/alpha".into())));
    assert!(!m.visible);
}

#[test]
fn escape_goes_back_from_submenu() {
    let mut m = make_menu();
    m.open();
    m.handle_char('x');
    assert_eq!(m.depth(), 1);

    m.handle_input(MenuInput::Escape);
    assert!(m.visible);
    assert_eq!(m.depth(), 0);

    m.handle_input(MenuInput::Escape);
    assert!(!m.visible);
}

#[test]
fn not_visible_returns_none() {
    let mut m = make_menu();
    let r = m.handle_char('s');
    assert!(r.is_none());
}

#[test]
fn breadcrumb_tracks_navigation() {
    let mut m = make_menu();
    m.open();
    assert!(m.breadcrumb().is_empty());

    m.handle_char('x');
    assert_eq!(m.breadcrumb(), &["extra"]);

    m.handle_input(MenuInput::Escape);
    assert!(m.breadcrumb().is_empty());
}
