use std::time::Duration;

use tokio::join;

use msg_channel::*;

pub struct MsgHandler;

impl MsgHandler {
    pub fn test(&mut self) {}
}

impl HandleSync<SyncMsgA> for MsgHandler {
    type Replay = ();

    fn handle(&mut self, _msg: SyncMsgA) -> Self::Replay {
        println!("SyncMsg");
    }
}
impl HandleSync<SyncMsgB> for MsgHandler {
    type Replay = ();

    fn handle(&mut self, _msg: SyncMsgB) -> Self::Replay {
        println!("SyncMsgB");
    }
}
impl HandleAsync<AsyncMsg> for MsgHandler {
    type Replay = ();

    async fn handle(&mut self, _msg: AsyncMsg) -> Self::Replay {
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("AsyncMsg")
    }
}
impl HandleAsyncConcurrent<AsyncConcurrentMsgA> for MsgHandler {
    type Replay = ();

    async fn handle(&self, _msg: AsyncConcurrentMsgA) -> Self::Replay {
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("AsyncConcurrentMsgA")
    }
}
impl HandleAsyncConcurrent<AsyncConcurrentMsgB> for MsgHandler {
    type Replay = ();

    async fn handle(&self, _msg: AsyncConcurrentMsgB) -> Self::Replay {
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("AsyncConcurrentMsgB")
    }
}

impl HandleSyncConcurrent<SyncConcurrentMsgA> for MsgHandler {
    type Replay = ();

    fn handle(&self, _msg: SyncConcurrentMsgA) -> Self::Replay {
        println!("SyncConcurrentMsgA")
    }
}
impl HandleSyncConcurrent<SyncConcurrentMsgB> for MsgHandler {
    type Replay = ();

    fn handle(&self, _msg: SyncConcurrentMsgB) -> Self::Replay {
        println!("SyncConcurrentMsgB")
    }
}

pub struct SyncMsgA;
pub struct SyncMsgB;
pub struct AsyncMsg;
pub struct SyncConcurrentMsgA;
pub struct SyncConcurrentMsgB;
pub struct AsyncConcurrentMsgA;
pub struct AsyncConcurrentMsgB;
pub struct TestMsgSet;

#[msg_set]
impl MessageSet for TestMsgSet {
    type Handler = MsgHandler;
    type Async = (AsyncMsg,);
    type Sync = (SyncMsgA, SyncMsgB);
    type AsyncConcurrent = (AsyncConcurrentMsgA, AsyncConcurrentMsgB);
    type SyncConcurrent = (SyncConcurrentMsgA, SyncConcurrentMsgB);
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let (sender, mut handler) = msg_channel::<TestMsgSet>();
    tokio::spawn(async move {
        let _r = sender.send(SyncMsgA)?.await;
        let _r = sender.send(SyncMsgB)?.await;
        let _r = sender.send(AsyncMsg)?.await;
        join!(
            sender.send(SyncConcurrentMsgA)?,
            sender.send(SyncConcurrentMsgB)?
        );
        let r = sender
            .send(TestMsgSetSyncVariant::SyncMsgA(SyncMsgA))?
            .await;
        match r {
            TestMsgSetSyncReplayVariant::SyncMsgA(_) => {}
            TestMsgSetSyncReplayVariant::SyncMsgB(_) => {}
        }
        let r = sender
            .send(TestMsgSetAsyncConcurrentVariant::AsyncConcurrentMsgB(
                AsyncConcurrentMsgB,
            ))?
            .await;
        match r {
            TestMsgSetAsyncConcurrentReplayVariant::AsyncConcurrentMsgA(_) => {}
            TestMsgSetAsyncConcurrentReplayVariant::AsyncConcurrentMsgB(_) => {}
        }
        Ok::<(), color_eyre::Report>(())
    });
    let mut msg_handler = MsgHandler;
    loop {
        tokio::select! {
            _ = std::future::pending() => {
                msg_handler.test();
            }
            result = async {
                handler.handle_next(&mut msg_handler).await
            } => {
                if let Some(_result) = result? {
                    msg_handler.test();
                }else{
                    break;
                }
            }
        }
    }
    Ok(())
}
