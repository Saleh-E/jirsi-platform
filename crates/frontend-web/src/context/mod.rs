//! Context modules

pub mod theme;
pub mod socket;
pub mod mobile;

pub use theme::{ThemeContext, provide_theme_context, use_theme};
pub use socket::{SocketProvider, SocketContext, use_socket, WsEvent};
pub use mobile::{MobileContext, provide_mobile_context, use_mobile, is_mobile};

