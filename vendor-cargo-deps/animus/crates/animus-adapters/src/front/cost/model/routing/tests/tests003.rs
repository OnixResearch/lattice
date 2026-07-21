    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ReceiptLeakFixture {
        raw_prompt: Option<&'static str>,
        provider_payload: Option<&'static str>,
        credential: Option<&'static str>,
        pricing_store_dump: Option<&'static str>,
        hidden_reasoning: Option<&'static str>,
    }

    impl ReceiptLeakFixture {
        const fn none() -> Self {
            Self {
                raw_prompt: None,
                provider_payload: None,
                credential: None,
                pricing_store_dump: None,
                hidden_reasoning: None,
            }
        }

        const fn raw_prompt() -> Self {
            Self {
                raw_prompt: Some(RAW_PROMPT),
                ..Self::none()
            }
        }

        const fn provider_payload() -> Self {
            Self {
                provider_payload: Some(PROVIDER_PAYLOAD),
                ..Self::none()
            }
        }

        const fn credential() -> Self {
            Self {
                credential: Some(CREDENTIAL),
                ..Self::none()
            }
        }

        const fn pricing_store_dump() -> Self {
            Self {
                pricing_store_dump: Some(PRICING_STORE_DUMP),
                ..Self::none()
            }
        }

        const fn hidden_reasoning() -> Self {
            Self {
                hidden_reasoning: Some(HIDDEN_REASONING),
                ..Self::none()
            }
        }
    }

    fn receipt(
        decision: &ModelRoutingDecision,
        fixture: ReceiptLeakFixture,
    ) -> ModelRoutingReceiptDraft<'_> {
        ModelRoutingReceiptDraft {
            decision,
            raw_prompt: fixture.raw_prompt,
            provider_payload: fixture.provider_payload,
            credential: fixture.credential,
            pricing_store_dump: fixture.pricing_store_dump,
            hidden_reasoning: fixture.hidden_reasoning,
        }
    }

    fn assert_receipt_leak(decision: &ModelRoutingDecision, fixture: ReceiptLeakFixture) {
        let error = ModelRoutingReceipt::validate(&receipt(decision, fixture))
            .expect_err("unsafe routing receipt should fail");
        assert!(matches!(error, ModelRoutingError::ReceiptLeak { .. }));
    }
