use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub enum DocumentType {
    Page,
    Block,
}
