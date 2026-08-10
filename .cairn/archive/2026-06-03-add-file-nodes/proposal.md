## Why

Local automation often starts or ends with files. First-class file nodes let workflows read input files, write generated output, and append logs without relying on ad hoc shell commands for every file operation.

## What Changes

- Add read, write, and append file node configurations.
- Execute file operations through a filesystem interface.
- Validate paths against configured safety policy before side effects.
- Expose file contents, bytes written, and errors as typed node outputs.
