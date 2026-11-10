# howdah

Typed Postgres queries for Rust, modeled on JetBrains Exposed.

> **Status: pre-alpha placeholder.** This release reserves the crate name. It
> contains no query API yet.

howdah is a Postgres-only, typed SQL query library built on `tokio-postgres` with both
fluent and imperative APIs.

## Design

Goals:

- Compile-time checking of value types, nullability and projection types.
- Ergonomics past basic examples: conditional filters and joins, PATCH-style
  updates, reusable query fragments as plain functions.
- Direct access to Postgres features: `RETURNING`, `ON CONFLICT`,
  `DISTINCT ON`, row comparison, jsonb and CTEs.

Non-goals: other databases, Diesel-style compile-time query-shape checking, a
DAO/active-record layer, migration generation, a custom wire-protocol driver.

## License

Licensed under the [MIT license](LICENSE-MIT).
