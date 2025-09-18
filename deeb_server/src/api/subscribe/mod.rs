use actix_web::{
    Error, HttpRequest, HttpResponse, get, rt,
    web::{Data, Payload},
};
use actix_ws::AggregatedMessage;
use deeb::{Entity, EntityName, FindManyOptions, Query};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{
    app_data::AppData,
    broker::{SenderValue, Subscriber},
};

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
    entity_name: Option<String>,
    data: Option<Value>,
}

#[get("/subscribe")]
async fn subscribe(
    req: HttpRequest,
    stream: Payload,
    app_data: Data<AppData>,
) -> Result<HttpResponse, Error> {
    let broker = app_data.broker.clone();
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(2_usize.pow(20));

    // Init Subscriptions
    let (tx, mut rx) = mpsc::channel::<SenderValue>(8);

    // This task will send messages *to the client* from the mpsc receiver.
    let mut session_clone = session.clone();
    rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let response = SubscribeResponse {
                data: Some(msg.value),
                status: SubscribeResponseStatus::Ok,
                entity_name: Some(msg.entity_name.to_string()),
            };
            if session_clone
                .text(serde_json::to_string(&response).unwrap())
                .await
                .is_err()
            {
                let error_response = SubscribeResponse {
                    data: None,
                    status: SubscribeResponseStatus::Error("Broker error".to_string()),
                    entity_name: None,
                };
                //TODO: Handle error response
                match session_clone
                    .text(serde_json::to_string(&error_response).unwrap())
                    .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        log::error!("Broker Error: {}", err);
                    }
                }
            }
        }
    });

    // start task but don't wait for it
    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let subscribe_options = match serde_json::from_str::<SubscribeOptions>(&text) {
                        Ok(options) => options,
                        Err(err) => {
                            let response = SubscribeResponse {
                                data: None,
                                entity_name: None,
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

                    // Subscribe
                    let subscriber = Subscriber::new(tx.clone());

                    // TODO: Handle Errors
                    broker
                        .subscribe(
                            &EntityName::from(subscribe_options.entity_name.as_str()),
                            &subscribe_options.query.clone().unwrap_or(Query::All),
                            &subscriber,
                        )
                        .await;

                    //TODO: Handle Applied Queries && Post Query Validation!!!!

                    let success_response = SubscribeResponse {
                        data: None,
                        status: SubscribeResponseStatus::Ok,
                        entity_name: Some(entity.name.to_string()),
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

    Ok(res)
}
