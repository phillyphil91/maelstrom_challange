// {"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1","n2","n3"]}}

#![allow(dead_code)]
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};
use unique_id::{random::RandomGenerator, Generator};

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
    Generate {},
    GenerateOk {
        id: usize,
    },
}

struct Node {
    msg_id: usize,
}

impl Node {
    fn generate_response(
        &mut self,
        stdout: &mut StdoutLock,
        message: Message,
    ) -> anyhow::Result<()> {
        match message.body.message_type {
            MessageType::Init { .. } => {
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: Payload {
                        msg_id: Some(self.msg_id),
                        in_reply_to: message.body.msg_id,
                        message_type: MessageType::InitOk,
                    },
                };

                serde_json::to_writer(&mut *stdout, &response).unwrap();
                stdout
                    .write_all(b"\n")
                    .context("couldn't successfully write to stdout")?;
                Ok(())
            }
            MessageType::Echo { echo } => {
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: Payload {
                        msg_id: Some(self.msg_id),
                        in_reply_to: message.body.msg_id,
                        message_type: MessageType::EchoOk { echo },
                    },
                };

                serde_json::to_writer(&mut *stdout, &response).unwrap();
                stdout
                    .write_all(b"\n")
                    .context("couldn't successfully write to stdout")?;
                self.msg_id += 1;
                Ok(())
            }
            MessageType::InitOk { .. } => todo!(),
            MessageType::EchoOk { .. } => todo!(),
            MessageType::Generate {} => {
                let unique_id = RandomGenerator::default();
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: Payload {
                        msg_id: Some(self.msg_id),
                        in_reply_to: message.body.msg_id,
                        message_type: MessageType::GenerateOk {
                            id: unique_id.next_id() as usize, // not pretty but does the job
                        },
                    },
                };

                serde_json::to_writer(&mut *stdout, &response).unwrap();
                stdout
                    .write_all(b"\n")
                    .context("couldn't successfully write to stdout")?;
                self.msg_id += 1;
                Ok(())
            }
            MessageType::GenerateOk { .. } => todo!(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();

    let deserialized = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = std::io::stdout().lock();

    let mut node_at_start_up = Node { msg_id: 1 };
    for input in deserialized {
        node_at_start_up.generate_response(
            &mut stdout,
            input.context("something went wrong with the input")?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod test {

    #[test]
    fn test_init() {
        todo!()
    }
}
