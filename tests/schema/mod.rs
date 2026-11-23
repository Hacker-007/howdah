// The tables used by the tests.

use howdah::{Column, Handle, Table};

#[derive(Debug, Clone, Copy)]
pub struct Users;

impl Table for Users {
    const NAME: &'static str = "users";
}

/// The columns of `users`.
#[allow(dead_code)]
pub trait UsersColumns {
    fn id(&self) -> Column<i64>;
    fn name(&self) -> Column<String>;
    fn email(&self) -> Column<String>;
    fn active(&self) -> Column<bool>;
}

impl UsersColumns for Handle<Users> {
    fn id(&self) -> Column<i64> {
        Column::new(*self, "id")
    }

    fn name(&self) -> Column<String> {
        Column::new(*self, "name")
    }

    fn email(&self) -> Column<String> {
        Column::new(*self, "email")
    }

    fn active(&self) -> Column<bool> {
        Column::new(*self, "active")
    }
}
