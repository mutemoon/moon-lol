//! User/Auth 子系统的领域层。
//!
//! 含用户领域类型、密码哈希校验、JWT token 生成与解析（纯逻辑，无 IO）。

use chrono::{Duration, Utc};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error as JwtError,
};
use serde::{Deserialize, Serialize};

/// 用户领域类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i32,
    pub phone: String,
}

/// JWT claims。
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub exp: usize,
}

/// 手机号校验：中国大陆 11 位手机号。
///
/// 规则：1 开头，第二位 3-9，共 11 位数字。
pub fn validate_phone(phone: &str) -> bool {
    let p = phone.trim();
    p.len() == 11
        && p.starts_with('1')
        && p.chars().all(|c| c.is_ascii_digit())
        && p.chars()
            .nth(1)
            .map(|c| ('3'..='9').contains(&c))
            .unwrap_or(false)
}

/// 密码强度校验：至少 6 位。
pub fn validate_password(password: &str) -> bool {
    password.len() >= 6
}

/// 哈希密码（bcrypt）。
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

/// 校验密码（bcrypt）。
pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}

/// 生成 JWT（有效期 30 天）。
pub fn generate_jwt(user_id: i32, secret: &str) -> Result<String, JwtError> {
    let exp = (Utc::now() + Duration::days(30)).timestamp() as usize;
    let claims = Claims { user_id, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

/// 解析 JWT，返回 claims。失败返回错误。
pub fn parse_jwt(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_phones() {
        assert!(validate_phone("13812345678"));
        assert!(validate_phone("15900000000"));
        assert!(validate_phone("18699998888"));
    }

    #[test]
    fn invalid_phones() {
        assert!(!validate_phone("12345678901"), "第二位不能是 2");
        assert!(!validate_phone("1281234567"), "10 位太短");
        assert!(!validate_phone("138123456789"), "12 位太长");
        assert!(!validate_phone("23812345678"), "必须 1 开头");
        assert!(!validate_phone("1381234abcd"), "必须纯数字");
        assert!(!validate_phone(""), "空串");
        // trim 后合法
        assert!(validate_phone("  13812345678  "));
    }

    #[test]
    fn password_min_length() {
        assert!(validate_password("123456"));
        assert!(validate_password("a_very_long_password"));
    }

    #[test]
    fn password_too_short() {
        assert!(!validate_password("12345"));
        assert!(!validate_password(""));
    }

    #[test]
    fn password_hash_and_verify_roundtrip() {
        let password = "my_secret_123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn jwt_generate_and_parse_roundtrip() {
        let secret = "test-secret-key";
        let token = generate_jwt(42, secret).unwrap();
        let claims = parse_jwt(&token, secret).unwrap();
        assert_eq!(claims.user_id, 42);
    }

    #[test]
    fn jwt_wrong_secret_fails() {
        let token = generate_jwt(42, "correct-secret").unwrap();
        assert!(parse_jwt(&token, "wrong-secret").is_err());
    }

    #[test]
    fn jwt_expired_token_detected() {
        // 构造一个已过期的 claims（exp 设为过去）
        let exp = (Utc::now() - Duration::days(1)).timestamp() as usize;
        let claims = Claims { user_id: 1, exp };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"secret"),
        )
        .unwrap();
        assert!(parse_jwt(&token, "secret").is_err());
    }
}
