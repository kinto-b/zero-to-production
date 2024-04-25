use crate::configuration::{ApplicationSettings, DatabaseSettings, EmailClientSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

pub struct Application {
    listener: TcpListener,
    pub connection: PgPool,
    email_client: EmailClient,
}

impl Application {
    pub fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection = get_connection(configuration.database);
        let email_client = get_email_client(configuration.email_client);
        let listener = get_listener(configuration.application)?;

        Ok(Self {
            listener,
            connection,
            email_client,
        })
    }

    pub fn run(self) -> Result<Server, std::io::Error> {
        let connection = web::Data::new(self.connection);
        let email_client = web::Data::new(self.email_client);

        let server = HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .route("/health_check", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .app_data(connection.clone())
                .app_data(email_client.clone())
        })
        .listen(self.listener)?
        .run();
        Ok(server)
    }

    pub fn port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }
}

fn get_email_client(configuration: EmailClientSettings) -> EmailClient {
    let sender_email = configuration
        .sender()
        .expect("Invalid sender email address");
    let timeout = configuration.timeout();

    EmailClient::new(
        configuration.base_url,
        sender_email,
        configuration.token,
        timeout,
    )
}

fn get_connection(configuration: DatabaseSettings) -> PgPool {
    PgPool::connect_lazy_with(configuration.with_db())
}

fn get_listener(configuration: ApplicationSettings) -> Result<TcpListener, std::io::Error> {
    let address = format!("{}:{}", configuration.host, configuration.port);
    TcpListener::bind(address)
}
