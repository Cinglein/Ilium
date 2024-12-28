use crate::account::Account;
use bevy::prelude::*;
use sqlx::{query::*, *};

/// Data associated with a user account.
#[trait_variant::make(Send)]
pub trait UserData:
    Component + Clone + Send + Sync + Unpin + for<'r> FromRow<'r, <Any as Database>::Row>
{
    type O: Ord;
    fn query<'q>(account: &Account) -> QueryAs<'q, Any, Self, <Any as Database>::Arguments<'q>>;
    fn matchmake_priority(&self) -> Self::O;
    fn matchmake_valid(&self, user_data: &Self) -> bool;
}

pub async fn query_data<U: UserData>(pool: &Pool<Any>, account: &Account) -> eyre::Result<U> {
    let res = <&Pool<Any> as Executor>::fetch_one(pool, U::query(account)).await?;
    let res = U::from_row(&res)?;
    Ok(res)
}
