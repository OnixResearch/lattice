# Update the Octet input

## Why

Lattice pins Octet revision `493eb43402e5f6ece57cbeec0893c84a0a008200`. Published Octet revision `aa3fb4a2483439f332418aa1df3cfda04158f8fd` changes finding identities and diagnostic text.

A direct baseline refresh can accept new warning debt. Lattice needs a bounded semantic migration before it can admit the new validation input.

## What changes

- Apply stack dependency plan `7076e99af7d400cc362bb2a9fc6b749fdb0b5282af9f56889cb6a019733378c7` through the Nix input adapter.
- Normalize only reviewed `ambiguous_params` and `numeric_units` wording changes.
- Compare semantic finding multiplicity so repeated identities cannot hide warning growth.
- Review each candidate-only finding before a source fix or explicit admission.
- Refresh the baseline only after the migration accepts the complete transition.

## Non-goals

This change does not weaken Octet policy or accept all candidate findings. It does not prove that accepted findings are correct or harmless.
