use alloc::{borrow::Cow, vec::Vec};

use crate::models::transactions::metadata::TransactionMetadata;
use crate::models::transactions::metadata::{NFTokenMetadata, Node};

/// Flattens objects into a single list, and applies func to every object in objects.
fn flatmap<T, R>(func: impl Fn(T) -> Vec<R>, list_of_items: Vec<T>) -> Vec<R> {
    let mut modified_items: Vec<R> = Vec::new();
    for item in list_of_items {
        modified_items.extend(func(item));
    }
    modified_items
}

fn get_nftoken_ids_from_nftokens(nftokens: Vec<NFTokenMetadata<'_>>) -> Vec<Cow<'_, str>> {
    nftokens
        .into_iter()
        .map(|token| token.nftoken.nftoken_id)
        .collect()
}

fn has_nftoken_page(node: &Node) -> bool {
    match node {
        Node::Created(node) => node.created_node.ledger_entry_type == "NFTokenPage",
        Node::Modified(node) => {
            node.modified_node.ledger_entry_type == "NFTokenPage"
                && node.modified_node.previous_fields.is_some()
                && node
                    .modified_node
                    .previous_fields
                    .as_ref()
                    .unwrap()
                    .nftokens
                    .is_some()
        }
        Node::Deleted(_) => false,
    }
}

fn get_previous_nftokens<'a>(node: &'a Node) -> Vec<NFTokenMetadata<'a>> {
    let mut nftokens: Vec<NFTokenMetadata> = Vec::new();
    if let Node::Modified(modified_node) = node {
        if let Some(previous_fields) = &modified_node.modified_node.previous_fields {
            if let Some(new_nftokens) = &previous_fields.nftokens {
                nftokens = new_nftokens.clone();
            }
        }
    }
    nftokens
}

fn get_new_nftokens<'a>(node: &'a Node) -> Vec<NFTokenMetadata<'a>> {
    match node {
        Node::Modified(modified_node) => {
            if let Some(final_fields) = &modified_node.modified_node.final_fields {
                if let Some(new_nftokens) = &final_fields.nftokens {
                    new_nftokens.clone()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }
        Node::Created(created_node) => {
            if let Some(new_nftokens) = &created_node.created_node.new_fields.nftokens {
                new_nftokens.clone()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

pub fn get_nftoken_id<'a: 'b, 'b>(meta: &'a TransactionMetadata<'a>) -> Option<Cow<'b, str>> {
    let affected_nodes: Vec<&Node> = meta
        .affected_nodes
        .iter()
        .filter(|node| has_nftoken_page(node))
        .collect();

    if affected_nodes.is_empty() {
        return None;
    }

    let previous_token_ids: Vec<Cow<'_, str>> =
        get_nftoken_ids_from_nftokens(flatmap(get_previous_nftokens, affected_nodes.clone()));

    let final_token_ids: Vec<Cow<'_, str>> =
        get_nftoken_ids_from_nftokens(flatmap(get_new_nftokens, affected_nodes));

    final_token_ids
        .into_iter()
        .find(|id| !previous_token_ids.contains(id))
}
