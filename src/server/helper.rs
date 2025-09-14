use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, channel},
};

use crate::server::{Server, mod_api::ModApi};

use log::error;
use log::info;
use log::warn;
use notify::Watcher;

pub enum ServerUpdate {
    ModApiChange(ModApi),
}

struct ServerWorker {
    mod_api_path: PathBuf,
    sender: Sender<ServerUpdate>,
    watcher_recv: Receiver<notify::Result<notify::Event>>,
}

impl ServerWorker {
    pub fn new(
        mod_api_path: PathBuf,
        sender: Sender<ServerUpdate>,
        watcher_recv: Receiver<notify::Result<notify::Event>>,
    ) -> ServerWorker {
        ServerWorker {
            mod_api_path,
            sender,
            watcher_recv,
        }
    }

    pub fn update(&mut self) {
        let recv = self.watcher_recv.try_recv();
        if let Ok(notify::Result::Err(err)) = &recv {
            warn!("What error: {:?}", err);
        }

        if let Ok(Ok(event)) = recv {
            if let notify::EventKind::Access(_) = event.kind {
            } else {
                info!("{:?}", event);
                if let Ok(json) = std::fs::read_to_string(&self.mod_api_path) {
                    let mod_api: Result<ModApi, serde_json::Error> = serde_json::from_str(&json);

                    match mod_api {
                        Ok(mod_api) => {
                            info!("Sending new mod_api: {:?}", mod_api);
                            self.sender
                                .send(ServerUpdate::ModApiChange(mod_api))
                                .unwrap();
                        }
                        Err(err) => {
                            error!("Error deserializing mod_api: {:?}", err);
                        }
                    }
                }
            }
        }
    }
}

pub fn spawn_worker(root_path: PathBuf) -> Option<Receiver<ServerUpdate>> {
    let (send, recv) = channel::<ServerUpdate>();

    let (watch_send, watch_recv) = channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(watch_send).ok()?;

    std::thread::spawn(move || {
        let mut worker = ServerWorker::new(root_path.join("mod_api.json"), send, watch_recv);

        info!("Initializing worker main loop");
        loop {
            while let notify::Result::Err(_) = watcher.watch(
                &root_path.join("mod_api.json"),
                notify::RecursiveMode::NonRecursive,
            ) {}
            worker.update();
        }
    });

    Some(recv)
}

impl Server {
    pub fn handle_worker_messages(&mut self) {
        if let Ok(message) = self.messages_chan.try_recv() {
            match message {
                ServerUpdate::ModApiChange(mod_api) => {
                    info!("New mod_api: {:?}", mod_api);
                    self.mod_api = mod_api;
                }
            }
        }
    }
}
