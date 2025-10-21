use crate::values;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, errors::Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: u64,
    pub username: String,
    pub team_id: u64,
}

pub fn decode_jwt(token: &str) -> Result<TokenData<Claims>, Error> {
    let mut valid = Validation::new(Algorithm::HS256);
    let key = values::config::get_config()
        .server
        .security
        .jwt_secret
        .clone();
    valid.set_required_spec_claims(&["exp", "iss", "iat"]);
    valid.set_issuer(&["rodan"]);
    valid.validate_exp = true;
    decode::<Claims>(token, &DecodingKey::from_secret(key.as_bytes()), &valid)
}
