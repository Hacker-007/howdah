# howdah: Typed Postgres Queries for Rust — Design

Nov 22, 2026

## Summary

howdah is a Postgres-only, typed SQL query library for Rust. Value types and nullability are checked at compile time. Which tables a column may be used with is checked when the statement is rendered, before anything is sent. It runs on tokio-postgres.

A query is an ordinary value. Adding a table to it returns a **handle**, a small `Copy` value that stands for that one use of the table and gives its columns. Queries and handles have no lifetimes, so they can be stored in structs, returned from functions, and built up across calls.

### Goals

- Compile-time checking of value types, nullability (including outer joins and SQL's three-valued logic) and row types.
- Queries that stay easy past basic examples: conditional filters and joins, PATCH-style updates, and reusable pieces as plain functions and structs.
- Direct access to Postgres features: RETURNING, ON CONFLICT, DISTINCT ON, row locking, CTEs and jsonb.
- Every macro's output can be written by hand.

### Non-goals

- Databases other than Postgres.
- Compile-time checking of query shape, as Diesel does it.
- An active-record layer or relation loading.
- Migrations or schema diffing. Migrations stay plain SQL.
- A custom wire-protocol driver.

## Background

Rust data layers tend to check too much at the type level or too little. Diesel encodes query shape in types, which brings boxed queries for dynamic filters and very long errors. sqlx checks SQL strings against a live database but offers no composition. SeaORM checks little around joins and partial selects. Exposed (Kotlin) checks value types and leaves query shape untyped, which is the trade howdah copies. howdah adds typed result rows and outer-join nullability on top.

Targeting only Postgres lets howdah render one dialect and expose Postgres features directly.

## Core model

| Concept | Type | Role |
| --- | --- | --- |
| Table | `Users` (marker, `Copy`) | Names a table; passed to `from`, `join`, `insert_into` |
| Shape | `Biggest` (row struct) | The columns of a query used as a table |
| Handle | `Handle<T, N>` | One use of a table or shape in a statement |
| Column | `Column<T, N>` | A handle's column, typed by its Rust type `T` and nullability `N` |
| Expression | `Expr<T, N>` | An untyped `Node` tree with phantom `T` and `N` |
| Query | `Query` | A select being built |
| Statement | `Select<R>`, `Insert<T>`, `Update<T>`, `Delete<T>`, `Returning<R>` | What gets rendered and run; `R` is the row type |

All type rules live in the signatures of expression constructors. The renderer sees only the untyped tree.

### Handles

A handle carries a process-wide unique id (from one atomic counter) and its table type. It never carries an alias. Aliases are assigned when the statement is rendered.

- `from`, `join`, `left_join` and `cross_join` return a new handle each time, so the same table joined twice gives two handles and needs no alias.
- `left_join` returns `Handle<T, Nullable>`. Every column read through it is nullable, without double-wrapping columns that already are.
- Cloning a query keeps its handles valid in the clone. A join added to one clone is not in the other.
- Handles are `Copy + Send + Sync + 'static`. Queries are `Clone + Send + Sync + 'static`.
- `Handle` has no methods of its own, so it cannot clash with a column name.

### Schema

`#[derive(Table)]` on a row struct generates the marker, the `Shape` and `Table` impls, and a trait with one method per column, implemented for every handle of the table:

```rust
#[derive(Table)]
#[table(name = "users", marker = Users)]
pub struct User {
    #[col(default)] pub id: i64,
    pub name: String,
    pub email: String,
    #[col(default)] pub active: bool,
    pub manager_id: Option<i64>,
}

// generated, and writable by hand:
pub trait UsersColumns {
    type Null: Nullability;
    fn id(&self) -> Column<i64, Self::Null>;           // as nullable as the handle
    fn manager_id(&self) -> Column<i64, Nullable>;    // always nullable
    // ...
}
impl<N: Nullability> UsersColumns for Handle<Users, N> {
    type Null = N;
    // ...
}
```

`Shape` holds the column list (`&'static [ColumnDef]` with name, nullability and default), the row type, and the column types as a tuple. `Table` adds the table name. `#[derive(Shape)]` on a row struct does the same for a query used as a table, without a name.

### Type rules

These carry over unchanged from the current crate.

- **Nullability** is a separate parameter: `Expr<T, N>` with `N` either `NeverNull` or `Nullable`. A row reads `T` or `Option<T>`. This makes `Option<Option<T>>` impossible.
- **Comparisons** require operands of the same `T`. The result is nullable if either side is, following three-valued logic. Comparisons are methods (`eq`, `ne`, `lt`, `gt`, `like`, `ilike`, `between`, `is_in`, ...), because `PartialEq` must return `bool`.
- **Operators.** `&`, `|` and `!` work on boolean expressions; `+ - * / %` on numbers, as the `Additive`, `Numeric` and `Remainder` marker traits allow.
- **Filters** and ON conditions accept `bool` of either nullability.
- **Aggregates** follow Postgres: `count` is `i64`, `sum(i32)` is `Option<i64>`, `sum(i64)` and `avg` of integers are `Option<Decimal>`, `min` and `max` work only on `Ordered` types.
- **Literals** always become `$n` parameters. An integer literal takes the type the other operand needs.
- **Assignments.** `column.set(value)` accepts a nullable value only for a nullable column.

```rust
let a: Expr<bool>           = u.id().eq(1);
let b: Expr<bool, Nullable> = m.id().eq(u.manager_id());
let c: Expr<bool, Nullable> = a & b;
```

## Selects

`from(source)` returns the query and the handle of its first table, so every query has a FROM.

```rust
let (mut q, u) = from(Users);
let m = q.left_join(Users, |m| m.id().eq(u.manager_id()));
q.filter(u.active());
if let Some(term) = &search {
    q.filter(u.name().ilike(format!("%{term}%")));
}
q.order_by(u.name().asc()).limit(50);

let rows: Vec<(String, Option<String>)> =
    q.select((u.name(), m.name())).fetch_all(&db).await?;
```

- Builder methods take `&mut self` and return `&mut Self`, so they chain and also work inside `if` and loops. The methods are `join`, `left_join`, `cross_join`, `filter`, `group_by`, `having`, `order_by`, `limit`, `offset`, `distinct`, `distinct_on`, `for_update` and `for_share`.
- A join's `on` closure gets the new handle. It may also use any handle already in the query.
- `q.select(selection)` takes `&self` and returns a `Select<R>` snapshot. The query stays usable, so one query gives both a page and its count.
- A selection is a column, an expression, a handle, or a tuple of them. A handle selects the whole row: `T::Row`, or `Option<T::Row>` through a nullable handle.
- `Select<R>` has no lifetime and no handles inside its type. It can be stored, cloned and run.

### Reuse

Reusable pieces are plain functions and structs. Nothing in their signatures mentions lifetimes.

```rust
fn visible<N: Nullability>(u: Handle<Users, N>) -> Expr<bool, N> {
    u.active().eq(true) & u.email().ilike("%@example.com")
}

fn active_users() -> (Query, Handle<Users>) {
    let (mut q, u) = from(Users);
    q.filter(u.active());
    (q, u)
}

struct UserSearch { q: Query, u: Handle<Users> }

impl UserSearch {
    fn named(mut self, term: &str) -> Self {
        self.q.filter(self.u.name().ilike(format!("%{term}%")));
        self
    }
    fn count(&self) -> Select<i64> { self.q.select(count_all()) }
    fn page(&self) -> Select<User> {
        self.q.clone().order_by(self.u.name().asc()).select(self.u)
    }
}
```

### Subqueries

A subquery is another `Query` or `Select`. It may use the handles of every query around the place it ends up.

```rust
let (mut big, o) = from(Orders);
big.filter(o.user_id().eq(u.id())).filter(o.amount_cents().gt(5_000));
q.filter(exists(big));                          // EXISTS (SELECT 1 ...)
```

- `exists(query)` takes the query by value, so it cannot be changed after it is used.
- `expr.in_query(select)` requires the select's row type to match `expr`'s type.
- `select.scalar()` is `(SELECT ...)` as a nullable value.

### Queries as tables

`select.as_table::<D>()` turns a select into a table with the columns of the shape `D`. The select's row type must equal `D`'s column types, in order. `from`, `join` and `left_join` take it like a table.

```rust
#[derive(Shape)]
struct Biggest { user_id: i64, biggest: Option<i64> }

let (mut per_user, o) = from(Orders);
per_user.group_by(o.user_id());
let biggest = per_user.select((o.user_id(), max(o.amount_cents()))).as_table::<Biggest>();

let (mut q, u) = from(Users);
let b = q.join(biggest, |b| b.user_id().eq(u.id()));
q.select((u.name(), b.biggest()))
// ... JOIN (SELECT ...) AS "biggest"("user_id", "biggest") ON ...
```

`select.as_cte::<D>()` returns a `Cte<D>`, taken by reference: `from(&active)`, `q.join(&active, ..)`. Each use gets its own handle. The statement that uses it gets one `WITH` entry, however often it is used, because clones of a `Cte` share its body. Every statement kind can use CTEs.

## Writes

`insert_into`, `update` and `delete_from` take a table and return the statement. `target()` returns the handle of the table being written; calling it again returns the same handle.

### Insert

```rust
let mut ins = insert_into(Users);
let u = ins.target();
ins.value(u.name(), "Ada").value(u.email(), email);
ins.on_conflict(u.email()).do_update(|excluded| [u.name().set(excluded.name())]);

let id: i64 = ins.returning(u.id()).fetch_one(&db).await?;
```

- `value(column, v)` builds one row. `values(columns, rows)` gives many rows for a tuple of columns, with each row's tuple type checked against them. `values_from(columns, select)` renders INSERT ... SELECT, with the select's row type checked the same way.
- Every column that has no default and cannot be null needs a value. This is checked at render time from `ColumnDef`.
- `on_conflict(columns)` returns a small builder with `do_nothing()` and `do_update(|excluded| assignments)`. `on_conflict_do_nothing()` takes no target. The `excluded` row is a handle, so its columns are typed.

### Update and delete

```rust
let mut up = update(Users);
let u = up.target();
up.filter(u.id().eq(id));
if let Some(name) = patch.name { up.set(u.name(), name); }
if let Some(active) = patch.active { up.set(u.active(), active); }
up.execute(&db).await?;
```

- `up.from(table)` adds an UPDATE ... FROM item and returns its handle. Its join condition goes in `filter`.
- `delete_from(Orders)` mirrors this. `del.using(table)` adds a DELETE ... USING item.

### RETURNING

`returning(selection)` takes `&self` on any write and returns a `Returning<R>` snapshot, like `select`. The write itself runs with `execute`, which returns rows affected.

## Rendering

The renderer writes SQL and checks scope in the same pass. Each statement renders on its own, from scratch.

### Aliases

Each table use is aliased by its table name: the first `users` is `"users"`, the next `"users_2"`, and so on. A shape uses its snake-case type name, and an alias already taken in the statement is skipped. Aliases are unique across the whole statement, subqueries included, so a correlated reference never depends on shadowing.

The SQL text depends only on the statement's shape, never on handle ids. The statement cache stays effective.

### Scope rules

A column may be used only where its handle is in scope:

1. In a query's select list, WHERE, GROUP BY, HAVING, ORDER BY and DISTINCT ON: that query's FROM items, and those of every query around it.
2. In a join's ON: the FROM items up to and including that join, and those of the queries around it.
3. In a subquery inside an expression (EXISTS, IN, scalar): its own FROM items, then outward as in rule 1.
4. In a query used as a table: its own FROM items and the queries around the one it is joined into, but not the items beside it, since there is no LATERAL. A CTE body sees only its own items.
5. In writes: the target and FROM or USING items for filters, SET values and RETURNING. VALUES sees no table. DO UPDATE sees the target and `excluded`.
6. A handle may appear only once along any path of nested queries. Nesting a query inside a copy of itself makes its columns ambiguous and is an error. Sibling subqueries made from clones are fine.

### Render errors

`to_sql()` returns `Result<(String, Vec<Value>), Error>`, and every executor terminal calls it first, so a render error means nothing was sent.

| Error | Cause |
| --- | --- |
| `NotInScope { table, column }` | A column used where its handle is not in scope: another query's handle, a subquery's handle used outside it, a sibling read from a query used as a table, `excluded` outside DO UPDATE, or a join used in an earlier ON |
| `HandleReused { table }` | Rule 6 |
| `NotTarget { table, column }` | A written column, or conflict target, from a handle other than the write's target |
| `DuplicateColumn`, `MissingColumn` | A column set twice; a required insert column without a value |
| `MixedInsertRows` | `value`, `values` and `values_from` combined on one insert |
| `EmptyUpdate`, `EmptyInsert` | No assignments; an empty `values` list |
| `NullComparison` | `eq(None)` or `ne(None)`, which is never true |
| `RawArguments` | `{}` placeholders and arguments that differ in number |
| `Encode` | A value that could not become a parameter, such as unserializable JSON |

`DuplicateTable`, `NotNullable` and `NullableView` from the current crate are gone. Handles replace alias strings, and the left-join handle replaces nullable views.

Scope errors depend only on how the statement was built, never on data. A test that renders each code path that builds a statement catches them.

## What is checked where

| When | What |
| --- | --- |
| Compile time | Value types, nullability of every column, expression and row, row types of selects, subqueries and queries used as tables, value types of insert rows and assignments, operator and function argument types |
| Render time | Scope (rules 1 to 6), written columns belonging to the target, required insert columns, duplicate columns, empty writes, null comparisons, raw SQL arguments, value encoding |
| Execution | Decoding, Postgres errors |

## Raw SQL

`sql::<T>("lower({}) || {}", [u.name().into(), "x".into()])` is a typed expression made from SQL text, with `{}` slots filled by columns, expressions or values. It cannot be null by its type; `.nullable()` changes that. `query_as::<R>("...").bind(v)` is a whole raw statement with a declared row type. Both require the caller to state the result type, so raw SQL never produces an untyped value. Handles inside `sql` follow the same scope rules.

## Execution

Execution is a thin layer over tokio-postgres and deadpool-postgres, behind the opt-in `executor` feature in `howdah::exec`. It carries over from the current crate.

- One `Executor` trait, implemented by `Database` (a pool), `Connection` and `Tx`.
- Terminals: `fetch_all`, `fetch_one` (an error unless exactly one row), `fetch_optional` and `stream` on `Select<R>`, `Returning<R>` and raw queries; `execute` (rows affected) on writes and raw queries.
- Each connection caches prepared statements by SQL text. `DatabaseBuilder::statement_cache(false)` turns it off for PgBouncer in transaction mode.
- `db.transaction(async |tx| { .. })` commits on `Ok` and rolls back on `Err` or panic. `transaction` on a `Tx` is a savepoint. The closure's error type is any `E: From<exec::Error>`.
- `exec::Error` has `Render`, `Postgres`, `Pool`, `Build` and `Decode` variants.

Known limits: TLS is not supported yet (`NoTls` only), and null parameters are sent untyped.

## Testing

- Rendering tests assert the SQL text and parameters for every builder feature, and that each render error is reported.
- `compile_fail` doctests pin each compile-time rule, with the expected error code.
- The repository has no database tests. Behavior against Postgres is checked in a scratch crate against a Postgres 17 container.
- `cargo clippy` with each feature combination, `cargo fmt` and `cargo doc` stay clean.

## Alternatives considered

- **Alias strings with nullable views** (the current crate). Columns name their table by alias, and left joins need `.nullable()`. Handles remove the alias strings, `DuplicateTable`, `NullableSource` and `NotNullable`, and keep the same render-time checks.
- **A closure per query that brands handles with a lifetime.** This moves scope errors to compile time. A prototype showed it costs lifetimes in every helper, forbids storing or extending an unfinished query, and gives lifetime-flavored errors. It also let the `excluded` row leak into VALUES and rejected a valid derived table that reads an outer query. Fixing either needs more lifetimes.
- **Type-level table lists.** These encode the query's tables in its type. Conditional joins change the type, and errors grow with the number of joins.
- **sea-query as the rendering backend.** It covers the Postgres features howdah needs and could still replace the renderer later, since the tree is untyped. It is not used now, to keep dependencies to the approved list.

## Open questions

- [ ] Whole-row select through a nullable handle: when is the row `None`? The proposal is that it is `None` when a NOT NULL column is null, and that a table with no NOT NULL column cannot be selected whole through a nullable handle.
- [ ] Whether FULL and RIGHT joins are needed. A full join would make earlier handles nullable, which their types cannot express.
- [ ] LATERAL joins: they need only a scope rule that lets the query see the items beside it.
- [ ] Recursive CTEs.
- [ ] Typed null parameters, by sending each null with its column's type.
- [ ] A struct projection (`#[derive(Selectable)]`, `select_as::<T>()`) beyond whole-row selects.
- [ ] `#[derive(PgEnum)]` for Postgres enum types.

## Risks

- **Render-time scope errors in rarely run code.** A branch no test renders can hide a scope error until it runs. Tests that render each statement path are the mitigation, and the error message names the table and column.
- **Error message quality.** Each public trait gets a `#[diagnostic::on_unimplemented]` message in user terms, and `compile_fail` tests pin the common cases.
- **Scope creep.** Relation loading and an active-record layer stay out until howdah has been used in a real project.

## Milestones

Each milestone ends with tests, clippy, fmt and doc clean, and a review stop.

1. Core: value types, `Column` and `Expr`, handles, `from`, `join`, `left_join`, `filter`, `order_by`, `limit`, `offset`, `select`, and the renderer with aliases and scope rules 1 and 2.
2. Expressions and grouping: operators, functions, CASE, intervals, `group_by`, `having`, `distinct`, `distinct_on` and locking.
3. Subqueries: `exists`, `in_query` and `scalar` (rules 3 and 6), queries as tables (rule 4), and CTEs.
4. Writes: insert with upsert, update, delete and RETURNING (rule 5).
5. Raw SQL.
6. Executor: pool, terminals, statement cache, transactions and savepoints.
7. `#[derive(Table)]` and `#[derive(Shape)]`.
8. Diagnostics and documentation.
