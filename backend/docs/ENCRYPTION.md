# Encryption System - FoxNIO v0.2.0

## Overview

FoxNIO implements a comprehensive encryption system for sensitive data storage using AES-256-GCM encryption. This system provides:

- **Transparent encryption/decryption** for sensitive fields
- **Key rotation support** for secure key management
- **Multiple encryption targets**: API Keys, OAuth tokens, TOTP secrets

## Architecture

### Encryption Service

Located at `src/utils/encryption.rs`, the core encryption service provides:

```rust
pub struct EncryptionService {
    master_key: [u8; 32],
    old_master_key: Option<[u8; 32]>, // For key rotation
}

impl EncryptionService {
    pub fn encrypt(&self, plaintext: &str) -> Result<String>
    pub fn decrypt(&self, ciphertext: &str) -> Result<String>
    pub fn hash_sensitive(&self, data: &str) -> Result<String>
}
```

### Encryption Format

Encrypted data is stored as Base64-encoded string with the following format:

```
[Nonce (12 bytes)] + [Ciphertext + Auth Tag]
```

Total overhead: 28 bytes (12 byte nonce + 16 byte auth tag)

## Configuration

### Environment Variables

```bash
# Single key mode
FOXNIO_MASTER_KEY="base64-encoded-32-byte-key"

# Key rotation mode (new_key:old_key)
FOXNIO_MASTER_KEY="new-base64-key:old-base64-key"
```

### Generate a New Key

You can generate a new master key using the built-in function:

```rust
let key = EncryptionService::generate_master_key();
println!("Set FOXNIO_MASTER_KEY={}", key);
```

Or using command line:

```bash
# Using openssl
openssl rand -base64 32

# Using Python
python3 -c "import secrets, base64; print(base64.b64encode(secrets.token_bytes(32)).decode())"
```

## Encrypted Fields

### 1. Account Credentials (API Keys)

**Table**: `accounts`  
**Field**: `credential`

```rust
// Encrypting API key when creating account
let enc = encryption_service();
let encrypted_key = enc.encrypt(&api_key)?;

// Store encrypted
account.credential = encrypted_key;
```

### 2. OAuth Tokens

**Table**: `oauth_tokens`  
**Fields**: `access_token`, `refresh_token`

```rust
// Creating OAuth token
let encrypted_access = enc.encrypt(&access_token)?;
let encrypted_refresh = enc.encrypt(&refresh_token)?;

oauth_token.access_token = encrypted_access;
oauth_token.refresh_token = Some(encrypted_refresh);
```

### 3. User TOTP Secrets

**Table**: `users`  
**Field**: `totp_secret`

```rust
// Storing TOTP secret
let encrypted_secret = enc.encrypt(&totp_secret)?;
user.totp_secret = Some(encrypted_secret);
```

## Usage Examples

### Basic Encryption

```rust
use foxnio::utils::{EncryptionService, encryption_service};

// Get global encryption service
let enc = encryption_service();

// Encrypt
let encrypted = enc.encrypt("my-secret-api-key")?;

// Decrypt
let decrypted = enc.decrypt(&encrypted)?;
assert_eq!(decrypted, "my-secret-api-key");
```

### Using EncryptedString Wrapper

```rust
use foxnio::utils::{EncryptionService, EncryptedString};

let enc = EncryptionService::from_env()?;

// Create from plaintext
let encrypted = EncryptedString::from_plain("secret", &enc)?;

// Store to database
let db_value = encrypted.encrypted();

// Later, read from database and decrypt
let from_db = EncryptedString::from_encrypted(db_value.to_string());
let plaintext = from_db.to_plain(&enc)?;
```

### Using CredentialService

```rust
use foxnio::service::CredentialService;

let cred_service = CredentialService::new(db);

// Set account credential (auto-encrypted)
cred_service.set_account_credential(account_id, &api_key).await?;

// Get account credential (auto-decrypted)
let api_key = cred_service.get_account_credential(account_id).await?;

// Create OAuth token (auto-encrypted)
let oauth_token = cred_service.create_oauth_token(CreateOAuthToken {
    account_id,
    provider: "anthropic".to_string(),
    access_token: oauth_access_token,
    refresh_token: Some(oauth_refresh_token),
    expires_in: Some(3600),
    ..Default::default()
}).await?;
```

## Key Rotation

### Step 1: Generate New Key

```bash
# Generate new key
NEW_KEY=$(openssl rand -base64 32)
echo "New key: $NEW_KEY"
```

### Step 2: Configure Rotation Mode

```bash
# Set both keys (new:old format)
export FOXNIO_MASTER_KEY="${NEW_KEY}:${OLD_KEY}"
```

### Step 3: Restart Application

The application will:
1. Use new key for encryption
2. Accept both keys for decryption (try new first, then old)

### Step 4: Re-encrypt Data (Optional)

To re-encrypt all data with new key:

```sql
-- This will trigger re-encryption on next read/write
-- Or run a migration script
```

### Step 5: Remove Old Key

Once all data is re-encrypted, remove the old key:

```bash
export FOXNIO_MASTER_KEY="${NEW_KEY}"
```

## Security Considerations

### Key Storage

- **NEVER** commit `FOXNIO_MASTER_KEY` to source control
- Use environment variables or secure secret management (HashiCorp Vault, AWS Secrets Manager, etc.)
- Rotate keys periodically (recommended: every 90 days)

### Key Requirements

- Must be exactly 32 bytes (256 bits)
- Must be cryptographically random
- Must be Base64-encoded in environment variable

### Data at Rest

All sensitive fields are encrypted before storage:
- API Keys: Encrypted
- OAuth tokens: Encrypted  
- TOTP secrets: Encrypted
- Passwords: Already hashed with Argon2 (not encrypted)

### Data in Transit

Ensure TLS is enabled for all connections:
- Database connections (SSL/TLS)
- Redis connections (TLS)
- API endpoints (HTTPS)

## Performance

### Benchmarks

| Operation | Data Size | Time |
|-----------|-----------|------|
| Encrypt | 1 KB | < 1 ms |
| Decrypt | 1 KB | < 1 ms |
| Encrypt | 100 KB | < 10 ms |
| Decrypt | 100 KB | < 10 ms |
| Encrypt | 1 MB | < 50 ms |
| Decrypt | 1 MB | < 50 ms |

### Recommendations

1. **Cache decrypted values** when possible (short-lived cache with secure memory)
2. **Batch operations** when updating multiple encrypted fields
3. **Use connection pooling** for database operations

## Troubleshooting

### "Encryption service not initialized"

**Cause**: `FOXNIO_MASTER_KEY` not set  
**Solution**: Set the environment variable and restart

### "Failed to decrypt with both new and old keys"

**Cause**: Data was encrypted with a key not in the current configuration  
**Solution**: Ensure old key is included during rotation

### "Invalid master key length"

**Cause**: Key is not 32 bytes  
**Solution**: Generate a proper key using the provided methods

### "Invalid ciphertext format"

**Cause**: Data corruption or not properly encrypted  
**Solution**: Check data integrity, may need to restore from backup

## Testing

Run encryption tests:

```bash
cargo test encryption -- --nocapture
```

Run performance benchmarks:

```bash
cargo test performance_tests -- --nocapture --test-threads=1
```

## API Reference

### EncryptionService

```rust
impl EncryptionService {
    /// Create from environment variable FOXNIO_MASTER_KEY
    pub fn from_env() -> Result<Self>
    
    /// Create with specific key
    pub fn new(master_key: &[u8]) -> Result<Self>
    
    /// Create with key rotation support
    pub fn with_rotation(master_key: &[u8], old_master_key: &[u8]) -> Result<Self>
    
    /// Generate a new random master key
    pub fn generate_master_key() -> String
    
    /// Encrypt string
    pub fn encrypt(&self, plaintext: &str) -> Result<String>
    
    /// Encrypt bytes
    pub fn encrypt_bytes(&self, plaintext: &[u8]) -> Result<String>
    
    /// Decrypt string
    pub fn decrypt(&self, ciphertext: &str) -> Result<String>
    
    /// Decrypt bytes
    pub fn decrypt_bytes(&self, ciphertext: &str) -> Result<Vec<u8>>
    
    /// Hash sensitive data (one-way)
    pub fn hash_sensitive(&self, data: &str) -> Result<String>
    
    /// Verify hash
    pub fn verify_hash(&self, data: &str, hash: &str) -> bool
    
    /// Check if old key is configured
    pub fn has_old_key(&self) -> bool
}
```

### EncryptedString

```rust
impl EncryptedString {
    /// Create from plaintext (encrypts)
    pub fn from_plain(plain: &str, enc: &EncryptionService) -> Result<Self>
    
    /// Create from encrypted data (no encryption)
    pub fn from_encrypted(encrypted: String) -> Self
    
    /// Decrypt to plaintext
    pub fn to_plain(&self, enc: &EncryptionService) -> Result<String>
    
    /// Get encrypted value
    pub fn encrypted(&self) -> &str
}
```

## Migration Guide

### From v0.1.0 to v0.2.0

1. **Generate encryption key**:
   ```bash
   export FOXNIO_MASTER_KEY=$(openssl rand -base64 32)
   ```

2. **Run migration**:
   ```bash
   # Migration will add oauth_tokens table
   cargo run -- migrate
   ```

3. **Encrypt existing data** (if any):
   ```sql
   -- Backup first!
   -- Then run encryption migration script
   ```

4. **Restart application**:
   ```bash
   cargo run
   ```
