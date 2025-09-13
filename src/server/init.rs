use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use vfs::MemoryFS;

use crate::{LoggableResult, Logger, rpc::RequestMessage, server::Server};

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
}

impl Server {
    pub fn from_request(logger: &mut Logger, json: serde_json::Value) -> Server {
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
                logger.log_str(format!("Failure parsing init request: {:?}", error));
                logger.log_str(format!("Request had this json: {:?}", json));
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
            logger.log_str("Couldnt get a root path");
            logger.log_str(format!("{:?}", json.to_string()));
        }
        let root_path = root_path.unwrap();
        let root_path = PathBuf::from_str(&root_path)
            .unwrap_or_log(logger, format!("root path could be parsed {}", root_path));

        Server {
            file_system: MemoryFS::new(),
            root_path,
            client_capabilities: request.params.capabilities,
            document_map: std::collections::HashMap::new(),
        }
    }
}
