use warp::Filter;

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    warp::serve(hello)
        // Set the IP address for docker to 0.0.0.0
        .run(([0, 0, 0, 0], 3030))
        .await;
}
