pub(super) const TOTAL_FUEL: u64 = 10_000;
pub(super) const QUANTUM_FUEL: u64 = 64;
pub(super) const MAX_SLICES: u64 = 1_024;
pub(super) const DEADLINE_TICKS: u64 = 10;
pub(super) const MIN_YIELDS: u64 = 2;
pub(super) const LOOP_ITERATIONS: u8 = 96;
pub(super) const ACTION_MARK: usize = 1;
pub(super) const CLEANUP_MARK: usize = 2;

pub(super) fn bounds() -> hardened_wasmtime_runtime::resource::admission::RuntimeBounds {
    let mut bounds = super::support::profile().limits;
    bounds.fuel = TOTAL_FUEL;
    bounds.max_concurrency = 1;
    bounds
}

pub(super) fn prepare(
    bytes: &[u8],
    bounds: hardened_wasmtime_runtime::resource::admission::RuntimeBounds,
    ticks: u64,
) -> (
    std::sync::Arc<hardened_wasmtime_shell::ComponentInvocation>,
    hardened_wasmtime_shell::EpochService,
) {
    let runtime = super::support::engine_with_bounds(bounds);
    let service = runtime.epoch_service();
    let facts = super::support::facts(
        bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "actor",
        &["invoke"],
    );
    let mut options = super::support::options(bytes, &facts, &service);
    options.deadline.budget_ticks = ticks;
    let prepared = hardened_wasmtime_shell::ComponentInvocation::prepare(&runtime, options)
        .expect("component");
    (std::sync::Arc::new(prepared), service)
}

pub(super) fn options(quantum_fuel: u64) -> hardened_wasmtime_shell::SliceOptions {
    hardened_wasmtime_shell::SliceOptions {
        export: String::from("invoke"),
        input: Vec::new(),
        quantum_fuel,
    }
}

pub(super) fn start(
    prepared: &std::sync::Arc<hardened_wasmtime_shell::ComponentInvocation>,
) -> hardened_wasmtime_shell::ResumableInvocation {
    std::sync::Arc::clone(prepared)
        .start_sliced(options(QUANTUM_FUEL), (), |_, _| {
            Err(hardened_wasmtime_shell::HostDenied)
        })
        .expect("admitted sliced invocation")
}
pub(super) fn checkpoint(
    invocation: &mut hardened_wasmtime_shell::ResumableInvocation,
) -> (
    Result<
        hardened_wasmtime_shell::InstantiationCheckpoint,
        hardened_wasmtime_shell::InvocationOutcome,
    >,
    u64,
) {
    let mut yields = 0;
    let result = (0..super::slicesupport::MAX_SLICES)
        .find_map(|_| match invocation.run_slice().expect("live invocation") {
            hardened_wasmtime_shell::SliceStep::Yielded(boundary) => {
                assert_eq!(
                    boundary.phase,
                    hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation
                );
                assert_eq!(boundary.quantum_fuel, super::slicesupport::QUANTUM_FUEL);
                yields += 1;
                None
            }
            hardened_wasmtime_shell::SliceStep::Instantiated(observation) => Some(Ok(observation)),
            hardened_wasmtime_shell::SliceStep::Completed(outcome) => Some(Err(outcome)),
        })
        .expect("finite initialization fuel budget");
    (result, yields)
}

pub(super) fn finish(
    invocation: &mut hardened_wasmtime_shell::ResumableInvocation,
) -> (hardened_wasmtime_shell::InvocationOutcome, u64) {
    let mut yields = 0;
    let mut previous_ordinal: Option<u64> = None;
    let mut has_checkpoint = false;
    let terminal =
        (0..MAX_SLICES).find_map(|_| match invocation.run_slice().expect("live invocation") {
            hardened_wasmtime_shell::SliceStep::Yielded(boundary) => {
                if let Some(previous) = previous_ordinal {
                    assert_eq!(
                        boundary.ordinal,
                        previous.checked_add(1).expect("bounded fixture ordinal")
                    );
                }
                previous_ordinal = Some(boundary.ordinal);
                assert_eq!(boundary.quantum_fuel, QUANTUM_FUEL);
                yields += 1;
                None
            }
            hardened_wasmtime_shell::SliceStep::Instantiated(_) => {
                assert!(!has_checkpoint, "instantiation checkpoint must not repeat");
                has_checkpoint = true;
                None
            }
            hardened_wasmtime_shell::SliceStep::Completed(outcome) => Some((outcome, yields)),
        });
    terminal.expect("invocation must finish within its finite cumulative fuel budget")
}

pub(super) fn stateful_component() -> Vec<u8> {
    format!(
        r#"(component
  (import "host-step" (func $mark (result u32)))
  (core func $lower (canon lower (func $mark)))
  (core instance $host (export "mark" (func $lower)))
  (core module $m
    (import "host" "mark" (func $mark (result i32)))
    (memory (export "memory") 1)
    (global $progress (mut i32) (i32.const 0))
    (func (export "realloc") (param i32 i32 i32 i32) (result i32) i32.const 1024)
    (func (export "run") (param i32 i32) (result i32)
      call $mark drop
      (loop $next
        global.get $progress i32.const 1 i32.add global.set $progress
        global.get $progress i32.const {iterations} i32.lt_u br_if $next)
      i32.const 32 global.get $progress i32.store8
      i32.const 0 i32.const 0 i32.store
      i32.const 4 i32.const 32 i32.store
      i32.const 8 i32.const 1 i32.store
      call $mark drop
      i32.const 0)
    (func (export "cleanup") (param i32) (local $left i32)
      i32.const {iterations} local.set $left
      (loop $next
        local.get $left i32.const 1 i32.sub local.tee $left br_if $next)
      i32.const 32 i32.const 0 i32.store8
      ))
  (core instance $i (instantiate $m (with "host" (instance $host))))
  (alias core export $i "memory" (core memory $memory))
  (alias core export $i "realloc" (core func $realloc))
  (alias core export $i "run" (core func $run))
  (alias core export $i "cleanup" (core func $cleanup))
  (func (export "invoke") (param "input" (list u8)) (result (result (list u8) (error string)))
    (canon lift (core func $run) (memory $memory) (realloc $realloc)
      (post-return $cleanup))))"#,
        iterations = LOOP_ITERATIONS,
    )
    .into_bytes()
}

pub(super) const FINITE_WORK: &str = "(local $left i32)
  i32.const 96 local.set $left
  (loop $next local.get $left i32.const 1 i32.sub local.tee $left br_if $next)";

pub(super) fn marker_descriptor(
    facts: &hardened_wasmtime_runtime::artifact::shape::MaterializationFacts,
) -> hardened_wasmtime_runtime::restricted::imports::Descriptor {
    hardened_wasmtime_runtime::restricted::imports::Descriptor {
        function: super::support::function(&["host-step"]),
        world: facts.world.clone(),
        capability_id: String::from("observed:slice-marker"),
        adapter_id: String::from("consumer:slice-marker-v1"),
    }
}

pub(super) fn marked_prepared(
    bytes: &[u8],
) -> std::sync::Arc<hardened_wasmtime_shell::ComponentInvocation> {
    let runtime = super::support::engine_with_bounds(bounds());
    let service = runtime.epoch_service();
    let mut facts = super::support::facts(
        bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "actor",
        &["invoke"],
    );
    let descriptor = marker_descriptor(&facts);
    facts.imports.insert(descriptor.function.clone());
    let mut options = super::support::options(bytes, &facts, &service);
    options.descriptors = std::slice::from_ref(&descriptor);
    std::sync::Arc::new(
        hardened_wasmtime_shell::ComponentInvocation::prepare(&runtime, options)
            .expect("component with marker"),
    )
}

pub(super) fn bind_marker(
    _: &hardened_wasmtime_runtime::restricted::imports::Descriptor,
    linker: &mut wasmtime::component::Linker<
        hardened_wasmtime_shell::State<std::sync::Arc<std::sync::atomic::AtomicUsize>>,
    >,
) -> Result<(), hardened_wasmtime_shell::HostDenied> {
    linker
        .root()
        .func_wrap("host-step", |store, (): ()| {
            store
                .data()
                .host
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok((0u32,))
        })
        .map_err(|_| hardened_wasmtime_shell::HostDenied)
}

pub(super) fn observe_marked(
    invocation: &mut hardened_wasmtime_shell::ResumableInvocation,
    marks: &std::sync::atomic::AtomicUsize,
) -> hardened_wasmtime_shell::InvocationOutcome {
    let mut action_yields = 0;
    let mut cleanup_yields = 0;
    let mut terminal = None;
    let mut ordinal = 0;
    let mut has_checkpoint = false;
    for _ in 0..MAX_SLICES {
        match invocation.run_slice().expect("live invocation") {
            hardened_wasmtime_shell::SliceStep::Yielded(boundary) => {
                ordinal += 1;
                assert_eq!(boundary.ordinal, ordinal);
                assert_eq!(boundary.quantum_fuel, QUANTUM_FUEL);
                match marks.load(std::sync::atomic::Ordering::SeqCst) {
                    ACTION_MARK => action_yields += 1,
                    CLEANUP_MARK => cleanup_yields += 1,
                    count => assert!(count <= CLEANUP_MARK, "guest action was replayed"),
                }
            }
            hardened_wasmtime_shell::SliceStep::Instantiated(_) => {
                assert!(!has_checkpoint, "instantiation checkpoint must not repeat");
                assert_eq!(marks.load(std::sync::atomic::Ordering::SeqCst), 0);
                has_checkpoint = true;
            }
            hardened_wasmtime_shell::SliceStep::Completed(outcome) => {
                assert_eq!(
                    marks.load(std::sync::atomic::Ordering::SeqCst),
                    CLEANUP_MARK
                );
                terminal = Some(outcome);
                break;
            }
        }
    }
    assert!(
        has_checkpoint,
        "guest callback needs the instantiation checkpoint"
    );
    assert!(
        action_yields >= MIN_YIELDS,
        "finite action must span multiple slices"
    );
    assert!(
        cleanup_yields >= MIN_YIELDS,
        "cleanup must yield before exposing output"
    );
    terminal.expect("bounded completion")
}
