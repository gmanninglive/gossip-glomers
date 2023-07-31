use rust_maelstrom::{Message, Node};

struct BasicNode<'a> {
    inner: Node<'a>,
}

/// BasicNode implementation
///
/// This implementation handles the echo and unique id generation examples
impl<'a> BasicNode<'a> {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            inner: Node::new()?,
        })
    }
    fn run(&mut self) -> anyhow::Result<()> {
        let input = serde_jsonlines::JsonLinesReader::new(std::io::stdin().lock());
        for msg in input.read_all::<Message>() {
            self.inner.reply(msg?)?;
        }

        Ok(())
    }
}
fn main() -> anyhow::Result<()> {
    BasicNode::new()?.run()
}
