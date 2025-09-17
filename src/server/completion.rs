use lsp_server::{Connection, Message, RequestId, Response};
use lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, Documentation};

use crate::server::{
    Server,
    document::Document,
    utils::{get_nearest_node, get_spot_info},
};

use log::info;

impl Server {
    pub fn get_completion(
        &self,
        document: &Document,
        node: &tree_sitter::Node<'_>,
    ) -> Vec<CompletionItem> {
        let mut items: Vec<CompletionItem> = Vec::new();

        let spot_info = get_spot_info(document, node);

        for var in spot_info.variables.iter() {
            items.push(CompletionItem {
                label: var.name.clone(),
                detail: Some(var.format()),
                documentation: None,
                kind: Some(CompletionItemKind::VARIABLE),

                ..Default::default()
            });
        }
        for helper in document.helpers.iter() {
            items.push(CompletionItem {
                label: helper.name.clone(),
                detail: Some(helper.format().clone()),
                documentation: None,
                kind: Some(CompletionItemKind::VARIABLE),

                ..Default::default()
            });
        }

        for (name, game_func) in self.mod_api.game_functions.iter() {
            items.push(CompletionItem {
                label: name.clone(),
                detail: Some(format!("{}\n", game_func.format(name))),
                documentation: Some(Documentation::String(game_func.description.clone())),
                kind: Some(CompletionItemKind::FUNCTION),

                ..Default::default()
            });
        }

        if "source_file"
            == node
                .parent()
                .map(|node| node.kind())
                .unwrap_or("source_file")
        {
            info!("on root");
            if let Some(entity) = self.mod_api.entities.get(&document.entity_type) {
                info!("on entity");
                for (func_name, func) in entity.on_functions.iter() {
                    if !document
                        .on_functions
                        .iter()
                        .any(|func| func.name == *func_name)
                    {
                        items.push(CompletionItem {
                            label: func_name.clone(),
                            detail: Some(func_name.clone()),
                            documentation: Some(Documentation::String(func.description.clone())),
                            kind: Some(CompletionItemKind::FUNCTION),

                            ..Default::default()
                        })
                    }
                }
            }
        }

        items
    }

    pub fn handle_completion(
        &self,
        params: CompletionParams,
        connection: &mut Connection,
        id: RequestId,
    ) {
        let uri = params.text_document_position.text_document.uri.as_str();
        let path = &uri["file.//".len()..];
        let document = self.document_map.get(path).unwrap();

        let node = get_nearest_node(document, params.text_document_position.position);

        info!("{}", node.kind());

        let completion = self.get_completion(document, &node);

        info!("Sending this completion: {:?}", completion);
        let response = Response::new_ok(id, completion);

        connection.sender.send(Message::Response(response)).unwrap();
    }
}
