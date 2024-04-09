use crate::message_set::{MessageSet, MessageSetReceiver, MessageSetSender};

pub mod handle;
pub mod macros;
pub mod message_set;

pub fn msg_channel<MS>() -> (MessageSetSender<MS>, MessageSetReceiver<MS>)
where
    MS: MessageSet,
{
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

    (
        MessageSetSender { sender },
        MessageSetReceiver {
            receiver,
            msg_queue: vec![],
            concurrent_msg_buf: Default::default(),
        },
    )
}
