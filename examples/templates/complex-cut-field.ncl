let default_delimiter = " " in
let default_field = 1 in
let minimum_field = 1 in
let maximum_field = 20 in
{
  name = "complex-cut-field",
  description = "Extract one delimited field from each input line",
  kind = "shell",
  command = "cut -d '${delimiter}' -f ${field}",
  parameters = [
    {
      name = "delimiter",
      type = "string",
      description = "Single-character field delimiter",
      default = default_delimiter,
      required = false,
    },
    {
      name = "field",
      type = "number",
      description = "One-based field number to extract",
      default = default_field,
      required = false,
      min = minimum_field,
      max = maximum_field,
    },
  ],
  input_ports = [
    { label = "stdin", type_tag = "string" },
  ],
  output_ports = [
    { label = "stdout", type_tag = "string" },
    { label = "stderr", type_tag = "string" },
    { label = "exit",   type_tag = "exitcode" },
  ],
}
