//! Handlers for various event types
//!
//! Sub modules exist for performing various processes such as unit conversion
//! and searching github

mod commandless_handler;
mod help_handler;
mod unit_conversion_handler;

use self::commandless_handler::commandless_handler;
use self::help_handler::help_handler;
use self::unit_conversion_handler::unit_conversion_handler;
use crate::config::{ListenerStorage, MatrixListenerConfig};
use crate::messages::{MatrixInviteMessage, MatrixInviteType, MatrixMessage, MatrixMessageType};
use ruma::{
    events::room::message::{Relation, TextMessageEventContent},
    RoomId, UserId,
};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, trace};

/// Dispatches incoming text events to a number of different handlers depending on various conditions
pub async fn handle_text_event(
    text: &TextMessageEventContent,
    relates_to: Option<&Relation>,
    sender: &UserId,
    room_id: &RoomId,
    storage: &mut ListenerStorage,
    config: &MatrixListenerConfig,
    api_client: &reqwest::Client,
    mut send: &mut Sender<MatrixMessage>,
) {
    if !&text.body.starts_with('!') {
        debug!("Entering no command path...");
        commandless_handler(
            text, relates_to, sender, room_id, storage, config, api_client, &mut send,
        )
        .await
    } else if text.body.to_lowercase().starts_with("!convert ") {
        debug!("Entering unit conversion path...");
        unit_conversion_handler(text, relates_to, room_id, &mut send).await
    } else if text.body.to_lowercase().starts_with("!help") {
        debug!("Entering help path...");
        help_handler(text, room_id, config, &mut send).await
    } else {
        debug!("Doing nothing...");
    }
}

/// Accepts or rejects invites to rooms from matrix users
pub async fn handle_invite_event(
    sender: &UserId,
    room_id: &RoomId,
    config: &MatrixListenerConfig,
    send: &mut Sender<MatrixMessage>,
) {
    trace!("Invited by {} to room {} ", &sender, &room_id);
    if config.admins.contains(&sender) {
        let message = MatrixInviteMessage {
            kind: MatrixInviteType::Accept,
            sender: sender.clone(),
        };
        match send
            .send(MatrixMessage {
                room_id: room_id.clone(),
                message: MatrixMessageType::Invite(message),
            })
            .await
        {
            Ok(_) => (),
            Err(_) => error!("Channel closed. Unable to send message."),
        };
    } else {
        let message = MatrixInviteMessage {
            kind: MatrixInviteType::Reject,
            sender: sender.clone(),
        };
        match send
            .send(MatrixMessage {
                room_id: room_id.clone(),
                message: MatrixMessageType::Invite(message),
            })
            .await
        {
            Ok(_) => (),
            Err(_) => error!("Channel closed. Unable to send message."),
        };
    }
}
