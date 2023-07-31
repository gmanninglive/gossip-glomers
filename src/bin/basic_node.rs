use rust_maelstrom::{Body, InnerNode, IntoResponse, Message, Node, Payload};

struct BasicNode<'a> {
    inner: InnerNode<'a>,
}

/// BasicNode implementation
///
/// This implementation handles the echo and unique id generation examples
impl<'a> Node for BasicNode<'a> {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            inner: Node::new()?,
        })
    }

    /// Responds to the current message and increments the msg_id
    fn reply(&mut self, msg: Message) -> anyhow::Result<()> {
        self.inner.output.write(&self.response(msg))?;

        self.inner.msg_id += 1;
        Ok(())
    }

    fn write(&mut self, msg: Message) -> anyhow::Result<()> {
        Ok(self.inner.output.write(&msg)?)
    }
}

impl<'a> BasicNode<'a> {
    fn run(&mut self) -> anyhow::Result<()> {
        let input = serde_jsonlines::JsonLinesReader::new(std::io::stdin().lock());
        for msg in input.read_all::<Message>() {
            self.reply(msg?)?;
        }

        Ok(())
    }
}

impl<'a> IntoResponse for BasicNode<'a> {
    fn response(&self, msg: Message) -> Message {
        Message {
            src: msg.dest,
            dest: msg.src,
            body: Body {
                msg_id: Some(self.inner.msg_id),
                in_reply_to: msg.body.msg_id,
                payload: match msg.body.payload {
                    Payload::Echo { echo } => Payload::EchoOk { echo },
                    Payload::Generate => Payload::GenerateOk {
                        id: format!("{}-{}", self.inner.node_id, self.inner.msg_id),
                    },
                    unexpected => {
                        panic!("unexpected message received: {:?}", unexpected)
                    }
                },
            },
        }
    }
}
fn main() -> anyhow::Result<()> {
    BasicNode::new()?.run()
}
