# Server phase placeholder

This directory is a legacy planning placeholder; its plan, task, acceptance and completion files are empty. It does not define an accepted server phase or indicate that all server code is unimplemented.

The current server lives in [`crates/server`](../../../../crates/server/) and is launched by `qai serve`. It provides a loopback-only Quran read API, normalization preview/profile routes, health endpoints and a labelled debug reader. Production multi-user deployment, authentication/RBAC and a full Web GUI remain future work.

Phase numbering differs between the PRD roadmaps and the directory tree; this README does not resolve or renumber those plans. See the [project README](../../../../README.md#local-api) for current usage and limitations.
