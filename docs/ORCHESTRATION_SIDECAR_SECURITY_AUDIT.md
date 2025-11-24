# Orchestration Sidecar Security Audit Report

**Date**: November 18, 2025
**Auditor**: Security Auditor (Claude Orchestra)
**Version**: 1.0.0
**Status**: PRE-IMPLEMENTATION DESIGN AUDIT
**Scope**: Architecture review and secure implementation guidance

---

## Executive Summary

### Audit Status: DESIGN PHASE REVIEW

This security audit evaluates the **Orchestration Sidecar Architecture** specification (v1.0.0) prior to implementation. The architecture document provides a comprehensive design for enabling autonomous coordination of 119 Claude Orchestra agents through an HTTP API sidecar service.

**Key Finding**: The architecture is **NOT YET IMPLEMENTED**. This audit provides:
1. Security assessment of the proposed design
2. Identification of security risks and mitigations
3. Secure implementation guidelines
4. Security requirements checklist for implementation

### Vulnerability Summary

| Severity | Count | Status |
|----------|-------|---------|
| **CRITICAL** | 3 | Design mitigations required |
| **HIGH** | 7 | Implementation guidance provided |
| **MEDIUM** | 8 | Best practices recommended |
| **LOW** | 4 | Optional enhancements |

### Overall Assessment

**CONDITIONAL APPROVAL**: The architecture design includes appropriate security controls, but implementation must follow the security requirements outlined in this audit. All CRITICAL and HIGH severity items must be addressed before production deployment.

---

## 1. Authentication & Authorization Analysis

### 1.1 JWT Token Security

#### Design Review
The architecture specifies JWT-based authentication with:
- RSA-256 signing algorithm
- 1-hour token expiration
- Automatic refresh before expiry
- Claims: `sub`, `agent_type`, `project_id`, `permissions`, `exp`, `iat`

#### CRITICAL Finding #1: Token Secret Management
**Severity**: CRITICAL
**Risk**: Hardcoded or weak JWT signing keys could allow token forgery

**Vulnerability**:
```rust
// ❌ NEVER DO THIS
const JWT_SECRET: &str = "hardcoded-secret-key";

// ❌ NEVER DO THIS
let secret = env::var("JWT_SECRET").unwrap_or("default-key");
```

**Secure Implementation Required**:
```rust
// ✅ REQUIRED: Generate cryptographically secure keys
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use ring::signature::{RsaKeyPair, RSA_PKCS1_SHA256};
use ring::rand::SystemRandom;

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    rng: SystemRandom,
}

impl JwtManager {
    pub fn new() -> Result<Self, JwtError> {
        // Generate RSA-2048 key pair on startup
        let rng = SystemRandom::new();
        let pkcs8_bytes = RsaKeyPair::generate(&rng, 2048)?;

        // Store keys securely in memory only (never log or persist)
        let encoding_key = EncodingKey::from_rsa_pem(&pkcs8_bytes)?;
        let decoding_key = DecodingKey::from_rsa_pem(&public_key_bytes)?;

        Ok(Self {
            encoding_key,
            decoding_key,
            rng,
        })
    }

    // Token generation with secure random nonce
    pub fn generate_token(&self, claims: &Claims) -> Result<String, JwtError> {
        // Add jti (JWT ID) for replay prevention
        let mut claims = claims.clone();
        claims.jti = Some(generate_secure_id(&self.rng));

        let header = Header::new(Algorithm::RS256);
        encode(&header, &claims, &self.encoding_key)
    }

    // Token validation with comprehensive checks
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.leeway = 0; // No clock skew tolerance for security
        validation.validate_exp = true;
        validation.validate_nbf = true;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;

        // Additional validation
        self.check_token_revocation(&token_data.claims.jti)?;
        self.check_token_refresh_window(&token_data.claims.exp)?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,           // agent UUID
    agent_type: String,    // agent type (validated against allowed types)
    project_id: String,    // project isolation
    permissions: Vec<String>,  // granular permissions
    exp: usize,            // expiration timestamp
    iat: usize,            // issued at timestamp
    nbf: usize,            // not before timestamp
    jti: Option<String>,   // unique token ID for revocation
    iss: String,           // issuer = "cco-orchestration-sidecar"
    aud: String,           // audience = "cco-agents"
}
```

**OWASP Reference**: A02:2021 – Cryptographic Failures
**Mitigation Status**: REQUIRED before implementation
**Testing**: Unit tests required for token generation/validation

---

#### HIGH Finding #1: Token Refresh Mechanism
**Severity**: HIGH
**Risk**: Token refresh without proper validation could extend compromised tokens

**Secure Implementation**:
```rust
pub struct TokenRefreshPolicy {
    refresh_window_seconds: u64,  // Only allow refresh in last 5 minutes
    max_refresh_count: u8,        // Maximum 3 refreshes per original token
    require_activity: bool,       // Require recent API activity
}

impl JwtManager {
    pub fn refresh_token(&self, old_token: &str) -> Result<String, JwtError> {
        // Validate existing token
        let claims = self.validate_token(old_token)?;

        // Check if within refresh window (last 5 minutes before expiry)
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let expires_in = claims.exp - now as usize;

        if expires_in > 300 {
            return Err(JwtError::TooEarlyToRefresh);
        }

        // Check refresh count (prevent infinite refresh chains)
        let refresh_count = self.get_refresh_count(&claims.jti)?;
        if refresh_count >= 3 {
            return Err(JwtError::MaxRefreshExceeded);
        }

        // Verify recent activity
        if !self.has_recent_activity(&claims.sub)? {
            return Err(JwtError::InactiveToken);
        }

        // Generate new token with same permissions but new expiry
        let new_claims = Claims {
            exp: (now + 3600) as usize,  // New 1-hour expiry
            iat: now as usize,
            jti: Some(generate_secure_id(&self.rng)),
            ..claims
        };

        // Revoke old token
        self.revoke_token(&claims.jti)?;

        // Issue new token
        self.generate_token(&new_claims)
    }
}
```

**OWASP Reference**: A07:2021 – Identification and Authentication Failures
**Mitigation Status**: Implementation guidance provided

---

#### MEDIUM Finding #1: Permission Granularity
**Severity**: MEDIUM
**Risk**: Coarse-grained permissions could allow privilege escalation

**Recommendation**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
enum Permission {
    // Context operations
    ReadContext,
    ReadOwnContext,           // Only agent's own context
    ReadProjectContext,       // All context in same project

    // Result operations
    WriteResults,
    WriteOwnResults,          // Only agent's own results
    ReadResults,
    ReadProjectResults,

    // Event operations
    PublishEvents,
    PublishToOwnTopics,       // Only subscribed topics
    SubscribeEvents,

    // Agent management
    SpawnAgents,              // Chief Architect only
    TerminateAgents,          // Chief Architect only

    // Administration
    ClearCache,
    ViewMetrics,
    ManageProjects,
}

impl Permission {
    pub fn validate_for_agent_type(agent_type: &str) -> Vec<Permission> {
        match agent_type {
            "chief-architect" => vec![
                Permission::ReadProjectContext,
                Permission::WriteResults,
                Permission::PublishEvents,
                Permission::SpawnAgents,
                Permission::TerminateAgents,
                Permission::ViewMetrics,
            ],
            "python-specialist" | "go-specialist" | "rust-specialist" => vec![
                Permission::ReadOwnContext,
                Permission::WriteOwnResults,
                Permission::PublishToOwnTopics,
                Permission::SubscribeEvents,
            ],
            "security-auditor" | "qa-engineer" => vec![
                Permission::ReadProjectContext,  // Need to read all code
                Permission::WriteResults,
                Permission::PublishEvents,
                Permission::SubscribeEvents,
            ],
            "documentation-expert" => vec![
                Permission::ReadProjectContext,
                Permission::WriteOwnResults,
                Permission::SubscribeEvents,
            ],
            _ => vec![
                Permission::ReadOwnContext,
                Permission::WriteOwnResults,
            ],
        }
    }
}
```

---

### 1.2 Authorization Enforcement

#### HIGH Finding #2: Per-Endpoint Authorization
**Severity**: HIGH
**Risk**: Missing authorization checks could allow unauthorized operations

**Secure Implementation**:
```rust
use axum::middleware::from_fn;

async fn auth_middleware(
    State(state): State<ServerState>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract JWT from Authorization header
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate token
    let claims = state.jwt_manager
        .validate_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Store claims in request extensions for downstream handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

async fn permission_check<F>(
    Extension(claims): Extension<Claims>,
    required_permission: Permission,
) -> Result<(), StatusCode> {
    if !claims.permissions.contains(&required_permission) {
        warn!(
            agent_id = %claims.sub,
            agent_type = %claims.agent_type,
            required = ?required_permission,
            "Permission denied"
        );
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

// Example protected endpoint
async fn get_context(
    State(state): State<ServerState>,
    AxumPath((issue_id, agent_type)): AxumPath<(String, String)>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Context>, StatusCode> {
    // Check permission
    permission_check(claims.clone(), Permission::ReadContext).await?;

    // Verify project isolation
    if claims.project_id != extract_project_from_issue(&issue_id)? {
        warn!("Cross-project access attempt blocked");
        return Err(StatusCode::FORBIDDEN);
    }

    // Verify agent type matches (prevent impersonation)
    if claims.agent_type != agent_type {
        warn!("Agent type mismatch: claimed {} but requested {}",
            claims.agent_type, agent_type);
        return Err(StatusCode::FORBIDDEN);
    }

    // Proceed with context gathering
    let context = state.context_gatherer
        .gather_context(&agent_type, &issue_id, &claims.project_id)
        .await?;

    Ok(Json(context))
}
```

---

## 2. Event System Security

### 2.1 Event Validation & Signing

#### CRITICAL Finding #2: Event Spoofing Prevention
**Severity**: CRITICAL
**Risk**: Malicious agents could publish fake events to manipulate workflow

**Secure Implementation**:
```rust
use ring::hmac;

#[derive(Debug, Serialize, Deserialize)]
struct SignedEvent {
    event: Event,
    signature: String,  // HMAC-SHA256 of event data
    publisher_token: String,  // JWT of publisher (validated)
}

pub struct EventBus {
    signing_key: hmac::Key,
    jwt_manager: Arc<JwtManager>,
}

impl EventBus {
    pub fn new(jwt_manager: Arc<JwtManager>) -> Self {
        // Generate dedicated HMAC key for event signing
        let rng = SystemRandom::new();
        let mut key_bytes = [0u8; 32];
        rng.fill(&mut key_bytes).expect("Failed to generate key");

        let signing_key = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);

        Self { signing_key, jwt_manager }
    }

    pub async fn publish_event(
        &self,
        event: Event,
        publisher_jwt: &str,
    ) -> Result<EventId, EventError> {
        // Validate publisher JWT
        let claims = self.jwt_manager.validate_token(publisher_jwt)?;

        // Verify publisher has permission to publish
        if !claims.permissions.contains(&Permission::PublishEvents) &&
           !claims.permissions.contains(&Permission::PublishToOwnTopics) {
            return Err(EventError::Unauthorized);
        }

        // Verify publisher matches event.publisher field
        if claims.sub != event.publisher {
            return Err(EventError::PublisherMismatch);
        }

        // Verify topic access
        if claims.permissions.contains(&Permission::PublishToOwnTopics) &&
           !self.is_topic_allowed(&claims.agent_type, &event.topic) {
            return Err(EventError::TopicUnauthorized);
        }

        // Sign event to prevent tampering
        let event_json = serde_json::to_string(&event)?;
        let signature = hmac::sign(&self.signing_key, event_json.as_bytes());
        let signature_hex = hex::encode(signature.as_ref());

        let signed_event = SignedEvent {
            event: event.clone(),
            signature: signature_hex,
            publisher_token: publisher_jwt.to_string(),
        };

        // Store signed event
        let event_id = self.store_event(signed_event).await?;

        // Notify subscribers
        self.notify_subscribers(&event).await?;

        Ok(event_id)
    }

    pub fn verify_event(&self, signed_event: &SignedEvent) -> Result<(), EventError> {
        // Verify signature
        let event_json = serde_json::to_string(&signed_event.event)?;
        let signature_bytes = hex::decode(&signed_event.signature)?;

        hmac::verify(
            &self.signing_key,
            event_json.as_bytes(),
            &signature_bytes,
        ).map_err(|_| EventError::InvalidSignature)?;

        // Verify publisher token is still valid
        self.jwt_manager.validate_token(&signed_event.publisher_token)?;

        Ok(())
    }
}
```

**OWASP Reference**: A08:2021 – Software and Data Integrity Failures
**Mitigation Status**: REQUIRED before implementation

---

#### HIGH Finding #3: Topic Access Control
**Severity**: HIGH
**Risk**: Agents subscribing to unauthorized topics could leak sensitive information

**Secure Implementation**:
```rust
pub struct TopicAccessControl {
    topic_permissions: HashMap<String, TopicPermissions>,
}

#[derive(Debug, Clone)]
struct TopicPermissions {
    allowed_publishers: Vec<String>,    // Agent types that can publish
    allowed_subscribers: Vec<String>,   // Agent types that can subscribe
    requires_project_match: bool,       // Enforce same project
    sensitivity_level: SensitivityLevel,
}

#[derive(Debug, Clone, PartialEq, Ord, PartialOrd, Eq)]
enum SensitivityLevel {
    Public,        // All agents can access
    Internal,      // Same project only
    Restricted,    // Specific agent types only
    Confidential,  // Architect + Security only
}

impl TopicAccessControl {
    pub fn new() -> Self {
        let mut topic_permissions = HashMap::new();

        // Architecture topic - only architect publishes, all subscribe
        topic_permissions.insert("architecture".to_string(), TopicPermissions {
            allowed_publishers: vec!["chief-architect".to_string()],
            allowed_subscribers: vec!["*".to_string()],  // All agents
            requires_project_match: true,
            sensitivity_level: SensitivityLevel::Internal,
        });

        // Security topic - restricted
        topic_permissions.insert("security".to_string(), TopicPermissions {
            allowed_publishers: vec!["security-auditor".to_string(), "qa-engineer".to_string()],
            allowed_subscribers: vec!["chief-architect".to_string(), "devops-engineer".to_string()],
            requires_project_match: true,
            sensitivity_level: SensitivityLevel::Confidential,
        });

        // Implementation topic - coding agents publish, QA/Security subscribe
        topic_permissions.insert("implementation".to_string(), TopicPermissions {
            allowed_publishers: vec![
                "python-specialist".to_string(),
                "go-specialist".to_string(),
                "rust-specialist".to_string(),
            ],
            allowed_subscribers: vec![
                "qa-engineer".to_string(),
                "security-auditor".to_string(),
                "chief-architect".to_string(),
            ],
            requires_project_match: true,
            sensitivity_level: SensitivityLevel::Internal,
        });

        Self { topic_permissions }
    }

    pub fn can_publish(&self, agent_type: &str, topic: &str, project_id: &str) -> bool {
        match self.topic_permissions.get(topic) {
            Some(perms) => {
                perms.allowed_publishers.contains(&agent_type.to_string()) ||
                perms.allowed_publishers.contains(&"*".to_string())
            },
            None => false,  // Unknown topics denied by default
        }
    }

    pub fn can_subscribe(&self, agent_type: &str, topic: &str, project_id: &str) -> bool {
        match self.topic_permissions.get(topic) {
            Some(perms) => {
                perms.allowed_subscribers.contains(&agent_type.to_string()) ||
                perms.allowed_subscribers.contains(&"*".to_string())
            },
            None => false,
        }
    }
}
```

---

### 2.2 Event Replay Prevention

#### MEDIUM Finding #2: Replay Attack Protection
**Severity**: MEDIUM
**Risk**: Old events could be replayed to trigger duplicate actions

**Secure Implementation**:
```rust
pub struct EventReplayProtection {
    processed_events: Arc<DashMap<String, Instant>>,  // event_id -> processed_time
    cleanup_interval: Duration,
}

impl EventReplayProtection {
    pub fn new() -> Self {
        let protection = Self {
            processed_events: Arc::new(DashMap::new()),
            cleanup_interval: Duration::from_secs(3600),  // 1 hour
        };

        // Start cleanup task
        let processed_events = protection.processed_events.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300));  // Every 5 minutes
            loop {
                interval.tick().await;
                Self::cleanup_old_events(&processed_events);
            }
        });

        protection
    }

    pub fn check_and_mark(&self, event: &SignedEvent) -> Result<(), EventError> {
        let event_id = &event.event.id;

        // Check if already processed
        if self.processed_events.contains_key(event_id) {
            warn!(event_id = %event_id, "Replay attack detected");
            return Err(EventError::ReplayDetected);
        }

        // Check event timestamp (reject events older than 5 minutes)
        let event_age = Utc::now().signed_duration_since(event.event.timestamp);
        if event_age.num_seconds() > 300 {
            warn!(event_id = %event_id, age_seconds = event_age.num_seconds(),
                "Event too old, potential replay");
            return Err(EventError::EventTooOld);
        }

        // Mark as processed
        self.processed_events.insert(event_id.clone(), Instant::now());

        Ok(())
    }

    fn cleanup_old_events(processed_events: &DashMap<String, Instant>) {
        let cutoff = Instant::now() - Duration::from_secs(3600);
        processed_events.retain(|_, processed_at| *processed_at > cutoff);
    }
}
```

---

## 3. Data Isolation Security

### 3.1 Project-Level Isolation

#### CRITICAL Finding #3: Cross-Project Data Leakage
**Severity**: CRITICAL
**Risk**: Agents could access data from other projects without proper isolation

**Secure Implementation**:
```rust
pub struct ProjectIsolationEnforcer {
    project_policies: HashMap<String, ProjectPolicy>,
}

#[derive(Debug, Clone)]
struct ProjectPolicy {
    allowed_paths: Vec<PathBuf>,
    denied_paths: Vec<PathBuf>,
    max_context_size: usize,
    encryption_required: bool,
}

impl ProjectIsolationEnforcer {
    pub fn validate_access(
        &self,
        claims: &Claims,
        resource_project_id: &str,
        resource_type: ResourceType,
    ) -> Result<(), IsolationError> {
        // Verify project ID match
        if claims.project_id != resource_project_id {
            error!(
                agent = %claims.sub,
                agent_project = %claims.project_id,
                resource_project = %resource_project_id,
                "Cross-project access attempt blocked"
            );
            return Err(IsolationError::CrossProjectAccess);
        }

        // Verify permission for resource type
        match resource_type {
            ResourceType::Context => {
                if !claims.permissions.contains(&Permission::ReadContext) &&
                   !claims.permissions.contains(&Permission::ReadOwnContext) {
                    return Err(IsolationError::Unauthorized);
                }
            },
            ResourceType::Results => {
                if !claims.permissions.contains(&Permission::WriteResults) &&
                   !claims.permissions.contains(&Permission::WriteOwnResults) {
                    return Err(IsolationError::Unauthorized);
                }
            },
            ResourceType::Events => {
                // Event access validated separately
            },
        }

        Ok(())
    }

    pub fn get_isolated_storage_path(
        &self,
        project_id: &str,
        resource_type: ResourceType,
    ) -> Result<PathBuf, IsolationError> {
        // Validate project ID format (prevent path traversal)
        if !is_valid_project_id(project_id) {
            return Err(IsolationError::InvalidProjectId);
        }

        let base_path = PathBuf::from("/tmp/cco-sidecar");

        let path = match resource_type {
            ResourceType::Context => {
                base_path.join("context-cache").join(project_id)
            },
            ResourceType::Results => {
                base_path.join("results").join(project_id)
            },
            ResourceType::Events => {
                base_path.join("events").join(project_id)
            },
        };

        // Ensure path is within base directory (prevent path traversal)
        if !path.starts_with(&base_path) {
            error!(path = ?path, "Path traversal attempt detected");
            return Err(IsolationError::PathTraversal);
        }

        Ok(path)
    }
}

fn is_valid_project_id(project_id: &str) -> bool {
    // Only allow alphanumeric and hyphens (prevent path traversal)
    let re = Regex::new(r"^[a-zA-Z0-9\-]{1,64}$").unwrap();
    re.is_match(project_id)
}

#[derive(Debug, Clone, Copy)]
enum ResourceType {
    Context,
    Results,
    Events,
}
```

**OWASP Reference**: A01:2021 – Broken Access Control
**Mitigation Status**: REQUIRED before implementation

---

### 3.2 Context Cache Isolation

#### HIGH Finding #4: Cache Poisoning
**Severity**: HIGH
**Risk**: Malicious agent could poison cache with false context data

**Secure Implementation**:
```rust
pub struct SecureContextCache {
    cache: Arc<Mutex<LruCache<CacheKey, CachedContext>>>,
    integrity_checks: bool,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    project_id: String,
    issue_id: String,
    agent_type: String,
}

#[derive(Clone)]
struct CachedContext {
    context: Context,
    checksum: String,  // SHA-256 of context data
    created_by: String,  // Agent ID that created it
    created_at: DateTime<Utc>,
    last_accessed: DateTime<Utc>,
    access_count: u32,
}

impl SecureContextCache {
    pub async fn get(
        &self,
        key: &CacheKey,
        requester_claims: &Claims,
    ) -> Option<Context> {
        // Verify requester has access to this project
        if requester_claims.project_id != key.project_id {
            warn!("Cache access denied: project mismatch");
            return None;
        }

        let mut cache = self.cache.lock().await;

        if let Some(mut cached) = cache.get_mut(key) {
            // Verify integrity
            if self.integrity_checks && !self.verify_checksum(&cached) {
                error!("Cache integrity check failed, evicting entry");
                cache.pop(key);
                return None;
            }

            // Update access metadata
            cached.last_accessed = Utc::now();
            cached.access_count += 1;

            return Some(cached.context.clone());
        }

        None
    }

    pub async fn set(
        &self,
        key: CacheKey,
        context: Context,
        creator_claims: &Claims,
    ) -> Result<(), CacheError> {
        // Verify creator has permission to write to cache
        if !creator_claims.permissions.contains(&Permission::WriteContext) {
            return Err(CacheError::Unauthorized);
        }

        // Compute checksum for integrity
        let checksum = self.compute_checksum(&context);

        let cached = CachedContext {
            context,
            checksum,
            created_by: creator_claims.sub.clone(),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
        };

        let mut cache = self.cache.lock().await;
        cache.put(key, cached);

        Ok(())
    }

    fn compute_checksum(&self, context: &Context) -> String {
        use sha2::{Sha256, Digest};
        let json = serde_json::to_string(context).unwrap();
        let hash = Sha256::digest(json.as_bytes());
        hex::encode(hash)
    }

    fn verify_checksum(&self, cached: &CachedContext) -> bool {
        let computed = self.compute_checksum(&cached.context);
        computed == cached.checksum
    }
}
```

---

## 4. API Security

### 4.1 Input Validation

#### HIGH Finding #5: Input Validation Gaps
**Severity**: HIGH
**Risk**: Malicious input could cause crashes, resource exhaustion, or injection attacks

**Secure Implementation**:
```rust
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
struct ContextRequest {
    #[validate(length(min = 1, max = 128))]
    #[validate(regex = "ALPHANUMERIC_HYPHEN")]
    issue_id: String,

    #[validate(length(min = 1, max = 64))]
    #[validate(regex = "AGENT_TYPE_PATTERN")]
    agent_type: String,
}

#[derive(Debug, Validate, Deserialize)]
struct ResultSubmission {
    #[validate(length(min = 1, max = 128))]
    agent_id: String,

    #[validate(length(min = 1, max = 64))]
    agent_type: String,

    #[validate(length(min = 1, max = 128))]
    issue_id: String,

    #[validate(length(min = 1, max = 128))]
    project_id: String,

    #[validate]
    result: ResultData,
}

#[derive(Debug, Validate, Deserialize)]
struct ResultData {
    #[validate(length(min = 1, max = 32))]
    status: String,

    #[validate(length(max = 1000))]
    files_created: Vec<String>,

    #[validate(length(max = 1000))]
    files_modified: Vec<String>,

    #[validate(length(max = 100))]
    decisions: Vec<String>,

    #[validate(custom = "validate_json_size")]
    artifacts: serde_json::Value,
}

fn validate_json_size(value: &serde_json::Value) -> Result<(), validator::ValidationError> {
    let json_str = serde_json::to_string(value).unwrap();
    if json_str.len() > 10_485_760 {  // 10 MB limit
        return Err(validator::ValidationError::new("json_too_large"));
    }
    Ok(())
}

lazy_static! {
    static ref ALPHANUMERIC_HYPHEN: Regex = Regex::new(r"^[a-zA-Z0-9\-]+$").unwrap();
    static ref AGENT_TYPE_PATTERN: Regex = Regex::new(r"^[a-z][a-z0-9\-]*$").unwrap();
}

async fn validate_input<T: Validate>(input: &T) -> Result<(), StatusCode> {
    input.validate()
        .map_err(|e| {
            warn!(validation_errors = ?e, "Input validation failed");
            StatusCode::BAD_REQUEST
        })
}
```

**OWASP Reference**: A03:2021 – Injection
**Mitigation Status**: Implementation guidance provided

---

### 4.2 Rate Limiting

#### HIGH Finding #6: Denial of Service via Rate Exhaustion
**Severity**: HIGH
**Risk**: Malicious or buggy agents could overwhelm the sidecar

**Secure Implementation**:
```rust
use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState};
use nonzero_ext::nonzero;

pub struct RateLimitingMiddleware {
    // Global rate limit: 100 requests/second total
    global_limiter: Arc<RateLimiter<String, InMemoryState, DefaultClock>>,

    // Per-agent rate limit: 10 requests/second per agent
    agent_limiters: Arc<DashMap<String, Arc<RateLimiter<String, InMemoryState, DefaultClock>>>>,

    // Per-project rate limit: 50 requests/second per project
    project_limiters: Arc<DashMap<String, Arc<RateLimiter<String, InMemoryState, DefaultClock>>>>,
}

impl RateLimitingMiddleware {
    pub fn new() -> Self {
        Self {
            global_limiter: Arc::new(RateLimiter::direct(
                Quota::per_second(nonzero!(100_u32))
            )),
            agent_limiters: Arc::new(DashMap::new()),
            project_limiters: Arc::new(DashMap::new()),
        }
    }

    pub async fn check_rate_limit(&self, claims: &Claims) -> Result<(), StatusCode> {
        // Check global limit
        if self.global_limiter.check().is_err() {
            warn!("Global rate limit exceeded");
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        // Check per-agent limit
        let agent_limiter = self.agent_limiters
            .entry(claims.sub.clone())
            .or_insert_with(|| {
                Arc::new(RateLimiter::direct(
                    Quota::per_second(nonzero!(10_u32))
                ))
            })
            .clone();

        if agent_limiter.check().is_err() {
            warn!(agent_id = %claims.sub, "Per-agent rate limit exceeded");
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        // Check per-project limit
        let project_limiter = self.project_limiters
            .entry(claims.project_id.clone())
            .or_insert_with(|| {
                Arc::new(RateLimiter::direct(
                    Quota::per_second(nonzero!(50_u32))
                ))
            })
            .clone();

        if project_limiter.check().is_err() {
            warn!(project_id = %claims.project_id, "Per-project rate limit exceeded");
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        Ok(())
    }
}

// Middleware integration
async fn rate_limit_middleware(
    State(state): State<ServerState>,
    Extension(claims): Extension<Claims>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    state.rate_limiter.check_rate_limit(&claims).await?;
    Ok(next.run(request).await)
}
```

---

### 4.3 CORS Configuration

#### MEDIUM Finding #3: CORS Misconfiguration
**Severity**: MEDIUM
**Risk**: Overly permissive CORS could allow unauthorized browser-based attacks

**Secure Implementation**:
```rust
use tower_http::cors::{CorsLayer, AllowOrigin};

fn configure_cors() -> CorsLayer {
    CorsLayer::new()
        // Only allow localhost origins (sidecar is local-only)
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            origin.as_bytes().starts_with(b"http://localhost:") ||
            origin.as_bytes().starts_with(b"http://127.0.0.1:")
        }))
        // Only allow necessary methods
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::DELETE,
        ])
        // Only allow necessary headers
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ])
        // Don't expose all headers
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
        ])
        // No credentials needed (JWT in header)
        .allow_credentials(false)
        // Short max age (1 hour)
        .max_age(Duration::from_secs(3600))
}
```

---

## 5. File Storage Security

### 5.1 Directory Permissions

#### HIGH Finding #7: Insecure File Permissions
**Severity**: HIGH
**Risk**: Other users on system could read sensitive agent data

**Secure Implementation**:
```rust
use std::fs::{create_dir_all, set_permissions, Permissions};
use std::os::unix::fs::PermissionsExt;

pub struct SecureStorage {
    base_path: PathBuf,
}

impl SecureStorage {
    pub fn new() -> Result<Self, StorageError> {
        // Use ~/.cco/orchestration/ instead of /tmp for better security
        let home = dirs::home_dir()
            .ok_or_else(|| StorageError::HomeNotFound)?;

        let base_path = home.join(".cco").join("orchestration");

        // Create directory with restrictive permissions
        create_dir_all(&base_path)?;

        // Set directory to 0700 (owner read/write/execute only)
        #[cfg(unix)]
        {
            let perms = Permissions::from_mode(0o700);
            set_permissions(&base_path, perms)?;
        }

        Ok(Self { base_path })
    }

    pub async fn write_result(
        &self,
        project_id: &str,
        issue_id: &str,
        agent_type: &str,
        data: &ResultData,
    ) -> Result<PathBuf, StorageError> {
        // Create project/issue directory structure
        let dir_path = self.base_path
            .join("results")
            .join(project_id)
            .join(issue_id);

        create_dir_all(&dir_path)?;

        // Set directory permissions
        #[cfg(unix)]
        {
            let perms = Permissions::from_mode(0o700);
            set_permissions(&dir_path, perms)?;
        }

        // Write result file
        let file_path = dir_path.join(format!("{}.json", agent_type));
        let json = serde_json::to_string_pretty(data)?;

        // Write atomically using temp file + rename
        let temp_path = file_path.with_extension("tmp");
        tokio::fs::write(&temp_path, json).await?;

        // Set file permissions to 0600 (owner read/write only)
        #[cfg(unix)]
        {
            let perms = Permissions::from_mode(0o600);
            set_permissions(&temp_path, perms)?;
        }

        // Atomic rename
        tokio::fs::rename(&temp_path, &file_path).await?;

        Ok(file_path)
    }
}
```

**OWASP Reference**: A01:2021 – Broken Access Control
**Mitigation Status**: Implementation guidance provided

---

### 5.2 Sensitive Data Handling

#### MEDIUM Finding #4: Credentials in Logs/Storage
**Severity**: MEDIUM
**Risk**: Accidental logging or storage of secrets

**Secure Implementation**:
```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct SanitizedContext {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    files: Vec<FileInfo>,

    #[serde(skip)]  // Never serialize sensitive data
    secrets: HashMap<String, String>,

    project_structure: ProjectStructure,
    git_context: GitContext,
}

impl SanitizedContext {
    pub fn from_context(context: Context) -> Self {
        // Remove sensitive patterns before serialization
        let files = context.files.into_iter()
            .map(|mut file| {
                file.content = sanitize_content(&file.content);
                file
            })
            .collect();

        Self {
            files,
            secrets: HashMap::new(),  // Never included
            project_structure: context.project_structure,
            git_context: sanitize_git_context(context.git_context),
        }
    }
}

fn sanitize_content(content: &str) -> String {
    let patterns = [
        (r"(password|passwd|pwd)\s*=\s*['\"]([^'\"]+)['\"]", "password=\"***\""),
        (r"(api[_-]?key)\s*=\s*['\"]([^'\"]+)['\"]", "api_key=\"***\""),
        (r"(secret)\s*=\s*['\"]([^'\"]+)['\"]", "secret=\"***\""),
        (r"Bearer\s+[A-Za-z0-9\-._~+/]+=*", "Bearer ***"),
    ];

    let mut sanitized = content.to_string();
    for (pattern, replacement) in &patterns {
        let re = Regex::new(pattern).unwrap();
        sanitized = re.replace_all(&sanitized, *replacement).to_string();
    }
    sanitized
}

// Custom Debug implementation that redacts sensitive fields
impl std::fmt::Debug for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Claims")
            .field("sub", &self.sub)
            .field("agent_type", &self.agent_type)
            .field("project_id", &self.project_id)
            .field("permissions", &self.permissions)
            .field("exp", &self.exp)
            .field("iat", &self.iat)
            .field("jti", &"***")  // Redacted in logs
            .finish()
    }
}
```

---

## 6. Network Security

### 6.1 Localhost-Only Binding

#### MEDIUM Finding #5: Network Exposure
**Severity**: MEDIUM
**Risk**: Binding to 0.0.0.0 could expose sidecar to network attacks

**Secure Implementation**:
```rust
pub async fn start_sidecar(port: u16) -> Result<(), ServerError> {
    // ALWAYS bind to localhost only
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Verify we're binding to localhost
    if !addr.ip().is_loopback() {
        error!("Attempted to bind to non-localhost address: {}", addr);
        return Err(ServerError::InvalidBindAddress);
    }

    info!("Starting orchestration sidecar on {}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
```

---

### 6.2 TLS/HTTPS (Future Enhancement)

#### LOW Finding #1: No TLS for Localhost
**Severity**: LOW
**Risk**: Traffic is unencrypted, but only on localhost loopback

**Recommendation**: For paranoid security, implement TLS even for localhost:
```rust
// Optional: TLS for localhost (defense in depth)
use rustls::ServerConfig;

pub async fn start_sidecar_tls(port: u16) -> Result<(), ServerError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Generate self-signed cert for localhost
    let cert = generate_localhost_cert()?;
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert.cert_chain, cert.private_key)?;

    let listener = TcpListener::bind(addr).await?;

    // Serve with TLS
    axum_server::from_tcp_rustls(listener.into_std()?, config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

---

### 6.3 Security Headers

#### MEDIUM Finding #6: Missing Security Headers
**Severity**: MEDIUM
**Risk**: Browser-based attacks if dashboard accesses sidecar

**Secure Implementation**:
```rust
use axum::middleware::from_fn;

async fn security_headers_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // XSS protection
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // CSP for API responses
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );

    // HSTS (if using TLS)
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Prevent caching of sensitive data
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, private"),
    );

    response
}
```

---

## 7. Threat Model & Attack Scenarios

### Threat Matrix

| Threat | Likelihood | Impact | Risk | Mitigation |
|--------|-----------|--------|------|------------|
| **Compromised Agent** | Medium | High | HIGH | JWT validation, permission checks, event signing |
| **Token Theft** | Low | High | MEDIUM | Short expiry, refresh limits, activity tracking |
| **Replay Attack** | Medium | Medium | MEDIUM | Nonce tracking, timestamp validation, event IDs |
| **Cross-Project Access** | Medium | Critical | CRITICAL | Project ID enforcement, path validation |
| **DoS (Rate Exhaustion)** | High | Medium | HIGH | Multi-level rate limiting, circuit breakers |
| **Cache Poisoning** | Low | High | MEDIUM | Checksum validation, creator tracking |
| **Event Spoofing** | Medium | High | HIGH | HMAC signatures, publisher verification |
| **File System Attack** | Low | High | MEDIUM | Strict permissions (0700/0600), path validation |
| **SQL Injection** | N/A | N/A | N/A | No SQL database in architecture |
| **Information Disclosure** | Medium | Medium | MEDIUM | Sanitization, redaction, error handling |

---

### Attack Scenario 1: Compromised Agent Escalation

**Attack**: Malicious agent attempts to escalate privileges by forging JWT token

**Attack Steps**:
1. Agent obtains valid JWT token
2. Attempts to modify token claims (change agent_type to "chief-architect")
3. Uses modified token to spawn unauthorized agents

**Mitigations**:
- ✅ RSA-256 signature prevents token modification
- ✅ Token validation checks signature before trusting claims
- ✅ Agent type verification on all operations
- ✅ Spawn permission limited to Chief Architect

**Residual Risk**: LOW (requires breaking RSA-256 crypto)

---

### Attack Scenario 2: Cross-Project Data Exfiltration

**Attack**: Agent attempts to read context from another project

**Attack Steps**:
1. Agent receives valid token for project-abc
2. Requests context for issue in project-xyz
3. Attempts to extract sensitive code/credentials

**Mitigations**:
- ✅ Project ID in JWT claims
- ✅ Project ID validation on every request
- ✅ Path isolation in storage layer
- ✅ Regex validation prevents path traversal

**Residual Risk**: LOW (multiple layers of defense)

---

### Attack Scenario 3: Event Bus Manipulation

**Attack**: Agent publishes fake "security audit passed" event

**Attack Steps**:
1. Coding agent obtains valid token
2. Publishes event to "security" topic claiming all clear
3. Bypasses actual security review

**Mitigations**:
- ✅ Topic access control (only security-auditor can publish to "security")
- ✅ HMAC signature verification
- ✅ Publisher JWT validation
- ✅ Event replay protection

**Residual Risk**: LOW (topic ACLs prevent unauthorized publishing)

---

## 8. Dependency Security

### Cargo.toml Dependency Audit

#### Current Dependencies (from CCO)
```toml
axum = "0.7"              # Web framework
tokio = "1.35"            # Async runtime
tower-http = "0.5"        # Middleware
serde = "1.0"             # Serialization
reqwest = "0.11"          # HTTP client
sqlx = "0.7"              # Database (NOT used by sidecar)
chrono = "0.4"            # Date/time
uuid = "1.6"              # UUIDs
dashmap = "5.5"           # Concurrent hashmap
```

#### Additional Dependencies Needed for Sidecar
```toml
jsonwebtoken = "9.2"      # JWT handling
ring = "0.17"             # Cryptography
governor = "0.6"          # Rate limiting
validator = "0.16"        # Input validation
rustls = "0.21"           # TLS (optional)
```

#### Known Vulnerabilities (as of Nov 2025)

**MEDIUM Finding #7**: Check all dependencies for CVEs before implementation
```bash
cargo audit
```

**Recommendation**: Run `cargo audit` and `cargo outdated` before implementation.

---

### Dependency Hardening

```toml
[dependencies]
# Use exact versions for security-critical dependencies
jsonwebtoken = "=9.2.0"
ring = "=0.17.7"

# Enable security features
rustls = { version = "0.21", features = ["dangerous_configuration"] }  # For custom cert validation

[profile.release]
# Harden release builds
opt-level = 3
lto = true
codegen-units = 1
strip = true  # Remove debug symbols
panic = "abort"  # Smaller binary, faster panics
```

---

## 9. Logging & Monitoring Security

### 9.1 Audit Logging

#### MEDIUM Finding #8: Insufficient Audit Logging
**Severity**: MEDIUM
**Risk**: Cannot detect or investigate security incidents

**Secure Implementation**:
```rust
pub struct AuditLogger {
    log_file: Arc<Mutex<File>>,
}

#[derive(Serialize)]
struct AuditEvent {
    timestamp: DateTime<Utc>,
    event_type: AuditEventType,
    agent_id: String,
    agent_type: String,
    project_id: String,
    action: String,
    resource: String,
    result: AuditResult,
    ip_address: String,
    metadata: HashMap<String, String>,
}

#[derive(Serialize)]
enum AuditEventType {
    Authentication,
    Authorization,
    ContextAccess,
    ResultStorage,
    EventPublish,
    AgentSpawn,
    CacheAccess,
    RateLimitExceeded,
    SecurityViolation,
}

#[derive(Serialize)]
enum AuditResult {
    Success,
    Failure(String),  // Reason for failure
}

impl AuditLogger {
    pub async fn log_event(&self, event: AuditEvent) {
        // Write to structured log file
        let json = serde_json::to_string(&event).unwrap();

        let mut file = self.log_file.lock().await;
        writeln!(file, "{}", json).ok();

        // Also log to tracing for monitoring
        match event.result {
            AuditResult::Success => {
                info!(
                    event_type = ?event.event_type,
                    agent_id = %event.agent_id,
                    action = %event.action,
                    "Audit event"
                );
            },
            AuditResult::Failure(ref reason) => {
                warn!(
                    event_type = ?event.event_type,
                    agent_id = %event.agent_id,
                    action = %event.action,
                    reason = %reason,
                    "Security violation"
                );
            },
        }
    }
}

// Example usage in endpoint
async fn get_context_with_audit(
    State(state): State<ServerState>,
    Extension(claims): Extension<Claims>,
    // ...
) -> Result<Json<Context>, StatusCode> {
    let audit_event = AuditEvent {
        timestamp: Utc::now(),
        event_type: AuditEventType::ContextAccess,
        agent_id: claims.sub.clone(),
        agent_type: claims.agent_type.clone(),
        project_id: claims.project_id.clone(),
        action: "get_context".to_string(),
        resource: format!("{}/{}", issue_id, agent_type),
        result: AuditResult::Success,
        ip_address: "127.0.0.1".to_string(),
        metadata: HashMap::new(),
    };

    state.audit_logger.log_event(audit_event).await;

    // ... proceed with normal logic
}
```

---

### 9.2 Error Handling Security

#### LOW Finding #2: Information Disclosure in Errors
**Severity**: LOW
**Risk**: Error messages could leak sensitive information

**Secure Implementation**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found")]
    NotFound,

    #[error("Invalid request")]
    InvalidRequest,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error")]
    InternalError,  // Never expose internal details
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Log detailed error internally
        error!(error = ?self, "API error occurred");

        // Return generic error to client
        let (status, message) = match self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            ApiError::InvalidRequest => (StatusCode::BAD_REQUEST, "Invalid request"),
            ApiError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded"),
            ApiError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(serde_json::json!({
            "error": message,
            "timestamp": Utc::now().to_rfc3339(),
        }));

        (status, body).into_response()
    }
}
```

---

## 10. Compliance & Standards

### OWASP Top 10 (2021) Compliance

| OWASP Category | Status | Notes |
|---------------|--------|-------|
| A01: Broken Access Control | ✅ ADDRESSED | Project isolation, JWT auth, permission checks |
| A02: Cryptographic Failures | ✅ ADDRESSED | RSA-256 JWT, HMAC event signing, secure key gen |
| A03: Injection | ✅ ADDRESSED | Input validation, regex patterns, no SQL |
| A04: Insecure Design | ✅ ADDRESSED | Security by design, threat modeling |
| A05: Security Misconfiguration | ⚠️ REQUIRES ATTENTION | Hardening checklist provided |
| A06: Vulnerable Components | ⚠️ REQUIRES AUDIT | Cargo audit before deployment |
| A07: ID & Auth Failures | ✅ ADDRESSED | JWT with refresh limits, MFA not applicable |
| A08: Software & Data Integrity | ✅ ADDRESSED | Event signing, checksum validation |
| A09: Logging & Monitoring | ✅ ADDRESSED | Comprehensive audit logging |
| A10: SSRF | ✅ ADDRESSED | Localhost-only binding, no external requests |

---

### Security Checklist for Implementation

#### Phase 1: Core Security (MUST HAVE)
- [ ] RSA-256 JWT token generation with secure random keys
- [ ] Token validation on all endpoints
- [ ] Project isolation enforcement in all operations
- [ ] Input validation using `validator` crate
- [ ] Path traversal prevention in file operations
- [ ] File permissions (0700 directories, 0600 files)
- [ ] Localhost-only binding (127.0.0.1)
- [ ] Rate limiting (global, per-agent, per-project)
- [ ] HMAC event signing
- [ ] Topic access control
- [ ] Audit logging for security events
- [ ] Error handling without info disclosure

#### Phase 2: Advanced Security (SHOULD HAVE)
- [ ] Token refresh with activity tracking
- [ ] Event replay protection with nonce tracking
- [ ] Cache integrity checking with checksums
- [ ] Content sanitization for secrets
- [ ] Security headers on all responses
- [ ] CORS hardening for localhost only
- [ ] Dependency vulnerability scanning (cargo audit)
- [ ] Automated security testing in CI/CD

#### Phase 3: Defense in Depth (NICE TO HAVE)
- [ ] TLS for localhost (optional paranoia)
- [ ] Certificate pinning for agents
- [ ] Advanced rate limiting with circuit breakers
- [ ] Automated log analysis for anomalies
- [ ] Integration with SIEM systems
- [ ] Penetration testing before production

---

## 11. Security Test Plan

### Unit Tests Required

```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[tokio::test]
    async fn test_jwt_token_validation() {
        let jwt_manager = JwtManager::new().unwrap();

        // Valid token
        let claims = Claims { /* ... */ };
        let token = jwt_manager.generate_token(&claims).unwrap();
        assert!(jwt_manager.validate_token(&token).is_ok());

        // Expired token
        let mut expired_claims = claims.clone();
        expired_claims.exp = 0;
        let expired_token = jwt_manager.generate_token(&expired_claims).unwrap();
        assert!(jwt_manager.validate_token(&expired_token).is_err());

        // Modified token (should fail signature verification)
        let mut parts: Vec<&str> = token.split('.').collect();
        parts[1] = "modified_payload";
        let tampered_token = parts.join(".");
        assert!(jwt_manager.validate_token(&tampered_token).is_err());
    }

    #[tokio::test]
    async fn test_project_isolation() {
        let enforcer = ProjectIsolationEnforcer::new();

        let claims = Claims {
            project_id: "project-abc".to_string(),
            // ...
        };

        // Same project - allowed
        assert!(enforcer.validate_access(&claims, "project-abc", ResourceType::Context).is_ok());

        // Different project - denied
        assert!(enforcer.validate_access(&claims, "project-xyz", ResourceType::Context).is_err());
    }

    #[tokio::test]
    async fn test_path_traversal_prevention() {
        let enforcer = ProjectIsolationEnforcer::new();

        // Valid project ID
        assert!(is_valid_project_id("project-abc-123"));

        // Path traversal attempts
        assert!(!is_valid_project_id("../etc/passwd"));
        assert!(!is_valid_project_id("project/../other"));
        assert!(!is_valid_project_id("project\0null"));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimitingMiddleware::new();
        let claims = Claims { /* ... */ };

        // First 10 requests should succeed
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(&claims).await.is_ok());
        }

        // 11th request should be rate limited
        assert_eq!(
            limiter.check_rate_limit(&claims).await.unwrap_err(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[tokio::test]
    async fn test_event_signing() {
        let event_bus = EventBus::new(/* ... */);

        let event = Event { /* ... */ };
        let token = "valid_jwt_token";

        // Publish event
        let event_id = event_bus.publish_event(event.clone(), token).await.unwrap();

        // Retrieve and verify
        let signed_event = event_bus.get_event(&event_id).await.unwrap();
        assert!(event_bus.verify_event(&signed_event).is_ok());

        // Tamper with event
        let mut tampered = signed_event.clone();
        tampered.event.data = json!({"tampered": true});
        assert!(event_bus.verify_event(&tampered).is_err());
    }
}
```

---

### Integration Tests Required

```rust
#[tokio::test]
async fn test_cross_project_access_blocked() {
    let app = create_test_app().await;

    // Agent A in project-abc
    let token_a = generate_test_token("project-abc", "python-specialist");

    // Attempt to access project-xyz context
    let response = app
        .get("/api/context/xyz-issue-1/python-specialist")
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_unauthorized_event_publishing() {
    let app = create_test_app().await;

    // Documentation agent tries to publish to security topic
    let token = generate_test_token("project-abc", "documentation-expert");

    let event = json!({
        "event_type": "security_audit_passed",
        "topic": "security",
        "data": {}
    });

    let response = app
        .post("/api/events/security_audit_passed")
        .header("Authorization", format!("Bearer {}", token))
        .json(&event)
        .send()
        .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
```

---

### Penetration Testing Scenarios

1. **Token Forgery**: Attempt to forge JWT tokens with different algorithms (HMAC vs RSA)
2. **Replay Attacks**: Capture and replay old events to trigger duplicate actions
3. **Rate Limit Bypass**: Attempt to bypass rate limits with multiple tokens
4. **Path Traversal**: Try various path traversal payloads in project/issue IDs
5. **Event Spoofing**: Attempt to publish events without proper signatures
6. **Cross-Project Leakage**: Try to access context/results from other projects
7. **Token Theft**: Test token refresh without valid activity
8. **Cache Poisoning**: Attempt to inject malicious data into context cache

---

## 12. Remediation Priorities

### CRITICAL (Fix Before Any Implementation)

1. **JWT Secret Management** (Finding #1)
   - Generate RSA-2048 keys on startup
   - Never hardcode or use weak secrets
   - Implement in `JwtManager` struct

2. **Event Spoofing Prevention** (Finding #2)
   - Implement HMAC signing for all events
   - Validate publisher JWT matches event.publisher
   - Implement in `EventBus` struct

3. **Cross-Project Isolation** (Finding #3)
   - Validate project ID on every request
   - Prevent path traversal in storage paths
   - Implement in `ProjectIsolationEnforcer` struct

---

### HIGH (Fix During Implementation)

4. **Token Refresh Security** (Finding #1)
   - Limit refresh count and window
   - Require recent activity
   - Revoke old tokens on refresh

5. **Per-Endpoint Authorization** (Finding #2)
   - Permission checks on all endpoints
   - Agent type verification
   - Implement in auth middleware

6. **Topic Access Control** (Finding #3)
   - Define topic permissions matrix
   - Enforce publish/subscribe rules
   - Implement in `TopicAccessControl` struct

7. **Input Validation** (Finding #5)
   - Use `validator` crate for all inputs
   - Regex validation for IDs
   - Size limits on payloads

8. **DoS via Rate Limiting** (Finding #6)
   - Multi-level rate limiting (global, agent, project)
   - Circuit breaker patterns
   - Implement in `RateLimitingMiddleware`

9. **File Permissions** (Finding #7)
   - Directory: 0700
   - Files: 0600
   - Use ~/.cco/orchestration/ not /tmp

---

### MEDIUM (Best Practices)

10-18. See detailed findings above for all MEDIUM severity items

---

### LOW (Optional Enhancements)

19-22. See detailed findings above for all LOW severity items

---

## 13. Final Recommendations

### Pre-Implementation Checklist

- [ ] Review this security audit with entire team
- [ ] Assign security champions for implementation
- [ ] Set up dependency scanning in CI/CD (cargo audit)
- [ ] Create security unit tests BEFORE implementing features
- [ ] Conduct threat modeling workshop
- [ ] Define incident response plan
- [ ] Set up security monitoring and alerting

### Implementation Approach

**Phase 1 - Security Foundation** (Week 1):
- Implement JWT manager with RSA-256
- Set up project isolation enforcer
- Configure localhost-only binding
- Implement basic auth middleware

**Phase 2 - Access Control** (Week 2):
- Implement permission system
- Add topic access control
- Set up rate limiting
- Add input validation

**Phase 3 - Event Security** (Week 3):
- Implement HMAC event signing
- Add replay protection
- Set up cache integrity checks
- Implement audit logging

**Phase 4 - Testing & Hardening** (Week 4):
- Security unit tests
- Integration tests
- Penetration testing
- Security code review
- Dependency audit

### Post-Implementation

- [ ] Security code review by independent auditor
- [ ] Penetration testing by security team
- [ ] Load testing to verify rate limits
- [ ] Chaos engineering to test resilience
- [ ] Document security runbook
- [ ] Train operators on security monitoring

---

## 14. Approval Decision

### Status: CONDITIONAL APPROVAL

**The orchestration sidecar architecture is APPROVED for implementation with the following conditions:**

1. ✅ All CRITICAL findings MUST be addressed in implementation
2. ✅ All HIGH findings MUST be addressed before production deployment
3. ✅ MEDIUM findings SHOULD be addressed (best effort)
4. ✅ Security tests MUST be written alongside implementation (TDD approach)
5. ✅ Dependency audit MUST be performed before each release
6. ✅ Security code review MUST occur before production deployment

**This approval is contingent on:**
- Following the secure implementation patterns provided in this audit
- Implementing the security test plan in full
- Addressing all CRITICAL and HIGH severity findings
- Conducting penetration testing before production use

**Next Steps:**
1. Development team reviews this audit
2. Security requirements integrated into implementation plan
3. TDD approach: Write security tests FIRST
4. Implement features following secure patterns
5. Conduct security code review
6. Perform penetration testing
7. Final security sign-off

---

## Appendix A: Code Examples Repository

All secure implementation examples are included inline in this document. For additional reference:

- **JWT Management**: Section 1.1
- **Event Signing**: Section 2.1
- **Project Isolation**: Section 3.1
- **Input Validation**: Section 4.1
- **Rate Limiting**: Section 4.2
- **File Security**: Section 5.1
- **Audit Logging**: Section 9.1

---

## Appendix B: Security Testing Scripts

### Token Validation Test
```bash
#!/bin/bash
# Test JWT token validation

# Attempt to use expired token
curl -H "Authorization: Bearer <expired-token>" \
  http://localhost:3001/api/context/issue-1/python-specialist

# Expected: 401 Unauthorized
```

### Path Traversal Test
```bash
#!/bin/bash
# Test path traversal prevention

# Attempt path traversal in project ID
curl -H "Authorization: Bearer <valid-token>" \
  http://localhost:3001/api/context/../etc/passwd/issue-1

# Expected: 400 Bad Request
```

### Rate Limit Test
```bash
#!/bin/bash
# Test rate limiting

for i in {1..15}; do
  curl -H "Authorization: Bearer <valid-token>" \
    http://localhost:3001/api/status
done

# Expected: First 10 succeed, then 429 Too Many Requests
```

---

## Document Metadata

**Audit Completed**: November 18, 2025
**Next Review**: Upon implementation completion
**Document Version**: 1.0.0
**Classification**: Internal - Security Sensitive
**Distribution**: Development Team, Security Team, Chief Architect

---

**END OF SECURITY AUDIT REPORT**
