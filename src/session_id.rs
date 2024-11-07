use axum_extra::extract::{cookie::Cookie, CookieJar};
use uuid::Uuid;

use crate::app_error::AppError;

pub fn get_or_create_session_id(cookies: CookieJar) -> (Uuid, CookieJar) {
    if let Some(session_id) = cookies
        .get("session_id")
        .map(|cookie| cookie.value().parse::<Uuid>().ok())
        .flatten()
    {
        return (session_id, cookies);
    } else {
        let new_session_id = Uuid::new_v4();
        let mut cookie = Cookie::new("session_id", new_session_id.to_string());
        cookie.set_max_age(time::Duration::days(30));

        return (new_session_id, cookies.add(cookie));
    }
}

pub fn assert_equal_ids(uuid1: &Uuid, uuid2: &Uuid) -> Result<(), AppError> {
    if uuid1 == uuid2 {
        return Ok(());
    } else {
        return Err(AppError::Unauthorized(
            "This session_id is not valid for this request".to_string(),
        ));
    }
}
