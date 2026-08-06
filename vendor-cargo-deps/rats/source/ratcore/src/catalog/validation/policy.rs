pub(super) fn registry_scope_issues(
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::super::FixtureIssue> {
    crate::targeting_core::validate_renderer_scope(registry, scope)
        .issues
        .into_iter()
        .map(|issue| {
            super::super::FixtureIssue::new(
                super::super::FixtureIssueCode::InvalidRendererPolicy,
                issue.subject,
                issue.message,
            )
        })
        .collect()
}

pub(super) fn fixture_cell_issues(
    fixture: &super::super::ShowcaseFixture,
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::super::FixtureIssue> {
    let views = fixture_views(fixture);
    family_cell_issues(
        FamilyCellContext {
            subject: fixture.id.as_str(),
            cells: &views,
            codes: CellIssueCodes {
                missing: super::super::FixtureIssueCode::MissingRendererCell,
                invalid: super::super::FixtureIssueCode::InvalidRendererCell,
                missing_reason: super::super::FixtureIssueCode::UnsupportedRenderer,
            },
        },
        registry,
        scope,
    )
}

fn fixture_views(fixture: &super::super::ShowcaseFixture) -> Vec<FamilyCellView<'_>> {
    fixture
        .renderer_cells
        .iter()
        .map(|cell| FamilyCellView {
            family: &cell.family,
            is_supported: cell.supported,
            reason: cell.reason.as_deref(),
        })
        .collect()
}

pub(super) fn parity_cell_issues(
    row: &super::super::FixtureParityRow,
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::super::FixtureIssue> {
    let views = parity_views(row);
    family_cell_issues(
        FamilyCellContext {
            subject: row.fixture_id.as_str(),
            cells: &views,
            codes: CellIssueCodes {
                missing: super::super::FixtureIssueCode::ParityMissingRenderer,
                invalid: super::super::FixtureIssueCode::InvalidRendererCell,
                missing_reason: super::super::FixtureIssueCode::ParityMissingUnsupportedReason,
            },
        },
        registry,
        scope,
    )
}

fn parity_views(row: &super::super::FixtureParityRow) -> Vec<FamilyCellView<'_>> {
    row.renderer_cells
        .iter()
        .map(|cell| FamilyCellView {
            family: &cell.family,
            is_supported: cell.supported,
            reason: cell.unsupported_reason.as_deref(),
        })
        .collect()
}

struct FamilyCellView<'a> {
    family: &'a crate::targeting_core::RendererFamilyId,
    is_supported: bool,
    reason: Option<&'a str>,
}

struct CellIssueCodes {
    missing: super::super::FixtureIssueCode,
    invalid: super::super::FixtureIssueCode,
    missing_reason: super::super::FixtureIssueCode,
}

struct FamilyCellContext<'a> {
    subject: &'a str,
    cells: &'a [FamilyCellView<'a>],
    codes: CellIssueCodes,
}

fn family_cell_issues(
    context: FamilyCellContext<'_>,
    registry: &crate::targeting_core::RendererRegistry,
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<super::super::FixtureIssue> {
    let required = required_families(scope);
    let mut seen = std::collections::BTreeSet::new();
    let mut issues = Vec::with_capacity(context.cells.len().saturating_add(required.len()));
    for cell in context.cells {
        issues.extend(cell_record_issues(&context, cell, registry, &required, &mut seen));
    }
    issues.extend(missing_cell_issues(&context, &required, &seen));
    issues.extend(cell_order_issue(&context, &required));
    issues
}

fn cell_order_issue(
    context: &FamilyCellContext<'_>,
    required: &[crate::targeting_core::RendererFamilyId],
) -> Option<super::super::FixtureIssue> {
    let actual = context.cells.iter().map(|cell| cell.family).collect::<Vec<_>>();
    let expected = required.iter().collect::<Vec<_>>();
    (actual != expected).then(|| {
        cell_issue(
            context.codes.invalid,
            CellIssueInput::new(
                context.subject,
                "renderer-order",
                "renderer family cells do not follow the ordered scope",
            ),
        )
    })
}

fn cell_record_issues(
    context: &FamilyCellContext<'_>,
    cell: &FamilyCellView<'_>,
    registry: &crate::targeting_core::RendererRegistry,
    required: &[crate::targeting_core::RendererFamilyId],
    seen: &mut std::collections::BTreeSet<crate::targeting_core::RendererFamilyId>,
) -> Vec<super::super::FixtureIssue> {
    [
        (!seen.insert(cell.family.clone())).then(|| {
            cell_issue(
                context.codes.invalid,
                CellIssueInput::new(context.subject, cell.family.as_str(), "renderer family cell is duplicated"),
            )
        }),
        (registry.family(cell.family).is_none() || !required.contains(cell.family)).then(|| {
            cell_issue(
                context.codes.invalid,
                CellIssueInput::new(
                    context.subject,
                    cell.family.as_str(),
                    "renderer family cell is unknown or outside the named scope",
                ),
            )
        }),
        (!cell.is_supported && cell.reason.is_none_or(|reason| reason.trim().is_empty())).then(|| {
            cell_issue(
                context.codes.missing_reason,
                CellIssueInput::new(
                    context.subject,
                    cell.family.as_str(),
                    "unsupported renderer family requires a reason",
                ),
            )
        }),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn missing_cell_issues(
    context: &FamilyCellContext<'_>,
    required: &[crate::targeting_core::RendererFamilyId],
    seen: &std::collections::BTreeSet<crate::targeting_core::RendererFamilyId>,
) -> Vec<super::super::FixtureIssue> {
    required
        .iter()
        .filter(|family| !seen.contains(*family))
        .map(|family| {
            cell_issue(
                context.codes.missing,
                CellIssueInput::new(context.subject, family.as_str(), "required renderer family cell is missing"),
            )
        })
        .collect()
}

fn required_families(
    scope: &crate::targeting_core::RendererRequirementScope,
) -> Vec<crate::targeting_core::RendererFamilyId> {
    scope
        .members
        .iter()
        .filter_map(|member| match member {
            crate::targeting_core::RendererSubject::Family(family) => Some(family.clone()),
            crate::targeting_core::RendererSubject::Target(_)
            | crate::targeting_core::RendererSubject::EvidenceAdapter(_) => None,
        })
        .collect()
}

struct CellIssueInput<'a> {
    subject: &'a str,
    family: &'a str,
    message: &'static str,
}

impl<'a> CellIssueInput<'a> {
    const fn new(subject: &'a str, family: &'a str, message: &'static str) -> Self {
        Self {
            subject,
            family,
            message,
        }
    }
}

fn cell_issue(code: super::super::FixtureIssueCode, input: CellIssueInput<'_>) -> super::super::FixtureIssue {
    super::super::FixtureIssue::new(code, input.subject, format!("{}: {}", input.family, input.message))
}
