//! Ephemeral credential material and public origin metadata.

use animus_adapters::secret_resolution::SecretProviderScheme;
use zeroize::Zeroizing;

/// Public, value-free origin metadata for one resolved credential.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialOrigin {
    /// BLAKE3 identity of the exact manifest bytes.
    pub manifest_identity: String,
    /// Explicit profile.
    pub profile: String,
    /// Explicit scope.
    pub scope: String,
    /// Exact declaration.
    pub declaration: String,
    /// Selected credential-free provider family.
    pub provider: SecretProviderScheme,
}

/// Injected representative provider-dispatch credential boundary.
pub trait ProviderCredentialSink {
    /// Sink-owned failure type.
    type Error;

    /// Deliver one borrowed value to one exact declaration slot.
    ///
    /// # Errors
    ///
    /// Returns the sink-owned failure without transferring value ownership.
    fn deliver(&mut self, declaration: &str, value: &[u8]) -> Result<(), Self::Error>;
}

/// Non-cloneable and non-debuggable ephemeral secret material.
///
/// The inner bytes are zeroized when this value is dropped. This type exposes
/// no serialization, hashing, equality, or owned-value accessor.
pub struct ResolvedCredential {
    material: Zeroizing<Vec<u8>>,
    origin: CredentialOrigin,
}

impl ResolvedCredential {
    pub(crate) fn new(material: Zeroizing<Vec<u8>>, origin: CredentialOrigin) -> Self {
        Self { material, origin }
    }

    /// Return public, value-free origin metadata.
    #[must_use]
    pub const fn origin(&self) -> &CredentialOrigin {
        &self.origin
    }

    /// Borrow the secret only for the duration of one injected delivery call.
    pub fn expose_to<R>(&self, deliver: impl FnOnce(&[u8]) -> R) -> R {
        deliver(self.material.as_slice())
    }

    /// Deliver the borrowed value to one injected provider-dispatch sink.
    ///
    /// This method does not grant caller authority. The sink remains responsible
    /// for dispatch authorization and for not retaining the borrowed bytes.
    ///
    /// # Errors
    ///
    /// Returns the sink-owned failure.
    pub fn deliver_to_provider<S: ProviderCredentialSink>(
        &self,
        sink: &mut S,
    ) -> Result<(), S::Error> {
        sink.deliver(&self.origin.declaration, self.material.as_slice())
    }
}
