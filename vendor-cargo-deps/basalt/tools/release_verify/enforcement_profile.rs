//! Pure admission of the selected normal dependency observation.

// r[impl crate_consumability.enforcement_profile_evidence]
pub(super) fn validate(observation: &str) -> Result<(), String> {
    let names: Vec<_> = observation
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    if !names.contains(&"basalt") || !names.contains(&"ucan") {
        return Err("minimal dependency observation lacks Basalt or UCAN".to_owned());
    }
    for forbidden in [
        "bounded_http",
        "bounded_http_core",
        "replay_ledger_core",
        "revocation_view_core",
    ] {
        if names.contains(&forbidden) {
            return Err(format!(
                "minimal dependency observation includes {forbidden}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;

    #[test]
    fn full_and_minimal_rails_select_distinct_profiles() {
        let steps = crate::plan::steps();
        for name in [
            "library tests",
            "OIDC exchange and source-boundary tests",
            "doctests",
            "library docs",
        ] {
            let step = steps
                .iter()
                .find(|step| step.name == name)
                .expect("maintained step");
            assert!(!step.args.contains(&"--no-default-features"), "{name}");
        }
        for name in [
            "minimal enforcement tests",
            "minimal enforcement dependencies",
        ] {
            let step = steps
                .iter()
                .find(|step| step.name == name)
                .expect("minimal step");
            assert!(step.args.contains(&"--no-default-features"), "{name}");
        }
    }

    // r[verify crate_consumability.enforcement_profile_evidence]
    #[test]
    fn accepts_enforcement_and_rejects_missing_or_forbidden_packages() {
        assert!(validate("basalt\nucan\nserde\nucan (*)\n").is_ok());
        for invalid in ["", "basalt", "ucan", "not_basalt\nnot_ucan"] {
            assert!(validate(invalid).is_err());
        }
        for forbidden in [
            "bounded_http",
            "bounded_http_core",
            "replay_ledger_core",
            "revocation_view_core",
        ] {
            assert!(validate(&format!("basalt\nucan\n{forbidden}\n")).is_err());
        }
    }
}
