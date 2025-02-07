use bcrypt::{BcryptError, DEFAULT_COST};

pub fn hash_password(password: String) -> Result<String, BcryptError> {
    let hash = match bcrypt::hash(&password, DEFAULT_COST) {
        Ok(pwd) => Ok(pwd),
        Err(e) => Err(e),
    };
    return hash;
}

#[allow(dead_code)]
pub fn verify_password(password: String, hashed_password: &str) -> Result<bool, BcryptError> {
    let verify = match bcrypt::verify(&password, &hashed_password) {
        Ok(result) => Ok(result),
        Err(e) => Err(e),
    };
    return verify;
}
