# Generated output — do not edit directly

Everything in this directory except this notice is derivative output. Existing file headers and `.cli-flags.toml`, where applicable, identify the current operational generator inputs. Edit repository-owned source inputs and regenerate instead of patching Rust, TypeScript, Dart, schema, or documentation output by hand.

## Contract authority

TypeSpec and JSON Schema/OpenAPI in `fanwaave-interfaces` are independent, human-authored peer authorities for semantic cross-language contracts. Neither may be generated from the other as the ultimate source of truth. A generated contract change is mergeable only when both authorities change together, their normalized outputs agree, and a machine-readable reconciliation receipt is committed under `../contracts/parity/` or linked from the authoritative interfaces repository.

A schema placed below `generated/` is still derivative output and never counts as the human-authored JSON Schema/OpenAPI authority. The generated-policy workflow enforces the local evidence path for future generated changes until cross-repository receipts are machine-verifiable. Existing output predates the complete peer-authority migration and must not be represented as already compliant.

## Service boundary

Generated bindings do not replace the Rust service implementation and do not imply that every process surface is public. Public, internal/server-only, edge-only, and client-visible contracts must remain explicitly separated in the authority sources. Shared Auth, Opto Sync, ores-otel, flags-2-env, and zed-pkg integration boundaries remain unchanged.

## Regenerate and freeze

Before regeneration, temporarily make only this tree writable:

```sh
chmod -R u+w generated
```

Run the documented generator, review the complete diff, then make only this tree read-only again:

```sh
find generated -depth -exec chmod a-w {} +
```

Git persists only the regular-versus-executable distinction, not arbitrary owner-write bits. A fresh checkout can therefore be writable even when a working tree was frozen locally. CI is the durable merge control; `chmod a-w` is an additional local deterrent.