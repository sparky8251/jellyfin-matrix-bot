// use crate::events::Event;
// use crate::messages::{MatrixMessage, MatrixMessageType};
// use rocket::http::Status;
// use rocket::request::{FromRequest, Outcome, Request};
// use rocket::State;
// use rocket_contrib::json::Json;
// use ruma::RoomId;
// use std::convert::TryFrom;
// use tokio::sync::mpsc::Sender as TokioSender;
// use tracing::{debug, error, info};
// use hmac::{Hmac, Mac, NewMac};
// use sha2::{Sha256};

// type HmacSha256 = Hmac<Sha256>;

// #[derive(Debug)]
// pub struct SignedPayload<T> {
//     inner: T
// }

// #[derive(Debug)]
// pub enum AccessTokenError {
//     #[allow(dead_code)]
//     InvalidToken,
//     MissingToken,
//     BadCount,
// }

// Issue # for validating token: https://github.com/SergioBenitez/Rocket/issues/775

// #[post("/", data = "<event>")]
// pub async fn event(event: SignedPayload<Json<Event>>, send: State<'_, TokioSender<MatrixMessage>>) -> Status {
//     // Verify provided token
//     // let mut mac = HmacSha256::new_varkey(b"testing").expect("HMAC can take a key of any size");
//     // mac.update(b"testing"); // TODO: get request body data as bytes here
//     // let result = mac.finalize().into_bytes().to_vec();
//     // let valid_token = format!("sha256={}", hex::encode(&result));
//     // debug!("Access Token is {}", access_token.0);
//     // debug!("Valid Token is {}", valid_token);
//     // if !access_token.0.eq(&valid_token) {
//     //     return Status::Unauthorized
//     // }
//     // Handle event after auth succeeded.
//     match event.inner.clone() {
//         Event::Release {
//             action,
//             release,
//             repository,
//             ..
//         } => {
//             if !action.eq_ignore_ascii_case("published") {
//                 debug!("Recieved release event that was not of type published");
//                 return Status::Ok;
//             }
//             let url = release.html_url;
//             let repo_name = repository.name.replace("-", " ").trim().to_string(); // TODO: Determine how to make the names work better given our repetitive naming sense
//             let release_name = match release.name {
//                 Some(v) => v,
//                 None => {
//                     error!(
//                         "No release name has been set. Unable to announce release for event {:?}",
//                         event
//                     );
//                     return Status::NotImplemented;
//                 }
//             };
//             let prerelease = if release.prerelease {
//                 "pre".to_string()
//             } else {
//                 String::new()
//             };
//             match release.body {
//                 Some(_) => (),
//                 None => {
//                     error!(
//                         "No release body has been set. Unable to annouce release for event {:?}",
//                         event
//                     );
//                     return Status::NotImplemented;
//                 }
//             };

//             let message = format!(
//                 "A new {}release has been made for {}! {} is ready for using.\n\nRead more here: {}\nFeel free to head on over here to discuss: https://old.reddit.com/r/jellyfin",
//                 prerelease, repo_name, release_name, url
//             );
//             let room_id = match RoomId::try_from("!KQLCpaQglvHLTKqgPC:matrix.org") {
//                 //FIXME: This should be configurable
//                 Ok(v) => v,
//                 Err(_) => panic!("This should never happen! Failed to parse hard coded room id!"),
//             };
//             match send
//                 .clone()
//                 .send(MatrixMessage {
//                     room_id,
//                     message: MatrixMessageType::Text(message),
//                 })
//                 .await
//             {
//                 Ok(_) => debug!("Announcement message sent to matrix"),
//                 Err(_) => error!("Channel closed. Unable to send message"),
//             };
//             Status::Ok
//         }
//         Event::Public { repository, .. } => {
//             info!("Recieved webhook ping from repo {}", repository.name);
//             Status::Ok
//         }
//         _ => Status::NotFound,
//     }
// }
