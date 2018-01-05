use uuid::Uuid;
use ws::{Error, ErrorKind, Handler, Sender, Result as WSResult, Message, CloseCode, Handshake as WSHandshake, listen};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use std::io::{Error as IOError, ErrorKind as IOErrorKind};

use websocket_api::identifier;
use websocket_api::handshake::*;
use websocket_api::project::{ProjectRequest, ProjectRequestError, Project as SerializableProject};
use websocket_api::serialize;
use websocket_api::Deserialize;

type WrappedProjectsHandler = Rc<RefCell<ProjectsHandler>>;

struct Project {
    id: Uuid,
    clients: HashMap<Uuid, Sender>,
}

impl Project {
    fn new() -> Project {
        Project::with_id(Uuid::new_v4())
    }

    fn with_id(id: Uuid) -> Project {
        Project {
            id: id,
            clients: HashMap::new(),
            // current_version: String::from("\\documentclass{report}\n\n\\begin{document}\n\tHello world!\n\\end{document}")
        }
    }

    fn broadcast(&self, data: Vec<u8>) {
        for (client_id, tx) in &self.clients {
            if let Err(e) = tx.send(data.clone()) {
                debug!("Error during broadcast to client {}: {:?}", client_id.simple().to_string(), e);
            }
        }
    }

    fn add_client(&mut self, client: &ClientHandler) {
        info!("Client {} joined project {}", client.id.simple().to_string(), self.id.simple().to_string());
        self.clients.insert(client.id, client.tx.clone());
    }

    fn remove_client(&mut self, id: &Uuid) {
        self.broadcast(vec![1, 2, 3, 4, 5, 6, 42]);
        info!("Client {} left project {}", id.simple().to_string(), self.id.simple().to_string());
        self.clients.remove(id);
    }
}

struct ProjectsHandler {
    projects: HashMap<Uuid, Project>
}

impl ProjectsHandler {
    fn new() -> ProjectsHandler {
        let mut projects = HashMap::new();

        let pid1 = Uuid::parse_str("deadbeef-dead-beef-dead-beefdeadbeef").unwrap();
        let pid2 = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        projects.insert(pid1, Project::with_id(pid1));
        projects.insert(pid2, Project::with_id(pid2));

        ProjectsHandler { projects }
    }

    fn join_project(&mut self, project_id: &Uuid, client: &ClientHandler) -> Result<(), &'static str> {
        if let Some(project) = self.projects.get_mut(project_id) {
            project.add_client(client);
            Ok(())
        } else {
            Err("Project does not exist!")
        }
    }

    fn leave_project(&mut self, project_id: &Uuid, client_id: &Uuid) {
        if let Some(project) = self.projects.get_mut(project_id) {
            project.remove_client(client_id);
        }
    }
}

pub struct ClientHandler {
    id: Uuid,
    tx: Sender,
    projects: WrappedProjectsHandler,
    joined_project: Option<Uuid>,
    handshake_completed: bool
}

impl ClientHandler {
    fn process_binary_message<'a, F, T: Deserialize<'a>>(data: &'a Vec<u8>, mut processor: F)
        -> Result<(), Error> where F : FnMut(T) -> Result<(), Error> {
        match serialize::deserialize::<T>(data) {
            Ok(deserialized_data) => processor(deserialized_data),
            Err(_) => Err(Error::new(ErrorKind::Io(IOError::new(IOErrorKind::InvalidData, "Unable to deserialize data")), "Unable to deserialize data"))
        }
    }

    fn handle_binary_message(&mut self, mut data: Vec<u8>) -> Result<(), Error> {
        if let Some(id) = data.pop() {

            if id != identifier::HANDSHAKE && !self.handshake_completed {
                let error = HandshakeError::new(String::from("Handshake not completed"));
                return self.tx.send(Message::binary(error.serialize()));
            }

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
                        let mut projects = self.projects.borrow_mut();

                        // Leave old project if applicable
                        if let Some(project_id) = self.joined_project {
                            projects.leave_project(&project_id, &self.id);
                        }

                        // Send new project
                        self.tx.send(
                            match projects.join_project(&request.id, &self) {
                                Ok(_) => Message::binary(SerializableProject::new(request.id, String::from("ProtoMesh")).serialize()),
                                Err(e) => Message::binary(ProjectRequestError::new(String::from(e)).serialize())
                            }
                        )
                    })
                }
                _ => Ok(())
            }
        } else {
            Ok(())
        }
    }
}

impl Handler for ClientHandler {

    fn on_open(&mut self, _: WSHandshake) -> WSResult<()> {
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> WSResult<()> {
        match msg {
            Message::Text(_) => Ok(()),
            Message::Binary(data) => self.handle_binary_message(data)
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        if let Some(project_id) = self.joined_project {
            self.projects.borrow_mut().leave_project(&project_id, &self.id);
        }

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
    let projects = WrappedProjectsHandler::new(RefCell::new(ProjectsHandler::new()));

    listen(addr, |tx| {
        ClientHandler {
            id: Uuid::new_v4(),
            tx: tx,
            projects: projects.clone(),
            joined_project: None,
            handshake_completed: false
        }
    }).unwrap();
}
