use crate::account::Account;
use bevy::prelude::*;
use core::future::Future;
use sqlx::*;

/// Data associated with a user account.
#[trait_variant::make(Send)]
pub trait UserData:
    Component
    + std::fmt::Debug
    + Clone
    + Send
    + Sync
    + Unpin
    + for<'r> FromRow<'r, <Self::DB as Database>::Row>
{
    type O: Ord;
    type DB: Database;
    fn query(
        pool: &Pool<Self::DB>,
        account: &Account,
    ) -> impl Future<Output = eyre::Result<Self>> + Send + Sync;
    fn matchmake_priority(&self) -> Self::O;
    fn matchmake_valid(&self, user_data: &Self) -> bool;
}
