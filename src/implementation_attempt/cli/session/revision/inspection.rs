//! The lifecycles one inspection reads: the worker definition's, for the
//! worker profile facts, and the session subject's, for the attempt's pin.

type Lifecycle = lattice_core::agent_revision::state::Lifecycle;
type Request = super::Request;

impl super::Store<'_> {
    /// The worker definition's lifecycle for profile facts and the session
    /// subject's lifecycle for its pin, both `None` without a store. They
    /// are one read unless the request names a workflow-agent revision.
    pub(in super::super::super) fn lifecycles(self, request: &Request) -> Result<Lifecycles, String> {
        let Some(path) = self.0 else {
            return Ok(Lifecycles {
                profile: None,
                workflow: None,
            });
        };
        let store = Self::open(path)?;
        let load = |definition: &str| {
            store
                .load_lifecycle(definition)
                .map(|stored| stored.map(|(_, lifecycle)| lifecycle))
                .map_err(|error| error.to_string())
        };
        let worker = crate::agent_revision::bundle::resolve(request)?.definition.identity;
        let subject = crate::agent_revision::seam::subject(request)?.definition;
        let workflow = (subject != worker).then(|| load(&subject)).transpose()?;
        Ok(Lifecycles {
            profile: load(&worker)?,
            workflow,
        })
    }
}

/// The lifecycles one inspection reads.
pub(in super::super::super) struct Lifecycles {
    /// The worker definition's.
    profile: Option<Lifecycle>,
    /// The workflow-agent definition's, read only when the session subject
    /// is not the worker definition.
    workflow: Option<Option<Lifecycle>>,
}

impl Lifecycles {
    /// The worker definition's lifecycle, for the worker profile facts.
    pub(in super::super::super) fn profile(&self) -> Option<&Lifecycle> {
        self.profile.as_ref()
    }

    /// The session subject's lifecycle, for the attempt's pin.
    pub(in super::super::super) fn session(&self) -> Option<&Lifecycle> {
        match &self.workflow {
            Some(workflow) => workflow.as_ref(),
            None => self.profile.as_ref(),
        }
    }
}
