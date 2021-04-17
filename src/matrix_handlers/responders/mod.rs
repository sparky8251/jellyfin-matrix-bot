use crate::config::ResponderStorage;
use ruma::events::AnyMessageEventContent;
use ruma::{
    api::client::r0::{
        membership::{join_room_by_id, leave_room},
        message::send_message_event,
    },
    events::room::message::MessageEventContent,
    RoomId, UserId,
};
use ruma_client::Client;
use tracing::{debug, error, info};

pub async fn send_notice(
    client: &Client,
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
) {
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_plain(message));
    let next_txn_id = storage.next_txn_id();
    let req = send_message_event::Request::new(room_id, &next_txn_id.as_str(), &content);
    match client.request(req).await {
        Ok(_) => (),
        Err(e) => error!("{:?}", e),
    }
}
pub async fn send_plain_text(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    client: &Client,
) {
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(message));
    let next_txn_id = storage.next_txn_id();
    let req = send_message_event::Request::new(room_id, &next_txn_id.as_str(), &content);
    match client.request(req).await {
        Ok(_) => (),
        Err(e) => error!("Unable to send response due to error {:?}", e),
    }
}

pub async fn send_formatted_text(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &Client,
) {
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::text_html(
        message,
        formatted_message.unwrap_or_default(),
    ));
    let next_txn_id = storage.next_txn_id();
    let req = send_message_event::Request::new(room_id, &next_txn_id.as_str(), &content);
    match client.request(req).await {
        Ok(_) => (),
        Err(e) => error!("Unable to send response due to error {:?}", e),
    }
}

pub async fn send_formatted_notice(
    room_id: &RoomId,
    storage: &mut ResponderStorage,
    message: String,
    formatted_message: Option<String>,
    client: &Client,
) {
    let content = AnyMessageEventContent::RoomMessage(MessageEventContent::notice_html(
        message,
        formatted_message.unwrap_or_default(),
    ));
    let next_txn_id = storage.next_txn_id();
    let req = send_message_event::Request::new(room_id, &next_txn_id.as_str(), &content);
    match client.request(req).await {
        Ok(_) => (),
        Err(e) => error!("{:?}", e),
    }
}

pub async fn accept_invite(sender: &UserId, room_id: &RoomId, client: &Client) {
    info!("Authorized user {} invited me to room {}", sender, room_id);
    let response = client.request(join_room_by_id::Request::new(room_id)).await;
    match response {
        Ok(_) => info!("Successfully joined room {}", room_id),
        Err(e) => debug!("Unable to join room {} because of error {:?}", room_id, e),
    }
}

/// Will reject an invite and print the user that tried to logs
pub async fn reject_invite(sender: &UserId, room_id: &RoomId, client: &Client) {
    let response = client.request(leave_room::Request::new(room_id)).await;
    match response {
        Ok(_) => info!("Rejected invite from unathorized user {}", sender),
        Err(e) => debug!("Unable to reject invite this loop because of error {:?}", e),
    }
}
