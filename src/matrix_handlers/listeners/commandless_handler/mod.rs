//! Contains handlers and response functions for text based non-command events

mod github_search;
mod group_ping;
mod link_url;
mod spellcheck;
mod unit_conversion;

use crate::config::{ListenerStorage, MatrixListenerConfig};
use crate::helpers::{check_format, MatrixFormattedTextResponse, MatrixNoticeResponse};
use crate::messages::{MatrixFormattedMessage, MatrixMessage, MatrixMessageType};
use crate::regex::{GITHUB_SEARCH, GROUP_PING, LINK_URL, UNIT_CONVERSION};
use github_search::github_search;
use group_ping::group_ping;
use link_url::link_url;
use ruma::{
    events::room::message::{Relation, TextMessageEventContent},
    RoomId, UserId,
};
use spellcheck::spellcheck;
use std::time::SystemTime;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, trace};
use unit_conversion::unit_conversion;

/// Handler for all text based non-command events
pub(super) async fn commandless_handler(
    text: &TextMessageEventContent,
    relates_to: Option<&Relation>,
    sender: &UserId,
    room_id: &RoomId,
    storage: &mut ListenerStorage,
    config: &MatrixListenerConfig,
    api_client: &reqwest::Client,
    send: &mut Sender<MatrixMessage>,
) {
    if sender == &config.mx_uname {
        // do nothing if message is from self
        trace!("Message is from self, doing nothing");
    } else {
        match check_format(text.formatted.as_ref().map(|f| &f.format)) {
            Ok(_) => {
                let mut notice_response = MatrixNoticeResponse::default();
                let mut text_response = MatrixFormattedTextResponse::default();
                if UNIT_CONVERSION.is_match(&text.body) && config.enable_unit_conversions {
                    debug!("Entering commandless unit conversion path");
                    unit_conversion(&text, &config, &mut notice_response);
                }
                if GITHUB_SEARCH.is_match(&text.body) && !config.repos.is_empty() {
                    debug!("Entering commandless github search path");
                    github_search(&text, &config, &api_client, &mut notice_response).await;
                }
                if LINK_URL.is_match(&text.body)
                    && !config.links.is_empty()
                    && !config.linkers.is_empty()
                {
                    debug!("Entering commandless url linking path");
                    link_url(&text, &config, &mut notice_response);
                }
                if GROUP_PING.is_match(&text.body) {
                    debug!("Entering commandless group ping path");
                    group_ping(&text, &sender, &config, &mut text_response);
                }

                let notice_response = notice_response;
                let text_response = text_response;

                if notice_response.is_some() {
                    match send
                        .send(MatrixMessage {
                            room_id: room_id.clone(),
                            message: MatrixMessageType::Notice(notice_response.to_string()),
                        })
                        .await
                    {
                        Ok(_) => (),
                        Err(_) => error!("Channel closed. Unable to send message."),
                    };
                }
                if text_response.is_some() {
                    let message = MatrixFormattedMessage {
                        plain_text: text_response.to_string(),
                        formatted_text: text_response.format_text(),
                    };
                    match send
                        .send(MatrixMessage {
                            room_id: room_id.clone(),
                            message: MatrixMessageType::FormattedText(message),
                        })
                        .await
                    {
                        Ok(_) => (),
                        Err(_) => error!("Channel closed. Unable to send message."),
                    };
                }
                if config.enable_corrections
                    && relates_to.is_none()
                    && storage.correction_time_cooldown(room_id)
                    && !config.correction_exclusion.contains(room_id)
                    && !notice_response.is_some()
                    && !text_response.is_some()
                {
                    if let Some(v) = spellcheck(text, sender, config) {
                        match send
                            .send(MatrixMessage {
                                room_id: room_id.clone(),
                                message: MatrixMessageType::Text(v),
                            })
                            .await
                        {
                            Ok(_) => {
                                storage
                                    .last_correction_time
                                    .insert(room_id.clone(), SystemTime::now());
                            }
                            Err(_) => error!("Channel closed. Unable to send message."),
                        };
                    }
                }
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}
