use crate::RepositoryError;

pub(crate) type Response<T> = Result<T, RepositoryError>;

pub(crate) trait IntoResponse<T> {
    fn into_response(self, message: &str) -> Response<T>;
}

impl<T> IntoResponse<T> for Result<T, sea_orm::DbErr> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| RepositoryError::InSeaOrmDbErr {
            message: message.to_string(),
            source: e,
        })
    }
}
