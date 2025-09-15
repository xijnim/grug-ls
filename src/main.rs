use grug_ls::{rpc::Rpc, server::ServerWrapper};
use structured_logger::{Builder, json::new_writer};

use log::error;
use log::info;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) {
        println!("1.0.0");
        return;
    }

    let log_file_path = "/home/xijnim/Projects/grug-ls/log.txt";

    // TODO: Automatically find the project directory
    let file_writer = std::fs::File::options()
        .create(true)
        .append(true)
        .open(log_file_path)
        .unwrap();

    file_writer.set_len(0).unwrap();

    Builder::with_level("INFO")
        .with_target_writer("*", new_writer(file_writer))
        .init();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_grug::LANGUAGE.into())
        .unwrap();

    let mut rpc = Rpc::new(std::io::stdin(), std::io::stdout());

    info!("LSP START");
    info!("Got these arguments: {:?}", args);

    let mut server = ServerWrapper::new();

    loop {
        let content = rpc.recv();
        let json = String::from_utf8(content.to_vec());

        if let Err(ref error) = json {
            error!("Error decoding message json: {:?}", error);
            panic!();
        }
        let json = json.unwrap();

        server.handle_message(json, &mut rpc, &mut parser);
    }
}
