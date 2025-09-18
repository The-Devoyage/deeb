use std::{collections::HashMap, sync::Arc};

use deeb::{EntityName, Query};
use serde_json::Value;
use tokio::sync::{Mutex, mpsc};

#[derive(Debug, Clone, PartialEq)]
pub struct SubscriberId(ulid::Ulid);

#[derive(Debug, Clone)]
pub struct Subscriber {
    pub id: SubscriberId,
    pub sender: mpsc::Sender<Vec<Value>>,
}

impl Subscriber {
    pub fn new(sender: mpsc::Sender<Vec<Value>>) -> Self {
        let subscriber_id = ulid::Ulid::new();
        Subscriber {
            id: SubscriberId(subscriber_id),
            sender,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Subscription {
    entity_name: EntityName,
    query: Query,
}

impl Subscription {
    pub fn new(entity_name: &EntityName, query: &Query) -> Self {
        Subscription {
            entity_name: entity_name.clone(),
            query: query.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Broker {
    clients: Arc<Mutex<HashMap<Subscription, Vec<Subscriber>>>>,
}

impl Broker {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Subscribe to a query
    pub async fn subscribe(
        &self,
        entity_name: &EntityName,
        query: &Query,
        subscriber: &Subscriber,
    ) {
        let mut clients = self.clients.lock().await;
        let subscription = Subscription::new(entity_name, query);
        clients
            .entry(subscription.clone())
            .or_insert(Vec::new())
            .push(subscriber.clone());
    }

    // Unsubscribe from a query
    pub async fn unsubscribe(&self, subscriber_id: &SubscriberId) {
        let mut clients = self.clients.lock().await;
        for (_, subscribers) in clients.iter_mut() {
            if let Some(index) = subscribers.iter().position(|s| s.id == *subscriber_id) {
                subscribers.remove(index);
            }
        }
        clients.retain(|_subscription, subscribers| !subscribers.is_empty());
    }

    // Publish an event to all subscribers
    pub async fn publish<T>(&self, entity_name: &T, values: Vec<Value>) -> Result<(), anyhow::Error>
    where
        T: Into<EntityName> + Clone,
    {
        let clients = self.clients.lock().await;
        let subscriptions = clients.keys().cloned().collect::<Vec<_>>();
        for subscription in subscriptions {
            if subscription.entity_name == entity_name.clone().into() {
                let mut matched_values = Vec::new();
                for value in values.iter() {
                    let should_publish = subscription.query.matches(&value)?;
                    if should_publish {
                        matched_values.push(value.clone());
                    }
                }
                if let Some(subscribers) = clients.get(&subscription) {
                    for subscriber in subscribers {
                        subscriber.sender.send(matched_values.clone()).await?;
                    }
                }
            }
        }
        Ok(())
    }
}
