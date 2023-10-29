use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_url: String,
    pub jwt_secret: String,

    pub mail_username: String,
    pub mail_password: String,
    pub mail_host: String,
    pub mail_port: u16,
    pub mail_email: String,
    pub mail_author: String,
    pub mail_tls: bool,
}

impl Config {
    pub fn from_env() -> Config {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let mail_username = env::var("MAIL_USERNAME").expect("MAIL_USERNAME must be set");
        let mail_password = env::var("MAIL_PASSWORD").expect("MAIL_PASSWORD must be set");
        let mail_host = env::var("MAIL_HOST").expect("MAIL_HOST must be set");
        let mail_port = env::var("MAIL_PORT")
            .expect("MAIL_PORT must be set")
            .parse::<u16>()
            .expect("MAIL_PORT must be a number");
        let mail_email = env::var("MAIL_EMAIL").expect("MAIL_EMAIL must be set");
        let mail_author = env::var("MAIL_AUTHOR").expect("MAIL_AUTHOR must be set");
        let mail_tls = env::var("MAIL_TLS")
            .map(|v| v.parse::<bool>())
            .unwrap_or(Ok(false))
            .expect("MAIL_TLS must be a boolean");

        Config {
            db_url,
            jwt_secret,
            mail_username,
            mail_password,
            mail_host,
            mail_port,
            mail_email,
            mail_author,
            mail_tls,
        }
    }
}
