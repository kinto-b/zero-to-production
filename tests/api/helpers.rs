use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// Initialise `tracing` once on startup
static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// Spin up an instance of our application and return its address
    pub async fn spawn() -> Self {
        Lazy::force(&TRACING);

        let configuration = {
            let mut cfg = get_configuration().expect("Failed to read configuration.");
            cfg.database.database_name = Uuid::new_v4().to_string();
            cfg.application.port = 0; // Use random OS allocated port
            cfg
        };

        let db_config = configuration.database.clone();
        configure_database(&db_config).await;

        let application = Application::build(configuration).expect("Failed to build application");
        let connection = application.connection.clone();
        let address = format!("http://127.0.0.1:{}", application.port());
        let server = application.run().expect("Failed to bind address");

        let _ = tokio::spawn(server);

        TestApp {
            address,
            db_pool: connection,
        }
    }
}

async fn configure_database(database: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&database.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, database.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let pool = PgPool::connect_with(database.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the database.");

    pool
}
