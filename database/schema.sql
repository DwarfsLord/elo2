-- SQL dump generated using DBML (dbml-lang.org)
-- Database: PostgreSQL
-- Generated at: 2023-11-22T13:12:03.923Z

CREATE TABLE "players" (
  "id" serial PRIMARY KEY NOT NULL,
  "name" varchar UNIQUE NOT NULL,
  "elo1" int NOT NULL,
  "elo2" int NOT NULL
);

CREATE TABLE "singles" (
  "id" serial PRIMARY KEY NOT NULL,
  "player_id_win" int NOT NULL,
  "player_id_loss" int NOT NULL,
  "old_elo_win" int NOT NULL,
  "old_elo_lose" int NOT NULL,
  "time" timestamp NOT NULL DEFAULT (CURRENT_TIMESTAMP)
);

CREATE TABLE "doubles" (
  "id" serial PRIMARY KEY NOT NULL,
  "player_id_win1" int NOT NULL,
  "player_id_win2" int NOT NULL,
  "player_id_loss1" int NOT NULL,
  "player_id_loss2" int NOT NULL,
  "old_elo_win1" int NOT NULL,
  "old_elo_win2" int NOT NULL,
  "old_elo_lose1" int NOT NULL,
  "old_elo_lose2" int NOT NULL,
  "time" timestamp NOT NULL DEFAULT (CURRENT_TIMESTAMP)
);

ALTER TABLE "singles" ADD FOREIGN KEY ("player_id_win") REFERENCES "players" ("id");

ALTER TABLE "singles" ADD FOREIGN KEY ("player_id_loss") REFERENCES "players" ("id");

ALTER TABLE "doubles" ADD FOREIGN KEY ("player_id_win1") REFERENCES "players" ("id");

ALTER TABLE "doubles" ADD FOREIGN KEY ("player_id_win2") REFERENCES "players" ("id");

ALTER TABLE "doubles" ADD FOREIGN KEY ("player_id_loss1") REFERENCES "players" ("id");

ALTER TABLE "doubles" ADD FOREIGN KEY ("player_id_loss2") REFERENCES "players" ("id");
