let contracts = import "../../../policy/operation-contracts.ncl" in
(
{
  schema = "lattice.operation-catalog.v2",
  types = [],
  instance_commands = ["forbidden"],
}
) | contracts.Catalog
