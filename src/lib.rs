use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
};

use serde::{Deserialize, Serialize};

use serde_jsonlines::JsonLinesWriter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]

pub struct Body {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,

    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Payload {
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
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Gossip {
        messages: HashSet<usize>,
    },
}

pub trait IntoResponse {
    fn response(&self, msg: Message) -> Message;
}

impl<'a> IntoResponse for InnerNode<'a> {
    fn response(&self, msg: Message) -> Message {
        Message {
            src: msg.dest,
            dest: msg.src,
            body: Body {
                msg_id: Some(self.msg_id),
                in_reply_to: msg.body.msg_id,
                payload: match msg.body.payload {
                    Payload::Init { .. } => Payload::InitOk,
                    unexpected => {
                        panic!("unexpected message received: {:?}", unexpected)
                    }
                },
            },
        }
    }
}

pub struct InnerNode<'a> {
    pub msg_id: usize,
    pub node_id: String,
    pub output: JsonLinesWriter<StdoutLock<'a>>,
}

impl<'a> Default for InnerNode<'a> {
    fn default() -> Self {
        Self {
            msg_id: 0,
            node_id: String::new(),
            output: serde_jsonlines::JsonLinesWriter::new(std::io::stdout().lock()),
        }
    }
}

pub enum Event {
    External(Message),
    Gossip,
    EOF,
}

pub trait Node {
    fn new() -> Result<Self, anyhow::Error>
    where
        Self: Sized;

    fn reply(&mut self, msg: Message) -> anyhow::Result<()>;

    fn write(&mut self, msg: Message) -> anyhow::Result<()>;
}

impl<'a> Node for InnerNode<'a> {
    /// Initialises a Maelstrom InnerNode, reading and responding to the init message
    fn new() -> Result<Self, anyhow::Error> {
        let mut input = serde_jsonlines::JsonLinesReader::new(std::io::stdin().lock());

        let init_msg = input
            .read::<Message>()
            .expect("did not receive init message")
            .expect("deserializing init message");

        match &init_msg.body.payload {
            Payload::Init {
                node_id,
                node_ids: _,
            } => {
                let mut node = Self {
                    node_id: node_id.clone(),
                    ..Default::default()
                };

                node.reply(init_msg)?;
                Ok(node)
            }
            unexpected => Err(anyhow::anyhow!(
                "did not receive init message. received: {:?}",
                unexpected
            )),
        }
    }

    /// Responds to the current message and increments the msg_id
    fn reply(&mut self, msg: Message) -> anyhow::Result<()> {
        self.write(self.response(msg))?;

        self.msg_id += 1;
        Ok(())
    }

    fn write(&mut self, msg: Message) -> anyhow::Result<()> {
        Ok(self.output.write(&msg)?)
    }
}
