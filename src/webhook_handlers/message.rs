use crate::config::WebhookListenerConfig;
use crate::helpers::MatrixFormattedTextResponse;
use crate::messages::{MatrixFormattedMessage, MatrixMessage, MatrixMessageType};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::State;
use rocket_contrib::json::Json;
use ruma::{RoomId, UserId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::sync::mpsc::Sender;

#[post("/", data = "<message>")]
pub async fn message(
    req_token: MessageToken,
    message: Json<Message>,
    conf: State<'_, WebhookListenerConfig>,
    send: State<'_, Sender<MatrixMessage>>,
) -> Status {
    if req_token.0.eq(&conf.token) {
        let matrix_message = MatrixMessage {
            room_id: message.room_id.clone(),
            message: MatrixMessageType::Notice(message.message.clone()),
        };
        match send.clone().send(matrix_message).await {
            Ok(_) => (),
            Err(_) => return Status::InternalServerError,
        };
        if let Some(pings) = &message.ping {
            let mut response = MatrixFormattedTextResponse::default();
            let pings: HashSet<UserId> = pings.iter().cloned().collect();
            response.set_users(pings);
            let matrix_message = MatrixMessage {
                room_id: message.room_id.clone(),
                message: MatrixMessageType::FormattedText(MatrixFormattedMessage {
                    plain_text: response.to_string(),
                    formatted_text: response.format_text(),
                }),
            };
            match send.clone().send(matrix_message).await {
                Ok(_) => (),
                Err(_) => return Status::InternalServerError,
            };
        };
        Status::Ok
    } else {
        Status::Unauthorized
    }
}

#[derive(Debug, Deserialize)]
pub struct Message {
    room_id: RoomId,
    message: String,
    ping: Option<Vec<UserId>>,
}

#[derive(Debug, Deserialize)]
pub struct MessageToken(String);

#[derive(Debug, Serialize)]
pub enum MessageError {
    NoToken,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for MessageToken {
    type Error = MessageError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if req.headers().contains("X-Webhook-Token") {
            let token = req.headers().get_one("X-Webhook-Token").unwrap();
            request::Outcome::Success(MessageToken(token.to_string()))
        } else {
            request::Outcome::Failure((Status::Unauthorized, MessageError::NoToken))
        }
    }
}
