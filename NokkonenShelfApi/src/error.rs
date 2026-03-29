use axum::{http::StatusCode, response::{IntoResponse, Response}};

pub struct AppError(StatusCode, anyhow::Error);

impl AppError {
    pub fn bad_request(msg: impl Into<anyhow::Error>) -> Self {
        AppError(StatusCode::BAD_REQUEST, msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if self.0 == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!("{:?}", self.1);
        }
        self.0.into_response()
    }
}

impl aide::OperationOutput for AppError {
    type Inner = ();
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(StatusCode::INTERNAL_SERVER_ERROR, e.into())
    }
}
