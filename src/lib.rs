use actix_web::{
    web::{resource, Data},
    App, HttpResponse, HttpServer, Responder,
};
use lazy_static::lazy_static;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde:: {Serialize, Deserialize};
use tera::Tera;

#[allow(unused_imports)]
pub(crate) mod entities;

mod index;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

#[derive(Deserialize)]
pub struct Settings{
    pub servers: Vec<Server>,
}

#[derive(Deserialize)]
pub struct Server{
    pub name: String,
    pub port: u16,
    pub doubles: bool,
}

#[derive(Serialize)]
struct Player {
    name: String,
    id: i32,
    rank: usize,
    elo1: i32,
    elo1_type: EloType,
    elo2: i32,
    elo2_type: EloType,
}

#[derive(Serialize)]
struct PlayerDetails {
    name: String,
    id: i32,
    elo1: i32,
    elo2: i32,
    games1: Vec<Game1Details>,
    games2: Vec<Game2Details>,
}

#[derive(Serialize, Debug, Clone)]
struct Game1Details{
    opponent: String,
    elo_diff: String,
    date: String,
    win: bool,
    internal_datetime: chrono::NaiveDateTime,
}

#[derive(Serialize)]
struct Game2Details{
    
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum EloType {
    Single,
    Double,
}

#[derive(serde::Deserialize)]
pub struct HistoryInput {
    player_id: i32,
}

pub async fn start_server() {
    let db = Data::new(
        Database::connect(ConnectOptions::new(
            "postgres://local:local@localhost:5432/elo_k".to_owned(),
        ))
        .await
        .unwrap(),
    );
    HttpServer::new(move || {
        App::new()
            .app_data(db.clone())
            .service(resource("/").to(index::route))
            .service(resource("/add_player").to(add_player))
            .service(resource("/history").to(history))
            .service(resource("/add_game1").to(add_game_1))
            .service(resource("/add_game2").to(add_game_2))
            .service(resource("/favicon.ico").to(favicon))
            .service(resource("/style.css").to(style))
    })
    .bind("localhost:8000")
    .unwrap()
    .run()
    .await
    .unwrap()
}

pub async fn add_game_1(db: Data<DatabaseConnection>) -> impl Responder {
    let context = index::get_players(db).await;

    let tera = &TEMPLATES;

    let html = tera.render("add_game1.html", &context).unwrap();

    HttpResponse::Ok().body(html)
}

pub async fn add_game_2(db: Data<DatabaseConnection>) -> impl Responder {
    let context = index::get_players(db).await;

    let tera = &TEMPLATES;

    let html = tera.render("add_game2.html", &context).unwrap();

    HttpResponse::Ok().body(html)
}

pub async fn add_player() -> impl Responder {
    let context = tera::Context::new();

    let tera = &TEMPLATES;

    let html = tera.render("add_player.html", &context).unwrap();

    HttpResponse::Ok().body(html)
}

pub async fn history(
    db: Data<DatabaseConnection>,
    path: actix_web::web::Query<HistoryInput>,
) -> impl Responder {
    let context = index::get_player(db, path.player_id).await;

    let tera = &TEMPLATES;

    let html = tera.render("history.html", &context).unwrap();

    HttpResponse::Ok().body(html)
}

pub async fn style() -> impl Responder {
    actix_files::NamedFile::open_async("./static/style.css")
        .await
        .unwrap()
}

pub async fn favicon() -> impl Responder {
    actix_files::NamedFile::open_async("./static/favicon.ico")
        .await
        .unwrap()
}
