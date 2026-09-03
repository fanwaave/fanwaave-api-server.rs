# Generated output — do not edit directly

Everything in this directory except this notice is derivative output. Change the human-authored source, run the owning generator, and review the complete regenerated diff. Do not patch a language-specific artifact by hand.

## Current source map

The committed Dart, Gleam, Rust, and TypeScript `env` and `runtime` files are configuration projections generated from the repository-root `.cli-flags.toml` catalog by the canonical `flags-2-env/flags-2-env` toolchain. `.cli-flags.toml` remains the authority for those process settings.

A CLI/environment catalog is not an application-domain, synchronization, identity, IPC, or network protocol. It therefore does **not** require a duplicate TypeSpec model merely to satisfy a file-layout rule. The generated-policy workflow requires `.cli-flags.toml` to change whenever these projections change.

## Semantic cross-language contracts

For generated Fanwaave domain models, sync messages, persisted interchange, local IPC, or remote wire types, TypeSpec and JSON Schema/OpenAPI are independent, human-authored peer authorities in `fanwaave-interfaces`. Neither may be generated from the other as the ultimate source of truth. Their independently generated normalized outputs must agree, and a machine-readable reconciliation receipt must accompany derivative output changes.

A TypeSpec, JSON Schema, or OpenAPI document placed below `generated/` is derivative output and never satisfies the human-authored peer-authority requirement. Public/client, private/server-only, edge-only, and isomorphic surfaces must remain explicitly separated.

## Service boundary

These configuration projections do not define the API surface. The Rust service, `fanwaave-interfaces`, `fanwaave-lib-core`, Shared Auth, and Opto Sync own their respective runtime, contract, authorization, and synchronization responsibilities. Generated code does not justify React/JSX or a webview in this service.

Never place credentials, raw tokens, private identity payloads, authorization headers, or secret values in generated artifacts or fixtures.

## Regenerate and freeze

Temporarily thaw only this tree before running the documented generator:

```sh
chmod -R u+w generated
```

After regeneration and review, freeze it locally:

```sh
find generated -depth -exec chmod a-w {} +
```

Git stores the regular-versus-executable bit, not arbitrary owner-write bits. A fresh checkout therefore restores ordinary files as writable. CI, provenance checks, source/output co-change rules, deterministic regeneration, and contract parity are the durable controls; local `chmod a-w` is an additional deterrent.
