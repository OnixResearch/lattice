//! Positive and negative interactive secret path fixtures.

#![cfg(unix)]

use std::os::unix::fs::{MetadataExt, symlink};

use animus_std_secretspec::{
    InteractiveSecretFileHost, MaskedSecretInput, SecretEntryAuthority, SecretEntryError,
    SecretEntryRequest, SecretEntrySessionClass, validate_path_only_record,
};
use zeroize::Zeroizing;

const REQUEST_ID: &str = "request-1";
const PROMPT: &str = "API token";
const SECRET: &[u8] = b"private-token-value";
const MAX_SECRET_BYTES: u32 = 128;
const PERMISSION_MASK: u32 = 0o777;
const PRIVATE_FILE_MODE: u32 = 0o600;

struct FakeMaskedTerminal {
    secret: &'static [u8],
    call_count: u32,
}

impl MaskedSecretInput for FakeMaskedTerminal {
    fn read_masked(&mut self, _prompt_label: &str) -> Result<Zeroizing<Vec<u8>>, SecretEntryError> {
        self.call_count += 1;
        Ok(Zeroizing::new(self.secret.to_vec()))
    }
}

fn request() -> SecretEntryRequest<'static> {
    SecretEntryRequest {
        request_identity: REQUEST_ID,
        prompt_label: PROMPT,
        max_secret_bytes: MAX_SECRET_BYTES,
    }
}

fn root_authority() -> SecretEntryAuthority {
    SecretEntryAuthority {
        session_class: SecretEntrySessionClass::RootInteractive,
        masked_terminal_admitted: true,
        secret_host_admitted: true,
        separate_authority_admitted: false,
    }
}

#[test]
fn masked_root_entry_returns_only_a_private_path_and_closes_cleanly() {
    // r[verify animus.secret.path.masked]
    // r[verify animus.secret.file.private]
    // r[verify animus.secret.file.cleanup]
    // r[verify animus.secret.scope.root]
    let scratch = tempfile::tempdir().expect("scratch");
    let mut host = InteractiveSecretFileHost::new(scratch.path()).expect("host");
    let mut terminal = FakeMaskedTerminal {
        secret: SECRET,
        call_count: 0,
    };
    let result = host
        .enter_secret(request(), root_authority(), &mut terminal)
        .expect("entry");
    assert_eq!(terminal.call_count, 1);
    assert!(result.path().starts_with(scratch.path()));
    assert_eq!(
        std::fs::metadata(result.path()).expect("metadata").mode() & PERMISSION_MASK,
        PRIVATE_FILE_MODE
    );
    assert_eq!(std::fs::read(result.path()).expect("secret file"), SECRET);
    assert_eq!(host.tracked_count(), 1);
    host.close().expect("close cleanup");
    assert!(!result.path().exists());
    assert_eq!(host.tracked_count(), 0);
}

#[test]
fn leak_headless_child_and_unmasked_requests_fail_before_secret_disclosure() {
    // r[verify animus.secret.path.value_leak]
    // r[verify animus.secret.scope.headless]
    assert_eq!(
        validate_path_only_record(b"result=private-token-value", SECRET),
        Err(SecretEntryError::SecretLeak)
    );
    assert_eq!(
        validate_path_only_record(b"result=/scratch/secret-request-1", SECRET),
        Ok(())
    );
    let scratch = tempfile::tempdir().expect("scratch");
    for session_class in [
        SecretEntrySessionClass::Headless,
        SecretEntrySessionClass::Child,
    ] {
        let mut host = InteractiveSecretFileHost::new(scratch.path()).expect("host");
        let mut terminal = FakeMaskedTerminal {
            secret: SECRET,
            call_count: 0,
        };
        let authority = SecretEntryAuthority {
            session_class,
            ..root_authority()
        };
        assert_eq!(
            host.enter_secret(request(), authority, &mut terminal),
            Err(SecretEntryError::AuthorityDenied)
        );
        assert_eq!(terminal.call_count, 0);
    }
    let mut host = InteractiveSecretFileHost::new(scratch.path()).expect("host");
    let mut terminal = FakeMaskedTerminal {
        secret: SECRET,
        call_count: 0,
    };
    assert_eq!(
        host.enter_secret(
            request(),
            SecretEntryAuthority {
                masked_terminal_admitted: false,
                ..root_authority()
            },
            &mut terminal,
        ),
        Err(SecretEntryError::MaskedTerminalUnavailable)
    );
    assert_eq!(terminal.call_count, 0);
}

#[test]
fn escape_existing_target_and_symlink_roots_fail_closed() {
    // r[verify animus.secret.file.private]
    let scratch = tempfile::tempdir().expect("scratch");
    let existing = scratch.path().join("secret-request-1");
    std::fs::write(existing.as_path(), b"existing").expect("existing target");
    let mut host = InteractiveSecretFileHost::new(scratch.path()).expect("host");
    let mut terminal = FakeMaskedTerminal {
        secret: SECRET,
        call_count: 0,
    };
    assert_eq!(
        host.enter_secret(request(), root_authority(), &mut terminal),
        Err(SecretEntryError::TargetRejected)
    );
    assert_eq!(terminal.call_count, 0);
    assert_eq!(
        host.enter_secret(
            SecretEntryRequest {
                request_identity: "../escape",
                ..request()
            },
            root_authority(),
            &mut terminal,
        ),
        Err(SecretEntryError::InvalidRequest)
    );

    let parent = tempfile::tempdir().expect("parent");
    let root = parent.path().join("root");
    std::fs::create_dir(root.as_path()).expect("root");
    let linked = parent.path().join("linked");
    symlink(root.as_path(), linked.as_path()).expect("scratch symlink");
    assert!(matches!(
        InteractiveSecretFileHost::new(linked.as_path()),
        Err(SecretEntryError::ScratchRootRejected)
    ));
}

#[test]
fn caller_can_remove_a_tracked_secret_before_runtime_close() {
    // r[verify animus.secret.file.cleanup]
    let scratch = tempfile::tempdir().expect("scratch");
    let mut host = InteractiveSecretFileHost::new(scratch.path()).expect("host");
    let mut terminal = FakeMaskedTerminal {
        secret: SECRET,
        call_count: 0,
    };
    let result = host
        .enter_secret(request(), root_authority(), &mut terminal)
        .expect("entry");
    host.consume(&result).expect("consume cleanup");
    assert!(!result.path().exists());
    assert_eq!(host.tracked_count(), 0);
    host.close().expect("empty close");
}
