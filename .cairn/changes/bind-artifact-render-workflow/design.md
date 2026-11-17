# Design: one thin render consumer

The immutable producer manifest and batch state machine remain the only scene,
partition, content, fence, scheduling, retry and assembly protocol. A sparse
byte-exact public import supplies pure contracts; the real selected producer CLI
supplies the existing authenticated iroh/effect boundary. No sibling dependency
or cloned scheduler/evaluator is introduced.

The exact operation plan freezes canonical request identity in addition to its
absolute location. Ordinary Lattice graph/effect execution supplies actual host
run/node/workflow/invocation provenance and the admitted authority RunIdentity;
numeric store-scoped IDs alone are insufficient. Initial Advance binds a compute
owner before a possible Submit. A new explicit Observe graph may acquire its own
observer provenance but cannot replace the original compute owner or submit.
Host-dispatch records are local provenance, never a new authorization token. The
existing configured effect provider must independently admit dispatch.

The private create-only journal has one OS-held writer lock, bounded record count
and record bytes, synchronized files/directories, immutable owner provenance and
one bounded download slot per accepted content identity. Possible dispatch is
recorded before Submit. Exact deterministic producer run identity permits
read-only recovery after ACK loss; absence remains Unknown. Publication of
content-addressed input objects does not itself dispatch a render. No UI/session
lifetime owns a second computation.

Accepted snapshots must match the complete exact job and canonical fences and
request/result identities, including current producer policy. Only accepted
content IDs are fetched. Producer assembly and strict final verification reject
malformed/conflicting coverage or self-consistent wrong pixels. Final metadata
binds the owning logical attempt, host compute execution and original budget.
The classifier rechecks the committed journal and retained bytes rather than
trusting process exit zero or an output JSON flag.

The cancellation tombstone precedes a remote request and wins over late success.
Only observed producer terminal cancellation permits Stopped. A stopped local
client supplies no such fact. Supersession/currentness and absolute acceptance
deadline are rechecked before publication/acceptance. Reservation ceilings include
all canonical task retry bounds; remote usage remains unavailable because the
producer result schema supplies none. Adapter wall timings are separately
measured and accumulated, not substituted for remote accounting.

Qualification is an opt-in retained same-host process campaign over actual
Lattice graph admission and the real producer component. The fixture owns no
acceptance shortcuts and records failure if a requested active interruption is
not actually observed. It compares every accepted tile/final identity plus PPM
bytes under the same actual component/backend/host. Pack identity is not
applicable to this closed builtin operation route; cross-backend comparator,
two-host execution, machine-loss durability and metered remote usage are not
fabricated. Imported producer tests are source, not passing Lattice evidence.
