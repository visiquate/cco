# TDD Coordination Protocol for Claude Orchestra

**Version**: 1.0
**Date**: 2025-11-04
**Purpose**: Define exact coordination patterns for TDD workflow

---

## Protocol Overview

This document defines the precise coordination protocol for the TDD-aware Claude Orchestra pipeline, ensuring proper test-first development with efficient parallel execution.

---

## Phase 0: Architecture (Independent)

### Chief Architect Protocol

```bash
# Store architectural decisions
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Architecture: microservices with REST API, JWT auth, PostgreSQL database. Components: Python/FastAPI (api), Swift/SwiftUI (mobile), Go/Gin (backend). Test requirements: 90% coverage, unit/integration/e2e tests with pytest." \
  --type decision --agent architect

# Alternative: Store detailed JSON in a separate decision
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Test Requirements: 90% coverage, types: unit/integration/e2e, framework: pytest" \
  --type decision --agent architect
```

---

## Phase 1: TDD & Implementation (qwen-fast)

### TDD Coding Agent Protocol

**Critical**: Must complete test writing before coding agents can implement

```bash
# Retrieve architecture decisions
node ~/git/cc-orchestra/src/knowledge-manager.js search "architecture decisions"

# Write failing tests for each component
# Python API Tests
cat > tests/test_api.py << 'EOF'
import pytest
from fastapi.testclient import TestClient
from app.main import app

client = TestClient(app)

class TestAPI:
    def test_health_check(self):
        """Test health endpoint returns correct status."""
        response = client.get("/health")
        assert response.status_code == 200
        assert response.json()["status"] == "healthy"

    def test_create_user(self):
        """Test user creation with valid data."""
        user_data = {
            "email": "test@example.com",
            "password": "SecurePass123!",
            "name": "Test User"
        }
        response = client.post("/api/users", json=user_data)
        assert response.status_code == 201
        assert response.json()["email"] == user_data["email"]

    def test_authentication(self):
        """Test JWT authentication flow."""
        login_data = {
            "email": "test@example.com",
            "password": "SecurePass123!"
        }
        response = client.post("/api/auth/login", json=login_data)
        assert response.status_code == 200
        assert "access_token" in response.json()
        assert response.json()["token_type"] == "bearer"

    def test_invalid_authentication(self):
        """Test authentication with invalid credentials."""
        login_data = {
            "email": "test@example.com",
            "password": "WrongPassword"
        }
        response = client.post("/api/auth/login", json=login_data)
        assert response.status_code == 401

    @pytest.mark.parametrize("invalid_email", [
        "not-an-email",
        "@example.com",
        "user@",
        "",
        None
    ])
    def test_invalid_email_validation(self, invalid_email):
        """Test email validation."""
        user_data = {
            "email": invalid_email,
            "password": "ValidPass123!",
            "name": "Test User"
        }
        response = client.post("/api/users", json=user_data)
        assert response.status_code == 422
EOF

# Run tests to verify they fail
pytest tests/test_api.py -v || true  # Expected to fail

# Store test artifacts
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Tests: tests/test_api.py - 6 failing tests created (RED phase), coverage target: 90%, types: unit/integration, fixtures needed: test_client/test_db/auth_token, run: pytest tests/test_api.py -v" \
  --type implementation --agent tdd-agent

# Swift iOS Tests
cat > Tests/AppTests.swift << 'EOF'
import XCTest
@testable import MyApp

final class AppTests: XCTestCase {
    func testLoginViewModel() {
        let viewModel = LoginViewModel()

        // Test valid login
        viewModel.email = "test@example.com"
        viewModel.password = "SecurePass123!"

        XCTAssertTrue(viewModel.isValid)

        // Test invalid email
        viewModel.email = "invalid"
        XCTAssertFalse(viewModel.isValid)
    }

    func testAPIClient() async throws {
        let client = APIClient()

        // Test health check
        let health = try await client.healthCheck()
        XCTAssertEqual(health.status, "healthy")

        // Test user creation
        let user = try await client.createUser(
            email: "test@example.com",
            password: "SecurePass123!"
        )
        XCTAssertNotNil(user.id)
    }

    func testAuthenticationFlow() async throws {
        let auth = AuthenticationManager()

        // Test login
        let token = try await auth.login(
            email: "test@example.com",
            password: "SecurePass123!"
        )
        XCTAssertNotNil(token)

        // Test token storage
        XCTAssertTrue(auth.isAuthenticated)
    }
}
EOF

# Store Swift tests
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Tests: Tests/AppTests.swift - 3 failing tests created (RED phase), coverage target: 85%, types: unit/integration, run: swift test" \
  --type implementation --agent tdd-agent

# Go Backend Tests
cat > backend/api_test.go << 'EOF'
package main

import (
    "testing"
    "net/http"
    "net/http/httptest"
    "github.com/stretchr/testify/assert"
)

func TestHealthEndpoint(t *testing.T) {
    router := setupRouter()
    w := httptest.NewRecorder()
    req, _ := http.NewRequest("GET", "/health", nil)
    router.ServeHTTP(w, req)

    assert.Equal(t, 200, w.Code)
    assert.Contains(t, w.Body.String(), "healthy")
}

func TestUserCreation(t *testing.T) {
    router := setupRouter()
    w := httptest.NewRecorder()

    body := `{"email":"test@example.com","password":"SecurePass123!"}`
    req, _ := http.NewRequest("POST", "/api/users", strings.NewReader(body))
    req.Header.Set("Content-Type", "application/json")
    router.ServeHTTP(w, req)

    assert.Equal(t, 201, w.Code)
    assert.Contains(t, w.Body.String(), "test@example.com")
}

func TestAuthentication(t *testing.T) {
    router := setupRouter()

    // Test valid credentials
    w := httptest.NewRecorder()
    body := `{"email":"test@example.com","password":"SecurePass123!"}`
    req, _ := http.NewRequest("POST", "/api/auth/login", strings.NewReader(body))
    req.Header.Set("Content-Type", "application/json")
    router.ServeHTTP(w, req)

    assert.Equal(t, 200, w.Code)
    assert.Contains(t, w.Body.String(), "token")
}
EOF

# Store Go tests
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Tests: backend/api_test.go - 3 failing tests created (RED phase), coverage target: 90%, types: unit/integration, run: go test ./..." \
  --type implementation --agent tdd-agent

# Create test fixtures and mocks
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Test Fixtures: Database mock (setup: create_test_db, teardown: drop_test_db), Auth (test user: test@example.com, test token), API client (base_url: http://localhost:8000, timeout: 5)" \
  --type implementation --agent tdd-agent

# Notify tests ready
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Status: All failing tests ready for implementation (Python/Swift/Go)" \
  --type status --agent tdd-agent
```

### Python Expert Protocol

```bash
# Retrieve tests and architecture
node ~/git/cc-orchestra/src/knowledge-manager.js search "failing tests python RED"
node ~/git/cc-orchestra/src/knowledge-manager.js search "architecture decisions"

# Implement minimal code to pass tests
cat > app/main.py << 'EOF'
from fastapi import FastAPI, HTTPException, status
from fastapi.security import OAuth2PasswordBearer
from pydantic import BaseModel, EmailStr
import jwt
from datetime import datetime, timedelta

app = FastAPI()
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")

class User(BaseModel):
    email: EmailStr
    password: str
    name: str

class Token(BaseModel):
    access_token: str
    token_type: str = "bearer"

@app.get("/health")
def health_check():
    return {"status": "healthy", "timestamp": datetime.now().isoformat()}

@app.post("/api/users", status_code=status.HTTP_201_CREATED)
def create_user(user: User):
    # Minimal implementation to pass test
    return {
        "id": "user-123",
        "email": user.email,
        "name": user.name
    }

@app.post("/api/auth/login")
def login(email: EmailStr, password: str):
    # Check credentials (minimal implementation)
    if email == "test@example.com" and password == "SecurePass123!":
        token = jwt.encode(
            {"sub": email, "exp": datetime.now() + timedelta(hours=1)},
            "secret-key",
            algorithm="HS256"
        )
        return Token(access_token=token)
    raise HTTPException(status_code=401, detail="Invalid credentials")
EOF

# Run tests to verify they pass
pytest tests/test_api.py -v

# Store implementation status
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Implementation: Python API (GREEN phase) - 6 tests passing, coverage: 92%, files: app/main.py/models.py/auth.py, refactoring needed: extract auth logic, add database layer" \
  --type implementation --agent python-specialist

node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Edit: app/main.py - Implemented FastAPI endpoints to pass all tests" \
  --type edit --agent python-specialist
```

### Swift Expert Protocol

```bash
# Retrieve tests
node ~/git/cc-orchestra/src/knowledge-manager.js search "failing tests swift RED"

# Implement code to pass tests
cat > Sources/LoginViewModel.swift << 'EOF'
import Foundation
import Combine

class LoginViewModel: ObservableObject {
    @Published var email: String = ""
    @Published var password: String = ""

    var isValid: Bool {
        return isValidEmail(email) && password.count >= 8
    }

    private func isValidEmail(_ email: String) -> Bool {
        let emailRegex = "[A-Z0-9a-z._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,64}"
        return NSPredicate(format: "SELF MATCHES %@", emailRegex).evaluate(with: email)
    }
}
EOF

# Run Swift tests
swift test

# Store status
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Implementation: Swift iOS app (GREEN phase) - 3 tests passing, coverage: 88%, files: LoginViewModel.swift/APIClient.swift/AuthManager.swift" \
  --type implementation --agent swift-specialist

node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Edit: Sources/LoginViewModel.swift - Implemented authentication view model" \
  --type edit --agent swift-specialist
```

### Integration Specialists Protocol

```bash
# API Explorer - Research and document API patterns
node ~/git/cc-orchestra/src/knowledge-manager.js search "architecture decisions"

# Research and store patterns
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "API Patterns: REST conventions (versioning: /api/v1, pagination: limit/offset, errors: RFC7807), authentication: JWT with refresh (expiry: 1h)" \
  --type decision --agent api-explorer

# Salesforce API Expert - Implement Salesforce integration
node ~/git/cc-orchestra/src/knowledge-manager.js search "failing tests salesforce"

# Store Salesforce integration
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Integration: Salesforce implementation complete - endpoints: contacts/opportunities/accounts, auth: OAuth2, sync: webhook" \
  --type implementation --agent salesforce-specialist

# Authentik API Expert - Store Authentik integration
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Integration: Authentik authentication - OAuth2/OIDC, SAML/OAuth2/LDAP providers, SSO and user sync enabled" \
  --type implementation --agent authentik-specialist
```

---

## Phase 2: Quality & Documentation (qwen-quality)

### Model Swap Protocol

```bash
# Phase 1 completion notification
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Status: Phase 1 complete (TDD and implementation finished), initiating Phase 2" \
  --type status --agent architect

# Wait for model swap (handled by orchestrator)
sleep 40

node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Status: Model swap complete, Phase 2 (quality and documentation) starting" \
  --type status --agent architect
```

### QA Engineer Protocol

```bash
# Retrieve all test data and implementations
node ~/git/cc-orchestra/src/knowledge-manager.js search "failing tests RED phase"
node ~/git/cc-orchestra/src/knowledge-manager.js search "implementation GREEN phase"

# Analyze test coverage and add edge cases
cat >> tests/test_api_edge_cases.py << 'EOF'
import pytest
from fastapi.testclient import TestClient
import concurrent.futures

class TestEdgeCases:
    def test_concurrent_user_creation(self, client):
        """Test race condition in user creation."""
        with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
            futures = []
            for i in range(10):
                future = executor.submit(
                    client.post,
                    "/api/users",
                    json={"email": f"user{i}@test.com", "password": "Pass123!"}
                )
                futures.append(future)

            results = [f.result() for f in futures]
            assert all(r.status_code == 201 for r in results)

    def test_sql_injection_prevention(self, client):
        """Test SQL injection prevention."""
        malicious_input = "'; DROP TABLE users; --"
        response = client.post("/api/auth/login", json={
            "email": malicious_input,
            "password": "test"
        })
        assert response.status_code in [401, 422]  # Should reject, not error

    def test_jwt_expiration(self, client, freezer):
        """Test JWT token expiration."""
        # Get token
        response = client.post("/api/auth/login", json={
            "email": "test@example.com",
            "password": "SecurePass123!"
        })
        token = response.json()["access_token"]

        # Move time forward 2 hours
        freezer.move_to("2 hours")

        # Try to use expired token
        response = client.get("/api/protected",
                            headers={"Authorization": f"Bearer {token}"})
        assert response.status_code == 401
EOF

# Run enhanced test suite
pytest tests/ -v --cov=app --cov-report=term-missing

# Store review results
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "QA Review: Original tests: 12, edge cases added: 15, integration: 8, performance: 5, security: 10, total: 50 tests, coverage: 94%. Recommendations: add load testing for concurrent users, implement contract testing for APIs, add mutation testing" \
  --type completion --agent qa-engineer

node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Status: QA review complete - 94% coverage, 50 total tests" \
  --type status --agent qa-engineer
```

### Security Auditor Protocol

```bash
# Retrieve implementation for security analysis
node ~/git/cc-orchestra/src/knowledge-manager.js search "implementation GREEN phase"

# Perform security analysis
# Check for vulnerabilities
bandit -r app/ -f json > security_report.json
safety check --json > dependencies_report.json

# Store findings
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Security Audit: Vulnerabilities (critical: 0, high: 0, medium: 2, low: 5), OWASP compliance (injection/broken_auth/xxe/access_control/security_config/xss/deserialization: passed, sensitive_data/components: review needed, logging: improve). Recommendations: implement rate limiting, add CSRF protection, enable security headers, implement audit logging, add input sanitization middleware. Threat model: 12 identified threats, 10 mitigated, 2 pending" \
  --type completion --agent security-auditor

node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Status: Security audit complete - no critical vulnerabilities found" \
  --type status --agent security-auditor
```

### Documentation Lead Protocol

```bash
# Retrieve all context for documentation
node ~/git/cc-orchestra/src/knowledge-manager.js search "architecture decisions"
node ~/git/cc-orchestra/src/knowledge-manager.js search "implementation GREEN phase"
node ~/git/cc-orchestra/src/knowledge-manager.js search "QA review"
node ~/git/cc-orchestra/src/knowledge-manager.js search "security audit"

# Create documentation
cat > docs/API_DOCUMENTATION.md << 'EOF'
# API Documentation

## Architecture Overview

The system follows a microservices architecture with:
- REST API (Python/FastAPI)
- Mobile Apps (Swift/Flutter)
- Backend Services (Go)
- PostgreSQL Database

## Authentication

JWT-based authentication with refresh tokens.

### Login Endpoint
`POST /api/auth/login`

Request:
```json
{
  "email": "user@example.com",
  "password": "SecurePassword123!"
}
```

Response:
```json
{
  "access_token": "eyJhbGc...",
  "token_type": "bearer",
  "expires_in": 3600
}
```

## Testing Strategy

- **Unit Tests**: 94% coverage
- **Integration Tests**: All API endpoints
- **Security Tests**: OWASP compliance
- **Performance Tests**: Load and stress testing

## Deployment

Docker-based deployment with:
- Kubernetes orchestration
- GitHub Actions CI/CD
- Monitoring with Prometheus/Grafana
EOF

# Store documentation status
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Documentation: Created API_DOCUMENTATION.md, DEPLOYMENT_GUIDE.md, TESTING_STRATEGY.md, SECURITY_GUIDE.md. Diagrams: architecture.mermaid, sequence_flows.mermaid, data_model.dbml. Status: complete" \
  --type completion --agent documentation-lead

node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Status: Technical documentation complete" \
  --type status --agent documentation-lead
```

---

## Memory Structure Summary

```yaml
architect/decisions:
  - System architecture
  - Component boundaries
  - Technology choices

tdd/failing-tests/*:
  - Test files by language
  - Coverage targets
  - Test types

tdd/fixtures:
  - Mock data
  - Test database setup
  - Authentication tokens

coder/implementation/*:
  - Implementation status
  - Files created
  - Test results

integration/*:
  - API patterns
  - Third-party integrations
  - Authentication setup

qa/review:
  - Test enhancements
  - Coverage metrics
  - Recommendations

security/findings:
  - Vulnerabilities
  - OWASP compliance
  - Threat model

docs/technical:
  - Documentation files
  - Diagrams
  - Guides
```

---

## Success Validation

```bash
# Validate TDD compliance
node ~/git/cc-orchestra/src/knowledge-manager.js search "failing tests RED phase"
# Should show TDD agent entries with "RED phase" initially
node ~/git/cc-orchestra/src/knowledge-manager.js search "implementation GREEN phase"
# Should show coding agent entries with "GREEN phase" after implementation

# Check test coverage
node ~/git/cc-orchestra/src/knowledge-manager.js search "QA review coverage"
# Should show QA engineer entries with ≥90% coverage

# Verify security
node ~/git/cc-orchestra/src/knowledge-manager.js search "security audit vulnerabilities"
# Should show security auditor entries with 0 critical vulnerabilities

# Confirm documentation
node ~/git/cc-orchestra/src/knowledge-manager.js search "documentation complete"
# Should show documentation lead entries with "complete" status
```

---

## Protocol Version History

- v1.0 (2025-11-04): Initial TDD-aware protocol with 15 agents