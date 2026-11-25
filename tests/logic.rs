mod schema;

use howdah::{Error, Expr, Expression, Nullable, Value, from};
use schema::*;

fn filtered(condition: &str) -> String {
    format!(r#"SELECT "users"."id" FROM "users" WHERE {condition}"#)
}

#[test]
fn renders_and_or_not() {
    let (mut q, u) = from(Users);
    q.filter(u.active() & !u.verified() | u.manager_id().is_null());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            filtered(
                r#""users"."active" AND NOT "users"."verified" OR "users"."manager_id" IS NULL"#
            ),
            vec![]
        )
    );
}

#[test]
fn renders_is_not_null() {
    let (mut q, u) = from(Users);
    q.filter(u.manager_id().is_not_null());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (filtered(r#""users"."manager_id" IS NOT NULL"#), vec![])
    );
}

#[test]
fn comparisons_inside_and_or_need_no_parentheses() {
    let (mut q, u) = from(Users);
    q.filter(u.id().eq(1) & u.name().eq("Ada") | u.id().gt(10));
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            filtered(r#""users"."id" = $1 AND "users"."name" = $2 OR "users"."id" > $3"#),
            vec![
                Value::BigInt(1),
                Value::Text("Ada".to_string()),
                Value::BigInt(10)
            ]
        )
    );
}

#[test]
fn looser_operands_are_parenthesized() {
    let (q, u) = from(Users);
    let condition = |condition: Expr<bool, _>| {
        let mut q = q.clone();
        q.filter(condition);
        q.select(u.id()).to_sql().unwrap().0
    };
    let cases = [
        (
            condition((u.active() | u.verified()) & u.active()),
            r#"("users"."active" OR "users"."verified") AND "users"."active""#,
        ),
        (
            condition(u.active() | u.verified() & u.active()),
            r#""users"."active" OR "users"."verified" AND "users"."active""#,
        ),
        (
            condition(!(u.active() & u.verified())),
            r#"NOT ("users"."active" AND "users"."verified")"#,
        ),
        (condition(!!u.verified()), r#"NOT NOT "users"."verified""#),
        (
            condition(u.active().eq(u.active() & u.verified())),
            r#""users"."active" = ("users"."active" AND "users"."verified")"#,
        ),
    ];

    for (sql, expected) in cases {
        assert_eq!(sql, filtered(expected));
    }
}

#[test]
fn is_null_operands_are_parenthesized_unless_atoms() {
    let (mut q, u) = from(Users);
    q.filter(u.active().eq(u.manager_id().is_null()))
        .filter(u.id().eq(u.manager_id()).is_null());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            filtered(
                r#""users"."active" = ("users"."manager_id" IS NULL) AND ("users"."id" = "users"."manager_id") IS NULL"#
            ),
            vec![]
        )
    );
}

#[test]
fn an_or_filter_is_parenthesized_among_filters() {
    let (mut q, u) = from(Users);
    q.filter(u.active() | u.verified())
        .filter(u.id().eq(u.id()));
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            filtered(r#"("users"."active" OR "users"."verified") AND "users"."id" = "users"."id""#),
            vec![]
        )
    );
}

#[test]
fn a_rust_bool_can_be_an_operand() {
    let (mut q, u) = from(Users);
    q.filter(u.active() | true);
    assert_eq!(
        q.select(u.id()).to_sql().unwrap(),
        (
            filtered(r#""users"."active" OR $1"#),
            vec![Value::Boolean(true)]
        )
    );
}

#[test]
fn logic_is_nullable_if_an_operand_is() {
    let (_, u) = from(Users);
    let _: Expr<bool> = u.active() & u.active();
    let _: Expr<bool, Nullable> = u.active() & u.verified();
    let _: Expr<bool, Nullable> = u.verified() | u.active();
    let _: Expr<bool> = !u.active();
    let _: Expr<bool, Nullable> = !u.verified();
    let _: Expr<bool> = u.manager_id().is_null();
    let _: Expr<bool> = u.verified().is_not_null();
}

#[test]
fn logic_with_a_column_of_another_select_is_not_in_scope() {
    let (_, other) = from(Users);
    let (mut q, u) = from(Users);
    q.filter(u.active() & !other.verified());
    assert_eq!(
        q.select(u.id()).to_sql().unwrap_err(),
        Error::NotInScope {
            table: "users",
            column: "verified",
        }
    );
}
