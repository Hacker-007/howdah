mod schema;

use howdah::{Error, Expr, Expression, NeverNull, Nullable, Select, Value, from};
use schema::*;

fn selected(expression: &str) -> String {
    format!(r#"SELECT {expression} FROM "orders""#)
}

#[test]
fn renders_each_operator() {
    let (q, o) = from(Orders);
    let sql = |expression: Expr<i64>| q.select(expression).to_sql().unwrap();
    let cases = [
        (sql(o.amount_cents() + o.id()), "+"),
        (sql(o.amount_cents() - o.id()), "-"),
        (sql(o.amount_cents() * o.id()), "*"),
        (sql(o.amount_cents() / o.id()), "/"),
        (sql(o.amount_cents() % o.id()), "%"),
    ];

    for (sql, op) in cases {
        assert_eq!(
            sql,
            (
                selected(&format!(r#""orders"."amount_cents" {op} "orders"."id""#)),
                vec![]
            )
        );
    }
}

#[test]
fn an_integer_literal_takes_the_column_type() {
    let (q, o) = from(Orders);
    assert_eq!(
        q.select(o.amount_cents() * 2).to_sql().unwrap(),
        (
            selected(r#""orders"."amount_cents" * $1"#),
            vec![Value::BigInt(2)]
        )
    );
}

#[test]
fn arithmetic_inside_a_comparison_needs_no_parentheses() {
    let (mut q, o) = from(Orders);
    q.filter((o.amount_cents() - 100).ge(o.id() * 10));
    assert_eq!(
        q.select(o.id()).to_sql().unwrap(),
        (
            r#"SELECT "orders"."id" FROM "orders" WHERE "orders"."amount_cents" - $1 >= "orders"."id" * $2"#
                .to_string(),
            vec![Value::BigInt(100), Value::BigInt(10)]
        )
    );
}

#[test]
fn parentheses_follow_postgres_precedence_and_left_grouping() {
    let (q, o) = from(Orders);
    let a = o.amount_cents();
    let b = o.id();
    let c = o.user_id();
    let sql = |expression: Expr<i64>| q.select(expression).to_sql().unwrap().0;
    let cases = [
        (
            sql(a + b * c),
            r#""orders"."amount_cents" + "orders"."id" * "orders"."user_id""#,
        ),
        (
            sql((a + b) * c),
            r#"("orders"."amount_cents" + "orders"."id") * "orders"."user_id""#,
        ),
        (
            sql(a - b - c),
            r#""orders"."amount_cents" - "orders"."id" - "orders"."user_id""#,
        ),
        (
            sql(a - (b - c)),
            r#""orders"."amount_cents" - ("orders"."id" - "orders"."user_id")"#,
        ),
        (
            sql(a + b + c),
            r#""orders"."amount_cents" + "orders"."id" + "orders"."user_id""#,
        ),
        (
            sql(a + (b + c)),
            r#""orders"."amount_cents" + ("orders"."id" + "orders"."user_id")"#,
        ),
        (
            sql(a + (b - c)),
            r#""orders"."amount_cents" + ("orders"."id" - "orders"."user_id")"#,
        ),
        (
            sql(a * b / c),
            r#""orders"."amount_cents" * "orders"."id" / "orders"."user_id""#,
        ),
        (
            sql(a / (b * c)),
            r#""orders"."amount_cents" / ("orders"."id" * "orders"."user_id")"#,
        ),
        (
            sql(a % (b % c)),
            r#""orders"."amount_cents" % ("orders"."id" % "orders"."user_id")"#,
        ),
        (
            sql(a * b - c / a),
            r#""orders"."amount_cents" * "orders"."id" - "orders"."user_id" / "orders"."amount_cents""#,
        ),
    ];

    for (sql, expected) in cases {
        assert_eq!(sql, selected(expected));
    }
}

#[test]
fn arithmetic_is_nullable_if_an_operand_is() {
    let (q, u) = from(Users);
    let _: Expr<i64, NeverNull> = u.id() + u.id();
    let _: Expr<i64, Nullable> = u.id() + u.manager_id();
    let _: Expr<i64, Nullable> = u.manager_id() * 2;
    let _: Select<Option<i64>> = q.select(u.manager_id() - u.id());
    let _: Select<i64> = q.select(u.id() % 7);
}

#[test]
fn arithmetic_with_a_column_of_another_select_is_not_in_scope() {
    let (_, other) = from(Orders);
    let (q, o) = from(Orders);
    assert_eq!(
        q.select(o.amount_cents() + other.amount_cents())
            .to_sql()
            .unwrap_err(),
        Error::NotInScope {
            table: "orders",
            column: "amount_cents",
        }
    );
}
