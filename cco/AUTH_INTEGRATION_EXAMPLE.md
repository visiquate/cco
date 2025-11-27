# CCO Auth Module - Integration Example

## Quick Start Integration

### 1. Add Commands to CLI

Edit `/Users/brent/git/cc-orchestra/cco/src/main.rs`:

```rust
// Add to Commands enum
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Login to CCO (OIDC device flow)
    Login,

    /// Logout from CCO
    Logout,

    /// Show authentication status
    Whoami,
}

// Add to match statement in main()
match command {
    // ... existing matches ...

    Commands::Login => {
        tracing_subscriber::fmt::init();
        cco::auth::login().await
    }

    Commands::Logout => {
        tracing_subscriber::fmt::init();
        cco::auth::logout().await
    }

    Commands::Whoami => {
        tracing_subscriber::fmt::init();
        match cco::auth::is_authenticated() {
            Ok(true) => {
                println!("✅ Authenticated");

                // Optionally show token info
                if let Ok(storage) = cco::auth::TokenStorage::new() {
                    if let Ok(tokens) = storage.get_tokens() {
                        println!("   Token expires: {}", tokens.expires_at);
                        println!("   Token type: {}", tokens.token_type);
                    }
                }
                Ok(())
            }
            Ok(false) => {
                println!("❌ Not authenticated");
                println!("   Run 'cco login' to authenticate");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
```

### 2. Use in API Clients

Example: Authenticated request to CCO API:

```rust
use cco::auth;

async fn fetch_user_data() -> anyhow::Result<UserData> {
    // Get access token (auto-refreshes if needed)
    let token = auth::get_access_token().await?;

    // Make authenticated request
    let client = reqwest::Client::new();
    let response = client
        .get("https://cco-api.visiquate.com/api/user/profile")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            // Token might be invalid, prompt re-login
            eprintln!("Authentication expired. Please run 'cco login' again.");
            std::process::exit(1);
        }
        anyhow::bail!("API request failed: HTTP {}", status);
    }

    let user_data = response.json().await?;
    Ok(user_data)
}
```

### 3. Middleware for Authenticated Endpoints

Create a helper for API clients:

```rust
// In src/api_client.rs or similar
use crate::auth;

pub struct AuthenticatedClient {
    client: reqwest::Client,
}

impl AuthenticatedClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Make authenticated GET request
    pub async fn get<T>(&self, url: &str) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let token = auth::get_access_token().await?;

        let response = self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make authenticated POST request
    pub async fn post<T, B>(&self, url: &str, body: &B) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let token = auth::get_access_token().await?;

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn handle_response<T>(&self, response: reqwest::Response) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            eprintln!("\n❌ Authentication failed");
            eprintln!("   Your session may have expired.");
            eprintln!("   Please run: cco login");
            std::process::exit(1);
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API error (HTTP {}): {}", status, error_text);
        }

        let data = response.json().await?;
        Ok(data)
    }
}

// Usage example
async fn example_usage() -> anyhow::Result<()> {
    let client = AuthenticatedClient::new();

    // GET request
    let user_profile: UserProfile = client
        .get("https://cco-api.visiquate.com/api/user/profile")
        .await?;

    // POST request
    let create_request = CreateResourceRequest {
        name: "My Resource".to_string(),
        // ...
    };
    let created: Resource = client
        .post("https://cco-api.visiquate.com/api/resources", &create_request)
        .await?;

    Ok(())
}
```

## Usage Examples

### Command Line

```bash
# Login (opens browser for authentication)
$ cco login
🔐 Initiating CCO login...

Please visit: https://auth.visiquate.com/device
And enter code: ABCD-EFGH

Waiting for authentication...

✅ Login successful!
   Tokens stored securely

# Check authentication status
$ cco whoami
✅ Authenticated
   Token expires: 2025-11-25 14:30:00 UTC
   Token type: Bearer

# Logout
$ cco logout
✅ Logout successful!
   Tokens cleared

# Try authenticated operation when not logged in
$ cco run
❌ Not authenticated. Please run 'cco login' first.
```

### Programmatic Usage

```rust
use cco::auth;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check if authenticated
    if !auth::is_authenticated()? {
        println!("Not authenticated. Please login:");
        auth::login().await?;
    }

    // Get access token (auto-refreshes if needed)
    let token = auth::get_access_token().await?;

    // Use token for API calls
    make_authenticated_request(&token).await?;

    Ok(())
}
```

## Environment Variable Override

For CI/CD or testing, allow token override:

```rust
pub async fn get_access_token() -> Result<String> {
    // Check for environment variable first (CI/CD)
    if let Ok(token) = std::env::var("CCO_ACCESS_TOKEN") {
        tracing::debug!("Using CCO_ACCESS_TOKEN from environment");
        return Ok(token);
    }

    // Otherwise use stored token with auto-refresh
    let storage = TokenStorage::new()?;
    let client = DeviceFlowClient::new(AUTH_API_URL);
    let tokens = storage.get_tokens()?;

    if tokens.is_expired(300) {
        let new_tokens = client.refresh_token(&tokens.refresh_token).await?;
        storage.store_tokens(&new_tokens)?;
        Ok(new_tokens.access_token)
    } else {
        Ok(tokens.access_token)
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticated_client() {
        // Set test token
        std::env::set_var("CCO_ACCESS_TOKEN", "test_token_123");

        let client = AuthenticatedClient::new();

        // Mock server testing would go here
        // Using something like mockito or wiremock
    }
}
```

### Integration Tests

```bash
# Manual integration test
$ cco login
$ cco whoami
$ cco run --some-authenticated-command
$ cco logout
```

## Security Best Practices

1. **Never log tokens**: Use `tracing::debug!` with care
2. **Clear on logout**: Ensure `logout()` removes all tokens
3. **Check expiration**: Always use auto-refresh mechanisms
4. **Handle 401s gracefully**: Prompt re-login on auth failures
5. **Use HTTPS only**: Never send tokens over HTTP
6. **Environment overrides**: Document `CCO_ACCESS_TOKEN` for testing only

## Common Patterns

### Retry on Auth Failure

```rust
async fn call_api_with_retry<T, F, Fut>(f: F) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    match f().await {
        Ok(result) => Ok(result),
        Err(e) if e.to_string().contains("401") || e.to_string().contains("Unauthorized") => {
            // Try to refresh token and retry once
            tracing::warn!("Auth failed, attempting token refresh");

            let storage = TokenStorage::new()?;
            let client = DeviceFlowClient::new(AUTH_API_URL);
            let tokens = storage.get_tokens()?;
            let new_tokens = client.refresh_token(&tokens.refresh_token).await?;
            storage.store_tokens(&new_tokens)?;

            // Retry once
            f().await
        }
        Err(e) => Err(e),
    }
}
```

### Background Token Refresh

```rust
use tokio::time::{interval, Duration};

async fn background_token_refresh() {
    let mut interval = interval(Duration::from_secs(60 * 10)); // Every 10 minutes

    loop {
        interval.tick().await;

        if let Ok(storage) = TokenStorage::new() {
            if let Ok(tokens) = storage.get_tokens() {
                if tokens.is_expired(60 * 15) { // 15 minutes before expiry
                    tracing::info!("Proactively refreshing token");
                    if let Ok(client) = DeviceFlowClient::new(AUTH_API_URL) {
                        if let Ok(new_tokens) = client.refresh_token(&tokens.refresh_token).await {
                            let _ = storage.store_tokens(&new_tokens);
                        }
                    }
                }
            }
        }
    }
}
```

## Next Steps

1. **Add commands to main.rs**: Login, Logout, Whoami
2. **Create AuthenticatedClient**: Reusable client for API calls
3. **Update existing API calls**: Use authenticated client
4. **Add documentation**: Update README with auth instructions
5. **Test integration**: Manual testing of full flow
6. **CI/CD setup**: Use `CCO_ACCESS_TOKEN` env var for automated tests

The auth module is ready for production use!
