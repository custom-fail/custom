use std::sync::Arc;
use warp::Filter;
use crate::response_type;
use crate::server::session::{Authenticator, AuthorizationInformation, authorize_user, Sessions};

pub fn run(
    authenticator: Arc<Authenticator>,
    sessions: Arc<Sessions>
) -> response_type!() {
   warp::get()
       .and(warp::path!("users" / "me"))
       .and(authorize_user(authenticator, sessions))
       .map(|info: Arc<AuthorizationInformation>| {
           warp::reply::json(&info.user)
       })
}