//! Deterministic profile, requirement, and aggregate matrix renderers.

use std::fmt::Write as _;

/// Renders one row per ordered profile package.
#[must_use]
pub fn render_portability_profiles_tsv(profiles: &[super::PortabilityPackageProfile]) -> String {
    let mut ordered = profiles.to_vec();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    let mut output = String::from(
        "profile\tprofile_role\tpackage\tdefault_features\tpackage_role\ttarget_scope\tpackage_set_hash\n",
    );
    for profile in ordered {
        for package in profile.packages {
            writeln!(
                output,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                profile.id,
                profile.role.as_str(),
                package.package,
                package.default_features,
                package.role.as_str(),
                package.target_scope,
                profile.package_set_hash,
            )
            .expect("writing to String cannot fail");
        }
    }
    output
}

/// Renders exact required evidence-grade cells.
#[must_use]
pub fn render_portability_requirements_tsv(requirements: &[super::PortabilityMatrixRequirement]) -> String {
    let mut ordered = requirements.to_vec();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    let mut output = String::from("id\tprofile\ttarget\tgrade\tcommand\tcommand_hash\tallows_cross_compile\n");
    for requirement in ordered {
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            requirement.id,
            requirement.profile,
            requirement.target,
            requirement.grade.as_str(),
            requirement.command,
            requirement.command_hash,
            requirement.allows_cross_compile,
        )
        .expect("writing to String cannot fail");
    }
    output
}

/// Renders aggregate matrix cells as TSV.
#[must_use]
pub fn render_portability_matrix_tsv(build: &super::PortabilityMatrixBuild) -> String {
    let mut output = String::from("schema\tprofile\ttarget\tstatus\trequired_grades\treason\n");
    for cell in &build.cells {
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}",
            super::MATRIX_SCHEMA,
            cell.profile,
            cell.target,
            cell.status.as_str(),
            cell.required_grades.iter().map(|grade| grade.as_str()).collect::<Vec<_>>().join(","),
            if cell.reason.is_empty() {
                "-"
            } else {
                cell.reason.as_str()
            },
        )
        .expect("writing to String cannot fail");
    }
    output
}

/// Renders generated package-profile documentation.
#[must_use]
pub fn render_portability_profiles_markdown(profiles: &[super::PortabilityPackageProfile]) -> String {
    let mut ordered = profiles.to_vec();
    ordered.sort_by(|left, right| left.id.cmp(&right.id));
    let mut output = String::from(
        "# Generated portability package profiles\n\nDerived from the accepted host-portability inventory. Membership is not support evidence.\n\n| Profile | Role | Packages | Package-set hash |\n| --- | --- | --- | --- |\n",
    );
    for profile in ordered {
        writeln!(
            output,
            "| `{}` | `{}` | {} | `{}` |",
            profile.id,
            profile.role.as_str(),
            profile
                .packages
                .iter()
                .map(|package| format!("`{}`", package.package))
                .collect::<Vec<_>>()
                .join(", "),
            profile.package_set_hash,
        )
        .expect("writing to String cannot fail");
    }
    output
}

/// Renders the exact target/profile matrix and bounded non-claims.
#[must_use]
pub fn render_portability_matrix_markdown(build: &super::PortabilityMatrixBuild) -> String {
    let mut output = String::from(
        "# Generated Rats host portability matrix\n\nOnly `supported` exact profile/target cells are claims. Missing, failed, stale, blocked, unsupported, partial, cross-only, experimental, and irrelevant states never promote another cell.\n\n",
    );
    writeln!(output, "Non-claims: `{}`.\n", super::MATRIX_NON_CLAIMS).expect("writing to String cannot fail");
    output
        .push_str("| Profile | Exact target | Status | Required grades | Reason |\n| --- | --- | --- | --- | --- |\n");
    for cell in &build.cells {
        writeln!(
            output,
            "| `{}` | `{}` | **{}** | {} | `{}` |",
            cell.profile,
            cell.target,
            cell.status.as_str(),
            cell.required_grades.iter().map(|grade| grade.as_str()).collect::<Vec<_>>().join(", "),
            cell.reason,
        )
        .expect("writing to String cannot fail");
    }
    if !build.issues.is_empty() {
        output.push_str("\n## Rejected receipt/policy diagnostics\n\n");
        for issue in &build.issues {
            writeln!(output, "- `{:?}` / `{}`", issue.code, issue.subject).expect("writing to String cannot fail");
        }
    }
    output
}
