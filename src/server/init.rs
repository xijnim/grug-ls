use std::{path::PathBuf, str::FromStr};

use lsp_types::InitializeParams;
use serde::{Deserialize, Serialize};
use vfs::MemoryFS;

use crate::{server::{helper::spawn_worker, mod_api::ModApi, Server}};

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

#[derive(Debug, Serialize)]
pub enum ServerInitError {
    NoRootPath,
    RootPathParseError,
    NoModApi,
    InitParseError(String),

    #[serde(rename = "modApiReadError")]
    ModApiIOError(String),
    ModApiParseError(String),
}
impl Server {
    pub fn from_request(params: InitializeParams) -> Result<Server, ServerInitError> {
        let mut root_path: Option<String> = None;

        #[allow(deprecated)]
        if let Some(ref folders) = params.workspace_folders {
            root_path = Some(folders[0].name.to_string());
        } else if let Some(ref uri) = params.root_uri {
            let uri = uri.as_str();
            assert!(uri.starts_with("file://"));
            root_path = Some(uri["file://".len()..].to_string());
        }

        let root_path = match root_path {
            Some(root_path) => root_path,
            None => {
                error!("Couldn't get a root path");
                error!("Init Request: {:?}", params);
                return Err(ServerInitError::NoRootPath);
            }
        };

        let mut root_path = match PathBuf::from_str(&root_path) {
            Ok(root_path) => root_path,
            Err(_) => {
                return Err(ServerInitError::RootPathParseError);
            }
        };

        if root_path.is_relative() {
            // Vscode
            if let Some(cwd) = std::env::current_dir().ok().map(|cwd| cwd.parent().map(|p| p.to_path_buf())).flatten() {
                root_path = cwd.join(root_path);
            }
        }

        let mod_api_json = match std::fs::read_to_string(&root_path.join("mod_api.json")) {
            Ok(json) => json,
            Err(err) => {
                return Err(ServerInitError::ModApiIOError(format!("At {}: {}", root_path.to_string_lossy().into_owned(), err.to_string())));
            }
        };
        let mod_api: ModApi = match serde_json::from_str(&mod_api_json) {
            Ok(mod_api) => mod_api,
            Err(err) => {
                return Err(ServerInitError::ModApiParseError(err.to_string()));
            }
        };

        let chan = spawn_worker(root_path.clone()).unwrap();

        let client_capabilities = params.capabilities;

        Ok(Server {
            file_system: MemoryFS::new(),
            root_path,
            client_capabilities,
            document_map: std::collections::HashMap::new(),
            messages_chan: chan,
            mod_api,
            should_exit: false,
        })
    }
}
