/// Model-generation adapter contract.
pub trait ModelAdapter: ComponentAdapter<crate::ModelRequest, crate::ModelResponse> {}

impl<T> ModelAdapter for T where T: ComponentAdapter<crate::ModelRequest, crate::ModelResponse> {}

/// Tool-invocation adapter contract.
pub trait ToolAdapter: ComponentAdapter<crate::ToolRequest, crate::ToolResponse> {}

impl<T> ToolAdapter for T where T: ComponentAdapter<crate::ToolRequest, crate::ToolResponse> {}

/// Host status-observation adapter contract.
pub trait HostStatusAdapter:
    ComponentAdapter<crate::HostStatusRequest, crate::HostStatusObservationBatch>
{
}

impl<T> HostStatusAdapter for T where
    T: ComponentAdapter<crate::HostStatusRequest, crate::HostStatusObservationBatch>
{
}

/// Generic port between harness orchestration and a swappable component.
pub trait ComponentAdapter<Request, Response> {
    /// Error returned by the concrete adapter implementation.
    type Error;

    /// Calls the component with one typed request and returns one typed response.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] when the concrete adapter cannot satisfy the
    /// request or rejects it at its own boundary.
    fn call(&mut self, request: &Request) -> Result<Response, Self::Error>;
}
