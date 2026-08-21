# Explicit workflow expression dependencies

<!-- r[impl lattice.workflow_validation.explicit_dependencies.evidence] -->

Lattice expressions can read one output port from another workflow node. Lattice now derives these references before graph admission.

Choregraph remains the owner of graph admission, cycle rejection, order, readiness, lifecycle, and replay. Lattice owns the source-field inventory and dependency projection.

## Supported fields

The checked inventory includes these fields:

- `shell.command`
- `shell.cwd`
- `shell.env[<name>]`
- `http.method`
- `http.url`
- `http.headers[<name>]`
- `http.body`
- `file.path`
- `file.content`
- `file.allowed_roots[<index>]`
- `image_input.path`
- `image_input.allowed_roots[<index>]`
- `agent.prompt`
- `agent.model_hint`
- `agent.tool_policy`
- `agent.context_refs[<index>]`
- `agent.cwd`
- `agent.session`

The `NodeConfig` match has no catch-all pattern. Thus, a new config variant or field causes a compile error until the inventory classifies it.

`agent.prompt_file` does not support expression syntax in its path. The loaded prompt content uses `agent.prompt` after the shell reads the file.

Typed operation plans do not evaluate Lattice expressions. Their arguments remain identity-bound operation data.

## Dependency projection

The pure projector extracts these facts for each reference:

```text
ExpressionDependency {
  consumer_node,
  config_field_path,
  producer_node,
  producer_output_port,
}
```

The projector rejects these conditions:

- invalid expression syntax
- a missing producer node
- a missing producer output port
- a self-reference
- expression syntax in a classified unsupported field

Duplicate references in one field become one fact. References in different fields retain their field paths.

The execution graph uses the union of normal graph edges and expression dependencies. Equal producer-consumer pairs become one execution dependency.

Lattice gives the complete union to Choregraph. It does not add a second cycle or readiness algorithm.

## Runtime context

For each consumer, the executor builds a context from its admitted reference facts. The context excludes outputs from unrelated completed nodes.

If an admitted value is unavailable, preparation fails before credential resolution, attempt creation, or an external effect.

The cache key for an expression consumer includes the exact producer node, output port, and value identity. Cache keys without expression values retain their prior framing.

## Compatibility

Portable workflow source does not gain derived edges or ports. Export and import preserve the source fields and bytes.

The Choregraph adapter adds deterministic internal input ports only to its evaluated projection. These ports represent expression-only producer-consumer pairs.

A workflow with a new derived pair receives a new Choregraph machine identity. Its runner-plan identity also changes.

The checked identity fixture records this disposition:

| Projection | Runner-plan identity | Choregraph machine identity |
|---|---|---|
| Two independent nodes | `blake3:75effcf83d3c2e3bc0fbdcd025b9dadacf1f1e79fb664c0b56e4768993f7c8e8` | `47fcaf97770cc65b0fb954690eff5f35599f08474a99cecd0554b894e37478d6` |
| One expression dependency | `blake3:a0469c4f953a465715c787b572835795c664f3b846e8dcd56cfed0b8ba952a1c` | `dfed62b3fcc0efba5dc6bd43aaadd293d3e00b2c48a4ed3229168b6938043c78` |

A stored machine or runner plan with another dependency set is stale. Lattice rejects it instead of using incidental serial order.

## Boundaries

This projection establishes supported data dependencies. It does not establish these properties:

- external effect commutativity
- provider success or truth
- arbitrary expression semantics
- runtime progress
- workflow correctness
- release eligibility
