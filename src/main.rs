// {"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1","n2","n3"]}}

#![allow(dead_code)]
use std::io::{StdoutLock, Write};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    src: String,
    dest: String,
    body: Payload,
}

#[derive(Debug, Deserialize, Serialize)]
struct Payload {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,

    #[serde(flatten)]
    message_type: MessageType,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum MessageType {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

struct Node {}

impl Node {
    fn generate_response(stdout: &mut StdoutLock, message: Message) {
        match message.body.message_type {
            MessageType::Init { .. } => {
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: Payload {
                        msg_id: Some(1),
                        in_reply_to: Some(1),
                        message_type: MessageType::InitOk,
                    },
                };

                serde_json::to_writer(&mut *stdout, &response).unwrap();
                stdout.write_all(b"\n").unwrap();
            }
            MessageType::Echo { echo } => {
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: Payload {
                        msg_id: Some(1),
                        in_reply_to: Some(1),
                        message_type: MessageType::EchoOk { echo },
                    },
                };

                serde_json::to_writer(&mut *stdout, &response).unwrap();
                stdout.write_all(b"\n").unwrap();
            }
            MessageType::InitOk { .. } => todo!(),
            MessageType::EchoOk { .. } => todo!(),
        }
    }
}

fn main() {
    let stdin = std::io::stdin().lock();

    let deserialized = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = std::io::stdout().lock();

    for input in deserialized {
        Node::generate_response(&mut stdout, input.unwrap());
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_init() {
        todo!()
    }
}
