use crate::account::Account;
use bevy::prelude::*;
use sqlx::*;

/// Data associated with a user account.
#[trait_variant::make(Send)]
pub trait UserData:
    Component + Clone + Send + Sync + Unpin + for<'r> FromRow<'r, <Self::DB as Database>::Row>
{
    type O: Ord;
    type DB: Database;
    async fn query(pool: &Pool<Self::DB>, account: &Account) -> eyre::Result<Self>;
    fn matchmake_priority(&self) -> Self::O;
    fn matchmake_valid(&self, user_data: &Self) -> bool;
}
