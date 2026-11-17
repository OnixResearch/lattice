## Implementation

- [x] [serial] Retain bounded source/member/parameter inputs separately from portable materialized workflow, with truthful snapshot and unavailable-origin boundaries. r[lattice.source_authoring.capture]
- [x] [serial] Persist independent source drafts and atomically admit exact successful previews with durable revision CAS, generation/ABA protection and last-admission preservation. r[lattice.source_authoring.admission]
- [x] [serial] Apply explicit materialized lineage forks across typed, graph and save/export paths; preserve legacy no-source behavior and old workflow clones. r[lattice.source_authoring.fork]
- [x] [serial] Supply actual CLI/TUI source editing and distinct exact source archive versus explicit expanded export paths. r[lattice.source_authoring.exports]
- [x] [serial] Define focused import/store/reopen/export, failure/stale/concurrency/drift/fork/legacy/snapshot and real CLI/TUI regression cases. r[lattice.source_authoring.admission]

## Integration qualification

- [x] [serial] Parent exercises the integrated source/format/template/CLI/TUI contracts and full workspace harness, retaining actual outcomes in evidence/focused-observation.json and the repository gate observation. r[lattice.source_authoring.admission]
- [x] [serial] Parent checks root help/export/source-archive wiring, updates root operator documentation and runs required repository/Cairn gates. r[lattice.source_authoring.exports]

The delegated implementation initially supplied source and definitions without
executing checks. Parent integration now records actual focused and full-harness
observations. Complete-module closure and Choregraph #6 origins remain outside
the available producer contract; #21 remains open for those criteria.
