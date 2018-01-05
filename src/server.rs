use uuid::Uuid;
use ws::{Handler, Factory, Sender, Result, Message, CloseCode, Handshake, listen};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

type WrappedProject = Rc<RefCell<Project>>;

struct Project {
    clients: HashMap<Uuid, Sender>
}

impl Project {
    fn new() -> Project {
        Project {
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
    project: WrappedProject
}

impl Handler for ClientHandler {

    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.project.borrow_mut().add_client(&self);
        // info!("{:?}", self.project.borrow().len());
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        // Echo the message back
        self.tx.send(msg)
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
    let project = WrappedProject::new(RefCell::new(Project::new()));

    listen(addr, |tx| {
        ClientHandler {
            id: Uuid::new_v4(),
            tx: tx,
            project: project.clone()
        }
    }).unwrap();
}
