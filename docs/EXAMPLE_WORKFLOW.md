# Example Workflow: Building a Full-Stack Application

This example demonstrates the Claude Orchestra in action, building a complete full-stack application with authentication, mobile app, and ML features.

## User Request

```
"Build a task management application with:
- Mobile app (Flutter)
- Backend API (Go)
- ML-powered task prioritization (Python)
- User authentication with JWT
- Real-time notifications
- End-to-end tests
- Complete documentation"
```

## Claude Code Response (Single Message)

```javascript
// Phase 1: MCP Coordination Setup (OPTIONAL)
// MCP swarm initialization can be used for advanced coordination
// but is not required - Knowledge Manager provides coordination

// Phase 2: Parallel Agent Spawn (ALL in ONE message)

// ARCHITECT - Strategic Leadership
Task(
  "Chief Architect",
  `You are the Chief Architect for this project.

USER REQUIREMENT:
Build a task management application with:
- Mobile app (Flutter)
- Backend API (Go)
- ML-powered task prioritization (Python)
- User authentication with JWT
- Real-time notifications
- End-to-end tests
- Complete documentation

YOUR RESPONSIBILITIES:
1. Analyze the requirement and design the system architecture
2. Define the component breakdown:
   - Mobile app architecture and screens
   - Backend API endpoints and services
   - ML service design and integration
   - Authentication flow
   - Real-time notification system
3. Make technology stack decisions
4. Store all decisions in shared memory
5. Coordinate with all coding and support agents
6. Review final implementation

COORDINATION:
- Store architecture: 'node ~/git/cc-army/src/knowledge-manager.js store "Architecture: [architecture details]" --type decision --agent architect'
- Notify agents when design is complete

OUTPUT:
- System architecture document
- Component specifications for each coding agent
- API contract definitions
- Database schema
- Security requirements
- Testing requirements`,
  "system-architect",
  { model: "opus" }
)

// CODING AGENTS - Implementation

Task(
  "Flutter Specialist",
  `You are the Flutter mobile development expert.

PROJECT: Task Management Application

YOUR RESPONSIBILITIES:
1. Check shared memory for architecture decisions from the Chief Architect
2. Build Flutter mobile app with:
   - Login/registration screens
   - Task list with priority indicators (ML-powered)
   - Task creation/editing
   - Real-time notifications
   - Offline support
3. Implement state management (Provider/Riverpod)
4. Integrate with Go backend API
5. Handle authentication tokens
6. Coordinate with QA for testing

COORDINATION:
Before: 'node ~/git/cc-army/src/knowledge-manager.js search "architecture"'
During: 'node ~/git/cc-army/src/knowledge-manager.js store "Edit: lib/main.dart - Implemented mobile app features" --type edit --agent flutter-specialist'
After: 'node ~/git/cc-army/src/knowledge-manager.js store "Implementation: Flutter mobile app complete" --type implementation --agent flutter-specialist'
Notify: 'node ~/git/cc-army/src/knowledge-manager.js store "Status: Mobile app features complete" --type status --agent flutter-specialist'

QUALITY STANDARDS:
- Material Design UI
- Responsive layouts
- Error handling
- Loading states
- Offline-first architecture`,
  "mobile-developer",
  { model: "sonnet" }
)

Task(
  "Go Specialist",
  `You are the Go backend development expert.

PROJECT: Task Management Application Backend

YOUR RESPONSIBILITIES:
1. Retrieve architecture from shared memory
2. Build Go REST API with:
   - User authentication endpoints (JWT)
   - Task CRUD operations
   - Real-time WebSocket for notifications
   - Integration with Python ML service
   - Database operations (PostgreSQL)
3. Implement middleware:
   - Authentication
   - Logging
   - Rate limiting
   - CORS
4. API documentation (OpenAPI/Swagger)
5. Coordinate with Python and Flutter agents

COORDINATION:
Before: 'node ~/git/cc-army/src/knowledge-manager.js search "architecture"'
During: 'node ~/git/cc-army/src/knowledge-manager.js store "Edit: main.go - Implemented backend API" --type edit --agent go-specialist'
Store API contracts: 'node ~/git/cc-army/src/knowledge-manager.js store "API Contracts: [contract details]" --type decision --agent go-specialist'
Notify: 'node ~/git/cc-army/src/knowledge-manager.js store "Status: Backend API complete" --type status --agent go-specialist'

QUALITY STANDARDS:
- Clean architecture
- Error handling
- Logging
- Input validation
- Performance optimization`,
  "backend-dev",
  { model: "sonnet" }
)

Task(
  "Python Specialist",
  `You are the Python ML/backend expert.

PROJECT: ML-Powered Task Prioritization Service

YOUR RESPONSIBILITIES:
1. Retrieve architecture from shared memory
2. Build Python ML service:
   - Task prioritization model (scikit-learn or simple heuristic)
   - FastAPI endpoint for priority predictions
   - Training pipeline (if ML-based)
   - Integration with Go backend
3. Implement features:
   - Priority scoring based on due date, dependencies, user history
   - REST API for predictions
   - Model versioning
4. Coordinate with Go backend for integration

COORDINATION:
Before: 'node ~/git/cc-army/src/knowledge-manager.js search "architecture"'
Before: 'node ~/git/cc-army/src/knowledge-manager.js search "api contracts"'
During: 'node ~/git/cc-army/src/knowledge-manager.js store "Edit: main.py - Implemented ML prioritization service" --type edit --agent python-specialist'
Store: 'node ~/git/cc-army/src/knowledge-manager.js store "Implementation: Python ML service complete" --type implementation --agent python-specialist'
Notify: 'node ~/git/cc-army/src/knowledge-manager.js store "Status: ML service complete" --type status --agent python-specialist'

QUALITY STANDARDS:
- Async operations
- Error handling
- Model performance metrics
- API documentation
- Testing`,
  "python-expert",
  { model: "sonnet" }
)

// SUPPORT AGENTS - Quality & Documentation

Task(
  "QA Engineer",
  `You are the QA and integration testing expert.

PROJECT: Task Management Application Testing

YOUR RESPONSIBILITIES:
1. Monitor shared memory for completed features
2. Create comprehensive test suite:
   - Integration tests for all API endpoints
   - Mobile app UI tests (Flutter widget tests)
   - End-to-end tests (user flows)
   - ML service accuracy tests
   - Authentication flow tests
   - Real-time notification tests
3. Set up test automation
4. Generate test coverage reports
5. Coordinate with all coding agents

COORDINATION:
Monitor: 'node ~/git/cc-army/src/knowledge-manager.js list'
Retrieve specs: 'node ~/git/cc-army/src/knowledge-manager.js search "api contracts"'
During: 'node ~/git/cc-army/src/knowledge-manager.js store "Edit: tests/integration_test.go - Added comprehensive test suite" --type edit --agent qa-engineer'
Store results: 'node ~/git/cc-army/src/knowledge-manager.js store "Test Results: [coverage and results]" --type completion --agent qa-engineer'
Notify: 'node ~/git/cc-army/src/knowledge-manager.js store "Status: Test suite complete - X% coverage" --type status --agent qa-engineer'

TEST REQUIREMENTS:
- Unit tests for all services
- Integration tests for API
- E2E tests for critical flows
- Performance tests
- Security tests
- 90%+ code coverage`,
  "test-automator",
  { model: "sonnet" }
)

Task(
  "Security Auditor",
  `You are the security analysis expert.

PROJECT: Task Management Application Security Audit

YOUR RESPONSIBILITIES:
1. Review all code for security vulnerabilities
2. Audit specific areas:
   - JWT implementation (token storage, expiration, refresh)
   - Authentication flows
   - Authorization (task ownership checks)
   - Input validation
   - SQL injection prevention
   - XSS prevention in mobile app
   - API rate limiting
   - Credential storage
   - WebSocket security
3. Check OWASP Top 10 compliance
4. Generate security report
5. Coordinate with Credential Manager

COORDINATION:
Retrieve code: 'node ~/git/cc-army/src/knowledge-manager.js search "go implementation"'
Retrieve code: 'node ~/git/cc-army/src/knowledge-manager.js search "flutter implementation"'
During: 'node ~/git/cc-army/src/knowledge-manager.js store "Edit: SECURITY_AUDIT.md - Documented security findings" --type edit --agent security-auditor'
Store findings: 'node ~/git/cc-army/src/knowledge-manager.js store "Security Audit Results: [findings]" --type completion --agent security-auditor'
Notify critical: 'node ~/git/cc-army/src/knowledge-manager.js store "Status: CRITICAL - JWT secret in code" --type status --agent security-auditor'

SECURITY FOCUS:
- Authentication/authorization
- Data validation
- Encryption
- Secure communication
- Dependency vulnerabilities
- Best practices compliance`,
  "security-auditor",
  { model: "sonnet" }
)

Task(
  "Documentation Lead",
  `You are the technical documentation expert.

PROJECT: Task Management Application Documentation

YOUR RESPONSIBILITIES:
1. Create comprehensive documentation:
   - README with setup instructions
   - Architecture documentation
   - API documentation (from Go specs)
   - Mobile app user guide
   - ML service documentation
   - Deployment guide
   - Development guide
   - Contributing guide
2. Create architecture diagrams
3. Document all API endpoints
4. Coordinate with all agents for accurate info

COORDINATION:
Retrieve arch: 'node ~/git/cc-army/src/knowledge-manager.js search "architecture"'
Retrieve APIs: 'node ~/git/cc-army/src/knowledge-manager.js search "api contracts"'
During: 'node ~/git/cc-army/src/knowledge-manager.js store "Edit: docs/API.md - Documented API endpoints" --type edit --agent documentation-lead'
Store: 'node ~/git/cc-army/src/knowledge-manager.js store "Status: Documentation complete" --type completion --agent documentation-lead'

DOCUMENTATION REQUIRED:
- System architecture
- API reference
- Setup/deployment
- User guides
- Code examples
- Troubleshooting
- FAQ`,
  "coder",
  { model: "haiku" }
)

Task(
  "Credential Manager",
  `You are the secure credential management expert.

PROJECT: Task Management Application Credentials

YOUR RESPONSIBILITIES:
1. Design secure credential storage strategy
2. Track all credentials needed:
   - JWT signing secret
   - Database connection strings
   - ML service API keys (if external)
   - WebSocket secrets
   - Third-party API keys (notifications)
3. Implement credential storage using the credential manager
4. Document credential rotation procedures
5. Coordinate with Security Auditor
6. Never store credentials in code

CREDENTIALS TO MANAGE:
- JWT_SECRET (high security, rotate monthly)
- DB_PASSWORD (high security, rotate quarterly)
- NOTIFICATION_API_KEY (medium security)
- ML_SERVICE_KEY (medium security)

COORDINATION:
Before: 'node ~/git/cc-army/src/knowledge-manager.js search "architecture"'
Store credentials: Use src/credential-manager.js
Document: 'node ~/git/cc-army/src/knowledge-manager.js store "Edit: docs/CREDENTIALS.md - Documented credential management" --type edit --agent credential-manager'
Notify: 'node ~/git/cc-army/src/knowledge-manager.js store "Status: Credential system ready" --type status --agent credential-manager'

OUTPUT:
- Credential inventory
- Storage strategy
- Rotation procedures
- Environment variable template
- Security best practices`,
  "coder",
  { model: "haiku" }
)

// TODOS - All in ONE call
TodoWrite({
  todos: [
    // Architecture Phase
    { content: "Analyze requirements and design architecture", status: "in_progress", activeForm: "Analyzing and designing architecture" },
    { content: "Define system components and APIs", status: "pending", activeForm: "Defining components and APIs" },
    { content: "Create database schema", status: "pending", activeForm: "Creating database schema" },

    // Backend Implementation
    { content: "Implement Go backend API structure", status: "pending", activeForm: "Implementing Go backend structure" },
    { content: "Build authentication endpoints with JWT", status: "pending", activeForm: "Building auth endpoints" },
    { content: "Create task CRUD operations", status: "pending", activeForm: "Creating task operations" },
    { content: "Implement WebSocket for real-time notifications", status: "pending", activeForm: "Implementing WebSocket" },

    // ML Service
    { content: "Build Python ML prioritization service", status: "pending", activeForm: "Building ML service" },
    { content: "Create priority prediction API", status: "pending", activeForm: "Creating prediction API" },

    // Mobile App
    { content: "Build Flutter app structure and navigation", status: "pending", activeForm: "Building Flutter structure" },
    { content: "Implement authentication screens", status: "pending", activeForm: "Implementing auth screens" },
    { content: "Create task list and detail views", status: "pending", activeForm: "Creating task views" },
    { content: "Integrate real-time notifications", status: "pending", activeForm: "Integrating notifications" },

    // Testing
    { content: "Write backend integration tests", status: "pending", activeForm: "Writing backend tests" },
    { content: "Create Flutter widget tests", status: "pending", activeForm: "Creating widget tests" },
    { content: "Build end-to-end test suite", status: "pending", activeForm: "Building E2E tests" },

    // Security & Credentials
    { content: "Security audit of authentication", status: "pending", activeForm: "Auditing authentication" },
    { content: "Review API security", status: "pending", activeForm: "Reviewing API security" },
    { content: "Setup credential management", status: "pending", activeForm: "Setting up credentials" },

    // Documentation
    { content: "Create architecture documentation", status: "pending", activeForm: "Creating architecture docs" },
    { content: "Write API documentation", status: "pending", activeForm: "Writing API docs" },
    { content: "Build user guides", status: "pending", activeForm: "Building user guides" }
  ]
})
```

## Expected Output Structure

After all agents complete their work:

```
claude-army/
├── backend/                    (Go Specialist)
│   ├── main.go
│   ├── handlers/
│   │   ├── auth.go
│   │   ├── tasks.go
│   │   └── websocket.go
│   ├── models/
│   │   └── task.go
│   ├── middleware/
│   │   └── auth.go
│   └── database/
│       └── postgres.go
│
├── mobile/                     (Flutter Specialist)
│   ├── lib/
│   │   ├── main.dart
│   │   ├── screens/
│   │   │   ├── login_screen.dart
│   │   │   ├── task_list_screen.dart
│   │   │   └── task_detail_screen.dart
│   │   ├── services/
│   │   │   ├── api_service.dart
│   │   │   └── auth_service.dart
│   │   └── models/
│   │       └── task.dart
│   └── test/
│       └── widget_test.dart
│
├── ml-service/                 (Python Specialist)
│   ├── main.py
│   ├── model/
│   │   └── prioritizer.py
│   ├── api/
│   │   └── routes.py
│   └── requirements.txt
│
├── tests/                      (QA Engineer)
│   ├── integration/
│   │   ├── test_auth.py
│   │   ├── test_tasks.py
│   │   └── test_notifications.py
│   ├── e2e/
│   │   └── test_user_flows.py
│   └── performance/
│       └── load_test.py
│
├── docs/                       (Documentation Lead)
│   ├── ARCHITECTURE.md
│   ├── API.md
│   ├── SETUP.md
│   ├── USER_GUIDE.md
│   └── CREDENTIALS.md
│
├── security/                   (Security Auditor)
│   ├── SECURITY_AUDIT.md
│   └── VULNERABILITY_REPORT.md
│
├── config/                     (Credential Manager)
│   ├── credential-inventory.json
│   └── .env.example
│
└── README.md                   (Documentation Lead)
```

## Agent Coordination Timeline

### Minutes 0-5: Architecture Phase
- **Architect**: Analyzes requirements, designs system
- **All agents**: Wait for architecture decisions in shared memory

### Minutes 5-15: Parallel Implementation
- **Flutter**: Builds mobile UI and screens
- **Go**: Implements backend API and auth
- **Python**: Creates ML service
- **Credentials**: Sets up credential system
- **Docs**: Starts documentation based on architecture

### Minutes 15-25: Integration & Testing
- **QA**: Runs integration tests as features complete
- **Security**: Reviews code for vulnerabilities
- **All coding agents**: Fix issues found by QA and Security
- **Docs**: Updates documentation

### Minutes 25-30: Final Review
- **Architect**: Reviews all outputs
- **Security**: Final security audit
- **QA**: Final test suite run
- **Docs**: Finalizes documentation
- **Credentials**: Documents all credential requirements

## Shared Memory Contents

Throughout execution, shared memory contains:

```json
{
  "architect/architecture": {
    "components": ["mobile-app", "backend-api", "ml-service"],
    "tech-stack": {
      "mobile": "Flutter",
      "backend": "Go + Gin",
      "ml": "Python + FastAPI",
      "database": "PostgreSQL"
    },
    "api-version": "v1"
  },
  "go/api-contracts": {
    "endpoints": [
      "POST /auth/login",
      "POST /auth/register",
      "GET /tasks",
      "POST /tasks",
      "PUT /tasks/:id",
      "DELETE /tasks/:id",
      "WS /notifications"
    ]
  },
  "flutter/implementation": {
    "screens": ["login", "register", "task-list", "task-detail"],
    "state-management": "Provider",
    "status": "complete"
  },
  "python/ml-service": {
    "model": "priority-predictor",
    "endpoint": "POST /predict",
    "status": "complete"
  },
  "qa/test-results": {
    "total-tests": 87,
    "passed": 87,
    "failed": 0,
    "coverage": "94.3%"
  },
  "security/audit-results": {
    "critical": 0,
    "high": 0,
    "medium": 2,
    "low": 5,
    "status": "approved-with-recommendations"
  }
}
```

## Benefits Demonstrated

1. **Speed**: 2.8-4.4x faster than sequential development
2. **Quality**: Built-in security auditing and testing
3. **Completeness**: Documentation generated in parallel
4. **Coordination**: Shared memory ensures consistency
5. **Security**: Credentials managed from the start
6. **Expertise**: Language-specific specialists for each component

## Lessons Learned

- The Architect's early decisions guide all agents efficiently
- Shared memory prevents duplication and ensures consistency
- QA and Security running in parallel catch issues early
- Documentation agent has full context from shared memory
- Credential management from the start prevents security issues
- All agents working in parallel significantly reduces total time
