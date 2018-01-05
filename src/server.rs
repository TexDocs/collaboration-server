use uuid::Uuid;
use ws::{Error, ErrorKind, Handler, Sender, Result as WSResult, Message, CloseCode, Handshake as WSHandshake, listen};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use websocket_api::identifier;
use websocket_api::handshake::*;
use websocket_api::project::*;
use websocket_api::serialize;
use websocket_api::Deserialize;

type WrappedProjectHandler = Rc<RefCell<ProjectHandler>>;

struct ProjectHandler {
    clients: HashMap<Uuid, Sender>
}

impl ProjectHandler {
    fn new() -> ProjectHandler {
        ProjectHandler {
            clients: HashMap::new()
        }
    }

    fn print_count(&self) {
        info!("Connected clients: {:?}", self.clients.len());
    }

    fn add_client(&mut self, client: &ClientHandler) {
        self.clients.insert(client.id, client.tx.clone());
        self.print_count();
    }

    fn remove_client(&mut self, id: &Uuid) {
        self.clients.remove(id);
        self.print_count();
    }
}

pub struct ClientHandler {
    id: Uuid,
    tx: Sender,
    project: WrappedProjectHandler,
    handshake_completed: bool
}

impl ClientHandler {
    fn process_binary_message<'a, F, T: Deserialize<'a>>(data: &'a Vec<u8>, mut processor: F)
        -> Result<(), Error> where F : FnMut(T) -> Result<(), Error> {

        match serialize::deserialize::<T>(data) {
            Ok(deserialized_data) => processor(deserialized_data),
            Err(_) => Err(Error::new(ErrorKind::Protocol, "Unable to deserialize data"))
        }

    }

    fn handle_binary_message(&mut self, mut data: Vec<u8>) -> Result<(), Error> {
        match data.pop() {
			Some(id) => {
				match id {
					identifier::HANDSHAKE => {
                        ClientHandler::process_binary_message(&data, |handshake: Handshake| {
                            if handshake.protocol_version != String::from(identifier::PROTOCOL_VERSION) {
                                let error = HandshakeError::new(String::from("Invalid protocol version"));
                                self.tx.send(Message::binary(error.serialize()))
                            } else {
                                self.handshake_completed = true;
                                let acknowledgement = HandshakeAcknowledgement::new(self.id);
                                self.tx.send(Message::binary(acknowledgement.serialize()))
                            }
                        })
					},
                    identifier::PROJECT_REQUEST => {
                        ClientHandler::process_binary_message(&data, |request: ProjectRequest| {
                            self.tx.send(Message::binary(Project::mock().serialize()))
                        })
                    }
					_ => Ok(())
				}
			},
			_ => Ok(())
		}
    }
}

impl Handler for ClientHandler {

    fn on_open(&mut self, _: WSHandshake) -> WSResult<()> {
        self.project.borrow_mut().add_client(&self);
        // info!("{:?}", self.project.borrow().len());
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> WSResult<()> {
        match msg {
            Message::Text(_) => Ok(()),
            Message::Binary(data) => self.handle_binary_message(data)
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        self.project.borrow_mut().remove_client(&self.id);

        // The WebSocket protocol allows for a utf8 reason for the closing state after the
        // close code. WS-RS will attempt to interpret this data as a utf8 description of the
        // reason for closing the connection. I many cases, `reason` will be an empty string.
        // So, you may not normally want to display `reason` to the user,
        // but let's assume that we know that `reason` is human-readable.
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away   => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {} {:?}", reason, code),
        }
    }
}

pub fn launch_server(addr: &'static str) {
    let project = WrappedProjectHandler::new(RefCell::new(ProjectHandler::new()));

    listen(addr, |tx| {
        ClientHandler {
            id: Uuid::new_v4(),
            tx: tx,
            project: project.clone(),
            handshake_completed: false
        }
    }).unwrap();
}
