mod auth;
mod dashboard;
mod dashboard_admin;
mod profile;
mod results;
mod speedtest;
mod tokens;

// Re-export all handlers
pub use auth::{login_page, login_post, logout};
pub use dashboard::home_dashboard;
pub use dashboard_admin::admin_dashboard;
pub use profile::{profile_page, profile_update};
pub use results::{delete_results, results_list};
pub use speedtest::{run_test_execute, run_test_page};
pub use tokens::{api_tokens_page, create_token, delete_token, edit_token_page, update_token};
