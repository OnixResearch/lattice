impl ValidatedBundle {
    /// The template identity, or for a bound bundle the identity over template, values, and revision.
    pub fn identity(&self) -> &ContentIdentity {
        &self.identity
    }

    pub fn manifest_identity(&self) -> &ContentIdentity {
        &self.manifest_identity
    }

    pub fn manifest(&self) -> &BundleManifest {
        &self.manifest
    }

    pub fn workflow_revision(&self) -> &crate::orchestration::WorkflowRevisionRef {
        &self.workflow_revision
    }

    pub fn workflow(&self) -> &crate::engine::Workflow {
        &self.workflow
    }

    pub fn orchestration_revision(&self) -> Option<&crate::orchestration::DefinitionRevision> {
        self.orchestration_revision.as_ref()
    }

    pub fn orchestration(&self) -> Option<&crate::orchestration::Definition> {
        self.orchestration.as_ref()
    }

    pub fn file_identities(&self) -> &[(String, ContentIdentity)] {
        &self.file_identities
    }

    /// `None` for a catalog template; `Some` once [`super::bind`] bound exact values.
    pub fn params(&self) -> Option<&super::params::Bound> {
        self.params.as_ref()
    }
}
