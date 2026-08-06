pub(super) struct AcceptedIds<'a> {
    selected: &'a str,
    focused: &'a str,
}

impl<'a> AcceptedIds<'a> {
    pub(super) const fn same(id: &'a str) -> Self {
        Self {
            selected: id,
            focused: id,
        }
    }
}

pub(super) fn codes(
    candidate: super::super::super::TabsCatalog,
    ids: AcceptedIds<'_>,
) -> std::collections::BTreeSet<super::super::super::TabsDiagnosticCode> {
    super::super::super::TabsState::new(
        candidate,
        super::super::super::TabId::new(ids.selected),
        super::super::super::TabId::new(ids.focused),
        super::super::super::TabsOrientation::Horizontal,
        super::super::super::TabsActivationPolicy::Automatic,
    )
    .expect_err("candidate must fail")
    .into_iter()
    .map(|issue| issue.code)
    .collect()
}
