//! Pages

pub mod home;
pub mod entity_list;
pub mod entity_detail;
pub mod login;
pub mod public;
pub mod profile;
pub mod settings;
pub mod dashboard;
pub mod reports;
pub mod inbox;

pub use home::HomePage;
pub use entity_list::EntityListPage;
pub use entity_detail::EntityDetailPage;
pub use login::LoginPage;
pub use profile::ProfilePage;
pub use settings::SettingsPage;
pub use dashboard::DashboardPage;
pub use reports::ReportsPage;
pub use inbox::InboxPage;
