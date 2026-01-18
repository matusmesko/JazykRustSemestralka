pub mod users;
pub mod messages;
pub mod auth;
pub mod session;
mod utils;
mod controller;
mod blocked;

use library::applicaton::ServerRun;

#[tokio::main]
async fn main() {
    ServerRun::start().await.unwrap();
}
