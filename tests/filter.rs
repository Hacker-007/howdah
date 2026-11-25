mod schema;

use howdah::{Error, from};
use schema::*;

#[test]
fn filters_by_a_boolean_column() {
    let (mut q, u) = from(Users);
    q.filter(u.active());
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (
            r#"SELECT "users"."name" FROM "users" WHERE "users"."active""#.to_string(),
            vec![]
        )
    );
}

#[test]
fn every_filter_must_hold() {
    let (mut q, u) = from(Users);
    q.filter(u.active()).filter(u.verified());
    assert_eq!(
        q.select(u.name()).to_sql().unwrap(),
        (
            r#"SELECT "users"."name" FROM "users" WHERE "users"."active" AND "users"."verified""#
                .to_string(),
            vec![]
        )
    );
}

#[test]
fn a_select_keeps_the_filters_it_was_made_with() {
    let (mut q, u) = from(Users);
    let all = q.select(u.name());
    q.filter(u.active());
    let active = q.select(u.name());
    assert_eq!(
        all.to_sql().unwrap(),
        (r#"SELECT "users"."name" FROM "users""#.to_string(), vec![])
    );
    assert_eq!(
        active.to_sql().unwrap(),
        (
            r#"SELECT "users"."name" FROM "users" WHERE "users"."active""#.to_string(),
            vec![]
        )
    );
}

#[test]
fn filter_on_a_column_of_another_select_is_not_in_scope() {
    let (_, other) = from(Users);
    let (mut q, u) = from(Users);
    q.filter(other.active());
    assert_eq!(
        q.select(u.name()).to_sql().unwrap_err(),
        Error::NotInScope {
            table: "users",
            column: "active",
        }
    );
}
