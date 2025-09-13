use grug_ls::{rpc::Rpc, server::ServerWrapper, Logger};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) {
        println!("1.0.0");
        return;
    }

    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_grug::LANGUAGE.into()).unwrap();

    let mut rpc = Rpc::new(std::io::stdin(), std::io::stdout());

    // TODO: Automatically find the project directory
    let mut logger = Logger::new("/home/xijnim/Projects/grug-ls/log.txt");
    logger.log_str("LSP START");

    let mut server = ServerWrapper::new();

    loop {
        let content = rpc.recv(&mut logger);
        let json = String::from_utf8(content.to_vec());

        if let Err(ref error) = json {
            logger.log_str(format!("Error decoding text: {:?}", error));
            panic!();
        }
        let json = json.unwrap();

        server.handle_message(json, &mut logger, &mut rpc, &mut parser);
    }
}
