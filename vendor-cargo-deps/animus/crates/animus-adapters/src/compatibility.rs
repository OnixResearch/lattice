fn assert_type_resolves<T>() {}

#[test]
fn existing_adapter_paths_compile() {
    assert_type_resolves::<crate::effect::HostAuthorityDecision>();
    assert_type_resolves::<crate::messaging::MessagingPolicy>();
    assert_type_resolves::<crate::pijul::PijulOperation>();
    assert_type_resolves::<crate::secret_resolution::SecretResolutionPlan>();
    assert_type_resolves::<crate::skill_source::FrozenSkillSource>();
    assert_type_resolves::<crate::trajectory::TrajectoryPolicy>();

    assert!(core::mem::size_of::<crate::messaging::MessagingPolicy>() > 0);
    assert!(core::mem::size_of::<crate::trajectory::TrajectoryPolicy>() > 0);
}
