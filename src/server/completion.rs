use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    rpc::{RequestMessage, ResponseMessage, Rpc},
    server::{
        Server,
        document::Document,
        text_sync::{Position, TextDocumentIdentifier},
        utils::get_spot_info,
    },
};

use log::info;

#[derive(Serialize, Deserialize)]
pub struct CompletionParams {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentIdentifier,

    position: Position,
}

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionItem {
    label: String,
    detail: String,
    documentation: Option<String>,
    kind: CompletionItemKind,
}

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
                detail: var.format(),
                documentation: None,
                kind: CompletionItemKind::Variable,
            });
        }
        for helper in document.helpers.iter() {
            items.push(CompletionItem {
                label: helper.name.clone(),
                detail: helper.format().clone(),
                documentation: None,
                kind: CompletionItemKind::Variable,
            });
        }

        for (name, game_func) in self.mod_api.game_functions.iter() {
            items.push(CompletionItem {
                label: name.clone(),
                detail: format!("{}\n", game_func.format(name)),
                documentation: Some(game_func.description.clone()),
                kind: CompletionItemKind::Function,
            });
        }

        info!("{:?}", document.on_functions);
        if "source_file" == node.parent().map(|node| node.kind()).unwrap_or("source_file") {
            info!("on root");
            if let Some(entity) = self.mod_api.entities.get(&document.entity_type) {
                info!("on entity");
                for (func_name, func) in entity.on_functions.iter() {
                    if !document.on_functions.iter().any(|func| func.name == *func_name) {
                        items.push(CompletionItem {
                            label: func_name.clone(),
                            detail: func_name.clone(),
                            documentation: Some(func.description.clone()),
                            kind: CompletionItemKind::Function,
                        })
                    }
                }
            }
        }

        items
    }

    pub fn handle_completion(&self, req: RequestMessage<CompletionParams>, rpc: &mut Rpc) {
        let path = &req.params.text_document.uri["file.//".len()..];
        let document = self.document_map.get(path).unwrap();

        let point = tree_sitter::Point {
            column: req.params.position.character,
            row: req.params.position.line,
        };
        let node = document
            .tree
            .root_node()
            .named_descendant_for_point_range(point, point)
            .unwrap();

        let completion = self.get_completion(document, &node);
        let result: ResponseMessage<Vec<CompletionItem>> = ResponseMessage::new(req.id, completion);
        let json = serde_json::to_string_pretty(&result).unwrap();

        info!("Sending this completion: {}", json);
        rpc.send(json.as_str());
    }
}
