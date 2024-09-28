use apistos::ApiComponent;
use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: u64,
    pub exp: usize,
    pub nbf: usize,
}

#[derive(Serialize, Deserialize, JsonSchema, ApiComponent)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, JsonSchema, ApiComponent, Validate)]
pub struct RegisterRequest {
    #[garde(length(min = 2))]
    #[schemars(length(min = 2))]
    pub first_name: String,
    #[garde(length(min = 2))]
    #[schemars(length(min = 2))]
    pub last_name: String,
    #[garde(email)]
    #[schemars(email)]
    pub email: String,
    #[garde(length(min = 3))]
    #[schemars(length(min = 3))]
    pub username: String,
    #[garde(length(min = 8), custom(validate_password))]
    #[schemars(length(min = 8), regex(pattern = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$"))]
    pub password: String,
    #[garde(matches(password))]
    #[schemars(length(min = 8), regex(pattern = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$"))]
    pub password_confirm: String,
}

#[derive(Serialize, Deserialize, JsonSchema, ApiComponent)]
pub struct AuthResponse {
    pub token: String,
}

fn validate_password(password: &str, _: &&&()) -> garde::Result {
    if password.len() < 8 {
        return Err(garde::Error::new("length is lower than 8"))
    } else if password.to_lowercase() == password || password.to_uppercase() == password {
        return Err(garde::Error::new("does not contain both upper and lower case letters"))
    } else if password.chars().all(|c| !c.is_digit(10)) {
        return Err(garde::Error::new("does not contain a digit"))
    }
    Ok(())
}
