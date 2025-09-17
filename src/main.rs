use std::path::PathBuf;
use std::str::FromStr;

use grug_ls::server::Server;

use log::error;
use log::info;
use lsp_server::{Connection, ErrorCode, Response};
use lsp_types::{CompletionOptions, InitializeResult, ServerInfo};
use lsp_types::HoverProviderCapability;
use lsp_types::InitializeParams;
use lsp_types::ServerCapabilities;
use lsp_types::TextDocumentSyncCapability;
use lsp_types::TextDocumentSyncKind;
use structured_logger::json::new_writer;
use structured_logger::Builder;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) {
        println!("1.0.0");
        return;
    }

    let log_file_path = include_str!("../log_path");
    let log_file_path: String = log_file_path.chars().filter(|c| *c != '\n').collect();
    let log_file_path = PathBuf::from_str(&log_file_path).expect("Didn't get a valid log file path");

    let file_writer = std::fs::File::options()
        .create(true)
        .append(true)
        .open(log_file_path)
        .unwrap();

    file_writer.set_len(0).unwrap();

    Builder::with_level("INFO")
        .with_target_writer("*", new_writer(file_writer))
        .init();

    let (mut connection, io_threads) = Connection::stdio();

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions::default()),

        ..Default::default()
    };
    
    let (mut server, id) = match connection.initialize_start() {
        Ok((req_id, value)) => {
            let params: InitializeParams = serde_json::from_value(value).unwrap();

            let server = Server::from_request(params);
            match server {
                Ok(server) => (server, req_id),
                Err(err) => {
                    let err = serde_json::to_string(&err).unwrap();
                    error!("{:?}", err);

                    let res = Response::new_err(req_id.clone(), ErrorCode::InvalidRequest as i32, err);
                    let res = serde_json::to_value(res).unwrap();
                    connection.initialize_finish(req_id, res).unwrap();

                    panic!();
                }
            }
        }
        Err(err) => {
            error!("Init Star err: {}", err);
            panic!()
        }
    };

    let init_data = InitializeResult {
        capabilities: server_capabilities,
        server_info: Some(ServerInfo {
            name: "Grug-LS".to_string(),
            version: Some("1.0.0".to_string()),
        }),
        ..Default::default()
    };
    let init_data = serde_json::to_value(init_data).unwrap();

    connection.initialize_finish(id, init_data).unwrap();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_grug::LANGUAGE.into())
        .unwrap();

    info!("LSP START");
    info!("Got these arguments: {:?}", args);

    loop {
        let message: lsp_server::Message = connection.receiver.recv().unwrap();

        server.handle_message(message, &mut connection, &mut parser);

        if server.should_exit {
            break;
        }
    }
    io_threads.join().unwrap();

}
