# Seed runbook catalog: a catalog-only operation pack that carries two purpose-made
# workflow-template bundles. Every bundle byte is a pinned `workflow_bundle` content item.
{
  schema = "lattice.operation-pack-manifest.v1",
  pack_id = "lattice.seed-runbooks",
  description = "Seed runbook catalog fixture with two reviewed shell runbooks",
  contents = [
    {
      id = "hello-runbook-manifest",
      role = "workflow_bundle",
      path = "workflow-templates/hello-runbook/manifest.ncl",
      content_identity = "b3:6673c392d49ad5896d58828424a4e4620c0ee7a421fdbe3d6b7c82bf2699a35a",
      byte_length = 708,
      media_class = "nickel",
    },
    {
      id = "hello-runbook-workflow",
      role = "workflow_bundle",
      path = "workflow-templates/hello-runbook/workflow.ncl",
      content_identity = "b3:c638275fb5cc6abc49f47750368647f2ec2940c227493c91ed2ff85cf75fa75e",
      byte_length = 1087,
      media_class = "nickel",
    },
    {
      id = "status-runbook-manifest",
      role = "workflow_bundle",
      path = "workflow-templates/status-runbook/manifest.ncl",
      content_identity = "b3:059cc0face34449688974be9a52839c75c19aa14c7bdd655217e71dcf0eec720",
      byte_length = 894,
      media_class = "nickel",
    },
    {
      id = "status-runbook-notes",
      role = "workflow_bundle",
      path = "workflow-templates/status-runbook/assets/notes.txt",
      content_identity = "b3:e3acb563cd78594bb77f0ef5a067ae4358a052aa9f76a15cba87374620ed8e7f",
      byte_length = 86,
      media_class = "text",
    },
    {
      id = "status-runbook-workflow",
      role = "workflow_bundle",
      path = "workflow-templates/status-runbook/workflow.ncl",
      content_identity = "b3:b9d01b44414acbbb212705d6804b56d4199fd920ea60116d7e5d6409541deef8",
      byte_length = 1052,
      media_class = "nickel",
    },
  ],
  workflow_bundles = [
    {
      id = "hello-runbook",
      bundle_identity = "b3:a1dd5bac617f49dfc376ff5125db2fa2db8a49dd4cf8252019d8ea917bd481ba",
    },
    {
      id = "status-runbook",
      bundle_identity = "b3:272ed8525a61a41e51155f4cf84e6b13dd336acba0e41a9db381d0d311148f80",
    },
  ],
  operations = [],
  runtimes = [],
  analyzers = [],
  skills = [],
  dependencies = [],
  platforms = ["aarch64-darwin", "aarch64-linux", "x86_64-darwin", "x86_64-linux"],
  admission = {
    identity = "b3:0f5b2d2c5d1b5f0c4e8e5c3f9a1d7b6e2c4a8f0d3b5e7c9a1f2d4b6c8e0a2c4e",
    required_evidence = [],
    allowed_licenses = ["AGPL-3.0-or-later"],
    allowed_capabilities = [],
  },
  release = {
    display_version = "0.1.0",
    channel = "beta",
    deprecated = false,
  },
  licenses = ["AGPL-3.0-or-later"],
  required_non_claims = [
    "pack identity proves exact admitted content only",
    "pack admission does not prove maintainer or source trust",
    "bounded process execution does not prove sandbox completeness",
    "pack evidence does not prove production or release readiness",
  ],
}
