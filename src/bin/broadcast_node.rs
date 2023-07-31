use rust_maelstrom::{Body, InnerNode, IntoResponse, Message, Node, Payload};
use std::{collections::HashSet, time::Duration};

enum Event {
    External(Message),
    Gossip,
    EOF,
}

struct BroadcastNode<'a> {
    inner: InnerNode<'a>,
    messages: HashSet<usize>,
    neighbours: Vec<String>,
}

/// BroadcastNode implementation
///
/// This implemenation handles the broadcast examples
/// - 3a: Single-Node Broadcast
/// - 3b: Multi-Node Broadcast
/// - 3c: Fault Tolerant Broadcast
/// - 3d: Efficient Broadcast, Part I
/// - 3e: Efficient Broadcast, Part II
///
impl<'a> Node for BroadcastNode<'a> {
    /// Initialises a Maelstrom Node, reading and responding to the init message
    fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            inner: Node::new()?,
            messages: HashSet::new(),
            neighbours: Vec::new(),
        })
    }

    /// Responds to the current message and increments the msg_id
    fn reply(&mut self, msg: Message) -> anyhow::Result<()> {
        self.write(self.response(msg))?;

        self.inner.msg_id += 1;
        Ok(())
    }

    fn write(&mut self, msg: Message) -> anyhow::Result<()> {
        Ok(self.inner.output.write(&msg)?)
    }
}

impl<'a> BroadcastNode<'a> {
    fn handle_message(&mut self, msg: Message) -> anyhow::Result<()> {
        match &msg.body.payload {
            Payload::Broadcast { message } => {
                self.messages.insert(*message);

                self.reply(msg)
            }
            Payload::Topology { topology } => {
                let t = topology
                    .get(&self.inner.node_id)
                    .expect("missing topology for node");
                self.neighbours.append(&mut t.clone());

                self.reply(msg)
            }
            Payload::Gossip { messages } => {
                self.messages.extend(messages);
                Ok(())
            }
            _ => self.reply(msg),
        }
    }

    fn run(&mut self) -> anyhow::Result<()> {
        let (tx, rx) = crossbeam_channel::bounded::<Event>(10);

        let stdin_jh = std::thread::spawn({
            let tx = tx.clone();
            move || {
                let input = serde_jsonlines::JsonLinesReader::new(std::io::stdin().lock());
                for msg in input.read_all::<Message>() {
                    let _ = tx.send(Event::External(msg.expect("error deserializing message")));
                }

                let _ = tx.send(Event::EOF);
            }
        });

        let gossip_jh = std::thread::spawn({
            let tx = tx.clone();
            move || loop {
                // Adjust this duration between challenge 3d and 3e
                std::thread::sleep(Duration::from_millis(200));

                let _ = tx.send(Event::Gossip);
            }
        });

        for event in rx.into_iter() {
            match event {
                Event::Gossip => {
                    for n in self.neighbours.clone() {
                        self.write(Message {
                            src: self.inner.node_id.clone(),
                            dest: n,
                            body: Body {
                                msg_id: None,
                                in_reply_to: None,
                                payload: Payload::Gossip {
                                    messages: self.messages.clone(),
                                },
                            },
                        })?;
                    }
                }
                Event::External(msg) => {
                    self.handle_message(msg)?;
                }
                Event::EOF => break,
            }
        }
        let _ = stdin_jh.join().expect("stdin thread paniced");
        let _ = gossip_jh.join().expect("gossip thread paniced");

        Ok(())
    }
}

impl<'a> IntoResponse for BroadcastNode<'a> {
    fn response(&self, msg: Message) -> Message {
        Message {
            src: msg.dest,
            dest: msg.src,
            body: Body {
                msg_id: Some(self.inner.msg_id),
                in_reply_to: msg.body.msg_id,
                payload: match msg.body.payload {
                    Payload::Broadcast { .. } => Payload::BroadcastOk,
                    Payload::Read => Payload::ReadOk {
                        messages: self.messages.clone(),
                    },
                    Payload::Topology { .. } => Payload::TopologyOk,
                    unexpected => {
                        panic!("unexpected message received: {:?}", unexpected)
                    }
                },
            },
        }
    }
}

fn main() -> anyhow::Result<()> {
    BroadcastNode::new()?.run()
}
