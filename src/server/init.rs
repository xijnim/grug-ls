use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use vfs::MemoryFS;

use crate::{rpc::RequestMessage, server::{helper::spawn_worker, mod_api::ModApi, Server}};

use std::collections::HashMap;

use log::error;

#[derive(Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct InitResponse {
    pub id: serde_json::Value,
    pub result: InitResult,
}

#[derive(Serialize, Deserialize)]
pub struct InitResult {
    pub capabilities: ServerCapabilities,

    #[serde(rename = "serverInfo")]
    pub server_info: Option<ServerInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(rename = "positionEncoding")]
    pub position_encoding: String,

    #[serde(rename = "textDocumentSync")]
    pub text_document_sync: usize,

    #[serde(rename = "hoverProvider")]
    pub hover_provider: bool,

    #[serde(rename = "completionProvider")]
    pub completion_provider: HashMap<(), ()>,
}

impl Server {
    pub fn from_request(json: serde_json::Value) -> Server {
        #[derive(Serialize, Deserialize, Debug)]
        struct WorkspaceFolder {
            uri: String,
            name: String,
        }

        #[derive(Serialize, Deserialize, Debug)]
        struct InitParams {
            capabilities: super::ClientCapabilities,

            #[serde(rename = "rootUri")]
            root_uri: Option<String>,

            #[serde(rename = "workspaceFolders")]
            workpace_folders: Option<Vec<WorkspaceFolder>>,
        }

        let request: Result<RequestMessage<InitParams>, serde_json::Error> =
            serde_json::from_value(json.clone());

        let request = match request {
            Err(error) => {
                error!("Failure parsing init request: {:?}", error);
                error!("Request had this json: {:?}", json);
                panic!()
            }
            Ok(req) => req,
        };

        let mut root_path: Option<String> = None;

        if let Some(folders) = request.params.workpace_folders {
            root_path = Some(folders[0].name.to_string());
        } else if let Some(uri) = request.params.root_uri {
            assert!(uri.starts_with("file://"));
            root_path = Some(uri["file://".len()..].to_string());
        }

        if root_path.is_none() {
            //TODO: Send the error message to the client
            error!("Couldn't get a root path");
            error!("Init Request: {:?}", json);
            panic!();
        }
        let root_path = root_path.unwrap();
        let root_path = PathBuf::from_str(&root_path)
            .unwrap();

        let mod_api_json = std::fs::read_to_string(&root_path.join("mod_api.json")).unwrap();
        let mod_api: ModApi = serde_json::from_str(&mod_api_json).unwrap();

        let chan = spawn_worker(root_path.clone()).unwrap();

        Server {
            file_system: MemoryFS::new(),
            root_path,
            client_capabilities: request.params.capabilities,
            document_map: std::collections::HashMap::new(),
            messages_chan: chan,
            mod_api,
        }
    }
}
