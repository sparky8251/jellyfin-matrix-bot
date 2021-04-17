//! Structs and functions that represent functional bots and allow for easy loading
//! plus main loop initialization.

use crate::config::{Config, ListenerStorage, MatrixListenerConfig};
use crate::matrix_handlers::listeners::{handle_invite_event, handle_text_event};
use crate::messages::MatrixMessage;
use ruma::{
    api::client::r0::sync::sync_events,
    events::{
        room::message::{MessageEventContent, MessageType, Relation},
        AnyStrippedStateEvent, AnySyncMessageEvent, AnySyncRoomEvent, SyncMessageEvent,
    },
    presence::PresenceState,
};
use ruma_client::Client;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, trace};

/// Struct representing all required data for a functioning bot instance.
pub struct MatrixListener {
    /// Storage data.
    pub storage: ListenerStorage,
    /// Configuration data.
    pub config: MatrixListenerConfig,
    /// Reqwest client used for external API calls.
    pub api_client: reqwest::Client,
    send: Sender<MatrixMessage>,
}

impl MatrixListener {
    /// Loads storage data, config data, and then creates a reqwest client and then returns a Bot instance.
    pub fn new(config: &Config, send: Sender<MatrixMessage>) -> Self {
        let storage = ListenerStorage::load_storage();
        let config = MatrixListenerConfig::new(&config);
        let api_client = reqwest::Client::new();
        Self {
            storage,
            config,
            api_client,
            send,
        }
    }

    /// Used to start main program loop for the bot.
    /// Will login then loop forever while waiting on new sync data from the homeserver.
    pub async fn start(&mut self, client: Client) {
        loop {
            let req = assign!(sync_events::Request::new(),
                {
                    filter: None,
                    since: match &self.storage.last_sync {
                        Some(v) => Some(v.as_str()),
                        None => None
                    },
                    full_state: false,
                    set_presence: &PresenceState::Unavailable,
                    timeout: Some(Duration::new(30, 0))
                }
            );
            let response = match client.request(req).await {
                Ok(v) => Some(v),
                Err(e) => {
                    debug!("Line 73: {:?}", e);
                    None
                }
            };

            match response {
                Some(v) => {
                    for (room_id, joined_room) in &v.rooms.join {
                        for raw_event in &joined_room.timeline.events {
                            let event = raw_event.deserialize();
                            match event {
                                Ok(AnySyncRoomEvent::Message(
                                    AnySyncMessageEvent::RoomMessage(SyncMessageEvent {
                                        content:
                                            MessageEventContent {
                                                msgtype: MessageType::Text(t),
                                                relates_to,
                                                ..
                                            },
                                        sender,
                                        ..
                                    }),
                                )) => {
                                    if matches!(relates_to, Some(Relation::Replacement(_))) {
                                        debug!("Message is an edit, skipping handling");
                                        continue;
                                    }
                                    handle_text_event(
                                        &t,
                                        relates_to.as_ref(),
                                        &sender,
                                        room_id,
                                        &mut self.storage,
                                        &self.config,
                                        &self.api_client,
                                        &mut self.send,
                                    )
                                    .await;
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    debug!("{:?}", e);
                                    trace!("Content: {:?}", raw_event.json())
                                }
                            }
                            self.storage.last_sync = Some(v.next_batch.clone());
                            self.storage.save_storage();
                        }
                    }
                    for (room_id, invited_room) in &v.rooms.invite {
                        trace!("Invited room data: {:?}", invited_room);
                        for raw_event in &invited_room.invite_state.events {
                            let event = raw_event.deserialize();
                            match event {
                                Ok(AnyStrippedStateEvent::RoomMember(s)) => {
                                    trace!("Invited by {}", s.sender);
                                    handle_invite_event(
                                        &s.sender,
                                        &room_id,
                                        &self.config,
                                        &mut self.send,
                                    )
                                    .await;
                                    trace!("Handled invite event")
                                }
                                Ok(_) => {
                                    // FIXME: Reject invite if there is no known sender
                                    error!("No known inviter. Will not join room. If you see this, report it.");
                                }
                                Err(e) => {
                                    debug!("{:?}", e);
                                    trace!("Content: {:?}", raw_event.json())
                                }
                            }
                        }
                    }
                }
                None => debug!("Response deserialization failed. Doing nothing this loop."),
            }
        }
    }
}
