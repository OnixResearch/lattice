# Design

A pure image-format classifier accepts PNG/JPEG suffixes and unspecified suffixes, while rejecting explicitly unsupported suffixes. Provider-backed workflow validation maps that decision to a fatal node diagnostic. The import shell runs existing pure validation before opening or mutating the store.

Maintained tests prove rejected image import preserves prior state, malformed control-flow conversion yields a node-bound error, credential references round trip without resolution, and malformed cookbook drift yields deterministic rejection. A byte-bound completion inventory reclassifies all 41 rows as exact while preserving the 222-item missing set and adding no selected markers.
