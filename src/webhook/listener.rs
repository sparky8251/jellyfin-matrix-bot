use crate::config::{Config, WebhookListenerConfig};
use crate::messages::MatrixMessage;
use crate::webhook_handlers::register_handlers;
use rocket::config::{Config as RocketConfig, LogLevel};
use tokio::sync::mpsc::Sender;

pub struct WebhookListener {
    send: Sender<MatrixMessage>,
    config: WebhookListenerConfig,
}

impl WebhookListener {
    pub fn new(config: &Config, send: Sender<MatrixMessage>) -> Self {
        let config = WebhookListenerConfig {
            token: config.webhook_token.clone(),
        };
        WebhookListener { send, config }
    }

    pub async fn start(self) {
        let rocket_config = RocketConfig {
            log_level: LogLevel::Off,
            port: 33333,
            ..RocketConfig::release_default()
        };

        let rocket = rocket::custom(rocket_config);
        match register_handlers(rocket)
            .manage(self.send)
            .manage(self.config)
            .launch()
            .await
        {
            Ok(_) => (),
            Err(e) => panic!("Unable to launch webhook listener due to error {:?}", e),
        }
    }
}
