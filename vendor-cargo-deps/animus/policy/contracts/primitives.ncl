# Shared authoring-time policy contracts for Animus.
# r[impl agent_harness_runtime.nickel_policy_contracts.shared_primitives]
#
# These contracts reject malformed reviewed source before projection. Runtime
# admission, effects, external artifact validation, and receipt authority stay
# in Rust and host shells.

let _stable_validator = fun issue_code predicate =>
  std.contract.from_validator (fun value =>
    if predicate value then
      'Ok
    else
      'Error { message = issue_code }
  )
in

let _count_equal = fun needle values =>
  std.array.fold_left
    (fun count value => if value == needle then count + 1 else count)
    0
    values
in

let _all_unique = fun values =>
  std.array.all (fun value => _count_equal value values == 1) values
in

let _unique_by = fun project values =>
  _all_unique (std.array.map project values)
in

let _starts_with = fun prefix value =>
  std.is_string value
  && std.is_string prefix
  && std.string.length value >= std.string.length prefix
  && std.string.substring 0 (std.string.length prefix) value == prefix
in

let _path_segments_are_safe = fun value =>
  value
  |> std.string.split "/"
  |> std.array.all (fun segment => segment != "" && segment != "." && segment != "..")
in

let _is_safe_repository_relative_path = fun value =>
  std.is_string value
  && std.string.length value > 0
  && std.string.substring 0 1 value != "/"
  && !std.string.contains "\\" value
  && std.string.is_match "^[A-Za-z0-9.][A-Za-z0-9._/-]*$" value
  && _path_segments_are_safe value
in

let _is_safe_absolute_policy_path = fun value =>
  std.is_string value
  && std.string.length value > 1
  && std.string.substring 0 1 value == "/"
  && !std.string.contains "\\" value
  && _path_segments_are_safe (std.string.substring 1 (std.string.length value - 1) value)
in

let _is_integer_between = fun minimum maximum value =>
  std.is_number value
  && value % 1 == 0
  && value >= minimum
  && value <= maximum
in

let _with_validator = fun shape validator =>
  std.contract.custom (fun label value =>
    let checked = value | shape in
    let result = validator checked in
    if result == 'Ok then
      'Ok checked
    else
      result
  )
in

let _exact = fun issue_code expected =>
  _stable_validator issue_code (fun value => value == expected)
in

let _integer_between = fun issue_code minimum maximum =>
  _stable_validator issue_code (_is_integer_between minimum maximum)
in

let _non_negative_integer = fun issue_code maximum =>
  _integer_between issue_code 0 maximum
in

let _positive_integer = fun issue_code maximum =>
  _integer_between issue_code 1 maximum
in

let _enum = fun issue_code allowed =>
  _stable_validator issue_code (fun value =>
    std.is_string value && std.array.elem value allowed
  )
in

let _non_empty_string = fun issue_code =>
  _stable_validator issue_code (fun value =>
    std.is_string value && std.string.length value > 0
  )
in

let _safe_identifier = fun issue_code =>
  _stable_validator issue_code (fun value =>
    std.is_string value
    && std.string.is_match "^[a-z0-9][a-z0-9._-]*$" value
  )
in

let _optional_safe_identifier = fun issue_code =>
  _stable_validator issue_code (fun value =>
    std.is_string value
    && (
      value == ""
      || std.string.is_match "^[a-z0-9][a-z0-9._-]*$" value
    )
  )
in

let _lowercase_blake3 = fun issue_code =>
  _stable_validator issue_code (fun value =>
    std.is_string value && std.string.is_match "^[0-9a-f]{64}$" value
  )
in

let _safe_repository_relative_path = fun issue_code =>
  _stable_validator issue_code _is_safe_repository_relative_path
in

let _safe_absolute_policy_path = fun issue_code =>
  _stable_validator issue_code _is_safe_absolute_policy_path
in

{
  stable_validator = _stable_validator,
  with_validator = _with_validator,
  exact = _exact,
  integer_between = _integer_between,
  non_negative_integer = _non_negative_integer,
  positive_integer = _positive_integer,
  enum = _enum,
  non_empty_string = _non_empty_string,
  safe_identifier = _safe_identifier,
  optional_safe_identifier = _optional_safe_identifier,
  lowercase_blake3 = _lowercase_blake3,
  safe_repository_relative_path = _safe_repository_relative_path,
  safe_absolute_policy_path = _safe_absolute_policy_path,
  all_unique = _all_unique,
  unique_by = _unique_by,
  count_equal = _count_equal,
  starts_with = _starts_with,
  is_integer_between = _is_integer_between,
  is_safe_repository_relative_path = _is_safe_repository_relative_path,
  is_safe_absolute_policy_path = _is_safe_absolute_policy_path,
}
