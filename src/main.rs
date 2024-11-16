use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main(){
    let console_layer = console_subscriber::spawn();
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt_layer)
        .init();

    warp::serve(warp::fs::dir("web"))
    .run(([0,0,0,0], 8080))
    .await;
}