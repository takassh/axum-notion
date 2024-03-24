use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};
use repositories::Repository;

mod request;
pub mod response;

use crate::util::into_response;

use self::response::{Event, GetEventResponse, GetEventsResponse};

pub async fn get_events(
    State(repo): State<Repository>,
) -> Result<Json<GetEventsResponse>, Response> {
    let events = repo
        .event
        .find_all()
        .await
        .map_err(|e| into_response(e, "find all"))?;

    let response = Json(GetEventsResponse {
        events: events
            .into_iter()
            .map(|a| Event {
                contents: a.contents,
            })
            .collect(),
    });

    Ok(response)
}

pub async fn get_event(
    State(repo): State<Repository>,
    Path(id): Path<String>,
) -> Result<Json<GetEventResponse>, Response> {
    let event = repo
        .event
        .find_by_event_id(id)
        .await
        .map_err(|e| into_response(e, "find by event id"))?;

    let Some(event) = event else {
        return Ok(Json(GetEventResponse { event: None }));
    };

    Ok(Json(GetEventResponse {
        event: Some(Event {
            contents: event.contents,
        }),
    }))
}
