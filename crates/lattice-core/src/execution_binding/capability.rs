//! `lattice.worker-capability.v1`: an operator's optional definition name and
//! declared task families and tools for one worker profile.
//!
//! The shape matches the optional `worker_capability` of
//! `contracts/worker-execution-binding.ncl` and of an implementation-attempt
//! request. Names are short lowercase identifiers, so a declaration cannot
//! carry a path, command line, prompt or URL. A declaration states support;
//! it grants no tool, credential or execution authority.

// r[impl lattice.worker_inventory.capability]
// r[impl lattice.worker_inventory.capability.scenario.malformed]

pub const CAPABILITY_SCHEMA: &str = "lattice.worker-capability.v1";
/// Names one declared set may carry.
pub const MAX_CAPABILITY_NAMES: usize = 32;
/// Bytes one name may carry.
pub const MAX_NAME_BYTES: usize = 64;

/// One profile's declared definition and capability sets.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub schema: String,
    /// Definition name; the definition identity is stable across revisions.
    pub definition: String,
    /// Absent means undeclared, which is published as unknown.
    #[serde(default = "undeclared", skip_serializing_if = "Option::is_none")]
    pub task_families: Option<NameSet>,
    /// Absent means undeclared, which is published as unknown.
    #[serde(default = "undeclared", skip_serializing_if = "Option::is_none")]
    pub tools: Option<NameSet>,
}

/// A declared set is complete and nonempty; `unavailable` offers none.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NameSet {
    Declared { names: Vec<String> },
    Unavailable {},
}

/// An undeclared set; requests and configurations without one decode to it.
#[must_use]
pub const fn undeclared() -> Option<NameSet> {
    None
}

/// An absent declaration; existing requests and configurations decode to it.
#[must_use]
pub const fn absent() -> Option<Declaration> {
    None
}

/// A declaration the core refuses before any projection or effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "defect", rename_all = "snake_case")]
pub enum Defect {
    Schema,
    Definition,
    /// A declared set with no names; use `unavailable` instead.
    Empty {
        field: &'static str,
    },
    TooMany {
        field: &'static str,
    },
    Name {
        field: &'static str,
    },
    Duplicate {
        field: &'static str,
    },
}

/// 1 to 64 bytes of `[a-z0-9]` followed by `[a-z0-9._:-]`.
#[must_use]
pub fn is_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let is_lead = bytes.next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    let is_rest = bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._:-".contains(&byte));
    is_lead && is_rest && value.len() <= MAX_NAME_BYTES
}

/// Check schema, definition name and both sets.
pub fn validate(declaration: &Declaration) -> Result<(), Defect> {
    if declaration.schema != CAPABILITY_SCHEMA {
        return Err(Defect::Schema);
    }
    if !is_name(&declaration.definition) {
        return Err(Defect::Definition);
    }
    validate_set(declaration.task_families.as_ref(), "task_families")?;
    validate_set(declaration.tools.as_ref(), "tools")?;
    debug_assert!(declaration.definition.len() <= MAX_NAME_BYTES);
    debug_assert_eq!(declaration.schema, CAPABILITY_SCHEMA);
    Ok(())
}

fn validate_set(set: Option<&NameSet>, field: &'static str) -> Result<(), Defect> {
    let Some(NameSet::Declared { names }) = set else {
        return Ok(());
    };
    if names.is_empty() {
        return Err(Defect::Empty { field });
    }
    if names.len() > MAX_CAPABILITY_NAMES {
        return Err(Defect::TooMany { field });
    }
    if !names.iter().all(|name| is_name(name)) {
        return Err(Defect::Name { field });
    }
    if !super::is_distinct(names.iter().map(String::as_str)) {
        return Err(Defect::Duplicate { field });
    }
    Ok(())
}

impl Declaration {
    /// The validated declaration with every declared set sorted, so equal
    /// declarations frame equal profile revisions whatever their order.
    pub fn canonical(&self) -> Result<Self, Defect> {
        validate(self)?;
        let sorted = |set: &Option<NameSet>| {
            set.as_ref().map(|set| match set {
                NameSet::Declared { names } => {
                    let mut names = names.clone();
                    names.sort_unstable();
                    NameSet::Declared { names }
                }
                NameSet::Unavailable {} => NameSet::Unavailable {},
            })
        };
        let canonical = Self {
            schema: self.schema.clone(),
            definition: self.definition.clone(),
            task_families: sorted(&self.task_families),
            tools: sorted(&self.tools),
        };
        debug_assert_eq!(validate(&canonical), Ok(()), "sorting keeps a valid declaration valid");
        Ok(canonical)
    }
}
