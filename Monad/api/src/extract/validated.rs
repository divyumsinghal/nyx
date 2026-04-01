use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S>,
{
    type Rejection = axum::response::Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(payload) = Json::<T>::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;

        payload.validate().map_err(|e| {
            let fields = e
                .field_errors()
                .iter()
                .flat_map(|(field, errs)| {
                    errs.iter().map(move |err| {
                        nun::error::FieldError::new(
                            (*field).to_string(),
                            err.code.to_string(),
                            err.message
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_else(|| err.code.to_string()),
                        )
                    })
                })
                .collect::<Vec<_>>();
            let err = nun::NyxError::validation(fields).to_error_response(None);
            (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(err)).into_response()
        })?;

        Ok(Self(payload))
    }
}
