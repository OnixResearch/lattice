let Digest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == 64
) in
let Source = {
  repository | String,
  release | String,
  revision | String,
  license | [| 'mit |],
  crates_io_sha256 | Digest,
  upstream_files_manifest_blake3 | Digest,
  upstream_generated_manifest_blake3 | Digest,
  patched_manifest_blake3 | Digest,
  rust_source_modified | Bool,
  manifest_only_patch | Bool,
  default_cli_or_tui_enabled | Bool,
  serde_json_preserve_order_enabled | Bool,
} in
{
  repository = "https://github.com/toon-format/toon-rust",
  release = "0.5.0",
  revision = "2136cb1a35f3ed63be733bb74d36b89d3b4592dd",
  license = 'mit,
  # crates.io requires SHA-256 for published package checksums.
  crates_io_sha256 = "8f89570c1a68d73941f728cca32a4345b2ffca36667ad921af336c60309a3e7e",
  upstream_files_manifest_blake3 = "7aef44e975bdab7a0a705bf824f5ffa0e292bd2d1cb122c6f1585364ad33dc01",
  upstream_generated_manifest_blake3 = "94e46fabf052aeaba2be0d42d6eeb45ccb63ada21d13c38e9c25064dd81f19b5",
  patched_manifest_blake3 = "2c55674a9d444937cc5f19c5a0b5d0bf7668de47dbafb2f72c32dade1833692f",
  rust_source_modified = false,
  manifest_only_patch = true,
  default_cli_or_tui_enabled = false,
  serde_json_preserve_order_enabled = false,
} | Source
