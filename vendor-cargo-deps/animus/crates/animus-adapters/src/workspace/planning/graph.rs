type Descriptor = crate::workspace::intent::Descriptor;
type Capability = crate::workspace::adapter::Capability;
type Capabilities = crate::workspace::adapter::Capabilities;

pub(super) fn find_descriptor<'a>(descriptors: &'a [Descriptor], workspace_id: &str) -> Option<&'a Descriptor> {
    descriptors.iter().find(|descriptor| descriptor.workspace_id() == workspace_id)
}

pub(super) fn has_duplicate_identity(descriptors: &[Descriptor]) -> bool {
    descriptors.iter().any(|candidate| {
        descriptors
            .iter()
            .filter(|descriptor| descriptor.workspace_id() == candidate.workspace_id())
            .count()
            > 1
    })
}

pub(super) fn has_missing_parent(descriptors: &[Descriptor]) -> bool {
    descriptors.iter().any(|descriptor| {
        descriptor
            .parent_workspace_id()
            .is_some_and(|parent| find_descriptor(descriptors, parent).is_none())
    })
}

pub(super) fn has_parent_cycle(descriptors: &[Descriptor]) -> bool {
    descriptors.iter().any(|descriptor| descriptor_has_parent_cycle(descriptor, descriptors))
}

pub(super) fn has_child(descriptors: &[Descriptor], workspace_id: &str) -> bool {
    descriptors.iter().any(|descriptor| descriptor.parent_workspace_id() == Some(workspace_id))
}

pub(super) fn missing_capabilities(selected: &Descriptor, adapter: &Capabilities) -> alloc::vec::Vec<Capability> {
    selected
        .required_capabilities()
        .iter()
        .copied()
        .filter(|capability| !adapter.supports(*capability))
        .collect()
}

fn descriptor_has_parent_cycle(descriptor: &Descriptor, descriptors: &[Descriptor]) -> bool {
    let mut visited = alloc::vec::Vec::<alloc::string::String>::with_capacity(descriptors.len());
    let mut current = Some(descriptor.workspace_id());
    while let Some(workspace_id) = current {
        if visited.iter().any(|visited_id| visited_id == workspace_id) {
            return true;
        }
        visited.push(alloc::string::String::from(workspace_id));
        current = find_descriptor(descriptors, workspace_id).and_then(Descriptor::parent_workspace_id);
    }
    false
}
