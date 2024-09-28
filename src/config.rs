use std::env;

use dotenvy::dotenv;

pub struct Config {
    pub unisender_api_key: String,
    pub unisender_sender_name: String,
    pub unisender_sender_email: String,
    pub github_app_id: u64,
    pub github_app_private_key: String,
    #[allow(unused)]
    pub jwt_secret: String,
    #[allow(unused)]
    pub database_url: String
}

impl Config {
    pub fn is_debug() -> bool {
        !env::var("PROD")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false)
    }

    pub fn get_env_param(param_name: &str) -> String {
        env::var(param_name).unwrap_or_else(|_| {
            log::error!("Variable {param_name} seems not specified.");
            std::process::exit(1);
        })
    }

    pub fn from_env() -> Config {
        if Self::is_debug() {
            dotenv().ok();
        }

        let unisender_api_key = Self::get_env_param("UNISENDER_API_KEY");
        let unisender_sender_name =
            env::var("UNISENDER_SENDER_NAME").unwrap_or("krakker".to_string());
        let unisender_sender_email =
            env::var("UNISENDER_SENDER_EMAIL").unwrap_or("noreply@krakker.org".to_string());
        let github_app_id = match Self::get_env_param("GITHUB_APP_ID").parse() {
            Ok(id) => id,
            Err(err) => {
                log::error!("Failed to parse GITHUB_APP_ID: {err}");
                std::process::exit(1);
            }
        };
        let github_app_private_key = Self::get_env_param("GITHUB_APP_PRIVATE_KEY");
        let jwt_secret = Self::get_env_param("JWT_SECRET");
        let database_url = Self::get_env_param("DATABASE_URL");

        Config {
            unisender_api_key,
            unisender_sender_name,
            unisender_sender_email,
            github_app_id,
            github_app_private_key,
            jwt_secret,
            database_url
        }
    }
}
