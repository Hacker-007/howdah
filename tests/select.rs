mod schema;

use howdah::{Column, Error, Handle, Select, Table, from};
use schema::*;

#[test]
fn selects_no_columns_until_chosen() {
    let (q, _) = from(Users);
    let _: &Select<()> = &q;
    assert_eq!(q.to_sql().unwrap(), r#"SELECT FROM "users""#);
}

#[test]
fn selects_a_column() {
    let (q, u) = from(Users);
    let s: Select<String> = q.select(u.name());
    assert_eq!(s.to_sql().unwrap(), r#"SELECT "users"."name" FROM "users""#);
}

#[test]
fn selects_a_tuple_of_columns() {
    let (q, u) = from(Users);
    let s: Select<(i64, String, bool)> = q.select((u.id(), u.name(), u.active()));
    assert_eq!(
        s.to_sql().unwrap(),
        r#"SELECT "users"."id", "users"."name", "users"."active" FROM "users""#
    );
}

#[test]
fn tuples_nest() {
    let (q, u) = from(Users);
    let s: Select<((i64, String), bool)> = q.select(((u.id(), u.name()), u.active()));
    assert_eq!(
        s.to_sql().unwrap(),
        r#"SELECT "users"."id", "users"."name", "users"."active" FROM "users""#
    );
}

#[test]
fn nullable_columns_read_as_option() {
    let (q, u) = from(Users);
    let s: Select<(String, Option<i64>)> = q.select((u.name(), u.manager_id()));
    assert_eq!(
        s.to_sql().unwrap(),
        r#"SELECT "users"."name", "users"."manager_id" FROM "users""#
    );
}

#[test]
fn one_query_gives_several_selects() {
    let (q, u) = from(Users);
    let ids: Select<i64> = q.select(u.id());
    let emails: Select<String> = q.select(u.email());
    assert_eq!(ids.to_sql().unwrap(), r#"SELECT "users"."id" FROM "users""#);
    assert_eq!(
        emails.to_sql().unwrap(),
        r#"SELECT "users"."email" FROM "users""#
    );
}

#[test]
fn column_of_another_select_is_not_in_scope() {
    let (_, other) = from(Users);
    let (q, _) = from(Users);
    let error = q.select(other.name()).to_sql().unwrap_err();
    assert_eq!(
        error,
        Error::NotInScope {
            table: "users",
            column: "name",
        }
    );
    assert_eq!(
        error.to_string(),
        "`users.name` is used where its table is not part of the statement"
    );
}

#[test]
fn quotes_in_names_are_escaped() {
    #[derive(Clone, Copy)]
    struct Odd;

    impl Table for Odd {
        const NAME: &'static str = r#"a "b""#;
    }

    let (q, o) = from(Odd);
    let c: Column<i64> = Column::new(o, r#"c"d"#);
    let s = q.select(c);
    assert_eq!(
        s.to_sql().unwrap(),
        r#"SELECT "a ""b"""."c""d" FROM "a ""b""""#
    );
}

#[test]
fn values_can_move_between_threads() {
    fn assert_send_sync<T: Send + Sync + 'static>() {}

    assert_send_sync::<Select>();
    assert_send_sync::<Select<(i64, Option<String>)>>();
    assert_send_sync::<Handle<Users>>();
    assert_send_sync::<Column<String>>();
    assert_send_sync::<Error>();
}
