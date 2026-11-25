mod schema;

use howdah::{Error, Expression, Value, from};
use schema::*;

#[test]
fn orders_rows() {
    let (mut q, u) = from(Users);
    q.order_by(u.name().asc()).order_by(u.id().desc());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            r#"SELECT "users"."id" FROM "users" ORDER BY "users"."name" ASC, "users"."id" DESC"#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn orders_by_an_expression() {
    let (mut q, u) = from(Users);
    q.order_by(u.id().eq(1).desc());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            r#"SELECT "users"."id" FROM "users" ORDER BY "users"."id" = $1 DESC"#.to_string(),
            vec![Value::BigInt(1)]
        )
    );
}

#[test]
fn limits_and_offsets() {
    let (mut q, u) = from(Users);
    q.limit(50).offset(10);
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            r#"SELECT "users"."id" FROM "users" LIMIT 50 OFFSET 10"#.to_string(),
            vec![]
        )
    );
}

#[test]
fn the_last_limit_and_offset_win() {
    let (mut q, u) = from(Users);
    q.limit(50).limit(10).offset(5).offset(2);
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            r#"SELECT "users"."id" FROM "users" LIMIT 10 OFFSET 2"#.to_string(),
            vec![]
        )
    );
}

#[test]
fn a_select_keeps_its_order_limit_and_offset() {
    let (mut q, u) = from(Users);
    q.order_by(u.name().asc()).limit(50);
    let page = q.select(u.id());
    q.order_by(u.id().desc()).limit(10).offset(20);
    let next_page = q.select(u.id());
    assert_eq!(
        page.to_sql().unwrap(),
        (
            r#"SELECT "users"."id" FROM "users" ORDER BY "users"."name" ASC LIMIT 50"#.to_string(),
            vec![]
        )
    );
    assert_eq!(
        next_page.to_sql().unwrap(),
        (
            r#"SELECT "users"."id" FROM "users" ORDER BY "users"."name" ASC, "users"."id" DESC LIMIT 10 OFFSET 20"#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn order_by_a_column_of_another_select_is_not_in_scope() {
    let (_, other) = from(Users);
    let (mut q, u) = from(Users);
    q.order_by(other.name().asc());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap_err(),
        Error::NotInScope {
            table: "users",
            column: "name",
        }
    );
}
