use basalt::CapabilityGrant;
use basalt::EnforcementRequest;
use basalt::enforce;
use basalt::parse_policy_json;

const POLICY_JSON: &str = r#"{
  "schema_version": "ucan-nickel-contracts.policy.v1",
  "contracts": {
    "document-write": {
      "id": "document-write",
      "description": "Write bounded document resources.",
      "resource_prefixes": ["did:key:z6Mkresource/docs/"],
      "abilities": ["document/write"]
    }
  }
}"#;

#[derive(Debug, Clone, Copy)]
struct FixtureToken {
    issuer: &'static str,
    audience: &'static str,
    resource: &'static str,
    ability: &'static str,
    valid_signature: bool,
    not_before: u64,
    expires_at: u64,
}

impl FixtureToken {
    fn translate_verified_grant(
        self,
        expected_issuer: &str,
        expected_audience: &str,
        now: u64,
    ) -> Option<CapabilityGrant> {
        if !self.valid_signature {
            return None;
        }
        if self.issuer != expected_issuer || self.audience != expected_audience {
            return None;
        }
        if now < self.not_before || now >= self.expires_at {
            return None;
        }
        CapabilityGrant::new(self.resource, self.ability).into()
    }
}

const fn fixture_token() -> FixtureToken {
    FixtureToken {
        issuer: "did:key:zFixtureIssuer",
        audience: "did:key:zFixtureAudience",
        resource: "did:key:z6Mkresource/docs/report",
        ability: "document/write",
        valid_signature: true,
        not_before: 100,
        expires_at: 200,
    }
}

fn enforce_with_token(token: FixtureToken, resource: &str, ability: &str) -> bool {
    let policy = parse_policy_json(POLICY_JSON).expect("fixture policy parses");
    let capabilities = token
        .translate_verified_grant("did:key:zFixtureIssuer", "did:key:zFixtureAudience", 150)
        .into_iter()
        .collect::<Vec<_>>();
    let request = EnforcementRequest::new("document-write", resource, ability).with_capabilities(capabilities);

    enforce(&policy, &request).expect("fixture grants are valid capability documents").is_allowed()
}

#[test]
fn deterministic_ucan_fixture_translates_verified_grants() {
    assert!(enforce_with_token(fixture_token(), "did:key:z6Mkresource/docs/report", "document/write"));
}

#[test]
fn deterministic_ucan_fixture_fails_closed_for_invalid_or_mismatched_tokens() {
    let cases = [
        FixtureToken {
            valid_signature: false,
            ..fixture_token()
        },
        FixtureToken {
            audience: "did:key:zWrongAudience",
            ..fixture_token()
        },
        FixtureToken {
            issuer: "did:key:zWrongIssuer",
            ..fixture_token()
        },
        FixtureToken {
            resource: "did:key:z6Mkresource/docs/other",
            ..fixture_token()
        },
        FixtureToken {
            ability: "document/delete",
            ..fixture_token()
        },
        FixtureToken {
            not_before: 175,
            ..fixture_token()
        },
        FixtureToken {
            expires_at: 125,
            ..fixture_token()
        },
    ];

    for token in cases {
        assert!(!enforce_with_token(token, "did:key:z6Mkresource/docs/report", "document/write"), "{token:?}");
    }
}

#[test]
fn fixture_material_is_local_deterministic_and_secret_free() {
    let token = fixture_token();

    for value in [token.issuer, token.audience, token.resource, token.ability] {
        assert!(!value.contains("password"));
        assert!(!value.contains("secret"));
        assert!(!value.contains("token="));
    }
    assert!(token.issuer.starts_with("did:key:zFixture"));
    assert!(token.audience.starts_with("did:key:zFixture"));
}
