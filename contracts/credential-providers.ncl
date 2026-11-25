# Typed Nickel contract for `animus-secretspec` credential providers and hosts.
# r[impl lattice.credentials.animus_secret_providers.config]
let MaxMetadataBytes = 128 in
let MaxInputBytes = 65536 in
let MaxValueBytes = 1048576 in
let FrameOverheadBytes = 32 in
let MaxStderrBytes = 65536 in
let MaxTimeoutMs = 60000 in
let MaxPollIntervalMs = 1000 in
let MaxTeardownTimeoutMs = 10000 in
let MaxDeclarations = 128 in
let ReviewedCohort = "0.17.0" in

let SafeLabel = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value >= 1
  && std.string.length value <= MaxMetadataBytes
  && std.string.is_match "^[A-Za-z0-9_.-]+$" value
) in

let NonEnvId = std.contract.Sequence [
  SafeLabel,
  std.contract.from_predicate (fun value => value != "env"),
] in

let Blake3Hex = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
) in

let Cohort = std.contract.from_predicate (fun value =>
  std.is_string value && value == ReviewedCohort
) in

let ProviderKind = std.contract.from_predicate (fun value =>
  std.is_string value && value == "animus-secretspec"
) in

let ProviderScheme = std.contract.from_predicate (fun value =>
  std.is_string value
  && (value == "environment" || value == "systemd-credential" || value == "sops")
) in

let SopsCredentialKind = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.array.elem value [
    "age-key",
    "aws-secret-access-key",
    "azure-client-secret",
    "google-oauth-access-token",
    "vault-token",
  ]
) in

let BoundedInteger = fun max => std.contract.from_predicate (fun value =>
  std.is_number value
  && value >= 1
  && value <= max
  && value % 1 == 0
) in

let RequestBoundsShape = {
  input_max_bytes | BoundedInteger MaxInputBytes,
  value_max_bytes | BoundedInteger MaxValueBytes,
  stdout_max_bytes | BoundedInteger (MaxValueBytes + FrameOverheadBytes),
  stderr_max_bytes | BoundedInteger MaxStderrBytes,
  timeout_ms | BoundedInteger MaxTimeoutMs,
  poll_interval_ms | BoundedInteger MaxPollIntervalMs,
  teardown_timeout_ms | BoundedInteger MaxTeardownTimeoutMs,
} in

let RequestBounds = std.contract.Sequence [
  RequestBoundsShape,
  std.contract.from_predicate (fun bounds =>
    bounds.stdout_max_bytes == bounds.value_max_bytes + FrameOverheadBytes
    && bounds.poll_interval_ms <= bounds.timeout_ms
  ),
] in

let OptionalSafeLabel = std.contract.from_predicate (fun value =>
  value == null
  || (
    std.is_string value
    && std.string.length value >= 1
    && std.string.length value <= MaxMetadataBytes
    && std.string.is_match "^[A-Za-z0-9_.-]+$" value
  )
) in

let DeclarationMapping = {
  key | SafeLabel,
  field | optional | OptionalSafeLabel,
  declaration | SafeLabel,
  scheme | ProviderScheme,
} in

let declaration_field_value = fun decl =>
  if std.record.has_field "field" decl then decl.field else null
in

let AnimusSecretSpecProviderShape = {
  kind | ProviderKind,
  id | NonEnvId,
  cohort | Cohort,
  manifest_identity | Blake3Hex,
  policy_identity | Blake3Hex,
  profile | SafeLabel,
  scope | SafeLabel,
  declarations | Array DeclarationMapping,
  allowed_schemes | Array ProviderScheme,
  reason | SafeLabel,
  bounds | RequestBounds,
} in

let provider_is_valid = fun provider =>
  let decl_count = std.array.length provider.declarations in
  let scheme_count = std.array.length provider.allowed_schemes in
  decl_count >= 1
  && decl_count <= MaxDeclarations
  && scheme_count >= 1
  && std.array.all (fun scheme =>
    std.array.length (std.array.filter (fun s => s == scheme) provider.allowed_schemes) == 1
  ) provider.allowed_schemes
  && std.array.all (fun decl =>
    std.array.elem decl.scheme provider.allowed_schemes
    && std.array.length (
      std.array.filter (fun other =>
        other.key == decl.key && declaration_field_value other == declaration_field_value decl
      ) provider.declarations
    ) == 1
  ) provider.declarations
in

let AnimusSecretSpecProvider = std.contract.Sequence [
  AnimusSecretSpecProviderShape,
  std.contract.from_predicate provider_is_valid,
] in

let AbsolutePath = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.is_match "^/" value
  && !(std.string.is_match "(^|/)\\.\\.(/|$)" value)
) in

let SopsCredentialVariable = {
  credential | SopsCredentialKind,
  variable | SafeLabel,
} in

let EnvironmentInput = {
  kind | std.contract.from_predicate (fun v => v == "environment"),
  variable | SafeLabel,
} in

let SystemdCredentialInput = {
  kind | std.contract.from_predicate (fun v => v == "systemd-credential"),
  directory | AbsolutePath,
} in

let SopsInputShape = {
  kind | std.contract.from_predicate (fun v => v == "sops"),
  guard_directory | AbsolutePath,
  executable | AbsolutePath,
  credential_variables | Array SopsCredentialVariable,
} in

let SopsInput = std.contract.Sequence [
  SopsInputShape,
  std.contract.from_predicate (fun input =>
    std.array.all (fun entry =>
      std.array.length (
        std.array.filter (fun other => other.credential == entry.credential) input.credential_variables
      ) == 1
    ) input.credential_variables
  ),
] in

let HostInput = std.contract.custom (fun label input =>
  if !(std.is_record input) || !(std.record.has_field "kind" input) then
    'Error { message = "host input must be a record with kind" }
  else if input.kind == "environment" then
    'Ok (std.contract.apply EnvironmentInput label input)
  else if input.kind == "systemd-credential" then
    'Ok (std.contract.apply SystemdCredentialInput label input)
  else if input.kind == "sops" then
    'Ok (std.contract.apply SopsInput label input)
  else
    'Error { message = "unsupported host input kind" }
) in

let SecretSpecHost = {
  provider_id | NonEnvId,
  policy_path | AbsolutePath,
  manifest_path | AbsolutePath,
  worker_executable | AbsolutePath,
  working_directory | AbsolutePath,
  input | HostInput,
} in

let hosts_match_providers = fun cfg =>
  let provider_count = std.array.length cfg.credential_providers in
  let host_count = std.array.length cfg.credential_hosts in
  provider_count == host_count
  && std.array.all (fun provider =>
    std.array.length (
      std.array.filter (fun other => other.id == provider.id) cfg.credential_providers
    ) == 1
    && std.array.length (
      std.array.filter (fun host => host.provider_id == provider.id) cfg.credential_hosts
    ) == 1
  ) cfg.credential_providers
in

let CredentialAppConfigShape = {
  data_dir | String,
  default_timeout_secs | Number,
  shell | String,
  credential_providers | Array AnimusSecretSpecProvider,
  credential_hosts | Array SecretSpecHost,
  ..
} in

let CredentialAppConfig = std.contract.Sequence [
  CredentialAppConfigShape,
  std.contract.from_predicate hosts_match_providers,
] in

let AnimusSecretSpecProviderContract = AnimusSecretSpecProvider in
let SecretSpecHostContract = SecretSpecHost in
let RequestBoundsContract = RequestBounds in
let DeclarationMappingContract = DeclarationMapping in
let CredentialAppConfigContract = CredentialAppConfig in

{
  AnimusSecretSpecProvider = AnimusSecretSpecProviderContract,
  SecretSpecHost = SecretSpecHostContract,
  RequestBounds = RequestBoundsContract,
  DeclarationMapping = DeclarationMappingContract,
  CredentialAppConfig = CredentialAppConfigContract,
}
