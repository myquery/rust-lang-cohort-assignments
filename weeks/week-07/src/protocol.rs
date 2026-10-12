use crate::{Block, NodeError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeRequest {
    Ping,
    Height,
    GetTip,
    GetBlock(String),
    SubmitBlock(Block),
    AddPeer(String),
    GetPeers,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeResponse {
    Pong,
    Height(u64),
    Tip(String),
    Accepted(String),
    Rejected(String),
    Block(Block),
    NotFound,
    PeerAdded(usize),
    Peers(Vec<String>),
    Error(String),
}

/// Parse a block in `<hash>|<previous_hash>|<height>|<payload>` format.
pub fn parse_block(input: &str) -> Result<Block, NodeError> {
    // Steps:
    // 1. Split `input` into exactly four fields using `|`.
    // 2. Trim each field.
    // 3. Reject empty hash, previous hash, height, or payload.
    // 4. Parse height as `u64`.
    // 5. Return `NodeError::MalformedMessage` on malformed input.
    let fields: Vec<&str> = input.split('|').map(str::trim).collect();
    if fields.len() != 4 || fields.iter().any(|field| field.is_empty()) {
        return Err(NodeError::MalformedMessage);
    }
    let height = fields[2].parse().map_err(|_| NodeError::MalformedMessage)?;
    Ok(Block::new(fields[0], fields[1], height, fields[3]))
}

/// Parse one text protocol request.
///
/// Supported commands:
/// - `ping`
/// - `height`
/// - `get_tip`
/// - `get_peers`
/// - `get_block <hash>`
/// - `add_peer <address>`
/// - `submit_block <hash>|<previous_hash>|<height>|<payload>`
pub fn parse_request(line: &str) -> Result<NodeRequest, NodeError> {
    // Steps:
    // 1. Trim trailing whitespace.
    // 2. Match exact commands without arguments first.
    // 3. For commands with arguments, split once on the first space.
    // 4. Reject missing arguments with `MalformedMessage`.
    // 5. Reject unknown commands with `UnknownCommand`.
    let line = line.trim();
    match line {
        "ping" => return Ok(NodeRequest::Ping),
        "height" => return Ok(NodeRequest::Height),
        "get_tip" => return Ok(NodeRequest::GetTip),
        "get_peers" => return Ok(NodeRequest::GetPeers),
        _ => {}
    }
    let (command, argument) = line.split_once(' ').unwrap_or((line, ""));
    let argument = argument.trim();
    match command {
        "get_block" if !argument.is_empty() && !argument.contains(char::is_whitespace) => {
            Ok(NodeRequest::GetBlock(argument.to_string()))
        }
        "add_peer" if !argument.is_empty() && !argument.contains(char::is_whitespace) => {
            Ok(NodeRequest::AddPeer(argument.to_string()))
        }
        "submit_block" if !argument.is_empty() => {
            Ok(NodeRequest::SubmitBlock(parse_block(argument)?))
        }
        "get_block" | "add_peer" | "submit_block" | "ping" | "height" | "get_tip" | "get_peers" => {
            Err(NodeError::MalformedMessage)
        }
        _ => Err(NodeError::UnknownCommand),
    }
}

/// Encode a response as one newline-terminated protocol line.
pub fn encode_response(response: &NodeResponse) -> String {
    // Steps:
    // 1. Match every response variant.
    // 2. Return exactly one line ending in `\n`.
    // 3. Use `block <wire_format>` for block responses.
    // 4. Use comma-separated peer addresses for `Peers`.
    let line = match response {
        NodeResponse::Pong => "pong".to_string(),
        NodeResponse::Height(height) => format!("height {}", height),
        NodeResponse::Tip(hash) => format!("tip {}", hash),
        NodeResponse::Accepted(hash) => format!("accepted {}", hash),
        NodeResponse::Rejected(reason) => format!("rejected {}", reason),
        NodeResponse::Block(block) => format!("block {}", block.wire_format()),
        NodeResponse::NotFound => "not_found".to_string(),
        NodeResponse::PeerAdded(count) => format!("peer_added {}", count),
        NodeResponse::Peers(peers) => format!("peers {}", peers.join(",")),
        NodeResponse::Error(message) => format!("error {}", message),
    };
    format!("{}\n", line)
}

/// Parse a response produced by `encode_response`.
///
/// This is intentionally smaller than a real P2P decoder, but it forces students
/// to handle both directions of a protocol boundary.
pub fn parse_response(line: &str) -> Result<NodeResponse, NodeError> {
    // Steps:
    // 1. Trim the response line.
    // 2. Parse `pong`, `not_found`, `height <n>`, `tip <hash>`,
    //    `accepted <hash>`, `rejected <reason>`, `error <message>`,
    //    `block <wire_block>`, and `peers <a,b,c>`.
    // 3. Return `MalformedMessage` for malformed known responses.
    // 4. Return `UnknownCommand` for unrecognized response prefixes.
    let line = line.trim();
    match line {
        "pong" => return Ok(NodeResponse::Pong),
        "not_found" => return Ok(NodeResponse::NotFound),
        _ => {}
    }
    let (command, argument) = line.split_once(' ').unwrap_or((line, ""));
    let argument = argument.trim();
    match command {
        "height" => Ok(NodeResponse::Height(
            argument.parse().map_err(|_| NodeError::MalformedMessage)?,
        )),
        "peer_added" => Ok(NodeResponse::PeerAdded(
            argument.parse().map_err(|_| NodeError::MalformedMessage)?,
        )),
        "tip" | "accepted" | "rejected" | "error" if !argument.is_empty() => match command {
            "tip" => Ok(NodeResponse::Tip(argument.to_string())),
            "accepted" => Ok(NodeResponse::Accepted(argument.to_string())),
            "rejected" => Ok(NodeResponse::Rejected(argument.to_string())),
            _ => Ok(NodeResponse::Error(argument.to_string())),
        },
        "block" => Ok(NodeResponse::Block(parse_block(argument)?)),
        "peers" => Ok(NodeResponse::Peers(if argument.is_empty() {
            vec![]
        } else {
            argument.split(',').map(str::to_string).collect()
        })),
        "tip" | "accepted" | "rejected" | "error" | "pong" | "not_found" => {
            Err(NodeError::MalformedMessage)
        }
        _ => Err(NodeError::UnknownCommand),
    }
}
