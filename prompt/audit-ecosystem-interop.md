<!-- audit-ecosystem-interop.md -->

Role: Rust Ecosystem Integrator & Library Maintainer.

Objective: Ensure Premix ORM integrates seamlessly with the standard Rust cloud native stack (Tokio, Serde, Tracing, Chrono/Time, Uuid).

Instructions for Agent:

1. 📦 Serialization Compatibility (Serde)
   Audit the `#[derive(Model)]` macro interaction with `serde`.

   Scenarios:
   - Can a Model be returned directly as JSON from an Axum handler? (`impl Serialize`)
   - Can a Model be deserialized from a request body? (`impl Deserialize`)
   - Field Renaming: Does `#[premix(rename = "foo")]` conflict with `#[serde(rename = "bar")]`?
   - Goal: Users should not have to create a separate DTO (Data Transfer Object) for simple CRUD.

2. 👁️ Observability & Logging (Tracing)
   Audit the instrumentation of queries.

   Check:
   - Do generated queries emit `tracing::info!` or `tracing::debug!` events?
   - Privacy: Ensure values are not logged by default (only SQL templates), or adhere to the `#[premix(sensitive)]` flag.
   - Correlation IDs: Can the logs be traced back to the parent request span?

3. 🕰️ Common Type Interop
   Verify strict type mapping for ecosystem standards.
   - UUID: Does it support `uuid::Uuid` crate natively?
   - Time: Does it support `chrono` or `time` crates?
   - JSON: Does it map database JSON types to `serde_json::Value` automatically?

4. 🚀 Async Runtime Agnosticism
   Check for hidden runtime dependencies.
   - Does the library accidentally depend on `tokio::spawn`?
   - Compatibility: Can it run on `async-std` if the user chooses? (Or explicitly state Tokio-only dependency).

Reporting Format:

- Friction Points: Where the user has to manually convert types (e.g., String -> Uuid).
- Logging Gaps: Critical operations (connections, heavy queries) that are silent in logs.
- Serde Conflicts: Issues where macro attributes clash.
