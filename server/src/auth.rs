use crate::account::Account;
use session::token::ClientToken;

pub fn auth(token: ClientToken, ip: std::net::SocketAddr) -> Account {
    match token {
        ClientToken::Guest => Account::Guest { ip },
    }
}
