use uuid::Uuid;

use websocket_api::{identifier, serialize};
use websocket_api::handshake::*;

pub struct Client {
	pub id: Uuid,
	pub handshake_completed: bool
}

impl Client {
	pub fn new() -> Client {
		Client {
			id: Uuid::new_v4(),
			handshake_completed: false,
		}
	}

	// pub fn handle_message(&mut self, msg: OwnedMessage) -> Option<OwnedMessage> {
	// 	debug!("Message from Client: {:?}", msg);
	// 	match msg {
	// 		OwnedMessage::Ping(p) => Some(OwnedMessage::Pong(p)),
	// 		OwnedMessage::Pong(_) => None,
	// 		OwnedMessage::Binary(mut data) => {
	// 			debug!("Binary message from Client!");
    //
	// 			match data.pop() {
	// 				Some(id) => {
	// 					match id {
	// 						identifier::HANDSHAKE => {
	// 							let handshake: Handshake = serialize::deserialize(&data).expect("Invalid data");
    //
	// 							if handshake.protocol_version != String::from(identifier::PROTOCOL_VERSION) {
	// 								let error = HandshakeError::new(String::from("Invalid protocol version"));
	// 								Some(Message::binary(error.serialize()).into())
	// 							} else {
	// 								self.handshake_completed = true;
	// 								let acknowledgement = HandshakeAcknowledgement::new(self.id);
	// 								Some(Message::binary(acknowledgement.serialize()).into())
	// 							}
	// 						},
	// 						_ => None
	// 					}
	// 				},
	// 				_ => None
	// 			}
	// 		},
	// 		_ => None,
	// 	}
	// }
}
