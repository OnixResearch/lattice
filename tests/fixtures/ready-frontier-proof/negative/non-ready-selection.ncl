let Positive = std.contract.from_predicate (fun value => std.is_number value && value > 0) in
let Fixture = {
  total_slots | Positive,
  lane_slots | Positive,
  candidates | Array { node_id | Positive, ready | Bool, conflict | String },
  claimed_selection | Array Number,
} in
{
  total_slots = 2,
  lane_slots = 2,
  candidates = [
    { node_id = 11, ready = true, conflict = "pure" },
    { node_id = 22, ready = false, conflict = "pure" },
  ],
  claimed_selection = [11, 22],
} | Fixture
