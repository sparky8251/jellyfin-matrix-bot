//! Handler for the unit conversion command

use crate::helpers::convert_unit;
use crate::helpers::MatrixNoticeResponse;
use crate::messages::{MatrixMessage, MatrixMessageType};
use crate::regex::UNIT_CONVERSION;
use ruma::{
    events::room::message::{Relation, TextMessageEventContent},
    RoomId,
};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error};

/// Command based unit conversion handler that will parse, generate a response body, and send it
pub(super) async fn unit_conversion_handler(
    text: &TextMessageEventContent,
    relates_to: Option<&Relation>,
    room_id: &RoomId,
    send: &mut Sender<MatrixMessage>,
) {
    if relates_to.is_none() && text.formatted.is_none() {
        let mut conversions = Vec::new();
        for cap in UNIT_CONVERSION.captures_iter(&text.body.to_lowercase()) {
            conversions.push((cap[1].to_string(), cap[2].to_string()));
        }
        let result = match convert_unit(conversions) {
            Some(v) => v,
            None => {
                debug!("No convertable units found. No reply will be constructed.");
                return;
            }
        };
        let mut response = MatrixNoticeResponse::default();
        response.set_unit_conversions(result);
        match send
            .send(MatrixMessage {
                room_id: room_id.clone(),
                message: MatrixMessageType::Notice(response.to_string()),
            })
            .await
        {
            Ok(_) => (),
            Err(_) => error!("Channel closed. Unable to send message."),
        };
    }
}
