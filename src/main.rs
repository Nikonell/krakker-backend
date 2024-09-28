use std::time::Duration;

use actix_cors::Cors;
use actix_web::{middleware::Logger, App, HttpServer};
use apistos::{
    app::{BuildConfig, OpenApiWrapper}, web::scope, ScalarConfig
};
use github::worker::GitHubWorker;
use mailer::mailer::Mailer;
use tokio_util::sync::CancellationToken;

use utils::{app_data::AppData, openapi::get_spec};
#[allow(unused)]
use services::common::create_prisma_client;

mod controllers;
mod models;
mod services;
mod middleware;
mod utils;
mod config;
mod mailer;
mod github;
#[allow(warnings, unused)]
mod prisma;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Config & logger initialization
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let config = config::Config::from_env();

    // Prisma migration
    {
        #[allow(unused)]
        let client = create_prisma_client().await.unwrap();
        #[cfg(not(debug_assertions))]
        client._migrate_deploy().await.unwrap();
    }

    // App data initialization
    let mut app_data = AppData {
        mailer: Mailer::new(
            &config.unisender_api_key,
            &config.unisender_sender_name,
            &config.unisender_sender_email,
            "ru"
        )
    };

    // Mailer initialization
    app_data.mailer.initialize_mail_list().await;

    // GitHub worker initialization
    let shutdown_token = CancellationToken::new();
    let gh_worker = GitHubWorker::new(
        config.github_app_id,
        &config.github_app_private_key,
        shutdown_token.clone(),
        Duration::from_secs(60)
    ).await.unwrap();

    actix_web::rt::spawn(async move {
        if let Err(e) = gh_worker.work().await {
            eprintln!("GitHub worker error: {}", e);
        }
    });

    // Http server start
    let server = HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(actix_web::web::Data::new(app_data.clone()))
            .document(get_spec())
            .service(scope("/api").configure(controllers::init_routes))
            .build_with(
                "/openapi.json",
                BuildConfig::default().with(ScalarConfig::new(&"/docs")),
            )
            .service(actix_web::web::scope("/uploads").configure(controllers::init_uploads))
    })
    .bind("0.0.0.0:1488")?
    .run();

    let srv = server.handle();

    tokio::spawn(async move {
        shutdown_token.cancelled().await;
        srv.stop(true).await;
    });

    server.await
}
