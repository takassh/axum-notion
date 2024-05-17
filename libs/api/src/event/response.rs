use serde::Serialize;

#[derive(Serialize)]
pub struct EventResp {
    pub contents: String,
}

#[derive(Serialize)]
pub struct GetEventsResp {
    pub events: Vec<EventResp>,
}

#[derive(Serialize)]
pub struct GetEventResp {
    pub event: Option<EventResp>,
}
