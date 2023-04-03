use actix_web::HttpResponse;
use actix_web::http::header::{ContentType, LOCATION};
use secrecy::Secret;
use crate::authentication::{validate_credentials, Credentials};
use sqlx::PgPool;
use crate::authentication::AuthError;
use actix_web::http::StatusCode;
use actix_web::{web, ResponseError};
use actix_web::error::InternalError;
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use crate::startup::HmacSecret;


#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}


fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}


#[tracing::instrument(
skip(form, pool, secret),
fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]

pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    secret: web::Data<HmacSecret>,
) -> Result<HttpResponse, InternalError<LoginError>>{
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };

    /*
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    Ok(HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish())

    */

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
        tracing::Span::current()
            .record("user_id", &tracing::field::display(&user_id));
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
    }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => {
                    LoginError::UnexpectedError(e.into())
                },
            };
            let query_string = format!(
                "error={}",
                urlencoding::Encoded::new(e.to_string())
            );

            let hmac_tag = {
                let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
                    secret.0.expose_secret().as_bytes()
                ).unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };

            let response = HttpResponse::SeeOther()
                .insert_header((
                LOCATION,
                format!("/login?{}&tag={:x}", query_string, hmac_tag), ))
                .finish();
            Err(InternalError::from_response(e, response))
        }
    }
}


#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

