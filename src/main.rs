// {"src":"c1","dest":"n1","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1","n2","n3"]}}
// {"src":"c1","dest":"n1","body":{"type":"topology","topology":{"n1":["n2","n3"],"n2":["n1"],"n3":["n1"]}}}

#![allow(dead_code)]
use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{StdoutLock, Write};
use unique_id::{random::RandomGenerator, Generator};

#[derive(Debug, Deserialize, Serialize)]
struct Message<'a> {
    src: String,
    dest: String,
    body: Payload<'a>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Payload<'a> {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,

    #[serde(flatten)]
    message_type: MessageType<'a>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum MessageType<'a> {
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
    Generate,
    GenerateOk {
        id: usize,
    },
    Broadcast {
        #[serde(rename = "message")]
        broadcast_message: usize,
    },
    BroadcastOk,

    Read,
    #[serde(skip_deserializing)]
    // skipped here because &vec needed. doesn't impl Deserialize
    ReadOk {
        messages: &'a Vec<usize>,
    },
    Topology {
        topology: Value,
    },
    TopologyOk {},
}

struct Node {
    msg_id: usize,
    broadcast_message: Vec<usize>,
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
            MessageType::InitOk { .. } => bail!("init_ok should not be received by a node"),
            MessageType::EchoOk { .. } => bail!("echo_ok should not be received by a node"),
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
            MessageType::GenerateOk { .. } => bail!("generate_ok should not be received by a node"),
            MessageType::Broadcast { broadcast_message } => {
                self.broadcast_message.push(broadcast_message);
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: Payload {
                        msg_id: Some(self.msg_id),
                        in_reply_to: message.body.msg_id,
                        message_type: MessageType::BroadcastOk,
                    },
                };

                serde_json::to_writer(&mut *stdout, &response).unwrap();
                stdout
                    .write_all(b"\n")
                    .context("couldn't successfully write to stdout")?;
                self.msg_id += 1;
                Ok(())
            }
            MessageType::BroadcastOk { .. } => {
                bail!("broadcast_ok should not be received by a node")
            }

            MessageType::Read { .. } => {
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: Payload {
                        msg_id: Some(self.msg_id),
                        in_reply_to: message.body.msg_id,
                        message_type: MessageType::ReadOk {
                            messages: &self.broadcast_message,
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
            MessageType::ReadOk { .. } => bail!("broadcast_ok should not be received by a node"),
            MessageType::Topology { .. } => {
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: Payload {
                        msg_id: Some(self.msg_id),
                        in_reply_to: message.body.msg_id,
                        message_type: MessageType::TopologyOk {},
                    },
                };
                serde_json::to_writer(&mut *stdout, &response).unwrap();
                stdout
                    .write_all(b"\n")
                    .context("couldn't successfully write to stdout")?;
                Ok(())
            }
            MessageType::TopologyOk {} => bail!("broadcast_ok should not be received by a node"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();

    let deserialized = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut stdout = std::io::stdout().lock();

    let mut node_at_start_up = Node {
        msg_id: 1,
        broadcast_message: vec![],
    };
    for input in deserialized {
        node_at_start_up.generate_response(
            &mut stdout,
            input.context("something went wrong with the input")?,
        )?;
    }
    Ok(())
}
