use std::future::{Future, ready};

use futures_util::{FutureExt, StreamExt};
use futures_util::future::Either;
use futures_util::stream::FuturesUnordered;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::task::{JoinError, JoinHandle};

use crate::handle::{
    HandleAsync, HandleAsyncConcurrent, HandleReplay, HandleSync, HandleSyncConcurrent,
};

pub enum MessageSetItem<MS>
where
    MS: MessageSet,
{
    Async(MS::AsyncVariant),
    Sync(MS::SyncVariant),
    AsyncConcurrent(MS::AsyncConcurrentVariant),
    SyncConcurrent(MS::SyncConcurrentVariant),
}
pub enum MessageSetReplayItem<MS>
where
    MS: MessageSet,
{
    Async(<MS::Handler as HandleAsync<MS::AsyncVariant>>::Replay),
    Sync(<MS::Handler as HandleSync<MS::SyncVariant>>::Replay),
    AsyncConcurrent(<MS::Handler as HandleAsyncConcurrent<MS::AsyncConcurrentVariant>>::Replay),
    SyncConcurrent(<MS::Handler as HandleSyncConcurrent<MS::SyncConcurrentVariant>>::Replay),
}

pub trait MessageVariantSet: 'static {
    type AsyncVariant: Send + 'static;
    type SyncVariant: Send + 'static;
    type AsyncConcurrentVariant: Send + 'static;
    type SyncConcurrentVariant: Send + 'static;
}

pub trait MessageSet: MessageVariantSet
where
    Self::Handler: HandleAsync<Self::AsyncVariant>,
    Self::Handler: HandleSync<Self::SyncVariant>,
    Self::Handler: HandleAsyncConcurrent<Self::AsyncConcurrentVariant>,
    Self::Handler: HandleSyncConcurrent<Self::SyncConcurrentVariant>,
{
    type Handler: 'static;
    type Async;
    type Sync;
    type AsyncConcurrent;
    type SyncConcurrent;
}

pub struct MessageSetSender<T>
where
    T: MessageSet,
{
    pub sender:
        mpsc::UnboundedSender<(MessageSetItem<T>, oneshot::Sender<MessageSetReplayItem<T>>)>,
}

impl<MS> MessageSetSender<MS>
where
    MS: MessageSet,
{
    pub fn send<M>(
        &self,
        msg: M,
    ) -> Result<
        impl Future<Output = <MS::Handler as HandleReplay<M>>::Replay>,
        mpsc::error::SendError<(
            MessageSetItem<MS>,
            oneshot::Sender<MessageSetReplayItem<MS>>,
        )>,
    >
    where
        M: Into<MessageSetItem<MS>>,
        MS::Handler: HandleReplay<M>,
        <MS::Handler as HandleReplay<M>>::MsgReplay: From<MessageSetReplayItem<MS>>,
    {
        let msg_variant = msg.into();
        let (replay_sender, replay_receiver) = oneshot::channel();
        self.sender.send((msg_variant, replay_sender))?;
        Ok(async move {
            let replay = replay_receiver.await.unwrap();
            let replay: <MS::Handler as HandleReplay<M>>::MsgReplay = replay.into();
            replay.into()
        })
    }
}


#[derive(Error, Debug)]
pub enum MsgSetRecvError {
    #[error("Disconnected")]
    Disconnected,
    #[error("JoinError {0:?}")]
    JoinError(JoinError)
}

type SyncConcurrentRelayAndSender<MS> = (
    <<MS as MessageSet>::Handler as HandleSyncConcurrent<
        <MS as MessageVariantSet>::SyncConcurrentVariant,
    >>::Replay,
    oneshot::Sender<MessageSetReplayItem<MS>>,
);


pub struct MessageSetReceiver<MS>
where
    MS: MessageSet,
{
    pub receiver: mpsc::UnboundedReceiver<(
        MessageSetItem<MS>,
        oneshot::Sender<MessageSetReplayItem<MS>>,
    )>,
    pub msg_queue: Vec<(
        MessageSetItem<MS>,
        oneshot::Sender<MessageSetReplayItem<MS>>,
    )>,
    pub concurrent_msg_buf: FuturesUnordered<
        Either<
            std::future::Ready<Result<SyncConcurrentRelayAndSender<MS>, JoinError>>,
            JoinHandle<SyncConcurrentRelayAndSender<MS>>,
        >,
    >,
}

impl<MS> MessageSetReceiver<MS>
where
    MS: MessageSet,
{
    /*
        pub async fn handle(
            &mut self,
            handler: &mut MS::Handler,
        ) -> Result<(), mpsc::error::TryRecvError> {
            while let Some(result) = self.handle_next(handler).await {
                result?;
            }
            Ok(())
        }
    */
    pub async fn recv(
        &mut self,
    ) -> Option<(
        MessageSetItem<MS>,
        oneshot::Sender<MessageSetReplayItem<MS>>,
    )> {
        if !self.msg_queue.is_empty() {
            self.msg_queue.pop()
        } else {
            self.receiver.recv().await
        }
    }
    pub async fn handle_next(
        &mut self,
        handler: &mut MS::Handler,
    ) -> Result<Option<()>, MsgSetRecvError> {
        println!("handle_next");
        if let Some((msg, replay_sender)) = self.recv().await {

            println!("msg");
            match msg {
                MessageSetItem::Sync(msg) => {
                    let replay = HandleSync::handle(handler, msg);
                    let _ = replay_sender.send(MessageSetReplayItem::Sync(replay));
                }
                MessageSetItem::Async(msg) => {
                    let replay = HandleAsync::handle(handler, msg).await;
                    let _ = replay_sender.send(MessageSetReplayItem::Async(replay));
                }
                MessageSetItem::SyncConcurrent(msg) => {
                    self.handle_concurrent(handler, (Either::Left(msg), replay_sender))
                        .await?;
                }
                MessageSetItem::AsyncConcurrent(msg) => {
                    self.handle_concurrent(handler, (Either::Right(msg), replay_sender))
                        .await?;
                }
            }
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    async fn handle_concurrent(
        &mut self,
        handler: &mut MS::Handler,
        (init_msg, sender): (
            Either<MS::SyncConcurrentVariant, MS::AsyncConcurrentVariant>,
            oneshot::Sender<MessageSetReplayItem<MS>>,
        ),
    ) -> Result<(), MsgSetRecvError> {
        let mut async_futures = None;
        self.concurrent_msg_buf.clear();
        let handler = unsafe { &*(handler as *const MS::Handler) };
        match init_msg {
            Either::Left(sync_msg) => {
                self.push_sync_concurrent((sync_msg, sender), unsafe {
                    force_send_sync::Sync::new(handler)
                });
            }
            Either::Right(async_msg) => {
                if async_futures.is_none() {
                    async_futures = Some(FuturesUnordered::new());
                }
                async_futures.as_mut().unwrap().push(
                    async {
                        (
                            HandleAsyncConcurrent::handle(handler, async_msg).await,
                            sender,
                        )
                    }
                    .left_future(),
                );
            }
        }

        loop {
            let (msg,replay_sender) = match self.receiver.try_recv() {
                Ok(r) => r,
                Err(TryRecvError::Disconnected) => {
                    return Err(MsgSetRecvError::Disconnected);
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
            };
            match msg {
                MessageSetItem::SyncConcurrent(msg) => {
                    self.push_sync_concurrent((msg, replay_sender), unsafe {
                        force_send_sync::Sync::new(handler)
                    });
                }
                MessageSetItem::AsyncConcurrent(msg) => {
                    if async_futures.is_none() {
                        async_futures = Some(FuturesUnordered::new());
                    }
                    async_futures.as_mut().unwrap().push(
                        async {
                            (
                                HandleAsyncConcurrent::handle(handler, msg).await,
                                replay_sender,
                            )
                        }
                        .right_future(),
                    );
                }
                msg => {
                    self.msg_queue.push((msg, replay_sender));
                    break;
                }
            }
        }
        let mut async_concurrent_futures = async_futures
            .as_mut()
            .map(|n| n.next().left_future())
            .unwrap_or(std::future::pending::<_>().right_future());
        let mut sync_concurrent_futures = self.concurrent_msg_buf.next().left_future();
        loop {
            match futures_util::future::select(async_concurrent_futures, sync_concurrent_futures)
                .await
            {
                Either::Left((left_result, right)) => {
                    sync_concurrent_futures = right;
                    match left_result {
                        None => {
                            if matches!(sync_concurrent_futures, Either::Right(_)) {
                                break;
                            }
                            async_concurrent_futures = std::future::pending::<_>().right_future();
                        }
                        Some((replay, sender)) => {
                            let _ = sender.send(MessageSetReplayItem::AsyncConcurrent(replay));
                            async_concurrent_futures = async_futures
                                .as_mut()
                                .map(|n| n.next().left_future())
                                .unwrap_or(std::future::pending::<_>().right_future());
                        }
                    }
                }
                Either::Right((right_result, left)) => {
                    async_concurrent_futures = left;
                    match right_result {
                        None => {
                            if matches!(async_concurrent_futures, Either::Right(_)) {
                                break;
                            }
                            sync_concurrent_futures = std::future::pending::<_>().right_future();
                        }
                        Some(replay) => {
                            let (replay, sender) = replay.map_err(|n| MsgSetRecvError::JoinError(n))?;
                            let _ = sender.send(MessageSetReplayItem::SyncConcurrent(replay));
                            sync_concurrent_futures = self.concurrent_msg_buf.next().left_future();
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn push_sync_concurrent(
        &mut self,
        (msg, replay_sender): (
            MS::SyncConcurrentVariant,
            oneshot::Sender<MessageSetReplayItem<MS>>,
        ),
        handler: force_send_sync::Sync<&'static MS::Handler>,
    ) {
        let is_blocking = HandleSyncConcurrent::is_blocking(*handler, &msg);
        if !is_blocking {
            let replay = HandleSyncConcurrent::handle(*handler, msg);
            self.concurrent_msg_buf
                .push(ready(Ok((replay, replay_sender))).left_future());
        } else {
            self.concurrent_msg_buf.push(
                tokio::task::spawn_blocking(move || {
                    (HandleSyncConcurrent::handle(*handler, msg), replay_sender)
                })
                .right_future(),
            );
        }
    }
}
