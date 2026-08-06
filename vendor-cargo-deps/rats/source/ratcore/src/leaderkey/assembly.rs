use super::*;

/// Result of building a leader menu: the menu and any conflicts.
pub type BuildResult<A> = (LeaderMenu<A>, Vec<Conflict>);

type GroupedMenuItems<A> = (Vec<MenuContribution<A>>, std::collections::HashMap<String, Vec<MenuContribution<A>>>);

/// Build a leader menu from contributors.
///
/// Collects all [`MenuContribution`] items, deduplicates by `(key, placement)`
/// with highest priority winning, removes hidden entries, and assembles the
/// menu tree.
pub fn build<A: Clone>(contributors: &[&dyn MenuContributor<A>], hidden: &HiddenSet) -> BuildResult<A> {
    let items = contributors.iter().flat_map(|c| c.menu_items()).collect();
    build_from_items(items, hidden)
}

fn group_contributions<A>(
    seen: std::collections::HashMap<(char, MenuPlacement), MenuContribution<A>>,
) -> GroupedMenuItems<A> {
    let grouped_item_count = seen.len();
    let mut root_items = Vec::with_capacity(grouped_item_count);
    let mut submenu_items: std::collections::HashMap<String, Vec<MenuContribution<A>>> =
        std::collections::HashMap::with_capacity(grouped_item_count);

    for ((_, placement), item) in seen {
        match placement {
            MenuPlacement::Root => root_items.push(item),
            MenuPlacement::Submenu(ref name) => {
                submenu_items.entry(name.clone()).or_default().push(item);
            }
        }
    }

    debug_assert!(root_items.len() <= grouped_item_count);
    debug_assert!(submenu_items.values().all(|items| !items.is_empty()));
    (root_items, submenu_items)
}

fn menu_items_from_contributions<A>(items: Vec<MenuContribution<A>>) -> Vec<LeaderMenuItem<A>> {
    items
        .into_iter()
        .map(|contribution| LeaderMenuItem {
            key: contribution.key,
            label: contribution.label,
            action: contribution.action,
        })
        .collect()
}

fn build_submenus<A>(
    submenu_items: std::collections::HashMap<String, Vec<MenuContribution<A>>>,
) -> Vec<LeaderMenuDef<A>> {
    let mut submenus = Vec::with_capacity(submenu_items.len());
    for (name, mut items) in submenu_items {
        items.sort_by_key(|item| item.key);
        submenus.push(LeaderMenuDef {
            label: name,
            items: menu_items_from_contributions(items),
        });
    }
    submenus
}

/// Builds a leader menu from a pre-collected list of contributions.
#[must_use]
pub fn build_from_items<A: Clone>(mut all_items: Vec<MenuContribution<A>>, hidden: &HiddenSet) -> BuildResult<A> {
    let contribution_count = all_items.len();
    let mut conflicts = Vec::with_capacity(contribution_count);

    // Sort by priority (lowest first so highest overwrites).
    all_items.sort_by_key(|i| i.priority);
    debug_assert!(all_items.windows(2).all(|pair| pair[0].priority <= pair[1].priority));
    debug_assert!(all_items.len() <= contribution_count);

    // Deduplicate by (key, placement) — last writer wins.
    let mut seen: std::collections::HashMap<(char, MenuPlacement), MenuContribution<A>> =
        std::collections::HashMap::with_capacity(contribution_count);
    for item in all_items {
        let slot = (item.key, item.placement.clone());
        if let Some(existing) = seen.get(&slot) {
            conflicts.push(Conflict {
                registry: "leader_menu",
                key: format!("'{}' in {:?}", item.key, item.placement),
                winner: item.source.clone(),
                loser: existing.source.clone(),
            });
        }
        seen.insert(slot, item);
    }

    // Remove hidden entries.
    for h in hidden {
        seen.remove(h);
    }

    let (mut root_items, submenu_items) = group_contributions(seen);
    let submenus = build_submenus(submenu_items);

    // Build root def (sorted by key).
    root_items.sort_by_key(|item| item.key);
    let root = LeaderMenuDef {
        label: "Leader".into(),
        items: menu_items_from_contributions(root_items),
    };

    let menu = LeaderMenu::from_parts(root, submenus);
    (menu, conflicts)
}
