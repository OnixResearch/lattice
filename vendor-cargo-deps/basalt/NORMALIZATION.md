# Basalt vendor normalization

Source revision: `bb8fb5f4a763f532df96b6e65407163ac668834e`

Original archive BLAKE3: `1492cc103aab54c9b5f9400b74cbbc623f3a8ab924197a9d4f47275876019e85`

UCAN snapshot revision: `277ab768eb532ac0c715a3bbff2a7dc37a702f11`

Lattice copied the owner-authorized Basalt source because anonymous canonical access was unavailable. It flattened nested Cargo workspace dependencies for this build. It also restored the two Trellis HDL golden fixtures referenced by the copied `verified-logic` test module.

Therefore, this directory is not a byte-identical copy of the original archive. `SOURCE` records the original identity. Cargo resolves Basalt only from this tracked directory. It has no runtime, sibling, or network fallback.
