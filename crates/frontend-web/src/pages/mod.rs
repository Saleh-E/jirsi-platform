//! Pages

pub mod home;
pub mod entity_list;
pub mod entity_detail;
pub mod login;
pub mod register;
pub mod public;
pub mod profile;
pub mod settings;
pub mod dashboard;
pub mod reports;
pub mod inbox;
pub mod workflow_editor;
pub mod workflow_list;
pub mod automation;
pub mod component_playground;

pub use home::HomePage;
pub use entity_list::EntityListPage;
pub use entity_detail::EntityDetailPage;
pub use login::LoginPage;
pub use register::RegisterPage;
pub use profile::ProfilePage;
pub use settings::SettingsPage;
pub use dashboard::Dashboard;
pub use reports::ReportsPage;
pub use inbox::InboxPage;
pub use workflow_editor::WorkflowEditorPage;
pub use workflow_list::WorkflowListPage;
pub use automation::AutomationPage;


