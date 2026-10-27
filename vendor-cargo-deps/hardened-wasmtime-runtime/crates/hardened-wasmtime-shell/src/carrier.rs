//! Import-free named `(string, list<u8>) -> list<u8>` component carrier.

// r[impl onix.wasmtime_runtime.named_byte_method]
/// A prepared component bound to one admitted synchronous named byte method.
/// Product method meaning and authority remain entirely consumer-owned.
pub struct ByteMethodInvocation {
    prepared: crate::ComponentInvocation,
    interface: String,
    function: String,
}

impl crate::ComponentInvocation {
    /// Bind an import-free component to one exact named byte-method signature.
    /// This reflects compiled types without instantiating or executing guest code.
    pub fn into_byte_method(
        self,
        interface: &str,
        function: &str,
    ) -> Result<ByteMethodInvocation, crate::PrepareError> {
        if !self.prepared.descriptors.is_empty() {
            return Err(crate::PrepareError::ArtifactMismatch);
        }
        let engine = &self.prepared.runtime.engine;
        let Some(wasmtime::component::types::ComponentItem::ComponentInstance(instance)) = self
            .component
            .component_type()
            .get_export(engine, interface)
        else {
            return Err(crate::PrepareError::ArtifactMismatch);
        };
        let Some(wasmtime::component::types::ComponentItem::ComponentFunc(signature)) =
            instance.get_export(engine, function)
        else {
            return Err(crate::PrepareError::ArtifactMismatch);
        };
        let mut parameters = signature.params();
        let mut results = signature.results();
        if signature.async_() {
            return Err(crate::PrepareError::ArtifactMismatch);
        }
        let is_parameters_match = matches!(
            parameters.next(),
            Some((_, wasmtime::component::types::Type::String))
        ) && parameters.next().is_some_and(|(_, ty)| byte_list(&ty))
            && parameters.next().is_none();
        if !is_parameters_match {
            return Err(crate::PrepareError::ArtifactMismatch);
        }
        let is_results_match =
            results.next().is_some_and(|ty| byte_list(&ty)) && results.next().is_none();
        if !is_results_match {
            return Err(crate::PrepareError::ArtifactMismatch);
        }
        Ok(ByteMethodInvocation {
            prepared: self,
            interface: interface.to_owned(),
            function: function.to_owned(),
        })
    }
}

fn byte_list(ty: &wasmtime::component::types::Type) -> bool {
    matches!(ty, wasmtime::component::types::Type::List(list) if matches!(list.ty(), wasmtime::component::types::Type::U8))
}

impl ByteMethodInvocation {
    /// Invoke the exact admitted carrier using cumulative budgets and no imports.
    /// The method's UTF-8 bytes and input share the input ceiling. Owned output
    /// is returned only after Wasmtime's automatic post-return has succeeded.
    pub fn run(&self, method: &str, input: &[u8]) -> crate::InvocationOutcome {
        let prepared = &self.prepared.prepared;
        let runtime = &prepared.runtime;
        let input_bytes = method
            .len()
            .checked_add(input.len())
            .and_then(|n| u64::try_from(n).ok());
        if !input_bytes.is_some_and(|n| n <= runtime.bounds.max_input_bytes) {
            return crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded,
            );
        }
        let Some(_permit) = runtime.enter() else {
            return crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::ConcurrencyExhausted,
            );
        };
        let mut store = match crate::store::create(runtime, (), prepared.ticks) {
            Ok(store) => store,
            Err(class) => return crate::store::failure(class),
        };
        store.data_mut().phase = hardened_wasmtime_runtime::outcomes::InvocationPhase::Linking;
        let linker = wasmtime::component::Linker::new(&runtime.engine);
        let result = self.execute(&mut store, &linker, method, input);
        crate::store::finish(&store, result)
    }

    fn execute(
        &self,
        store: &mut wasmtime::Store<crate::State<()>>,
        linker: &wasmtime::component::Linker<crate::State<()>>,
        method: &str,
        input: &[u8],
    ) -> Result<Vec<u8>, hardened_wasmtime_runtime::outcomes::Class> {
        store.data_mut().phase =
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation;
        let instance = linker
            .instantiate(&mut *store, &self.prepared.component)
            .map_err(crate::store::instantiation_error)?;
        store.data_mut().phase = hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation;
        let interface = instance
            .get_export_index(&mut *store, None, &self.interface)
            .ok_or(hardened_wasmtime_runtime::outcomes::Class::ArtifactDenied)?;
        let function = instance
            .get_export_index(&mut *store, Some(&interface), &self.function)
            .ok_or(hardened_wasmtime_runtime::outcomes::Class::ArtifactDenied)?;
        let function = instance
            .get_typed_func::<(&str, &[u8]), (Vec<u8>,)>(&mut *store, function)
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::ArtifactDenied)?;
        let (output,) = function
            .call(&mut *store, (method, input))
            .map_err(|error| crate::store::classify(&error))?;
        Ok(output)
    }
}
