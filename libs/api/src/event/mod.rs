use axum::{
    extract::{Path, State},
    Json,
};
use repositories::Repository;

mod request;
pub mod response;

use crate::{ApiResponse, IntoApiResponse};

use self::response::{Event, GetEventResponse, GetEventsResponse};

pub async fn get_events(
    State(repo): State<Repository>,
) -> ApiResponse<Json<GetEventsResponse>> {
    let events = repo.event.find_all().await.into_response("502-005")?;

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
) -> ApiResponse<Json<GetEventResponse>> {
    let event = repo.event.find_by_event_id(id).await.into_response("502-006")?;

    let Some(event) = event else {
        return Ok(Json(GetEventResponse { event: None }));
    };

    Ok(Json(GetEventResponse {
        event: Some(Event {
            contents: event.contents,
        }),
    }))
}
