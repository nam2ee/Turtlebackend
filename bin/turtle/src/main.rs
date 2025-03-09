use turtle_net::server::build_server;

#[tokio::main]
async fn main() {
    // build our application with a single route
    build_server().await;

}