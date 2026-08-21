## Tasks

- [x] [serial] Record current expression, workflow-validation, runner-plan, Choregraph, cache, replay, and portable-format baselines. r[lattice.workflow_validation.explicit_dependencies.baseline]
- [x] [depends:isolate-workflow-domain-boundaries] Add the exhaustive expression-bearing field inventory and fail its guard for unclassified additions. r[lattice.workflow_validation.explicit_dependencies.inventory]
- [x] [depends:lattice.workflow_validation.explicit_dependencies.inventory] Implement pure typed expression-reference extraction with stable field-path diagnostics. r[lattice.workflow_validation.explicit_dependencies.extraction]
- [x] [depends:lattice.workflow_validation.explicit_dependencies.extraction] Validate producer nodes, output ports, self-references, supported shapes, duplicates, and union acyclicity. r[lattice.workflow_validation.explicit_dependencies.admission]
- [x] [depends:lattice.workflow_validation.explicit_dependencies.admission] Merge graph and expression dependencies before Choregraph admission and bind the result to machine and runner-plan identities. r[lattice.workflow_validation.explicit_dependencies.projection]
- [x] [depends:lattice.workflow_validation.explicit_dependencies.projection] Build runtime expression contexts only from exact admitted references and declared workflow inputs. r[lattice.workflow_validation.explicit_dependencies.context]
- [x] [parallel] Add positive direct, duplicate, transitive, cache, and round-trip fixtures. r[lattice.workflow_validation.explicit_dependencies.compatibility]
- [x] [parallel] Add negative missing, stale, self, cyclic, malformed, unsupported, context-leak, source-mutation, and pre-effect fixtures. r[lattice.workflow_validation.explicit_dependencies.negative]
- [x] [serial] Update boundary documentation, migration notes, projection identities, and bounded non-claims. r[lattice.workflow_validation.explicit_dependencies.evidence]
- [x] [serial] Run focused tests, `scripts/check.rs`, Octet, Cairn validation and gates, operator checks, and relevant Nix checks. r[lattice.workflow_validation.explicit_dependencies.validation]
