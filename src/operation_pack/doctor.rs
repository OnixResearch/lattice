#![allow(
    unknown_lints,
    assertion_density,
    reason = "doctor converts incomplete or invalid observations into findings instead of terminating with assertions"
)]
#![allow(
    function_length,
    reason = "the doctor keeps one ordered evaluation pass so finding precedence and inert behavior stay visible"
)]
#![allow(
    ambiguous_params,
    reason = "finding constructors keep identifier, message, and remediation adjacent at every private call site"
)]

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Fatal,
    Warning,
    Healthy,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: String,
    pub severity: Severity,
    pub message: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub pack_identity: Option<String>,
    pub healthy: bool,
    pub findings: Vec<Finding>,
    pub ordinary_effect_invoked: bool,
    pub credentials_resolved: bool,
    pub probe_outcome: Option<super::shell::probe::Outcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellFacts {
    pub cache_present: bool,
    pub registry_configured: bool,
    pub registry_available: bool,
}

pub fn evaluate(
    candidate: Result<&super::model::Candidate, &super::Error>,
    lock: Option<&super::model::Lock>,
    admission: Option<&super::model::AdmissionFacts>,
    shell: ShellFacts,
) -> Report {
    let mut findings = Vec::new();
    let mut pack_identity = None;
    match candidate {
        Ok(candidate) => {
            pack_identity = Some(candidate.pack_identity.clone());
            findings.push(healthy("manifest", "pack manifest and declared content are valid"));
            findings.push(healthy("identity", "manifest, content, and pack identities are exact"));
            findings.push(healthy("runtimes", "runtime descriptors use admitted closed execution classes"));
            if candidate.manifest.runtimes.iter().any(|runtime| runtime.kind == "wasm-component") {
                findings.push(skipped(
                    "component_execution",
                    "component bytes, Mantle member bytes, exact shared runtime availability, and cleanup were not exercised by inert doctor",
                ));
            }
            findings.push(healthy("capabilities", "runtime capabilities remain inside the manifest profile"));
            if candidate.manifest.dependencies.is_empty() {
                findings.push(skipped("dependencies", "pack declares no dependencies"));
            } else {
                findings.push(healthy("dependencies", "dependency identities are present in the exact closure"));
            }
            if candidate.manifest.admission.required_evidence.is_empty() {
                findings.push(skipped("evidence", "admission profile requires no external evidence roles"));
            } else {
                findings.push(healthy("evidence", "required evidence roles are declared for admission checking"));
            }
            match lock {
                Some(lock) => match super::admission::validate_lock(candidate, lock) {
                    Ok(()) => findings.push(healthy("lock", "pack lock matches exact candidate identities")),
                    Err(_) => findings.push(fatal(
                        "lock",
                        "pack lock differs from the candidate",
                        "recreate the exact lock after reviewing trust policy",
                    )),
                },
                None => findings.push(skipped("lock", "no pack lock was supplied")),
            }
            match admission {
                Some(admission) if candidate.manifest.platforms.contains(&admission.platform) => {
                    findings.push(healthy("platform", "observed platform is admitted by the pack"));
                }
                Some(_) => findings.push(fatal(
                    "platform",
                    "observed platform is not admitted",
                    "select an admitted platform artifact",
                )),
                None => findings.push(skipped("platform", "no admission platform fact was supplied")),
            }
            match (lock, admission) {
                (Some(lock), Some(admission)) => match super::admission::admit(candidate, lock, admission) {
                    Ok(_) => findings.push(healthy("admission", "pack lock and admission facts agree")),
                    Err(_) => findings.push(fatal(
                        "admission",
                        "pack lock or admission facts differ",
                        "recreate the exact lock after reviewing trust policy",
                    )),
                },
                (None, None) => findings.push(skipped("admission", "pack admission was not requested")),
                _ => findings.push(fatal(
                    "admission",
                    "pack doctor needs both lock and admission facts",
                    "supply the exact lock and trust facts together",
                )),
            }
            if candidate.manifest.analyzers.is_empty() {
                findings.push(skipped("analyzers", "pack declares no analyzers"));
            } else {
                findings.push(healthy("analyzers", "analyzer manifests and bounded views are valid"));
            }
            if candidate.manifest.skills.iter().all(super::skill::validate_routing_cases) {
                findings.push(healthy("skills", "skill identities and routing fixtures are valid"));
            } else {
                findings.push(fatal(
                    "skills",
                    "skill routing fixtures are incomplete",
                    "add positive and neighboring negative routes",
                ));
            }
            if candidate.manifest.probe.is_some() {
                findings.push(healthy("probe", "no-effect probe contract is admitted but was not executed"));
            } else {
                findings.push(skipped("probe", "pack declares no no-effect probe"));
            }
        }
        Err(_) => findings.push(fatal(
            "manifest",
            "pack manifest or content observation failed",
            "repair the pack manifest and exact content",
        )),
    }
    if shell.cache_present {
        findings.push(healthy("cache", "exact local pack content is present"));
    } else {
        findings.push(warning("cache", "exact local pack content is not installed", "run explicit pack installation"));
    }
    match (shell.registry_configured, shell.registry_available) {
        (false, _) => findings.push(skipped("registry", "no registry was configured")),
        (true, true) => findings.push(healthy("registry", "configured registry metadata is available")),
        (true, false) => findings.push(warning(
            "registry",
            "configured registry metadata is unavailable",
            "run explicit registry freshness inspection later",
        )),
    }
    findings.sort_by(|left, right| left.severity.cmp(&right.severity).then_with(|| left.id.cmp(&right.id)));
    let is_healthy = !findings.iter().any(|finding| finding.severity == Severity::Fatal);
    Report {
        pack_identity,
        healthy: is_healthy,
        findings,
        ordinary_effect_invoked: false,
        credentials_resolved: false,
        probe_outcome: None,
    }
}

fn healthy(id: &str, message: &str) -> Finding {
    Finding {
        id: id.to_string(),
        severity: Severity::Healthy,
        message: message.to_string(),
        remediation: None,
    }
}

fn skipped(id: &str, message: &str) -> Finding {
    Finding {
        id: id.to_string(),
        severity: Severity::Skipped,
        message: message.to_string(),
        remediation: None,
    }
}

fn warning(id: &str, message: &str, remediation: &str) -> Finding {
    Finding {
        id: id.to_string(),
        severity: Severity::Warning,
        message: message.to_string(),
        remediation: Some(remediation.to_string()),
    }
}

fn fatal(id: &str, message: &str, remediation: &str) -> Finding {
    Finding {
        id: id.to_string(),
        severity: Severity::Fatal,
        message: message.to_string(),
        remediation: Some(remediation.to_string()),
    }
}
