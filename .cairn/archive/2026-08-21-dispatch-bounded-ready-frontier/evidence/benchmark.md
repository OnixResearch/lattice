# Bounded ready-frontier comparative fixture

<!-- r[verify lattice.workflow_execution.ready_frontier.benchmark] -->

Command:

```text
nix develop -c cargo test --test test_ready_frontier_dispatch benchmark_serial_and_bounded_frontier_on_the_same_effect_fixture -- --ignored --nocapture
```

Workload identity:

```text
blake3:55604e2765f89046f53ba81f44514e782e53436905613e9912b7d143dfc1543a
```

The identity frames two 300ms shell nodes, `done` outputs, disjoint aliases, serial capacity one, and frontier capacity two.

Observed on 2026-08-21:

```text
serial=626.427825ms bounded_ready_frontier=313.475304ms
```

Both measured modes ran the same two independent shell-effect nodes with the same delays and explicit disjoint aliases. Both produced two `done` outputs.

The ideal shell-delay critical paths are 600ms and 300ms. The observed excess was 26.427825ms for serial and 13.475304ms for the bounded frontier. This excess combines dispatch, process startup, timer, and host effects; it is not a scheduler-only measurement.

Measured maximum in-flight work was one for serial and two for the bounded frontier. Idle slot time was not instrumented for the host measurement.

The bounded duration model also compares one fork fixture:

```text
serial=370ms dependency_level_barrier=340ms ready_frontier=310ms
```

The model uses a 10ms source, a 300ms slow sibling, a 30ms fast sibling, and a 30ms fast-branch child. The ready frontier releases the child before the slow sibling completes.

At two available slots, modeled idle slot time is 310 slot-ms for the dependency-level barrier and 250 slot-ms for the ready frontier. Modeled dispatch overhead is zero. Maximum in-flight work is one for serial and two for both concurrent modes.

Both fixtures passed. The bounded frontier completed in less elapsed time than capacity-one execution and the dependency-level barrier model.

This is one local comparative observation. It is not a portable performance claim, a release target, or evidence for the paper's hardware-specific results. Process startup, host load, and timer resolution remain outside the deterministic scheduler contract.
