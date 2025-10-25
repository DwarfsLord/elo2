use clap::Parser;

#[actix_web::main]
async fn main() {
    let args = elo2::Args::parse();

    elo2::start_server(args).await;
}
