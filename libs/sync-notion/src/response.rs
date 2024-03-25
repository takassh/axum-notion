use notion_client::NotionClientError;
use repository::RepositoryError;

use crate::SyncNotionError;

type Response<T> = Result<T, SyncNotionError>;

pub trait IntoResponse<T> {
    fn into_response(self, message: &str) -> Response<T>;
}

impl<T> IntoResponse<T> for Result<T, std::io::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncNotionError::StdIoError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, toml::de::Error> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncNotionError::TomlDeError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, RepositoryError> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncNotionError::RepositoryError {
            source: e,
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Result<T, NotionClientError> {
    fn into_response(self, message: &str) -> Response<T> {
        self.map_err(|e| SyncNotionError::NotionClientError {
            source: Box::new(e),
            message: message.to_string(),
        })
    }
}

impl<T> IntoResponse<T> for Option<T> {
    fn into_response(self, message: &str) -> Response<T> {
        self.ok_or_else(|| SyncNotionError::Option {
            message: message.to_string(),
        })
    }
}
