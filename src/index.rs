use actix_web::{
    web::{Data, Query},
    HttpResponse, Responder,
};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect,
    RelationTrait, Set, Statement,
};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{
    entities::{self, prelude::*},
    EloType, Game1Details, Player, PlayerDetails,
};

const CALC_ELO_MAX: f64 = 50.;
const CALC_ELO_A: f64 = 43.4294;

const PLAYER_DETAIL_PAGINATION_SIZE: usize = 10;

#[derive(Serialize)]
struct IndexPlayers {
    players: Vec<Player>,
}

impl From<Vec<crate::entities::players::Model>> for IndexPlayers {
    fn from(players: Vec<crate::entities::players::Model>) -> Self {
        let mut out = vec![];
        for (i, p) in players.into_iter().enumerate() {
            let player = if p.elo2 > p.elo1 {
                Player {
                    id: p.id,
                    name: p.name,
                    rank: i + 1,
                    elo1: p.elo2,
                    elo1_type: EloType::Double,
                    elo2: p.elo1,
                    elo2_type: EloType::Single,
                }
            } else {
                Player {
                    id: p.id,
                    name: p.name,
                    rank: i + 1,
                    elo1: p.elo1,
                    elo1_type: EloType::Single,
                    elo2: p.elo2,
                    elo2_type: EloType::Double,
                }
            };
            out.push(player);
        }
        Self { players: out }
    }
}

impl IndexPlayers {
    fn get_context(&self) -> Result<Context, tera::Error> {
        Context::from_serialize(&self)
    }
}

impl From<crate::entities::players::Model> for Player {
    fn from(value: crate::entities::players::Model) -> Self {
        Self {
            name: value.name,
            id: value.id,
            rank: 0,
            elo1: value.elo1,
            elo1_type: EloType::Single,
            elo2: value.elo2,
            elo2_type: EloType::Double,
        }
    }
}

// impl Player {
//     fn get_context(&self) -> Result<Context, tera::Error> {
//         Context::from_serialize(&self)
//     }
// }

#[derive(Deserialize)]
// #[serde(rename_all = "snake_case")]
pub struct IndexInput {
    add_player_name: Option<String>,
    winner: Option<String>,
    loser: Option<String>,
    winner1: Option<String>,
    winner2: Option<String>,
    loser1: Option<String>,
    loser2: Option<String>,
}

pub async fn route(db: Data<DatabaseConnection>, query: Query<IndexInput>) -> impl Responder {
    let query = query.into_inner();
    if let Some(name) = query.add_player_name {
        add_player(&db, name).await;
    }

    if let (Some(winner), Some(loser)) = (query.winner, query.loser) {
        add_game_1(&db, winner, loser).await;
    }

    if let (Some(winner1), Some(winner2), Some(loser1), Some(loser2)) =
        (query.winner1, query.winner2, query.loser1, query.loser2)
    {
        add_game_2(&db, winner1, winner2, loser1, loser2).await;
    }

    // let players: IndexPlayers = get_players(db).await.into();

    // let context = players.get_context().unwrap();

    let context = get_players(db).await;

    let tera = &crate::TEMPLATES;

    let html = tera.render("index.html", &context).unwrap();

    HttpResponse::Ok().body(html)
}

pub async fn get_players(db: Data<DatabaseConnection>) -> Context {
    let db: &DatabaseConnection = &db;

    let players: IndexPlayers = Players::find()
        .from_raw_sql(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            r#"SELECT * FROM players ORDER BY GREATEST(elo1,elo2) DESC"#,
        ))
        .all(db)
        .await
        .unwrap()
        .into();
    players.get_context().unwrap()
}

pub async fn get_player(db: Data<DatabaseConnection>, id: i32) -> Context {
    let db: &DatabaseConnection = &db;

    let player = Players::find_by_id(id).one(db).await.unwrap().unwrap();

    // let alias = Alias::new("winner");
    let wins1 = entities::singles::Entity::find()
        .filter(entities::singles::Column::PlayerIdWin.eq(player.id))
        .join(
            sea_orm::JoinType::Join,
            entities::singles::Relation::Players2.def(),
        )
        .select_also(entities::players::Entity)
        .all(db)
        .await
        .unwrap();

    let loss1 = entities::singles::Entity::find()
        .filter(entities::singles::Column::PlayerIdLoss.eq(player.id))
        .join(
            sea_orm::JoinType::Join,
            entities::singles::Relation::Players1.def(),
        )
        .select_also(entities::players::Entity)
        .all(db)
        .await
        .unwrap();

    let wins1: Vec<Game1Details> = wins1
        .into_iter()
        .map(|g| Game1Details {
            date: g.0.time.format("%d-%m %H:%M").to_string(),
            opponent: g.1.unwrap().name,
            elo_diff: (g.0.old_elo_win - g.0.old_elo_lose).to_string(),
            win: true,
            internal_datetime: g.0.time,
        })
        .collect();

    let loss1: Vec<Game1Details> = loss1
        .into_iter()
        .map(|g| Game1Details {
            date: g.0.time.format("%d-%m %H:%M").to_string(),
            opponent: g.1.unwrap().name,
            elo_diff: (g.0.old_elo_lose - g.0.old_elo_win).to_string(),
            win: false,
            internal_datetime: g.0.time,
        })
        .collect();

    let mut games1 = [wins1, loss1].concat();

    games1.sort_unstable_by_key(|g|g.internal_datetime);

    games1.reverse();

    let games1 = games1.chunks(PLAYER_DETAIL_PAGINATION_SIZE).next().unwrap().to_vec();

    // dbg!(&games1);

    let player_details = PlayerDetails {
        id: player.id,
        name: player.name,
        elo1: player.elo1,
        elo2: player.elo2,
        games1,
        games2: vec![],
    };
    Context::from_serialize(player_details).unwrap()
}

async fn add_player(db: &DatabaseConnection, name: String) {
    crate::entities::players::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: Set(name),
        elo1: Set(1000),
        elo2: Set(1000),
    }
    .insert(db)
    .await
    .unwrap();
}

async fn add_game_1(db: &DatabaseConnection, winner: String, loser: String) {
    let winner = Players::find()
        .filter(crate::entities::players::Column::Name.eq(winner))
        .one(db)
        .await
        .unwrap()
        .unwrap();
    let loser = Players::find()
        .filter(crate::entities::players::Column::Name.eq(loser))
        .one(db)
        .await
        .unwrap()
        .unwrap();

    crate::entities::singles::ActiveModel {
        player_id_win: Set(winner.id),
        player_id_loss: Set(loser.id),
        old_elo_win: Set(winner.elo1),
        old_elo_lose: Set(loser.elo1),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    let loser_chance = 1. / (((winner.elo1 as f64 - loser.elo1 as f64) / CALC_ELO_A).exp() + 1.);

    let gain = (loser_chance * CALC_ELO_MAX).ceil() as i32;

    let winner_elo = winner.elo1 + gain;
    let mut winner: crate::entities::players::ActiveModel = winner.into();
    winner.elo1 = Set(winner_elo);
    winner.update(db).await.unwrap();

    let loser_elo = loser.elo1 - gain;
    let mut loser: crate::entities::players::ActiveModel = loser.into();
    loser.elo1 = Set(loser_elo);
    loser.update(db).await.unwrap();
}

async fn add_game_2(
    db: &DatabaseConnection,
    winner1: String,
    winner2: String,
    loser1: String,
    loser2: String,
) {
    let winner1 = Players::find()
        .filter(crate::entities::players::Column::Name.eq(winner1))
        .one(db)
        .await
        .unwrap()
        .unwrap();
    let winner2 = Players::find()
        .filter(crate::entities::players::Column::Name.eq(winner2))
        .one(db)
        .await
        .unwrap()
        .unwrap();
    let loser1 = Players::find()
        .filter(crate::entities::players::Column::Name.eq(loser1))
        .one(db)
        .await
        .unwrap()
        .unwrap();
    let loser2 = Players::find()
        .filter(crate::entities::players::Column::Name.eq(loser2))
        .one(db)
        .await
        .unwrap()
        .unwrap();

    crate::entities::doubles::ActiveModel {
        player_id_win1: Set(winner1.id),
        player_id_win2: Set(winner2.id),
        player_id_loss1: Set(loser1.id),
        player_id_loss2: Set(loser2.id),
        old_elo_win1: Set(winner1.elo2),
        old_elo_win2: Set(winner2.elo2),
        old_elo_lose1: Set(loser1.elo2),
        old_elo_lose2: Set(loser2.elo2),
        ..Default::default()
    }
    .insert(db)
    .await
    .unwrap();

    let winner_elo = join_elo(winner1.elo2, winner2.elo2);
    let loser_elo = join_elo(loser1.elo2, loser2.elo2);

    let loser_chance = 1. / (((winner_elo as f64 - loser_elo as f64) / CALC_ELO_A).exp() + 1.);

    let gain = (loser_chance * CALC_ELO_MAX).ceil() as i32;

    let (w1, w2) = split_elo(winner1.elo2, winner2.elo2, gain * 2);
    let (l1, l2) = split_elo(loser1.elo2, loser2.elo2, gain * -2);

    set_elo2(db, w1, winner1.into()).await;
    set_elo2(db, w2, winner2.into()).await;
    set_elo2(db, l1, loser1.into()).await;
    set_elo2(db, l2, loser2.into()).await;
}

fn join_elo(a: i32, b: i32) -> f64 {
    let a = a as f64;
    let b = b as f64;
    a.max(b)
        .min(a.min(b) + (a - b).abs() * 0.5 + ((a - b).abs() * 0.03).powi(2))
}

fn split_elo(a: i32, b: i32, gain: i32) -> (i32, i32) {
    let a = a;
    let b = b;

    const DIVISOR: f64 = 70.;

    let mut responsibility_a = 1. / (1. + ((a as f64 - b as f64) / DIVISOR).exp2());
    if gain < 0 {
        responsibility_a = 1. - responsibility_a;
    }

    let mut res_a = (responsibility_a * gain as f64).round() as i32;
    let mut res_b = ((1. - responsibility_a) * gain as f64).round() as i32;

    //Guarantee change
    if res_a == 0 {
        res_a += gain.signum();
        res_b -= gain.signum();
    } else if res_b == 0 {
        res_b += gain.signum();
        res_a -= gain.signum();
    }

    res_a += a;
    res_b += b;

    //Guarantee overall positive change (for victors)
    if gain > 0 {
        let old_elo = join_elo(a, b);
        while old_elo > join_elo(res_a, res_b) {
            res_a += (res_a - res_b).signum();
            res_b -= (res_a - res_b).signum();
        }
    }

    (res_a, res_b)
}

async fn set_elo2(
    db: &DatabaseConnection,
    new_elo: i32,
    mut model: crate::entities::players::ActiveModel,
) {
    model.elo2 = Set(new_elo);
    model.update(db).await.unwrap();
}
