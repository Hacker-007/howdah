mod schema;

use howdah::{Column, Error, Expression, Handle, Nullable, Select, Value, from};
use schema::*;

#[test]
fn joins_the_same_table_twice() {
    let (mut q, u) = from(Users);
    let m = q.join(Users, |m| m.id().eq(u.manager_id()));
    let s: Select<(String, String)> = q.select((u.name(), m.name()));
    assert_eq!(
        s.to_sql().unwrap(),
        (
            r#"SELECT "users"."name", "users_2"."name" FROM "users" JOIN "users" AS "users_2" ON "users_2"."id" = "users"."manager_id""#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn aliases_count_uses_of_a_table() {
    let (mut q, u) = from(Users);
    let m = q.join(Users, |m| m.id().eq(u.manager_id()));
    let g = q.join(Users, |g| g.id().eq(m.manager_id()));
    let s: Select<(String, String, String)> = q.select((u.name(), m.name(), g.name()));
    assert_eq!(
        s.to_sql().unwrap(),
        (
            r#"SELECT "users"."name", "users_2"."name", "users_3"."name" FROM "users" JOIN "users" AS "users_2" ON "users_2"."id" = "users"."manager_id" JOIN "users" AS "users_3" ON "users_3"."id" = "users_2"."manager_id""#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn a_table_joined_once_gets_no_alias() {
    let (mut q, u) = from(Users);
    let o = q.join(Orders, |o| o.user_id().eq(u.id()));
    let s: Select<(String, i64)> = q.select((u.name(), o.amount_cents()));
    assert_eq!(
        s.to_sql().unwrap(),
        (
            r#"SELECT "users"."name", "orders"."amount_cents" FROM "users" JOIN "orders" ON "orders"."user_id" = "users"."id""#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn left_joins_make_columns_nullable() {
    let (mut q, u) = from(Users);
    let m: Handle<Users, Nullable> = q.left_join(Users, |m| m.id().eq(u.manager_id()));
    let name: Column<String, Nullable> = m.name();
    let id: Column<i64, Nullable> = m.id();
    type Row = (String, Option<String>, Option<i64>, Option<i64>);
    let s: Select<Row> = q.select((u.name(), name, id, m.manager_id()));
    assert_eq!(
        s.to_sql().unwrap(),
        (
            r#"SELECT "users"."name", "users_2"."name", "users_2"."id", "users_2"."manager_id" FROM "users" LEFT JOIN "users" AS "users_2" ON "users_2"."id" = "users"."manager_id""#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn values_in_on_are_parameters_numbered_before_where() {
    let (mut q, u) = from(Users);
    let _o = q.join(Orders, |o| {
        o.user_id().eq(u.id()) & o.amount_cents().gt(100)
    });
    q.filter(u.name().eq("Ada"));
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (
            r#"SELECT "users"."name" FROM "users" JOIN "orders" ON "orders"."user_id" = "users"."id" AND "orders"."amount_cents" > $1 WHERE "users"."name" = $2"#
                .to_string(),
            vec![Value::BigInt(100), Value::Text("Ada".to_string())]
        )
    );
}

#[test]
fn filters_and_orders_can_use_joined_handles() {
    let (mut q, u) = from(Users);
    let m = q.join(Users, |m| m.id().eq(u.manager_id()));
    q.filter(m.verified().is_not_null())
        .order_by(m.name().asc());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            r#"SELECT "users"."id" FROM "users" JOIN "users" AS "users_2" ON "users_2"."id" = "users"."manager_id" WHERE "users_2"."verified" IS NOT NULL ORDER BY "users_2"."name" ASC"#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn a_select_keeps_the_joins_it_was_made_with() {
    let (mut q, u) = from(Users);
    let before = q.select(u.name());
    let m = q.join(Users, |m| m.id().eq(u.manager_id()));
    let after = q.select((u.name(), m.name()));
    assert_eq!(
        before.to_sql().unwrap(),
        (r#"SELECT "users"."name" FROM "users""#.to_string(), vec![])
    );
    assert_eq!(
        after.to_sql().unwrap(),
        (
            r#"SELECT "users"."name", "users_2"."name" FROM "users" JOIN "users" AS "users_2" ON "users_2"."id" = "users"."manager_id""#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn a_handle_of_another_select_is_not_in_scope_in_on() {
    let (_, other) = from(Users);
    let (mut q, u) = from(Users);
    let _m = q.join(Users, |m| m.id().eq(other.id()));
    assert_eq!(
        q.select(u.name()).to_sql().unwrap_err(),
        Error::NotInScope {
            table: "users",
            column: "id",
        }
    );
}

#[test]
fn a_joined_handle_of_another_select_is_not_in_scope() {
    let (mut other_q, other_u) = from(Users);
    let other_m = other_q.join(Users, |m| m.id().eq(other_u.id()));
    let (q, _) = from(Users);
    assert_eq!(
        q.select(other_m.name()).to_sql().unwrap_err(),
        Error::NotInScope {
            table: "users",
            column: "name",
        }
    );
}
