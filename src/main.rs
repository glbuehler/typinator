use typinator::*;

#[tokio::main]
async fn main() {
    enter();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        exit();
        hook(info);
    }));

    run().await;

    exit();
}
