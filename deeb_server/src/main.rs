use actix_web::{App, HttpServer, web::Data};
use api::{delete_many, delete_one, find_many, find_one, insert_many, insert_one, update_one};
use app_data::AppData;
use clap::Parser;
use cli::Cli;
use database::Database;

mod api;
pub mod app_data;
mod cli;
pub mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    log::info!("Deeb Server Starting...");

    HttpServer::new(|| {
        let database = Database::new();
        let app_data = AppData { database };

        App::new()
            .app_data(Data::new(app_data))
            .service(insert_one::insert_one)
            .service(find_one::find_one)
            .service(find_many::find_many)
            .service(insert_many::insert_many)
            .service(delete_one::delete_one)
            .service(delete_many::delete_many)
            .service(update_one::update_one)
    })
    .bind((cli.host, cli.port))?
    .run()
    .await
}
