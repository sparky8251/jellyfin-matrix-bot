mod github;
mod message;

// use github::event as event_fn;
use message::message as message_fn;
use rocket::{Build, Rocket};

pub fn register_handlers(rocket: Rocket<Build>) -> Rocket<Build> {
    rocket.mount("/message", routes![message_fn])
    // .mount("/event", routes![event_fn])
}
