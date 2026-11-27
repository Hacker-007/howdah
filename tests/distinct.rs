mod schema;

use howdah::{Error, Expression, Value, from};
use schema::*;

#[test]
fn selects_distinct_rows() {
    let (mut q, u) = from(Users);
    q.distinct();
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (
            r#"SELECT DISTINCT "users"."name" FROM "users""#.to_string(),
            vec![]
        )
    );
}

#[test]
fn selects_distinct_on_each_expression() {
    let (mut q, u) = from(Users);
    q.distinct_on(u.manager_id())
        .distinct_on(u.active())
        .order_by(u.manager_id().asc())
        .order_by(u.active().asc())
        .order_by(u.id().desc());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            r#"SELECT DISTINCT ON ("users"."manager_id", "users"."active") "users"."id" FROM "users" ORDER BY "users"."manager_id" ASC, "users"."active" ASC, "users"."id" DESC"#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn distinct_on_parameters_come_first() {
    let (mut q, u) = from(Users);
    q.distinct_on(u.id() % 10).filter(u.id().gt(5));
    assert_eq!(
        q.select(u.id() + 1).to_sql().unwrap(),
        (
            r#"SELECT DISTINCT ON ("users"."id" % $1) "users"."id" + $2 FROM "users" WHERE "users"."id" > $3"#
                .to_string(),
            vec![Value::BigInt(10), Value::BigInt(1), Value::BigInt(5)]
        )
    );
}

#[test]
fn the_last_of_distinct_and_distinct_on_wins() {
    let (q, u) = from(Users);

    let mut on_then_rows = q.clone();
    on_then_rows.distinct_on(u.name()).distinct();
    assert_eq!(
        on_then_rows.select(u.name()).to_sql().unwrap().0,
        r#"SELECT DISTINCT "users"."name" FROM "users""#
    );

    let mut rows_then_on = q.clone();
    rows_then_on.distinct().distinct_on(u.name());
    assert_eq!(
        rows_then_on.select(u.name()).to_sql().unwrap().0,
        r#"SELECT DISTINCT ON ("users"."name") "users"."name" FROM "users""#
    );
}

#[test]
fn a_select_keeps_its_distinct() {
    let (mut q, u) = from(Users);
    let all = q.select(u.name());
    q.distinct();
    let distinct = q.select(u.name());
    assert_eq!(
        all.to_sql().unwrap().0,
        r#"SELECT "users"."name" FROM "users""#
    );
    assert_eq!(
        distinct.to_sql().unwrap().0,
        r#"SELECT DISTINCT "users"."name" FROM "users""#
    );
}

#[test]
fn distinct_on_may_use_a_joined_table() {
    let (mut q, u) = from(Users);
    let o = q.join(Orders, |o| o.user_id().eq(u.id()));
    q.distinct_on(o.user_id());
    assert_eq!(
        q.select(o.id()).to_sql().unwrap().0,
        r#"SELECT DISTINCT ON ("orders"."user_id") "orders"."id" FROM "users" JOIN "orders" ON "orders"."user_id" = "users"."id""#
    );
}

#[test]
fn distinct_on_a_column_of_another_select_is_not_in_scope() {
    let (_, other) = from(Users);
    let (mut q, u) = from(Users);
    q.distinct_on(other.name());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap_err(),
        Error::NotInScope {
            table: "users",
            column: "name",
        }
    );
}
