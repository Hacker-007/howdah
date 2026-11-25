mod schema;

use howdah::{Column, Expr, Expression, Nullable, Value, from};
use schema::*;

#[test]
fn a_literal_becomes_a_parameter() {
    let (mut q, u) = from(Users);
    q.filter(u.id().eq(1));
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (
            r#"SELECT "users"."name" FROM "users" WHERE "users"."id" = $1"#.to_string(),
            vec![Value::BigInt(1)]
        )
    );
}

#[test]
fn parameters_are_numbered_in_order() {
    let (mut q, u) = from(Users);
    q.filter(u.id().gt(1))
        .filter(u.name().eq("Ada"))
        .filter(u.active().eq(true));
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (
            r#"SELECT "users"."name" FROM "users" WHERE "users"."id" > $1 AND "users"."name" = $2 AND "users"."active" = $3"#.to_string(),
            vec![
                Value::BigInt(1),
                Value::Text("Ada".to_string()),
                Value::Boolean(true)
            ]
        )
    );
}

#[test]
fn each_rust_type_becomes_its_value() {
    let (mut q, u) = from(Users);
    let small: Column<i16> = Column::new(u, "small");
    let medium: Column<i32> = Column::new(u, "medium");
    let single: Column<f32> = Column::new(u, "single");
    let double: Column<f64> = Column::new(u, "double");
    q.filter(small.eq(1))
        .filter(medium.eq(2))
        .filter(single.eq(3.5))
        .filter(double.eq(4.5))
        .filter(u.email().eq("a@example.com".to_string()));
    let (_, parameters) = q.to_sql().unwrap();
    assert_eq!(
        parameters,
        vec![
            Value::SmallInt(1),
            Value::Integer(2),
            Value::Real(3.5),
            Value::DoublePrecision(4.5),
            Value::Text("a@example.com".to_string()),
        ]
    );
}

#[test]
fn a_literal_is_never_null() {
    let (_, u) = from(Users);
    let _: Expr<bool> = u.id().eq(1);
    let _: Expr<bool, Nullable> = u.manager_id().eq(1);
}

#[test]
fn a_literal_inside_a_nested_comparison() {
    let (mut q, u) = from(Users);
    q.filter(u.active().eq(u.id().lt(5)));
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (
            r#"SELECT "users"."name" FROM "users" WHERE "users"."active" = ("users"."id" < $1)"#
                .to_string(),
            vec![Value::BigInt(5)]
        )
    );
}
