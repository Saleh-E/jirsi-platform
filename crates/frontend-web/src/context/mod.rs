//! Context modules

pub mod theme;
pub mod theme_context;
pub mod socket;
pub mod mobile;
pub mod network_status;

pub use theme::{ThemeContext as SimpleThemeContext, provide_theme_context as provide_simple_theme, use_theme as use_simple_theme, ThemeToggle};
pub use theme_context::{Theme, ThemeMode, ThemeContext, provide_theme_context as provide_jirsi_theme, use_theme as use_jirsi_theme};
pub use socket::{SocketProvider, SocketContext, use_socket, WsEvent};
pub use mobile::{MobileContext, provide_mobile_context, use_mobile, is_mobile};
pub use network_status::{NetworkStatus, NetworkStatusContext, provide_network_status, use_network_status, NetworkStatusBadge};

