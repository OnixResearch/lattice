# Operation-data Octet baseline migration

Date: 2026-08-15

The operation-data change added one table-initialization call to `src/store/mod.rs`. The existing `excessive_file_length` finding changed identity because its reviewed source snippet changed. The lint, file, normalized message, severity, crate, and effect family did not change.

Reviewed semantic pair:

- stale: `b3:d638efd8bfa74e5f`;
- current: `b3:07ed5522abaaae4a`.

Set evidence:

- new-set hash: `b3:4806e94da6207705757a0ffdca041d4688b45443239e47de05256e54c02f9b8c`;
- stale-set hash: `b3:5408c0c116d3e007591d1a598eee6d8391c25dc1b5e2efb9d185d8fe7d01d856`;
- old baseline: 855 identities, `b3:990b93716039ed0608a4a3614b46808ff359591f0ea75b4973460c95ef726ad7`;
- new baseline: 855 identities, `b3:ed5a97105f81114b4b8688e8194142fe9fda01fbf2f8ca05dd17cfe3e7383b00`.

The migration script passed its positive and negative self-test. The final Octet run reports zero new, stale, or error findings. No lint category grew.
