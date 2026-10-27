//! Actual Component Model invocation with bounded owned output before cleanup.
// r[impl onix.wasmtime_runtime.resource_limits.output]
// r[impl onix.wasmtime_runtime.artifact_shape]
// r[impl onix.wasmtime_runtime.failure_phase]

/// Component prepared against one exact world and admitted import set.
pub struct ComponentInvocation {
    pub(crate) prepared: crate::preparation::Prepared,
    pub(crate) component: wasmtime::component::Component,
}
impl ComponentInvocation {
    /// Validate the component's identity, class, shape and deadline before effects.
    pub fn prepare(
        runtime: &crate::BoundedRuntime,
        options: crate::PrepareOptions<'_>,
    ) -> Result<Self, crate::PrepareError> {
        let prepared = crate::preparation::admit(
            runtime,
            &options,
            hardened_wasmtime_runtime::artifact::shape::Class::Component,
        )?;
        let component =
            wasmtime::component::Component::new(&runtime.engine, options.artifact.bytes)
                .map_err(|_| crate::PrepareError::CompileFailed)?;
        let ty = component.component_type();
        let incoming = names(
            &runtime.engine,
            ty.imports(&runtime.engine),
            runtime.bounds.max_imports,
        )?;
        let outgoing = names(
            &runtime.engine,
            ty.exports(&runtime.engine),
            runtime.bounds.max_exports,
        )?;
        crate::preparation::names_match(
            incoming,
            &options.artifact.facts.imports,
            runtime.bounds.max_imports,
        )?;
        crate::preparation::names_match(
            outgoing,
            &options.artifact.facts.exports,
            runtime.bounds.max_exports,
        )?;
        Ok(Self {
            prepared,
            component,
        })
    }

    /// Invoke a root export with `(list<u8>) -> result<list<u8>, string>`.
    /// The carrier defines no product effects; imports are bound by the consumer.
    pub fn run<T: 'static, F>(
        &self,
        export: &str,
        input: &[u8],
        host: T,
        mut bind: F,
    ) -> crate::InvocationOutcome
    where
        F: FnMut(
            &hardened_wasmtime_runtime::restricted::imports::Descriptor,
            &mut wasmtime::component::Linker<crate::State<T>>,
        ) -> Result<(), crate::HostDenied>,
    {
        let runtime = &self.prepared.runtime;
        if input.len() as u64 > runtime.bounds.max_input_bytes {
            return crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded,
            );
        }
        let Some(_permit) = runtime.enter() else {
            return crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::ConcurrencyExhausted,
            );
        };
        let mut store = match crate::store::create(runtime, host, self.prepared.ticks) {
            Ok(store) => store,
            Err(class) => return crate::store::failure(class),
        };
        let mut linker = wasmtime::component::Linker::new(&runtime.engine);
        store.data_mut().phase = hardened_wasmtime_runtime::outcomes::InvocationPhase::Linking;
        for descriptor in &self.prepared.descriptors {
            if bind(descriptor, &mut linker).is_err() {
                return crate::store::finish(
                    &store,
                    Err(hardened_wasmtime_runtime::outcomes::Class::HostFailed),
                );
            }
        }
        let result = self.execute(&mut store, &linker, export, input);
        crate::store::finish(&store, result)
    }

    fn execute<T: 'static>(
        &self,
        store: &mut wasmtime::Store<crate::State<T>>,
        linker: &wasmtime::component::Linker<crate::State<T>>,
        export: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, hardened_wasmtime_runtime::outcomes::Class> {
        store.data_mut().phase =
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation;
        let instance = linker
            .instantiate(&mut *store, &self.component)
            .map_err(crate::store::instantiation_error)?;
        store.data_mut().phase = hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation;
        let function = instance
            .get_typed_func::<(&[u8],), (Result<Vec<u8>, wasmtime::component::WasmStr>,)>(
                &mut *store,
                export,
            )
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::ArtifactDenied)?;
        // Store hostcall fuel rejects an oversized list before Wasmtime copies
        // it. Owned lifting finishes before automatic guest post-return cleanup.
        let (result,) = function
            .call(&mut *store, (input,))
            .map_err(|error| crate::store::classify(&error))?;
        result.map_err(|_| hardened_wasmtime_runtime::outcomes::Class::GuestDenied)
    }
}

fn names<'a>(
    engine: &wasmtime::Engine,
    items: impl IntoIterator<Item = (&'a str, wasmtime::component::types::ComponentItem)>,
    limit: u32,
) -> Result<Vec<String>, crate::PrepareError> {
    let slot_count = usize::try_from(limit).map_err(|_| crate::PrepareError::ArtifactTooLarge)?;
    let pending = queue_names(Vec::with_capacity(slot_count), "", items, slot_count)?;
    collect_names(engine, pending, slot_count)
}

fn collect_names(
    engine: &wasmtime::Engine,
    mut pending: Vec<(String, wasmtime::component::types::ComponentItem)>,
    limit: usize,
) -> Result<Vec<String>, crate::PrepareError> {
    let mut output = Vec::with_capacity(limit);
    let mut remaining = limit;
    while let Some((path, item)) = pending.pop() {
        remaining = take_slot(remaining)?;
        if is_function(&item) {
            output.push(path);
            continue;
        }
        let wasmtime::component::types::ComponentItem::ComponentInstance(instance) = item else {
            return Err(crate::PrepareError::ArtifactMismatch);
        };
        pending = queue_names(pending, &path, instance.exports(engine), limit)?;
    }
    Ok(output)
}

fn take_slot(remaining: usize) -> Result<usize, crate::PrepareError> {
    remaining
        .checked_sub(1)
        .ok_or(crate::PrepareError::ArtifactTooLarge)
}

fn is_function(item: &wasmtime::component::types::ComponentItem) -> bool {
    matches!(
        item,
        wasmtime::component::types::ComponentItem::ComponentFunc(_)
    )
}

fn queue_names<'a>(
    mut pending: Vec<(String, wasmtime::component::types::ComponentItem)>,
    parent: &str,
    items: impl IntoIterator<Item = (&'a str, wasmtime::component::types::ComponentItem)>,
    limit: usize,
) -> Result<Vec<(String, wasmtime::component::types::ComponentItem)>, crate::PrepareError> {
    for member in items {
        if pending.len() >= limit {
            return Err(crate::PrepareError::ArtifactTooLarge);
        }
        pending.push(qualify_member(parent, member)?);
    }
    Ok(pending)
}

fn qualify_member(
    parent: &str,
    (name, item): (&str, wasmtime::component::types::ComponentItem),
) -> Result<(String, wasmtime::component::types::ComponentItem), crate::PrepareError> {
    // A 64-bit platform length needs at most 20 decimal digits plus the colon.
    const LENGTH_PREFIX_CAPACITY_BYTES: usize = 21;
    let capacity_bytes = parent
        .len()
        .checked_add(name.len())
        .and_then(|bytes| bytes.checked_add(LENGTH_PREFIX_CAPACITY_BYTES))
        .ok_or(crate::PrepareError::ArtifactTooLarge)?;
    let mut path = String::with_capacity(capacity_bytes);
    path.push_str(parent);
    Ok((crate::preparation::append_qualified(path, name), item))
}
