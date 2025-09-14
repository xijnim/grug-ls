use serde::{Serialize, Deserialize};
use serde_repr::{Serialize_repr, Deserialize_repr};

use crate::{rpc::{RequestMessage, ResponseMessage, Rpc}, server::{document::Document, mod_api::ModApi, text_sync::{Position, TextDocumentIdentifier}, Server}};

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
    documentation: String,
    kind: CompletionItemKind,
}


impl Server {
    pub fn get_completion(
        &self,
        document: &Document,
        node: &tree_sitter::Node<'_>,
    ) -> Vec<CompletionItem> {
        let mut items: Vec<CompletionItem> = Vec::new();

        items.push(CompletionItem {
            label: "Test".to_string(),
            detail: "Test".to_string(),
            documentation: "Test".to_string(),
            kind: CompletionItemKind::Text,
        });

        items
    }

    pub fn handle_completion(&self, req: RequestMessage<CompletionParams>, rpc: &mut Rpc) {
        let path = &req.params.text_document.uri["file.//".len()..];
        let document = self.document_map.get(path).unwrap();

        let point = tree_sitter::Point {
            column: req.params.position.character,
            row: req.params.position.line,
        };
        let node = document.tree.root_node().named_descendant_for_point_range(point, point).unwrap();

        let completion = self.get_completion(document, &node);
        let result: ResponseMessage<Vec<CompletionItem>> = ResponseMessage::new(req.id, completion);
        let json = serde_json::to_string_pretty(&result).unwrap();

        info!("Sending this completion: {}", json);
        rpc.send(json.as_str());
    }
}
