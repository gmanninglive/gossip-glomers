use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
};

use anyhow::Error;
use serde::{Deserialize, Serialize};

use serde_jsonlines::JsonLinesWriter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]

struct Body {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,

    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Generate,
    GenerateOk {
        id: String,
    },
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: HashSet<usize>,
    },
    Topology {
        topology: Topology,
    },
    TopologyOk,
}

impl Message {
    pub fn response(self, ctx: &Maelstrom) -> Self {
        Self {
            src: self.dest,
            dest: self.src,
            body: Body {
                msg_id: Some(ctx.msg_id),
                in_reply_to: self.body.msg_id,
                payload: match self.body.payload {
                    Payload::Echo { echo } => Payload::EchoOk { echo },
                    Payload::Generate => Payload::GenerateOk {
                        id: format!("{}-{}", ctx.node_id, ctx.msg_id),
                    },
                    Payload::Init { .. } => Payload::InitOk,
                    Payload::Broadcast { .. } => Payload::BroadcastOk,
                    Payload::Read => Payload::ReadOk {
                        messages: ctx.store.clone(),
                    },
                    Payload::Topology { .. } => Payload::TopologyOk,
                    _ => {
                        panic!("unexpected message received")
                    }
                },
            },
        }
    }
}

struct Maelstrom<'a> {
    msg_id: usize,
    node_id: String,
    output: JsonLinesWriter<StdoutLock<'a>>,
    store: HashSet<usize>,
    neighbours: Vec<String>,
}

type Topology = HashMap<String, Vec<String>>;

impl<'a> Maelstrom<'a> {
    fn init() -> Result<Self, anyhow::Error> {
        let mut input = serde_jsonlines::JsonLinesReader::new(std::io::stdin().lock());

        let init_msg = input
            .read::<Message>()
            .expect("did not receive init message")
            .expect("deserializing init message");

        let output = serde_jsonlines::JsonLinesWriter::new(std::io::stdout().lock());

        match init_msg.body.payload.clone() {
            Payload::Init { node_id, node_ids } => {
                let mut m = Self {
                    msg_id: 0,
                    node_id,
                    output,
                    store: HashSet::with_capacity(1000),
                    neighbours: node_ids,
                };

                m.reply(init_msg)?;
                Ok(m)
            }
            _ => Err(Error::msg("did not receive init message")),
        }
    }

    fn reply(&mut self, msg: Message) -> Result<(), anyhow::Error> {
        self.output.write(&msg.response(self))?;

        self.msg_id += 1;
        Ok(())
    }

    fn handle_message(&mut self, msg: Message) -> Result<(), anyhow::Error> {
        match &msg.body.payload {
            Payload::Broadcast { message } => {
                if !self.store.contains(message) {
                    self.store.insert(*message);

                    for id in self.neighbours.clone().into_iter() {
                        self.output.write(&Message {
                            src: self.node_id.clone(),
                            dest: id,
                            body: Body {
                                msg_id: None,
                                in_reply_to: None,
                                payload: msg.body.payload.clone(),
                            },
                        })?;
                    }
                }

                if !self.neighbours.contains(&msg.src) {
                    self.reply(msg)
                } else {
                    Ok(())
                }
            }
            _ => self.reply(msg),
        }
    }

    fn drain(&mut self) -> Result<(), anyhow::Error> {
        let input = serde_jsonlines::JsonLinesReader::new(std::io::stdin().lock());
        for msg in input.read_all::<Message>() {
            self.handle_message(msg?)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    let mut maelstrom = Maelstrom::init()?;

    maelstrom.drain()
}
