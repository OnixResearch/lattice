# Design: Bounded opaque source values

`SourceValue` owns a UTF-8 string with a 256-byte maximum. It rejects empty strings, ASCII control characters, and delete.

Provider remains a lowercase `Label`. Repository and revision use `SourceValue` because provider-native identities are case-sensitive and must not be normalized.

Request identity frames the exact source bytes. No shell, filesystem, network, or authority behavior enters the contract core.
