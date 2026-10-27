/// Explicit bounds for silence detection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SilencePolicy {
    /// Maximum silent interval before detachment.
    pub max_silent_ms: u64,
    /// Maximum observed output before detachment.
    pub max_output_bytes: u64,
}

/// Supplied observation for one running call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SilenceObservation {
    /// Elapsed silent interval.
    pub silent_ms: u64,
    /// Output bytes observed so far.
    pub output_bytes: u64,
    /// Whether the call already reached a terminal state.
    pub terminal: bool,
}

/// Pure silence enforcement disposition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SilenceDisposition {
    /// Call remains within bounds.
    Continue,
    /// Call completed before detachment.
    Complete,
    /// Caller returns while supervised work remains addressable.
    Detach {
        /// Store-typed handle for follow-up.
        handle: super::Handle,
        /// Explicit follow command.
        follow_command: alloc::string::String,
    },
}

/// One suspended work item for a diagnostic dump.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Suspended {
    /// Stable call identity.
    pub call_id: alloc::string::String,
    /// Store-typed follow handle.
    pub handle: super::Handle,
    /// Supplied suspension duration.
    pub duration_ms: u64,
}

/// Bounded dump that explains every supplied suspended item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuspensionDump {
    /// Named suspended work in input order.
    pub items: alloc::vec::Vec<Suspended>,
}

/// Plans silence handling from supplied facts only.
///
/// # Errors
///
/// Returns [`super::Error::InvalidSupervision`] for zero bounds.
pub fn plan_silence(
    policy: SilencePolicy,
    observation: SilenceObservation,
    handle: &super::Handle,
) -> Result<SilenceDisposition, super::Error> {
    if policy.max_silent_ms == 0 || policy.max_output_bytes == 0 {
        return Err(super::Error::InvalidSupervision);
    }
    if observation.terminal {
        return Ok(SilenceDisposition::Complete);
    }
    if observation.silent_ms <= policy.max_silent_ms
        && observation.output_bytes <= policy.max_output_bytes
    {
        return Ok(SilenceDisposition::Continue);
    }
    Ok(SilenceDisposition::Detach {
        handle: handle.clone(),
        follow_command: alloc::format!("tool.follow {}:{}", handle.kind().label(), handle.token()),
    })
}

/// Projects a bounded explanation of supplied suspended work.
///
/// # Errors
///
/// Returns [`super::Error::InvalidSupervision`] for missing, duplicate, blank,
/// zero-duration, or over-bound items.
pub fn project_suspension_dump(
    items: &[Suspended],
    max_items: u32,
) -> Result<SuspensionDump, super::Error> {
    let item_count = u32::try_from(items.len()).map_err(|_| super::Error::InvalidSupervision)?;
    if max_items == 0 || item_count == 0 || item_count > max_items {
        return Err(super::Error::InvalidSupervision);
    }
    for (index, item) in items.iter().enumerate() {
        if item.call_id.trim().is_empty() || item.duration_ms == 0 {
            return Err(super::Error::InvalidSupervision);
        }
        if items[..index]
            .iter()
            .any(|prior| prior.handle.identity() == item.handle.identity())
        {
            return Err(super::Error::InvalidSupervision);
        }
    }
    debug_assert!(item_count <= max_items);
    debug_assert!(items.iter().all(|item| item.duration_ms > 0));
    Ok(SuspensionDump {
        items: items.to_vec(),
    })
}
