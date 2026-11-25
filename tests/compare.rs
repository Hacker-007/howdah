mod schema;

use howdah::{Error, Expr, Expression, Nullable, from};
use schema::*;

#[test]
fn filters_by_a_comparison() {
    let (mut q, u) = from(Users);
    q.filter(u.id().eq(u.manager_id()));
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (
            r#"SELECT "users"."name" FROM "users" WHERE "users"."id" = "users"."manager_id""#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn renders_every_comparison() {
    let (q, u) = from(Users);
    let cases = [
        (u.id().eq(u.id()), "="),
        (u.id().ne(u.id()), "<>"),
        (u.id().lt(u.id()), "<"),
        (u.id().le(u.id()), "<="),
        (u.id().gt(u.id()), ">"),
        (u.id().ge(u.id()), ">="),
    ];

    for (condition, op) in cases {
        let mut q = q.clone();
        q.filter(condition);
        assert_eq!(
            q.select(u.name()).to_sql().unwrap(),
            (
                format!(
                    r#"SELECT "users"."name" FROM "users" WHERE "users"."id" {op} "users"."id""#
                ),
                vec![]
            )
        );
    }
}

#[test]
fn a_comparison_is_nullable_if_either_side_is() {
    let (_, u) = from(Users);
    let _: Expr<bool> = u.id().eq(u.id());
    let _: Expr<bool, Nullable> = u.id().eq(u.manager_id());
    let _: Expr<bool, Nullable> = u.manager_id().eq(u.id());
    let _: Expr<bool, Nullable> = u.manager_id().eq(u.manager_id());
}

#[test]
fn nested_comparisons_are_parenthesized() {
    let (mut q, u) = from(Users);
    q.filter(u.active().eq(u.id().lt(u.manager_id())));
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (r#"SELECT "users"."name" FROM "users" WHERE "users"."active" = ("users"."id" < "users"."manager_id")"#.to_string(), vec![])
    );
}

#[test]
fn comparison_with_a_column_of_another_select_is_not_in_scope() {
    let (_, other) = from(Users);
    let (mut q, u) = from(Users);
    q.filter(u.id().eq(other.manager_id()));
    assert_eq!(
        q.select(u.name()).to_sql().unwrap_err(),
        Error::NotInScope {
            table: "users",
            column: "manager_id",
        }
    );
}
