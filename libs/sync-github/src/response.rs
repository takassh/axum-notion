use repository::RepositoryError;

use crate::SyncGithubError;

type Response<T> = Result<T, SyncGithubError>;

pub trait IntoResponse<T> {
    fn into_response(self, message: &str) -> Response<T>;
}

impl<T> IntoResponse<T> for Result<T, std::io::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncGithubError::StdIoError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, toml::de::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncGithubError::TomlDeError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, reqwest::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncGithubError::ReqwestError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, RepositoryError> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncGithubError::RepositoryError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Option<T> {
    fn into_response(self, message: &str) -> Response<T> {
        self.ok_or_else(|| SyncGithubError::Option {
            message: message.to_string(),
        })
    }
}
