use serde::Serialize;

#[derive(Serialize)]
pub struct Event {
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetEventsResponse {
    pub events: Vec<Event>,
}

#[derive(Serialize)]
pub struct GetEventResponse {
    pub event: Option<Event>,
}
