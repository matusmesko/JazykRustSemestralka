use library::applicaton;
use library::applicaton::ServerRun;

#[tokio::main]
async fn main() {
    ServerRun::start().await.unwrap();

}
