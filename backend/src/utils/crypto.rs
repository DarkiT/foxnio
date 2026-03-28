//! 加密工具
//!
//! 注意：部分工具函数暂未使用，保留供未来扩展

#![allow(dead_code)]

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

/// Base64 编码
pub fn base64_encode(data: &[u8]) -> String {
    BASE64.encode(data)
}

/// Base64 解码
pub fn base64_decode(data: &str) -> Result<Vec<u8>> {
    let decoded = BASE64.decode(data)?;
    Ok(decoded)
}

/// 生成随机字节
pub fn random_bytes(len: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut bytes = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

/// 生成随机十六进制字符串
pub fn random_hex(len: usize) -> String {
    hex::encode(random_bytes(len / 2))
}

/// HMAC-SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC initialization failed");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// SHA256 哈希
pub fn sha256(data: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// 哈希密码
pub fn hash_password(password: &str) -> Result<String> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
        .to_string();
    Ok(hash)
}

/// 验证密码
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Invalid hash format: {}", e))?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
