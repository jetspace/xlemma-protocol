# Deterministic bundle vector

This directory is a cross-implementation XLMP/1 storage vector. A conforming
bundle builder supplied with `inputs.json`, the files in this directory, and
the exact parameters below must produce the same object and byte-identical RFC
8785 canonical JSON as `expected-bundle.json`:

```text
lean_toolchain       = leanprover/lean4:v4.33.1
dependency_lock_hash = blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
source_commit        = vector-1
build_image_digest   = sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
created_at           = 2026-09-04T12:00:00Z
```

Entries are sorted by safe normalized relative path. `created_at` and
`source_commit` remain in the complete manifest but are intentionally excluded
from `ArtifactID`; file content, paths, media types, encryption flags,
toolchain, dependency lock, and build image remain identity-critical. The
artifact root itself binds path, media type, content hash, byte length, and
encryption status for each sorted entry.
