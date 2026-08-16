#![allow(
    unknown_lints,
    assertion_density,
    reason = "untrusted registry metadata returns typed errors instead of terminating the process"
)]
#![allow(
    compound_condition,
    reason = "registry conditions each express one closed entry or locator admission rule"
)]

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema: String,
    pub generated_epoch_seconds: i64,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub pack_id: String,
    pub pack_identity: String,
    pub description: String,
    pub display_version: String,
    pub channel: super::model::ReleaseChannel,
    pub platforms: Vec<String>,
    pub deprecated: bool,
    pub superseded_by: Option<String>,
    pub content_locator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Freshness {
    pub observed_epoch_seconds: i64,
    pub maximum_age_seconds: u64,
    pub current: bool,
    pub unavailable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestorePlan {
    pub pack_identity: String,
    pub content_locator: String,
    pub destination_identity: String,
    pub destination_omitted: bool,
    pub requires_confirmation: bool,
    pub network_contacted: bool,
    pub pack_installed: bool,
    pub lock_changed: bool,
}

pub fn validate_catalog(mut catalog: Catalog) -> Result<Catalog, super::Error> {
    if catalog.schema != super::model::REGISTRY_SCHEMA || catalog.entries.len() > super::model::MAXIMUM_CONTENT_ITEMS {
        return Err(super::Error::Invalid("registry schema or bound"));
    }
    catalog.entries.sort_by(|left, right| {
        left.pack_id.cmp(&right.pack_id).then_with(|| left.pack_identity.cmp(&right.pack_identity))
    });
    let mut keys = std::collections::BTreeSet::new();
    for entry in &mut catalog.entries {
        super::validation::validate_identity(&entry.pack_identity)?;
        validate_content_locator(&entry.content_locator)?;
        if entry.pack_id.split('.').count() < 2
            || entry.description.is_empty()
            || entry.description.len() > super::model::MAXIMUM_TEXT_BYTES
            || entry.platforms.is_empty()
        {
            return Err(super::Error::Invalid("registry entry"));
        }
        entry.platforms.sort();
        let before = entry.platforms.len();
        entry.platforms.dedup();
        if entry.platforms.len() != before || !keys.insert((&entry.pack_id, &entry.pack_identity)) {
            return Err(super::Error::Invalid("registry ambiguity"));
        }
        if let Some(identity) = &entry.superseded_by {
            super::validation::validate_identity(identity)?;
            if identity == &entry.pack_identity {
                return Err(super::Error::Invalid("registry supersession"));
            }
        }
    }
    Ok(catalog)
}

pub fn search(catalog: &Catalog, query: &str) -> Vec<Entry> {
    let normalized = query.to_ascii_lowercase();
    catalog
        .entries
        .iter()
        .filter(|entry| {
            entry.pack_id.to_ascii_lowercase().contains(&normalized)
                || entry.description.to_ascii_lowercase().contains(&normalized)
        })
        .cloned()
        .collect()
}

pub fn exact(catalog: &Catalog, pack_identity: &str) -> Result<Entry, super::Error> {
    super::validation::validate_identity(pack_identity)?;
    let matches = catalog
        .entries
        .iter()
        .filter(|entry| entry.pack_identity == pack_identity)
        .cloned()
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [entry] => Ok(entry.clone()),
        [] => Err(super::Error::Invalid("registry candidate missing")),
        [_, ..] => Err(super::Error::Invalid("registry candidate ambiguous")),
    }
}

pub fn restore_plan(
    catalog: &Catalog,
    pack_identity: &str,
    destination: &std::path::Path,
) -> Result<RestorePlan, super::Error> {
    if !destination.is_absolute()
        || destination.components().any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(super::Error::Invalid("registry restore destination"));
    }
    let selected = exact(catalog, pack_identity)?;
    Ok(RestorePlan {
        pack_identity: selected.pack_identity,
        content_locator: selected.content_locator,
        destination_identity: super::identity::content_identity(destination.as_os_str().as_encoded_bytes()),
        destination_omitted: true,
        requires_confirmation: true,
        network_contacted: false,
        pack_installed: false,
        lock_changed: false,
    })
}

pub fn promote(
    catalog: &Catalog,
    pack_identity: &str,
    target: super::model::ReleaseChannel,
) -> Result<Catalog, super::Error> {
    let mut updated = validate_catalog(catalog.clone())?;
    let matching = updated
        .entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| (entry.pack_identity == pack_identity).then_some(index))
        .collect::<Vec<_>>();
    let [index] = matching.as_slice() else {
        return Err(super::Error::Invalid("registry promotion selection"));
    };
    if !forward(updated.entries[*index].channel, target) {
        return Err(super::Error::Invalid("registry promotion direction"));
    }
    updated.entries[*index].channel = target;
    validate_catalog(updated)
}

pub fn freshness(catalog: Option<&Catalog>, observed_epoch_seconds: i64, maximum_age_seconds: u64) -> Freshness {
    let Some(catalog) = catalog else {
        return Freshness {
            observed_epoch_seconds,
            maximum_age_seconds,
            current: false,
            unavailable: true,
        };
    };
    let age_seconds = observed_epoch_seconds.saturating_sub(catalog.generated_epoch_seconds);
    let is_current = u64::try_from(age_seconds).is_ok_and(|age_seconds| age_seconds <= maximum_age_seconds);
    Freshness {
        observed_epoch_seconds,
        maximum_age_seconds,
        current: is_current,
        unavailable: false,
    }
}

fn validate_content_locator(locator: &str) -> Result<(), super::Error> {
    if !locator.contains("://") {
        return super::validation::validate_safe_relative_path(locator)
            .map_err(|_| super::Error::Invalid("registry content locator"));
    }
    if locator.len() > super::model::MAXIMUM_TEXT_BYTES {
        return Err(super::Error::Invalid("registry content locator"));
    }
    let parsed = url::Url::parse(locator).map_err(|_| super::Error::Invalid("registry content locator"))?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(super::Error::Invalid("registry content locator"));
    }
    Ok(())
}

fn forward(current: super::model::ReleaseChannel, target: super::model::ReleaseChannel) -> bool {
    matches!(
        (current, target),
        (super::model::ReleaseChannel::Beta, super::model::ReleaseChannel::Rc)
            | (super::model::ReleaseChannel::Beta, super::model::ReleaseChannel::Stable)
            | (super::model::ReleaseChannel::Rc, super::model::ReleaseChannel::Stable)
    )
}
