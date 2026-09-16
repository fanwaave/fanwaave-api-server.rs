# Fanwaave contact worker parity

Tracks fanwaave-api-server.rs#18 and the behavior of ORESoftware/k8s-cluster `dd-email-sms-contact-rs`.

## Required transports

- Email: SendGrid primary with a provider abstraction that permits SES.
- SMS: Twilio.
- NATS command lanes use queue groups so one replica claims each command.
- Results are published on a dedicated result subject.
- HTTP and NATS auth fail closed when their configured secret is required.

## Durable job lifecycle

`queued -> claimed -> processing -> delivered`

Failure transitions are `processing -> queued` for retryable failures and `processing -> failed -> dead_letter` when attempts are exhausted or the failure is permanent. Jobs may also become `cancelled` before delivery. An expired claim/lease returns to the claimable set.

Each persisted job must include a stable job id, channel, redacted target metadata, payload reference/body, idempotency key, status, priority, attempt count, max attempts, `available_at`, claim/lease owner and expiry, provider message id, bounded/redacted last error, and created/updated/completed timestamps.

The database is the durable authority. NATS is a wake-up/distribution mechanism, not the sole source of truth.

## Worker semantics

Workers claim jobs atomically, use bounded concurrency, renew or bound leases, recover expired leases, classify provider errors, and retry transient errors with exponential backoff plus jitter. Provider 429 and 5xx responses and timeouts are retryable unless provider-specific evidence says otherwise. Invalid destinations/auth/configuration are permanent failures. Exhausted jobs enter a DLQ and can be explicitly replayed without losing the original audit trail.

Idempotency must prevent duplicate logical sends across HTTP retries, NATS redelivery, process restarts, and competing replicas.

## Configuration and secrets

`.fanwaave-cfg.toml` owns non-secret provider/worker/queue policy: enabled transports, provider selection, concurrency, lease duration, max attempts, retry/backoff bounds, rate limits, NATS subjects/queue group, Redis integration and readiness policy.

Provider credentials and auth secrets MUST NOT be committed to TOML. Resolve them from deployment secrets / sops+age env/enc.

## Hardening

- Lock email sender identity to configured verified senders.
- Validate recipient, subject and body shapes and bound request sizes.
- Bound and redact upstream error text.
- Never log credentials, full SMS destinations, message bodies, or provider tokens.
- Apply per-channel rate limiting. When replicas >1, global limits must use shared state (Redis or equivalent), not per-process buckets.
- Health/readiness report provider, database, queue and worker readiness without exposing secrets.
- Prefer canonical lowercase `x-ores-*` extension headers in authored contracts while accepting incoming HTTP header names case-insensitively.

## Observability

Instrument through ores-otel/OpenTelemetry: enqueue latency, queue depth, oldest-job age, claim latency, active workers, attempts, retry reasons, provider latency/status, delivered/failed counters, expired leases and DLQ depth/replay.

## Ownership

- `fanwaave-interfaces`: public request/result/job contracts and RPC definitions.
- `fanwaave-lib-core`: reusable contact, retry, idempotency and worker-domain logic.
- `fanwaave-orm-core`: SeaORM entities/repositories/migrations for durable jobs and attempts.
- `fanwaave-api-server.rs`: authenticated HTTP/RPC/NATS adapters and worker process wiring.

## Acceptance tests

Cover idempotent enqueue, duplicate NATS delivery, two competing workers, atomic claims, expired leases, retry/backoff, 429/5xx/timeouts, permanent provider errors, attempt exhaustion to DLQ, replay, cancellation, auth failure, input bounds, graceful shutdown, readiness degradation, redaction, and multi-replica shared rate limiting.
