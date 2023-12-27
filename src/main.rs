#[actix_web::main]
async fn main() {
    elo2::start_server().await;
}
