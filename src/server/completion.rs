use lsp_server::{Connection, Message, RequestId, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Documentation, InsertTextFormat,
    MarkupContent, MarkupKind,
};

use crate::server::{
    document::{Document, KEYWORDS, PRIMITIVE_TYPES, STATEMENT_SNIPPETS}, utils::{get_nearest_node, get_spot_info}, Server
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
            let markup = MarkupContent {
                kind: MarkupKind::Markdown,
                value: game_func.description.clone(),
            };

            let mut snippet = format!("{}(", name);
            for (idx, param) in game_func.arguments.iter().enumerate() {
                snippet.push_str(&format!("${{{}:{}}}", idx + 1, param.get_name()));
                if idx < game_func.arguments.len() - 1 {
                    snippet.push_str(", ");
                }
            }
            snippet.push(')');

            items.push(CompletionItem {
                label: name.clone(),
                detail: Some(format!("{}\n", game_func.format(name))),
                documentation: Some(Documentation::MarkupContent(markup)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                insert_text: Some(snippet),
                kind: Some(CompletionItemKind::FUNCTION),

                ..Default::default()
            });
        }

        for snippet in STATEMENT_SNIPPETS.values() {
            let markup = MarkupContent {
                kind: MarkupKind::Markdown,
                value: snippet.doc.to_string(),
            };
            items.push(CompletionItem {
                label: snippet.label.to_string(),
                kind: Some(CompletionItemKind::SNIPPET),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                insert_text: Some(snippet.snippet.to_string()),
                documentation: Some(Documentation::MarkupContent(markup)),
                ..Default::default()
            })
        }

        for (name, desc) in KEYWORDS.iter() {
            let markup = MarkupContent {
                kind: MarkupKind::Markdown,
                value: desc.to_string(),
            };
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                documentation: Some(Documentation::MarkupContent(markup)),
                ..Default::default()
            });
        }

        if "source_file"
            == node
                .parent()
                .map(|node| node.kind())
                .unwrap_or("source_file")
        {
            if let Some(entity) = self.mod_api.entities.get(&document.entity_type) {
                for (func_name, func) in entity.on_functions.iter() {
                    if !document
                        .on_functions
                        .iter()
                        .any(|func| func.name == *func_name)
                    {
                        let markup = MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: func.description.clone(),
                        };
                        items.push(CompletionItem {
                            label: func_name.clone(),
                            detail: Some(func_name.clone()),
                            documentation: Some(Documentation::MarkupContent(markup)),
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

        let text = if let Ok(src) = str::from_utf8(&document.content) {
            src
        } else {
            log::error!("Invalid message: {:?}", document.content);
            return;
        };

        let line = if let Some(line) = text
            .lines()
            .nth(params.text_document_position.position.line as usize)
        {
            line
        } else {
            return;
        };

        let line = &line[0..params.text_document_position.position.character as usize];
        let mut is_type = false;
        let mut can_skip = false;
        for chr in line.chars().rev() {
            if chr == ':' {
                is_type = true;
                break;
            }

            if !matches!(chr, ' ' | '\t') {
                if can_skip {
                    break;
                }
            } else {
                can_skip = true;
            }
        }
        let node = get_nearest_node(document, params.text_document_position.position);

        let is_string = node.kind() == "string" || node.kind() == "comment";

        let completion = if is_string {
            Vec::new()
        } else if is_type {
            let mut completion: Vec<CompletionItem> = Vec::new();

            for (name, entity) in self.mod_api.entities.iter() {
                completion.push(CompletionItem {
                    label: name.to_string(),
                    documentation: Some(Documentation::String(entity.description.clone())),
                    kind: Some(CompletionItemKind::CLASS),

                    ..Default::default()
                });
            }

            for (name, desc) in PRIMITIVE_TYPES.iter() {
                completion.push(CompletionItem {
                    label: name.to_string(),
                    documentation: Some(Documentation::String(desc.to_string())),
                    kind: Some(CompletionItemKind::TYPE_PARAMETER),

                    ..Default::default()
                });
            }

            completion
        } else {
            self.get_completion(document, &node)
        };

        info!("Sending this completion: {:?}", completion);
        let response = Response::new_ok(id, completion);

        connection.sender.send(Message::Response(response)).unwrap();
    }
}
