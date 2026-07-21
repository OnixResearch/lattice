
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ReceiptLeakFixture {
        raw_history: Option<&'static str>,
        credential: Option<&'static str>,
        shell_command: Option<&'static str>,
        full_path: Option<&'static str>,
        transcript: Option<&'static str>,
    }

    impl ReceiptLeakFixture {
        const fn none() -> Self {
            Self {
                raw_history: None,
                credential: None,
                shell_command: None,
                full_path: None,
                transcript: None,
            }
        }

        const fn raw_history() -> Self {
            Self {
                raw_history: Some(RAW_HISTORY),
                ..Self::none()
            }
        }

        const fn credential() -> Self {
            Self {
                credential: Some(CREDENTIAL),
                ..Self::none()
            }
        }

        const fn shell_command() -> Self {
            Self {
                shell_command: Some(SHELL_COMMAND),
                ..Self::none()
            }
        }

        const fn full_path() -> Self {
            Self {
                full_path: Some(FULL_PATH),
                ..Self::none()
            }
        }

        const fn transcript() -> Self {
            Self {
                transcript: Some(TRANSCRIPT),
                ..Self::none()
            }
        }
    }

    fn receipt(plan: &RestorePlan, fixture: ReceiptLeakFixture) -> RestoreReceiptDraft<'_> {
        RestoreReceiptDraft {
            plan,
            raw_history: fixture.raw_history,
            credential: fixture.credential,
            shell_command: fixture.shell_command,
            full_path: fixture.full_path,
            transcript: fixture.transcript,
        }
    }

    fn assert_receipt_leak(plan: &RestorePlan, fixture: ReceiptLeakFixture) {
        let error = RestoreReceipt::validate(&receipt(plan, fixture))
            .expect_err("unsafe restore receipt should fail");
        assert!(matches!(error, SessionRestoreError::ReceiptLeak(_)));
    }
