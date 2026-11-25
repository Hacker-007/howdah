// The tables used by the tests.

use howdah::{Column, Handle, Nullability, Nullable, Table};

#[derive(Debug, Clone, Copy)]
pub struct Users;

impl Table for Users {
    const NAME: &'static str = "users";
}

#[derive(Debug, Clone, Copy)]
pub struct Orders;

impl Table for Orders {
    const NAME: &'static str = "orders";
}

/// The columns of `users`. Columns that cannot be null in the table are as
/// nullable as the handle: through a left join's handle they are nullable.
#[allow(dead_code)]
pub trait UsersColumns {
    type Null: Nullability;

    fn id(&self) -> Column<i64, Self::Null>;
    fn name(&self) -> Column<String, Self::Null>;
    fn email(&self) -> Column<String, Self::Null>;
    fn active(&self) -> Column<bool, Self::Null>;
    fn manager_id(&self) -> Column<i64, Nullable>;
    fn verified(&self) -> Column<bool, Nullable>;
}

impl<N: Nullability> UsersColumns for Handle<Users, N> {
    type Null = N;

    fn id(&self) -> Column<i64, N> {
        Column::new(*self, "id")
    }

    fn name(&self) -> Column<String, N> {
        Column::new(*self, "name")
    }

    fn email(&self) -> Column<String, N> {
        Column::new(*self, "email")
    }

    fn active(&self) -> Column<bool, N> {
        Column::new(*self, "active")
    }

    fn manager_id(&self) -> Column<i64, Nullable> {
        Column::new(*self, "manager_id")
    }

    fn verified(&self) -> Column<bool, Nullable> {
        Column::new(*self, "verified")
    }
}

/// The columns of `orders`.
#[allow(dead_code)]
pub trait OrdersColumns {
    type Null: Nullability;

    fn id(&self) -> Column<i64, Self::Null>;
    fn user_id(&self) -> Column<i64, Self::Null>;
    fn amount_cents(&self) -> Column<i64, Self::Null>;
}

impl<N: Nullability> OrdersColumns for Handle<Orders, N> {
    type Null = N;

    fn id(&self) -> Column<i64, N> {
        Column::new(*self, "id")
    }

    fn user_id(&self) -> Column<i64, N> {
        Column::new(*self, "user_id")
    }

    fn amount_cents(&self) -> Column<i64, N> {
        Column::new(*self, "amount_cents")
    }
}
