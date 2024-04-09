#[macro_export]
macro_rules! define_msg_variant {
    ($set_name:ident;$handler:ident;$prefix:ident;$handler_trait:ident;$($msg:ident),*$(,)?) => {
        msg_channel::internal::paste! {
            pub enum [<$set_name $prefix Variant>] {
                $($msg($msg),)*
            }
            pub enum [<$set_name $prefix ReplayVariant>] {
                $($msg(<$handler as $handler_trait<$msg>>::Replay),)*
            }

            pub struct [<$handler $set_name $prefix VariantReplay>](pub [<$set_name $prefix ReplayVariant>]);

            impl Into<[<$set_name $prefix ReplayVariant>]> for [<$handler $set_name $prefix VariantReplay>] {
                fn into(self) -> [<$set_name $prefix ReplayVariant>] {
                    self.0
                }
            }

            impl HandleReplay<[<$set_name $prefix Variant>]> for $handler {
                type Replay = [<$set_name $prefix ReplayVariant>];
                type MsgReplay = [<$handler $set_name $prefix VariantReplay>];
            }

            impl Into<MessageSetItem<$set_name>> for [<$set_name $prefix Variant>] {
                fn into(self) -> MessageSetItem<$set_name>{
                    MessageSetItem::<$set_name>::$prefix(self)
                }
            }

            impl From<MessageSetReplayItem<$set_name>> for [<$handler $set_name $prefix VariantReplay>] {
                fn from(value: MessageSetReplayItem<$set_name>)-> Self {
                    let MessageSetReplayItem::<$set_name>::$prefix(replay) = value else {
                        unreachable!()
                    };
                    [<$handler $set_name $prefix VariantReplay>] (replay)
                }
            }

            $(
                pub struct [<$handler $msg Replay>](pub <$handler as $handler_trait<$msg>>::Replay);

                impl Into<<$handler as $handler_trait<$msg>>::Replay> for [<$handler $msg Replay>] {
                    fn into(self) -> <$handler as $handler_trait<$msg>>::Replay{
                        self.0
                    }
                }

                impl HandleReplay<$msg> for $handler {
                    type Replay = <$handler as $handler_trait<$msg>>::Replay;
                    type MsgReplay = [<$handler $msg Replay>];
                }

                impl Into<MessageSetItem<$set_name>> for $msg {
                    fn into(self) -> MessageSetItem<$set_name>{
                        MessageSetItem::<$set_name>::$prefix([<$set_name $prefix Variant>]::$msg(self))
                    }
                }

                impl From<MessageSetReplayItem<$set_name>> for [<$handler $msg Replay>] {
                    fn from(value: MessageSetReplayItem<$set_name>)-> Self {
                        let MessageSetReplayItem::<$set_name>::$prefix(replay) = value else {
                            unreachable!()
                        };
                        let [<$set_name $prefix ReplayVariant>]::$msg(replay) = replay else {
                            unreachable!()
                        };
                        [<$handler $msg Replay>](replay)
                    }
                }
            )*
        }
    };
}
#[macro_export]
macro_rules! impl_msg_handle_for_variant {
    ($name:ident;$handler:ident;$prefix:ident;$handler_trait:ident;$($msg:ident),*$(,)?) => {
        msg_channel::internal::define_msg_variant!($name;$handler;$prefix;$handler_trait;$($msg),*);
        msg_channel::internal::paste! {
            impl $handler_trait<[<$name $prefix Variant>]> for $handler {
                type Replay = [<$name $prefix ReplayVariant>];

                async fn handle(&mut self, msg: [<$name $prefix Variant>]) -> Self::Replay {
                    match msg {
                        $(
                        [<$name $prefix Variant>]::$msg(msg) => {
                            [<$name $prefix ReplayVariant>]::$msg(
                                $handler_trait::<$msg>::handle(self,msg).await
                            )
                        }
                        )*
                    }
                }
            }
        }
    };
}
#[macro_export]
macro_rules! impl_sync_msg_handle_for_variant {
    ($name:ident;$handler:ident;$prefix:ident;$handler_trait:ident;$($msg:ident),*$(,)?) => {
        msg_channel::internal::define_msg_variant!($name;$handler;$prefix;$handler_trait;$($msg),*);
        msg_channel::internal::paste! {
            impl $handler_trait<[<$name $prefix Variant>]> for $handler {
                type Replay = [<$name $prefix ReplayVariant>];

                fn is_blocking(&self, msg: &[<$name $prefix Variant>]) -> bool {
                    match msg {
                        $(
                        [<$name $prefix Variant>]::$msg(msg) => {
                            $handler_trait::<$msg>::is_blocking(self,msg)
                        }
                        )*
                        _ => false
                    }
                }

                fn handle(&mut self, msg: [<$name $prefix Variant>]) -> Self::Replay {
                    match msg {
                        $(
                        [<$name $prefix Variant>]::$msg(msg) => {
                            [<$name $prefix ReplayVariant>]::$msg(
                                $handler_trait::<$msg>::handle(self,msg)
                            )
                        }
                        )*
                    }
                }
            }
        }
    };
}
#[macro_export]
macro_rules! impl_concurrent_msg_handle_for_variant {
    ($name:ident;$handler:ident;$prefix:ident;$handler_trait:ident;$($msg:ident),*$(,)?) => {
        msg_channel::internal::define_msg_variant!($name;$handler;$prefix;$handler_trait;$($msg),*);
        msg_channel::internal::paste! {
            impl $handler_trait<[<$name $prefix Variant>]> for $handler {
                type Replay = [<$name $prefix ReplayVariant>];

                async fn handle(&self, msg: [<$name $prefix Variant>]) -> Self::Replay {
                    match msg {
                        $(
                        [<$name $prefix Variant>]::$msg(msg) => {
                            [<$name $prefix ReplayVariant>]::$msg(
                                $handler_trait::<$msg>::handle(self,msg).await
                            )
                        }
                        )*
                    }
                }
            }
        }
    };
}
#[macro_export]
macro_rules! impl_sync_concurrent_msg_handle_for_variant {
    ($name:ident;$handler:ident;$prefix:ident;$handler_trait:ident;$($msg:ident),*$(,)?) => {
        msg_channel::internal::define_msg_variant!($name;$handler;$prefix;$handler_trait;$($msg),*);
        msg_channel::internal::paste! {
            impl $handler_trait<[<$name $prefix Variant>]> for $handler {
                type Replay = [<$name $prefix ReplayVariant>];

                fn is_blocking(&self, msg: &[<$name $prefix Variant>]) -> bool {
                    match msg {
                        $(
                        [<$name $prefix Variant>]::$msg(msg) => {
                            $handler_trait::<$msg>::is_blocking(self,msg)
                        }
                        )*
                        _ => false
                    }
                }

                fn handle(&self, msg: [<$name $prefix Variant>]) -> Self::Replay {
                    match msg {
                        $(
                        [<$name $prefix Variant>]::$msg(msg) => {
                            [<$name $prefix ReplayVariant>]::$msg(
                                $handler_trait::<$msg>::handle(self,msg)
                            )
                        }
                        )*
                    }
                }
            }
        }
    };
}
