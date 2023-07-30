use std::io::StdoutLock;

use anyhow::Error;
use serde::{Deserialize, Serialize};

use serde_jsonlines::JsonLinesWriter;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message<Payload> {
    src: String,
    dest: String,
    body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]

struct Body<Payload> {
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
}

impl Message<Payload> {
    pub fn response(self, msg_id: usize) -> Self {
        Self {
            src: self.dest,
            dest: self.src,
            body: Body {
                msg_id: Some(msg_id),
                in_reply_to: self.body.msg_id,
                payload: match self.body.payload {
                    Payload::Echo { echo } => Payload::EchoOk { echo },
                    Payload::Init { .. } => Payload::InitOk,
                    _ => {
                        panic!("init_ok or echo_ok received")
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
}

impl<'a> Maelstrom<'a> {
    fn init() -> Result<Self, anyhow::Error> {
        let mut input = serde_jsonlines::JsonLinesReader::new(std::io::stdin().lock());

        let init_msg = input
            .read::<Message<Payload>>()
            .expect("did not receive init message")
            .expect("deserializing init message");

        let output = serde_jsonlines::JsonLinesWriter::new(std::io::stdout().lock());

        match init_msg.body.payload.clone() {
            Payload::Init {
                node_id,
                node_ids: _,
            } => {
                let mut m = Self {
                    msg_id: 0,
                    node_id,
                    output,
                };

                m.reply(init_msg)?;
                Ok(m)
            }
            _ => Err(Error::msg("did not receive init message")),
        }
    }

    pub fn reply(&mut self, msg: Message<Payload>) -> Result<(), anyhow::Error> {
        self.output.write(&msg.response(self.msg_id))?;

        self.msg_id += 1;
        Ok(())
    }

    fn drain(&mut self) -> Result<(), anyhow::Error> {
        let input = serde_jsonlines::JsonLinesReader::new(std::io::stdin().lock());
        for msg in input.read_all::<Message<Payload>>() {
            self.reply(msg?)?;
        }

        Ok(())
    }
}

fn main() -> Result<(), anyhow::Error> {
    let mut maelstrom = Maelstrom::init()?;

    maelstrom.drain()
}
