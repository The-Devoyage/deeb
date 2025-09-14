use actix_web::{
    Error, HttpRequest, HttpResponse, get, rt,
    web::{Data, Payload},
};
use actix_ws::AggregatedMessage;
use deeb::{Entity, FindManyOptions, Query};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::app_data::AppData;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct SubscribeOptions {
    entity_name: String,
    query: Option<Query>,
    find_many_options: Option<FindManyOptions>,
}

#[derive(Debug, Clone, Serialize)]
pub enum SubscribeResponseStatus {
    Ok,
    Error(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscribeResponse {
    status: SubscribeResponseStatus,
    data: Option<Vec<Value>>,
}

#[get("/subscribe")]
async fn subscribe(
    req: HttpRequest,
    stream: Payload,
    app_data: Data<AppData>,
) -> Result<HttpResponse, Error> {
    let database = app_data.database.clone();
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            let database = database.clone();
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let subscribe_options = match serde_json::from_str::<SubscribeOptions>(&text) {
                        Ok(options) => options,
                        Err(err) => {
                            let response = SubscribeResponse {
                                data: None,
                                status: SubscribeResponseStatus::Error(format!(
                                    "Error parsing JSON: {}",
                                    err
                                )),
                            };
                            session
                                .text(serde_json::to_string(&response).unwrap())
                                .await
                                .unwrap();
                            continue;
                        }
                    };
                    let entity = Entity::new(&subscribe_options.entity_name);

                    //TODO: Handle Applied Queries && Post Query Validation!!!!

                    let data = match database
                        .deeb
                        .find_many::<Value>(
                            &entity,
                            subscribe_options.query.unwrap_or(Query::All),
                            subscribe_options.find_many_options,
                            None,
                        )
                        .await
                    {
                        Ok(data) => data,
                        Err(err) => {
                            let response = SubscribeResponse {
                                data: None,
                                status: SubscribeResponseStatus::Error(format!(
                                    "Error fetching data: {}",
                                    err
                                )),
                            };
                            session
                                .text(serde_json::to_string(&response).unwrap())
                                .await
                                .unwrap();
                            continue;
                        }
                    };

                    let success_response = SubscribeResponse {
                        data,
                        status: SubscribeResponseStatus::Ok,
                    };

                    session
                        .text(serde_json::to_string(&success_response).unwrap())
                        .await
                        .unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    // respond to PING frame with PONG frame
                    session.pong(&msg).await.unwrap();
                }
                _ => {
                    log::warn!("Unknown message type received");
                }
            }
        }
    });

    // respond immediately with response connected to WS session
    Ok(res)
}
