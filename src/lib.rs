pub use handle::*;
pub use message_set::*;
use msg_channel_core::{handle,message_set};
pub use msg_channel_core::msg_channel;
pub use msg_channel_macro::msg_set;

pub mod internal {
    pub use msg_channel_core::{
        define_msg_variant, impl_concurrent_msg_handle_for_variant, impl_msg_handle_for_variant,
        impl_sync_concurrent_msg_handle_for_variant, impl_sync_msg_handle_for_variant,
    };
    pub use paste::paste;
}
