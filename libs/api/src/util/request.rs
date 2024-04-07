use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use utoipa::ToSchema;

#[serde_as]
#[derive(Deserialize, ToSchema)]
pub struct Pagination {
    #[serde_as(as = "DisplayFromStr")]
    pub limit: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub page: u64,
}
