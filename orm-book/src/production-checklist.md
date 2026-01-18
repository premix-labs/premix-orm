# Production Checklist

Use this list to sanity check a deployment.

## Schema and Migrations

- Prefer versioned SQL migrations in production.
- Avoid `Premix::sync` for critical databases unless you control the schema.

## Database Configuration

- Set `DATABASE_URL` for the runtime and for the CLI.
- Enable the correct `sqlx` features for your target database.

## Observability

- Enable `tracing` in your application if you want query timing visibility.
- Log slow queries and review them regularly.

## Safety and Consistency

- Use transactions for multi-step writes.
- Consider `version` fields for optimistic locking where needed.
- Use soft delete (`deleted_at`) when you need recovery.

## Verification

- Run `scripts/ci/check_all.ps1` before release.
- Run example apps for critical flows.
