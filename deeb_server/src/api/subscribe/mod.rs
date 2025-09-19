use std::thread::current;

use actix_web::{
    Error, HttpRequest, HttpResponse, get, rt,
    web::{Data, Payload},
};
use actix_ws::AggregatedMessage;
use deeb::{Entity, EntityName, Query};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

use crate::{
    app_data::AppData,
    broker::{EventType, SenderValue, Subscriber, SubscriberId},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscribeAction {
    Subscribe,
    Unsubscribe,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SubscribeOptions {
    action: SubscribeAction,
    entity_name: String,
    query: Option<Query>,
    subscriber_id: Option<SubscriberId>,
}

#[derive(Debug, Clone, Serialize)]
pub enum SubscribeResponseStatus {
    Ok,
    Subscribed,
    Unsubscribed,
    Error,
    NotSubscribed,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscribeResponse {
    status: SubscribeResponseStatus,
    entity_name: Option<String>,
    data: Option<Value>,
    message: Option<String>,
    subscriber_id: Option<SubscriberId>,
    event_type: Option<EventType>,
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
    let mut current_subscriptions = Vec::new();

    // This task will send messages *to the client* from the mpsc receiver.
    let mut session_clone = session.clone();
    rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let response = SubscribeResponse {
                data: Some(msg.value),
                status: SubscribeResponseStatus::Ok,
                entity_name: Some(msg.entity_name.to_string()),
                message: None,
                subscriber_id: Some(msg.subscriber_id.clone()),
                event_type: Some(msg.event_type.clone()),
            };
            if session_clone
                .text(serde_json::to_string(&response).unwrap())
                .await
                .is_err()
            {
                let error_response = SubscribeResponse {
                    data: None,
                    status: SubscribeResponseStatus::Error,
                    entity_name: None,
                    message: Some("Broker error".to_string()),
                    subscriber_id: Some(msg.subscriber_id.clone()),
                    event_type: Some(msg.event_type.clone()),
                };
                match session_clone
                    .text(serde_json::to_string(&error_response).unwrap())
                    .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        log::error!("Fatal Broker Error: {}", err);
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
                                status: SubscribeResponseStatus::Error,
                                message: Some(format!("Error parsing JSON: {}", err)),
                                subscriber_id: None,
                                event_type: None,
                            };
                            session
                                .text(serde_json::to_string(&response).unwrap())
                                .await
                                .unwrap();
                            continue;
                        }
                    };
                    let entity = Entity::new(&subscribe_options.entity_name);

                    match subscribe_options.action {
                        SubscribeAction::Subscribe => {
                            let subscriber = Subscriber::new(tx.clone());
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
                                status: SubscribeResponseStatus::Subscribed,
                                entity_name: Some(entity.name.to_string()),
                                message: Some(format!(
                                    "Successfully subscribed to entity {}",
                                    entity.name
                                )),
                                subscriber_id: Some(subscriber.id.clone()),
                                event_type: None,
                            };

                            current_subscriptions.push(subscriber.id);

                            session
                                .text(serde_json::to_string(&success_response).unwrap())
                                .await
                                .unwrap();
                        }
                        SubscribeAction::Unsubscribe => {
                            let subscriber_id = subscribe_options.subscriber_id;
                            if subscriber_id.is_none() {
                                let error_response = SubscribeResponse {
                                    data: None,
                                    status: SubscribeResponseStatus::Error,
                                    entity_name: None,
                                    message: Some(
                                        "Subscriber ID is required for unsubscribe".to_string(),
                                    ),
                                    subscriber_id: None,
                                    event_type: None,
                                };

                                session
                                    .text(serde_json::to_string(&error_response).unwrap())
                                    .await
                                    .unwrap();
                            }
                            let subscriber_id = subscriber_id.unwrap();

                            // Check if user is subscribed
                            let is_subscribed = current_subscriptions.contains(&subscriber_id);
                            if !is_subscribed {
                                let error_response = SubscribeResponse {
                                    data: None,
                                    status: SubscribeResponseStatus::NotSubscribed,
                                    entity_name: None,
                                    message: Some(
                                        "Not subscribed to current Subscriber ID.".to_string(),
                                    ),
                                    subscriber_id: Some(subscriber_id),
                                    event_type: None,
                                };
                                session
                                    .text(serde_json::to_string(&error_response).unwrap())
                                    .await
                                    .unwrap();
                                continue;
                            }

                            current_subscriptions.retain(|s| *s != subscriber_id);

                            broker.unsubscribe(&subscriber_id).await;

                            let success_response = SubscribeResponse {
                                data: None,
                                status: SubscribeResponseStatus::Unsubscribed,
                                entity_name: None,
                                message: Some("Unsubscribed successfully".to_string()),
                                subscriber_id: Some(subscriber_id),
                                event_type: None,
                            };

                            session
                                .text(serde_json::to_string(&success_response).unwrap())
                                .await
                                .unwrap();
                        }
                    }
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }
                Ok(AggregatedMessage::Close(_)) => {
                    log::info!("Unsubscribing from: {:?}", current_subscriptions);
                    for subscriber_id in current_subscriptions.iter() {
                        broker.unsubscribe(&subscriber_id).await;
                    }
                }
                _ => {
                    log::warn!("Unknown message type received");
                }
            }
        }
    });

    Ok(res)
}
