# Production Checklist

Use this list to sanity check a deployment.

## Schema and Migrations

- Prefer versioned SQL migrations in production.
- Avoid `Premix::sync` for critical databases unless you control the schema.
- Review and test migrations in staging before release.

## Database Configuration

- Set `DATABASE_URL` for the runtime and for the CLI.
- Enable the correct `sqlx` features for your target database.
- Configure pool size and timeouts for your workload.

## Observability

- Enable `tracing` in your application if you want query timing visibility.
- Log slow queries and review them regularly.
- Capture failed queries and migration errors in logs.

## Safety and Consistency

- Use transactions for multi-step writes.
- Consider `version` fields for optimistic locking where needed.
- Use soft delete (`deleted_at`) when you need recovery.

## Verification

- Run `scripts/ci/check_all.ps1` before release.
- Run example apps for critical flows.
- Run `scripts/ci/check_coverage.ps1` if you track coverage trends.

## Operations

- Back up your database before major schema changes.
- Monitor connection pool saturation and query latency.
- Keep release notes and changelog entries aligned.
