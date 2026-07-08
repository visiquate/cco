# TDD: Test-Driven Development

## Trigger
`/tdd <requirement>`

Implements test-driven development workflow with emphasis on writing tests first, then implementation.

## Purpose
The TDD skill ensures:
- Tests are written BEFORE code
- Implementation is focused on passing tests
- High test coverage from the start
- Refactoring safety through comprehensive test suite
- Clear requirements expressed as test cases

## Workflow

### Phase 1: Analyze Requirements

1. **Parse requirement**
   - Identify acceptance criteria
   - Extract edge cases
   - Note performance/security constraints

2. **Spawn Query Clarifier** if needed (Haiku)
   - Get clear, unambiguous requirements
   - Identify assumptions
   - Document acceptance criteria

### Phase 2: Design Test Suite

1. **Spawn TDD Agent** (Sonnet)
   - Design test architecture
   - Identify test categories:
     - Unit tests (functions, methods)
     - Integration tests (component interactions)
     - Edge case tests (boundaries, error conditions)
     - Performance tests (if applicable)
     - Security tests (if applicable)

2. **TDD Agent creates test file structure**
   ```
   tests/
   ├── unit/
   │   ├── test_core_logic.py
   │   ├── test_utils.py
   │   └── test_validators.py
   ├── integration/
   │   ├── test_api_endpoints.py
   │   └── test_database_integration.py
   ├── edge_cases/
   │   └── test_boundary_conditions.py
   └── conftest.py (fixtures)
   ```

3. **TDD Agent designs test cases**
   - Input: Acceptance criteria
   - Output: Named test cases
   - Pattern: test_[feature]_[scenario]_[expected_result]

### Phase 3: Write Tests FIRST (TDD Agent)

**Key Principle**: No implementation code until tests are written

1. **Write unit tests**
   ```python
   def test_jwt_token_creation_with_valid_payload():
       token = create_jwt_token({"user_id": 123})
       assert token is not None
       assert isinstance(token, str)

   def test_jwt_token_expires_in_24_hours():
       token = create_jwt_token({"user_id": 123})
       decoded = decode_jwt_token(token)
       assert decoded["exp"] - decoded["iat"] == 86400  # 24 hours

   def test_jwt_token_validation_fails_with_invalid_signature():
       token = "invalid.jwt.token"
       with pytest.raises(JWTInvalidSignatureError):
           decode_jwt_token(token)
   ```

2. **Write integration tests**
   ```python
   def test_login_endpoint_returns_jwt_token():
       response = client.post("/auth/login",
           json={"email": "user@example.com", "password": "secret"})
       assert response.status_code == 200
       assert "token" in response.json()

   def test_protected_endpoint_requires_valid_jwt():
       response = client.get("/api/profile",
           headers={"Authorization": "Bearer invalid"})
       assert response.status_code == 401
   ```

3. **Write edge case tests**
   ```python
   def test_jwt_token_rejects_expired_token():
       # Create token that expired 1 second ago
       past_time = int(time.time()) - 1
       token = create_jwt_token({"user_id": 123, "exp": past_time})
       with pytest.raises(JWTExpiredError):
           decode_jwt_token(token)

   def test_jwt_handles_empty_payload():
       with pytest.raises(InvalidPayloadError):
           create_jwt_token({})
   ```

4. **Document test coverage goals**
   - Store in knowledge base:
   ```bash
   cco knowledge store "TDD: [feature] - Test Suite Design" \
     --type status \
     --agent tdd-agent
   ```

### Phase 4: Run Tests (Should Fail)

**Red Phase**: Tests fail because code doesn't exist yet

```bash
# TDD Agent runs test suite
pytest tests/ -v

# All tests FAIL (expected)
FAILED tests/test_auth.py::test_jwt_token_creation_with_valid_payload
FAILED tests/test_auth.py::test_jwt_token_expires_in_24_hours
FAILED tests/test_auth.py::test_jwt_token_validation_fails_with_invalid_signature
...
```

### Phase 5: Implement Code (Language Specialists)

**Green Phase**: Implement minimum code to pass tests

1. **Spawn Language Specialist** (Haiku) with test file
   - Input: Test suite
   - Task: Write minimum code to pass tests
   - Constraint: Don't over-engineer

2. **Language Specialist implements**
   ```python
   import jwt
   import time
   from datetime import datetime, timedelta

   SECRET_KEY = "your-secret-key"
   ALGORITHM = "HS256"

   def create_jwt_token(payload):
       if not payload:
           raise InvalidPayloadError("Payload cannot be empty")

       payload["iat"] = int(time.time())
       payload["exp"] = payload["iat"] + 86400  # 24 hours

       return jwt.encode(payload, SECRET_KEY, algorithm=ALGORITHM)

   def decode_jwt_token(token):
       try:
           return jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
       except jwt.ExpiredSignatureError:
           raise JWTExpiredError("Token has expired")
       except jwt.InvalidSignatureError:
           raise JWTInvalidSignatureError("Invalid token signature")
   ```

3. **Run tests - Should PASS**
   ```bash
   pytest tests/ -v

   PASSED tests/test_auth.py::test_jwt_token_creation_with_valid_payload
   PASSED tests/test_auth.py::test_jwt_token_expires_in_24_hours
   PASSED tests/test_auth.py::test_jwt_token_validation_fails_with_invalid_signature
   ... (all tests pass)
   ```

### Phase 6: Refactor (Language Specialists)

**Refactor Phase**: Improve code quality while keeping tests passing

1. **Code Reviewer examines** (Sonnet)
   - Check for code smells
   - Identify optimization opportunities
   - Suggest refactoring

2. **Language Specialist refactors** (Haiku)
   - Extract common patterns
   - Improve error handling
   - Add type hints
   - Optimize performance

3. **Run tests after each refactoring**
   - Tests validate refactoring safety
   - No regressions

### Phase 7: Test Coverage Analysis

1. **Spawn Test Engineer** (Sonnet)
   - Run coverage analysis:
   ```bash
   pytest tests/ --cov=src --cov-report=html
   ```

2. **Identify gaps**
   - Missing edge cases
   - Untested error paths
   - Low coverage areas

3. **TDD Agent writes additional tests** if needed
   - Target: Minimum 85% coverage
   - Ideally 90%+ for critical paths

### Phase 8: Integration & Validation

1. **Spawn Test Engineer** (Sonnet)
   - Run full test suite
   - Run integration tests
   - Test with real dependencies

2. **Run performance tests** if applicable
   - Load testing
   - Stress testing
   - Performance benchmarks

3. **Final validation**
   - All tests pass
   - Coverage meets target
   - No performance regressions

## Agent Spawning Pattern

```
TDD Workflow
    │
    ├─→ TDD Agent (Sonnet)
    │      Designs test architecture
    │      Writes test suite
    │      Monitors test coverage
    │
    ├─→ Language Specialist (Haiku)
    │      Implements code to pass tests
    │      Refactors with tests as safety net
    │      Runs tests frequently
    │
    ├─→ Code Reviewer (Sonnet)
    │      Reviews implementation
    │      Suggests refactorings
    │
    ├─→ Test Engineer (Sonnet)
    │      Coverage analysis
    │      Integration testing
    │      Performance testing
    │
    └─→ Performance Engineer (Sonnet) - Optional
           If performance-critical
```

## Knowledge Base Integration

### Store Test Design
```bash
cco knowledge store "TDD Design: [feature]
- Test categories: [unit, integration, edge]
- Target coverage: [%]
- Key scenarios: [list]" \
  --type decision \
  --agent tdd-agent
```

### Track Progress
```bash
# After tests written
cco knowledge store "TDD Phase: Tests written for [feature]
- Test count: [N]
- All failing as expected" \
  --type status \
  --agent tdd-agent

# After implementation
cco knowledge store "TDD Phase: Code implemented for [feature]
- All tests passing
- Coverage: [%]" \
  --type status \
  --agent language-specialist
```

### Store Coverage Report
```bash
cco knowledge store "Test Coverage: [feature]
- Line coverage: [%]
- Branch coverage: [%]
- Gaps: [list]" \
  --type status \
  --agent test-engineer
```

## Test Design Patterns

### Arrange-Act-Assert Pattern
```python
def test_login_with_valid_credentials():
    # ARRANGE - Set up test data
    user = create_test_user(email="test@example.com", password="secret")

    # ACT - Perform action
    response = client.post("/login",
        json={"email": "test@example.com", "password": "secret"})

    # ASSERT - Verify results
    assert response.status_code == 200
    assert "token" in response.json()
```

### Given-When-Then Pattern
```python
def test_expired_token_rejected():
    # GIVEN - An expired JWT token
    token = create_expired_token()

    # WHEN - Requesting a protected endpoint
    response = client.get("/api/profile",
        headers={"Authorization": f"Bearer {token}"})

    # THEN - Request is rejected
    assert response.status_code == 401
    assert response.json()["error"] == "Token expired"
```

### Parametrized Tests
```python
@pytest.mark.parametrize("invalid_token,expected_error", [
    ("malformed.token", "Invalid format"),
    ("expired_token_data", "Token expired"),
    ("invalid_signature", "Invalid signature"),
    ("", "Missing token"),
])
def test_various_invalid_tokens(invalid_token, expected_error):
    with pytest.raises(AuthenticationError) as exc_info:
        verify_token(invalid_token)
    assert expected_error in str(exc_info.value)
```

## Best Practices

1. **Test FIRST** - Write tests before implementation
2. **One assertion focus** - Each test validates one behavior
3. **Clear test names** - Name describes what is being tested
4. **DRY tests** - Use fixtures for common setup
5. **Isolated tests** - No dependencies between tests
6. **Fast tests** - Unit tests < 100ms, integration tests < 1s
7. **Deterministic** - Tests always pass/fail consistently
8. **No test-only code** - Keep implementation clean
9. **Document why** - Comments explain non-obvious tests
10. **Coverage goals** - Aim for 85%+ coverage

## Test Categories

### Unit Tests
- Test individual functions/methods
- No external dependencies (mocked)
- Fast and focused
- Largest test category (70% of tests)

### Integration Tests
- Test component interactions
- Use real dependencies (DB, API)
- Slower but comprehensive
- Critical workflows (20% of tests)

### End-to-End Tests
- Test complete user workflows
- No mocks, full system
- Slowest but most realistic
- Happy path coverage (10% of tests)

### Edge Case Tests
- Boundary conditions
- Error conditions
- Invalid inputs
- Performance limits

## Metrics to Track

| Metric | Target | Tool |
|--------|--------|------|
| **Line Coverage** | 85%+ | pytest-cov |
| **Branch Coverage** | 80%+ | pytest-cov |
| **Test Pass Rate** | 100% | pytest |
| **Test Speed** | < 30s total | pytest -v |
| **Critical Path Coverage** | 95%+ | Manual review |

## Refactoring Safety

Tests enable safe refactoring:

1. **Red-Green-Refactor Cycle**
   - Red: Tests fail (intentional)
   - Green: Tests pass (implementation)
   - Refactor: Improve code (tests stay green)

2. **Regression Prevention**
   - Change code
   - Run tests
   - If tests fail, you broke something
   - Fix and re-run

3. **Refactoring Confidence**
   - With 85%+ test coverage
   - Can refactor without fear
   - Tests catch regressions

## Example Workflows

### Feature with TDD
```
1. TDD Agent designs tests for JWT auth
2. TDD Agent writes test suite (all fail)
3. Python Specialist implements JWT code
4. Run tests → all pass
5. Code Reviewer suggests refactoring
6. Python Specialist refactors (tests still pass)
7. Test Engineer analyzes coverage
8. TDD Agent adds missing edge case tests
9. Final validation: 90% coverage, all tests pass

Result: Robust, well-tested JWT authentication
Timeline: 25-35 minutes
```

### Bug Fix with TDD
```
1. Error Detective: Root cause is missing validation
2. TDD Agent: Write regression test for bug
3. Test runs → fails (reproduces bug)
4. Language Specialist: Add validation
5. Test runs → passes
6. Test Engineer: Validate no regressions
7. Code Reviewer: Approve fix

Result: Bug fixed, regression test prevents recurrence
Timeline: 15-20 minutes
```

## Output Format

Return comprehensive test report with:
- Test suite overview
- Test design rationale
- Coverage analysis
- Test execution results
- Refactoring recommendations
- Outstanding items
- Knowledge base references
