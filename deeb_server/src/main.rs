use actix_web::{App, HttpServer, web::Data};
use api::{
    auth as auth_api, delete_many, delete_one, find_many, find_one, insert_many, insert_one,
    update_many, update_one,
};
use app_data::AppData;
use clap::Parser;
use cli::Cli;

mod api;
pub mod app_data;
pub mod auth;
mod cli;
pub mod database;
pub mod environment;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();
    let app_data = AppData::new()?;

    log::info!("Deeb Server Starting...");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_data.clone()))
            .service(insert_one::insert_one)
            .service(find_one::find_one)
            .service(find_many::find_many)
            .service(insert_many::insert_many)
            .service(delete_one::delete_one)
            .service(delete_many::delete_many)
            .service(update_one::update_one)
            .service(update_many::update_many)
            .service(auth_api::me::me)
            .service(auth_api::register::register_user)
            .service(auth_api::login::login)
    })
    .bind((cli.host, cli.port))?
    .run()
    .await
}
