mod schema;

use howdah::{Select, from};
use schema::*;

#[test]
fn selects_no_columns_until_chosen() {
    let (q, _) = from(Users);
    let _: &Select<()> = &q;
    assert_eq!(q.to_sql(), r#"SELECT FROM "users""#);
}

#[test]
fn selects_a_column() {
    let (q, u) = from(Users);
    let s: Select<String> = q.select(u.name());
    assert_eq!(s.to_sql(), r#"SELECT "users"."name" FROM "users""#);
}

#[test]
fn selects_a_tuple_of_columns() {
    let (q, u) = from(Users);
    let s: Select<(i64, String, bool)> = q.select((u.id(), u.name(), u.active()));
    assert_eq!(
        s.to_sql(),
        r#"SELECT "users"."id", "users"."name", "users"."active" FROM "users""#
    );
}

#[test]
fn tuples_nest() {
    let (q, u) = from(Users);
    let s: Select<((i64, String), bool)> = q.select(((u.id(), u.name()), u.active()));
    assert_eq!(
        s.to_sql(),
        r#"SELECT "users"."id", "users"."name", "users"."active" FROM "users""#
    );
}

#[test]
fn one_query_gives_several_selects() {
    let (q, u) = from(Users);
    let ids: Select<i64> = q.select(u.id());
    let emails: Select<String> = q.select(u.email());
    assert_eq!(ids.to_sql(), r#"SELECT "users"."id" FROM "users""#);
    assert_eq!(emails.to_sql(), r#"SELECT "users"."email" FROM "users""#);
}
