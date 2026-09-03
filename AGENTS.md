# Fanwaave — api-server.rs

Canonical `api-server.rs` repository for [`fanwaave`](https://github.com/fanwaave).

- Internal runtimes: Rust, TypeScript, Dart.
- Contracts: TypeSpec and JSON Schema/OpenAPI are independent, human-authored, top-level peer authorities in `fanwaave-interfaces`. Neither is generated from the other as the ultimate source of truth. Normalize and compare their independently generated outputs, fail closed on discrepancies, and commit a machine-readable parity receipt before derivative interfaces or wire clients change.
- Contract surfaces: keep public/client, private/server-only, edge-only, and isomorphic definitions explicitly separated in the authority sources and generated output.
- Auth: github.com/shared-auth.
- Sync: github.com/opto-sync.
- Telemetry: github.com/ores-otel.
- Flags: github.com/flags-2-env.
- Packages: github.com/zed-pkg.
- Never use React/JSX or webviews.
- Never place credentials, raw tokens, private identity payloads, or secret values in generated fixtures, logs, or source control.
- Resolve git conflicts semantically; never rebase, stash, or reset.
