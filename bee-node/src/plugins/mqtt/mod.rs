// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod manager;
mod topics;

pub mod config;

use std::{any::Any, convert::Infallible};

use async_trait::async_trait;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::event::{LatestMilestoneChanged, SolidMilestoneChanged};
use futures::stream::StreamExt;
use log::{debug, warn};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use self::{config::MqttConfig, manager::MqttManager, topics::*};

#[derive(Default)]
pub struct Mqtt;

fn topic_handler<N, E, T, P, F>(node: &mut N, topic: &'static str, f: F)
where
    N: Node,
    E: Any + Clone + Send + Sync,
    T: Into<String> + Send,
    P: Into<Vec<u8>> + Send,
    F: 'static + Fn(&E) -> (T, P) + Send + Sync,
{
    let bus = node.bus();
    let manager = node.resource::<MqttManager>();
    let (tx, rx) = mpsc::unbounded_channel();

    node.spawn::<Mqtt, _, _>(|shutdown| async move {
        debug!("Mqtt {} topic handler running.", topic);

        let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

        while let Some(event) = receiver.next().await {
            let (topic, payload) = f(&event);
            manager.send(topic, payload).await;
        }

        debug!("Mqtt {} topic handler stopped.", topic);
    });

    bus.add_listener::<Mqtt, _, _>(move |event: &E| {
        if tx.send((*event).clone()).is_err() {
            warn!("Sending event to mqtt {} topic handler failed.", topic)
        }
    });
}

#[async_trait]
impl<N: Node> Worker<N> for Mqtt {
    type Config = MqttConfig;
    type Error = Infallible;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        match MqttManager::new(config) {
            Ok(manager) => {
                // TODO log connected
                node.register_resource(manager);

                topic_handler(node, TOPIC_MILESTONES_LATEST, |_event: &LatestMilestoneChanged| {
                    (TOPIC_MILESTONES_LATEST, "")
                });
                topic_handler(node, TOPIC_MILESTONES_SOLID, |_event: &SolidMilestoneChanged| {
                    (TOPIC_MILESTONES_SOLID, "")
                });
                // topic_handler(node, _TOPIC_MESSAGES, |_event: &_| (_TOPIC_MESSAGES, ""));
                // topic_handler(node, _TOPIC_MESSAGES_REFERENCED, |_event: &_| {
                //     (_TOPIC_MESSAGES_REFERENCED, "")
                // });
                // topic_handler(node, _TOPIC_MESSAGES_INDEXATION, |_event: &_| {
                //     (_TOPIC_MESSAGES_INDEXATION, "")
                // });
                // topic_handler(node, _TOPIC_MESSAGES_METADATA, |_event: &_| {
                //     (_TOPIC_MESSAGES_METADATA, "")
                // });
                // topic_handler(node, _TOPIC_OUTPUTS, |_event: &_| (_TOPIC_OUTPUTS, ""));
                // topic_handler(node, _TOPIC_ADDRESSES_OUTPUTS, |_event: &_| {
                //     (_TOPIC_ADDRESSES_OUTPUTS, "")
                // });
                // topic_handler(node, _TOPIC_ADDRESSES_ED25519_OUTPUT, |_event: &_| {
                //     (_TOPIC_ADDRESSES_ED25519_OUTPUT, "")
                // });
            }
            Err(_e) => {
                // error!("Creating mqtt manager failed {:?}.", e);
            }
        }

        Ok(Self::default())
    }
}
