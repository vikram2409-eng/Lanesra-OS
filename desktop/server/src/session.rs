use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use rusqlite::Connection;

use lanesra_core::repositories::session_repo;

pub const SESSION_COOKIE: &str = "lanesra_session";

/// Resolves the logged-in user_id for this request from its session
/// cookie, if any. A missing or expired session is not an error - it just
/// means the caller isn't authenticated.
pub fn current_actor(conn: &Connection, jar: &CookieJar) -> Option<String> {
    let token = jar.get(SESSION_COOKIE)?.value().to_string();
    session_repo::resolve_and_touch(conn, &token).ok().flatten()
}

pub fn set_session_cookie(jar: CookieJar, token: String) -> CookieJar {
    let cookie = Cookie::build((SESSION_COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::hours(session_repo::SESSION_LIFETIME_HOURS))
        .build();
    jar.add(cookie)
}

pub fn clear_session_cookie(jar: CookieJar) -> CookieJar {
    let mut cookie = Cookie::from(SESSION_COOKIE);
    cookie.set_path("/");
    jar.remove(cookie)
}
