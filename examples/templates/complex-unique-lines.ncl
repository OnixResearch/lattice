let default_unique_flags = "" in
{
  name = "complex-unique-lines",
  description = "Collapse adjacent duplicate lines from stdin with optional uniq flags",
  kind = "shell",
  command = "uniq ${flags}",
  parameters = [
    {
      name = "flags",
      type = "string",
      description = "Optional uniq flags, for example -c",
      default = default_unique_flags,
      required = false,
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
