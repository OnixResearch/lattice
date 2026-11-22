//! Positive and negative cases for the secret-like input rule.

fn alnum(length: usize) -> String {
    "aB3dE5fG7hJ9kL1mN3pQ".chars().cycle().take(length).collect()
}

#[test]
fn secret_identifiers_are_word_matched_ignoring_case() {
    for name in [
        "api_token",
        "github_pat",
        "db_password",
        "api_key",
        "PRIVATE_KEY",
        "auth",
        "Client_Secret",
    ] {
        assert!(super::is_secret_identifier(name), "{name}");
    }
    for name in [
        "catalog_root",
        "author",
        "sort_key",
        "tokenizer",
        "path",
        "key",
        "monkey_key_path",
    ] {
        assert!(!super::is_secret_identifier(name), "{name}");
    }
}

/// A deterministic uniform base64url string, from a xorshift generator.
fn base64url(state: &mut u64, length: usize) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    (0..length)
        .map(|_| {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            char::from(ALPHABET[usize::from(state.to_le_bytes()[0] & 63)])
        })
        .collect()
}

#[test]
fn each_provider_token_family_is_refused() {
    // Synthetic tokens are assembled at runtime so no literal looks like a credential.
    let tokens = [
        format!("ghp_{}", alnum(36)),
        format!("gho_{}", alnum(36)),
        format!("ghs_{}", alnum(36)),
        format!("dir/github_pat_11{}_{}", alnum(20), alnum(59)),
        format!("glpat-{}", alnum(20)),
        format!("glpat-{}.01.{}", alnum(27), "0a1b2c3d4"),
        format!("xoxb-{}-{}-{}", "1".repeat(10), "2".repeat(13), alnum(24)),
        format!("xoxp-1234567890-{}", alnum(20)),
        format!("xapp-1-A{}-{}-{}", "0".repeat(10), "3".repeat(13), "c4".repeat(32)),
        format!("sk-proj-{}-{}_{}", alnum(30), alnum(20), alnum(20)),
        format!("sk-{}", alnum(48)),
        format!("sk_live_{}", alnum(24)),
        format!("rk_live_{}", alnum(24)),
        "AKIAIOSFODNN7EXAMPLE".to_owned(),
        format!("keys/ASIA{}", "Y34FZKBOKMUTVV7A"),
        format!("AIza{}", alnum(35)),
        format!("ya29.{}", alnum(40)),
        format!("npm_{}", alnum(36)),
        format!("pypi-AgEIcHlwaS5vcmc{}", alnum(60)),
        format!("hf_{}", alnum(34)),
        format!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.{}", alnum(20)),
        format!("tokens/hf_{}.txt", alnum(34)),
        "creds/AKIAIOSFODNN7EXAMPLE.json".to_owned(),
    ];
    for token in &tokens {
        assert!(super::is_secret_value(token), "{token}");
    }
}

#[test]
fn random_runs_are_token_shaped() {
    for value in [
        alnum(20),
        format!("out/{}", alnum(40)),
        format!("build-{}.log", alnum(32)),
    ] {
        assert!(super::is_secret_value(&value), "{value}");
    }
    let short = alnum(19);
    let lower_hex = "0123456789abcdef0123456789abcdef01234567";
    let upper_words = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123";
    for value in [short.as_str(), lower_hex, upper_words] {
        assert!(!super::is_secret_value(value), "{value}");
    }
}

#[test]
fn generated_base64url_secrets_are_refused() {
    // Expected refusal is about 98.0%, 99.2% and 99.8% (see `WORD_RUN_MEAN_BYTES`).
    const SAMPLES: usize = 2000;
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for (length, minimum_refused) in [(32, 1920), (43, 1960), (64, 1980)] {
        let refused = (0..SAMPLES).filter(|_| super::is_secret_value(&base64url(&mut state, length))).count();
        assert!(refused >= minimum_refused, "{refused} of {SAMPLES} at {length} characters");
    }
}

#[test]
fn ordinary_refs_and_repo_paths_are_not_secret_values() {
    for value in [
        "dependabot/npm_and_yarn/express-4.18.2",
        ".github/workflows/pypi-publish.yml",
        "scripts/npm_publish.sh",
        "scripts/hf_upload_model.py",
        "data/ASIA_PACIFIC_REGIONS.csv",
        "npm_and_yarn",
        "feature/PROJ-1234-add-retry-to-upload-client",
        "bugfix/ABC-123-Fix-login-timeout-Safari15",
        "src/OAuth2AuthorizationServerConfig.java",
        "Jackson2ObjectMapperBuilder.java",
        "AmazonS3ClientBuilder.java",
        "X509CertificateValidator.cs",
        "CustomerControllerV2.java",
        "SHA256DigestComputation.swift",
        "builtins/agent-evaluations",
        "src/ghp_",
        "sk-a",
        "Readme.md",
        "a/B/c1",
        "AKIA/ASIA",
    ] {
        assert!(!super::is_secret_value(value), "{value}");
    }
}

fn declaration(
    name: &str,
    value_type: super::super::super::params::ParamType,
) -> super::super::super::params::ParamDeclaration {
    super::super::super::params::ParamDeclaration {
        name: name.to_owned(),
        value_type,
        required: true,
        default: None,
        bindings: vec![super::super::super::params::ParamTarget {
            node: "probe".to_owned(),
            shell_env: format!("LATTICE_PARAM_{}", name.to_ascii_uppercase()),
        }],
    }
}

#[test]
fn only_relative_path_parameters_are_screened() {
    use super::super::super::params::ParamType;
    use super::super::super::params::ParamValue;
    let path = ParamType::RelativePath { max_bytes: 128 };
    let tokens = ParamType::Integer { min: 1, max: 4096 };
    let modes = ParamType::Enum {
        values: vec!["oauth".into(), "npm_and_yarn".into()],
    };
    let declared = vec![
        declaration("max_tokens", tokens),
        declaration("auth_mode", modes),
        declaration("target", path.clone()),
    ];
    let supplied = [
        ("max_tokens".to_owned(), ParamValue::Integer(2048)),
        ("auth_mode".to_owned(), ParamValue::Text("npm_and_yarn".into())),
        ("target".to_owned(), ParamValue::Text("scripts/npm_publish.sh".into())),
    ]
    .into();
    assert_eq!(super::screen(&declared, &supplied), Ok(()));

    let secret_path = vec![declaration("api_token", path.clone())];
    let refused = super::screen(&secret_path, &std::collections::BTreeMap::new());
    assert_eq!(
        refused,
        Err(super::super::InputError::SecretLike {
            param: "api_token".into(),
            reason: super::super::SecretReason::Name,
        })
    );
    let mut keyed = declaration("target", path.clone());
    keyed.bindings[0].shell_env = "LATTICE_PARAM_PRIVATE_KEY".into();
    let refused = super::screen(&[keyed], &std::collections::BTreeMap::new());
    assert!(matches!(
        refused,
        Err(super::super::InputError::SecretLike {
            reason: super::super::SecretReason::Name,
            ..
        })
    ));

    let token = [("target".to_owned(), ParamValue::Text(format!("out/ghp_{}", alnum(36))))].into();
    let refused = super::screen(&[declaration("target", path)], &token);
    assert_eq!(
        refused,
        Err(super::super::InputError::SecretLike {
            param: "target".into(),
            reason: super::super::SecretReason::Value,
        })
    );
}
