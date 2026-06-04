let default_sort_flags = "" in
{
  name = "complex-sort-lines",
  description = "Sort text from stdin with optional sort flags",
  kind = "shell",
  command = "sort ${flags}",
  parameters = [
    {
      name = "flags",
      type = "string",
      description = "Optional sort flags, for example -r or -n",
      default = default_sort_flags,
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
