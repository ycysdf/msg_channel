## Example 

```rust
use std::time::Duration;

use tokio::join;

use msg_channel::*;

pub struct MsgHandler;
impl MsgHandler {
    pub fn mut_test(&mut self) {}
}

pub struct MsgA;
pub struct MsgB;
impl HandleAsync<MsgA> for MsgHandler {
    type Replay = String;

    async fn handle(&mut self, _msg: MsgA) -> Self::Replay {
        tokio::time::sleep(Duration::from_secs(2)).await;
        println!("handle MsgA");
        "Msg A Say Hello".to_string()
    }
}

impl HandleAsync<MsgB> for MsgHandler {
    type Replay = u32;

    async fn handle(&mut self, _msg: MsgB) -> Self::Replay {
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("handle  MsgB");
        22
    }
}

pub struct MsgC(bool);
pub struct MsgD(String);
impl HandleSync<MsgC> for MsgHandler {
    type Replay = bool;

    fn handle(&mut self, _msg: MsgC) -> Self::Replay {
        println!("handle MsgC {}", _msg.0);
        _msg.0
    }
}
impl HandleSync<MsgD> for MsgHandler {
    type Replay = String;

    fn handle(&mut self, _msg: MsgD) -> Self::Replay {
        println!("handle MsgD {:?}", _msg.0);
        _msg.0
    }
}

pub struct TestMsgSet;

#[msg_set]
impl MessageSet for TestMsgSet {
    type Handler = MsgHandler;
    type Async = (MsgA, MsgB);
    type Sync = (MsgC, MsgD);
    type AsyncConcurrent = ();
    type SyncConcurrent = ();
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let (sender, mut handler) = msg_channel::<TestMsgSet>();
    tokio::spawn(async move {
        let r = sender.send(MsgA)?.await;
        println!("MsgA replay: {}", r);
        let r = sender.send(MsgB)?.await;
        println!("MsgB replay: {}", r);

        let (r1, r2) = join!(
            sender.send(MsgC(true))?,
            sender.send(MsgD("Hello".to_string()))?
        );
        println!("MsgC,MsgD replay: {},{}", r1, r2);

        Ok::<(), color_eyre::Report>(())
    });
    let mut msg_handler = MsgHandler;
    loop {
        tokio::select! {
            _ = std::future::pending() => {
                msg_handler.mut_test();
            }
            result = async {
                handler.handle_next(&mut msg_handler).await
            } => {
                if let Some(_result) = result? {
                    msg_handler.mut_test();
                }else{
                    break;
                }
            }
        }
    }
    Ok(())
}
```

## License 

MIT License ([LICENSE-MIT](https://github.com/ycysdf/msg_channel/blob/main/LICENSE-MIT))
