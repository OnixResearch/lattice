let SafeRoot = std.contract.from_predicate (fun value =>
  std.is_string value
  && std.string.length value > 0
  && !std.string.contains "\\" value
  && std.array.all
    (fun component => component != "" && component != "." && component != "..")
    (std.string.split "/" value)
) in
let Source = { root | SafeRoot, .. } in
{
  Policy = {
    profiles | Array {
      requirement_sources | Array Source,
      evidence_sources | Array Source,
      ..
    },
    ..
  },
}
