use serde::Deserialize;
use serde_with::serde_as;
use utoipa::ToSchema;
use serde_with::DisplayFromStr;

#[serde_as]
#[derive(Deserialize, ToSchema)]
pub struct Pagination {
    #[serde_as(as = "DisplayFromStr")]
    pub limit: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub offset: u64,
}
