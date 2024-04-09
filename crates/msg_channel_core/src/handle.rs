use std::future::Future;

pub trait HandleReplay<M> {
    type Replay: Send + 'static;
    type MsgReplay: Into<Self::Replay>;
}

pub trait HandleSync<M>: Sync {
    fn is_blocking(&self, _msg: &M) -> bool {
        true
    }
    type Replay: Send + 'static;
    fn handle(&mut self, msg: M) -> Self::Replay;
}

impl<T: Sync> HandleSync<()> for T {
    type Replay = ();

    fn handle(&mut self, _: ()) -> Self::Replay {}
}

pub trait HandleAsync<M> {
    type Replay: Send + 'static;
    fn handle(&mut self, msg: M) -> impl Future<Output = Self::Replay>;
}

impl<T> HandleAsync<()> for T {
    type Replay = ();

    async fn handle(&mut self, _: ()) -> Self::Replay {}
}
pub trait HandleAsyncConcurrent<M> {
    type Replay: Send + 'static;
    fn handle(&self, msg: M) -> impl Future<Output = Self::Replay>;
}
impl<T> HandleAsyncConcurrent<()> for T {
    type Replay = ();

    async fn handle(&self, _: ()) -> Self::Replay {}
}
pub trait HandleSyncConcurrent<M>: Sync {
    fn is_blocking(&self, _msg: &M) -> bool {
        true
    }
    type Replay: Send + 'static;
    fn handle(&self, msg: M) -> Self::Replay;
}
impl<T: Sync> HandleSyncConcurrent<()> for T {
    type Replay = ();

    fn handle(&self, _: ()) -> Self::Replay {}
}
