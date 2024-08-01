use tower_sessions::Session;
use uuid::Uuid;

use crate::app_error::AppError;

#[derive(Clone)]
pub struct AuthToken {
    pub token: Uuid,
}

impl AuthToken {
    pub async fn get_or_create(session: &Session) -> Result<Self, AppError> {
        if let Some(token) = session
            .get::<Uuid>("auth_token")
            .await
            .map_err(|e| AppError::DatabaseError(e))?
        {
            return Ok(AuthToken { token });
        }

        let new_token = Uuid::new_v4();

        session
            .insert("auth_token", new_token)
            .await
            .map_err(|e| AppError::DatabaseError(e))?;

        return Ok(AuthToken { token: new_token });
    }

    pub async fn verify(&self, session: &Session) -> Result<(), AppError> {
        if self.token == Self::get_or_create(session).await?.token {
            return Ok(());
        } else {
            return Err(AppError::BadRequest("Invalid auth_token".to_string()));
        }
    }
}
