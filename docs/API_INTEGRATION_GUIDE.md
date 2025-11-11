# API Integration Specialists Guide

## Overview

The Claude Orchestra includes three specialized API integration agents to handle external service integrations. Your organization uses:

- **Authentik** heavily for centralized authentication across all applications
- **Salesforce** extensively via API for business operations
- **Other Third-Party APIs** as needed (handled by API Explorer)

These are independent integrations - Authentik and Salesforce are not related to each other.

## Integration Agents

### 1. API Explorer (General-Purpose)

**Role:** Explore, test, and document third-party APIs

**Capabilities:**
- REST and GraphQL API exploration
- Authentication flow testing (OAuth, API keys, JWT)
- OpenAPI/Swagger documentation generation
- Rate limit and quota analysis
- Integration POC development
- API client code generation
- Change monitoring and deprecation tracking

**Use Cases:**
- Exploring new third-party APIs
- Creating integration POCs
- Documenting API capabilities
- Testing authentication flows
- Analyzing API performance

**Example Scenarios:**
```
"Explore the Stripe API and create a payment integration POC"
"Test the GitHub API and document webhook capabilities"
"Analyze the Twilio API rate limits and create a client wrapper"
```

---

### 2. Salesforce API Specialist

**Role:** Salesforce API integration expert

**Capabilities:**
- **REST API**: CRUD operations, custom endpoints
- **SOAP API**: Enterprise/partner API integrations
- **SOQL/SOSL**: Optimized queries for Salesforce objects
- **Bulk API 2.0**: Large-scale data operations
- **Streaming API**: Real-time event subscriptions
- **Platform Events**: Pub/sub event-driven architecture
- **Change Data Capture**: Track object changes
- **Metadata API**: Deploy custom objects and configurations
- **OAuth 2.0**: Salesforce authentication flows

**Salesforce-Specific Knowledge:**
- Governor limits and best practices
- Salesforce object relationships
- Custom object mapping
- Apex integration patterns
- Lightning Platform capabilities
- Sandbox vs production environments

**Example Scenarios:**

#### Scenario 1: Import Leads from External System to Salesforce
```
"Import leads from our marketing database to Salesforce
and keep them in sync"

Agents Deployed:
- Architect: Designs sync architecture
- Salesforce API Expert: Creates/updates Salesforce Leads
- Python Expert: Implements sync logic and ETL
- API Explorer: Documents marketing DB API
- QA: Tests sync with various scenarios
- Security: Reviews API authentication
- Docs: Documents sync process

Deliverables:
- Lead import/sync script
- Field mapping configuration
- Error handling and retry logic
- Sync status dashboard
- Documentation
```

#### Scenario 2: Salesforce Event-Driven Integration
```
"Listen to Salesforce Platform Events and trigger workflows in our system"

Agents Deployed:
- Architect: Event-driven architecture design
- Salesforce API Expert: Platform Events integration
- Go Expert: Event handler service
- DevOps: Containerize and deploy
- QA: Event testing scenarios
- Docs: Event documentation

Deliverables:
- Platform Events listener
- Event handler service
- Dead letter queue for failed events
- Monitoring and alerting
- Documentation
```

#### Scenario 3: Bulk Data Export
```
"Export all Opportunities from Salesforce and store in our data warehouse"

Agents Deployed:
- Architect: Data pipeline design
- Salesforce API Expert: Bulk API export
- Python Expert: ETL pipeline
- DevOps: Schedule and deploy
- QA: Data validation tests
- Docs: Pipeline documentation

Deliverables:
- Bulk export script using Bulk API 2.0
- Data transformation logic
- Incremental sync support
- Error handling
- Scheduling configuration
```

---

### 3. Authentik API Specialist

**Role:** Authentik authentication and API integration expert

**Capabilities:**
- **OAuth2/OIDC**: Configure providers and flows
- **User Management**: Create, update, delete users via API
- **Group Management**: Manage groups and memberships
- **Application Providers**: Configure OAuth2, SAML, LDAP providers
- **SAML Integration**: SAML 2.0 service provider setup
- **Flow Customization**: Custom authentication flows
- **MFA Setup**: Multi-factor authentication configuration
- **Policy Engine**: Attribute-based access control
- **Events API**: Audit logging and event streaming
- **Outposts**: Proxy provider management

**Authentik-Specific Knowledge:**
- Core API structure (/api/v3/)
- OAuth2 vs SAML provider differences
- Flow and stage architecture
- Property mappings
- User attributes and custom fields
- Webhook integration patterns

**Example Scenarios:**

#### Scenario 1: Centralized Authentication for New App
```
"Configure Authentik OAuth2 provider for our new Flutter app"

Agents Deployed:
- Architect: Authentication flow design
- Authentik API Expert: OAuth2 provider configuration
- Flutter Expert: OAuth2 client implementation
- Security: Review authentication security
- Docs: Authentication documentation

Deliverables:
- Authentik OAuth2 provider configuration
- Flutter OAuth2 client
- Token refresh logic
- User attribute mapping
- MFA enforcement policy
- Documentation
```

#### Scenario 2: User Provisioning from HR System
```
"Automatically provision users in Authentik when they're added to our HR system"

Agents Deployed:
- Architect: Provisioning workflow design
- API Explorer: Explore HR system API
- Authentik API Expert: User creation via API
- Python Expert: Provisioning service
- DevOps: Deploy and monitor
- QA: Provisioning test scenarios
- Security: Review security
- Credentials: Manage API keys

Deliverables:
- HR system webhook integration
- Authentik user provisioning service
- Attribute mapping configuration
- Group assignment logic
- Error handling and notifications
- Monitoring dashboard
```

#### Scenario 3: SAML SSO for Enterprise App
```
"Configure SAML SSO with Authentik for our enterprise SaaS app"

Agents Deployed:
- Architect: SAML flow design
- Authentik API Expert: SAML provider configuration
- Python/Go Expert: SAML assertion validation
- Security: SAML security review
- QA: SSO testing
- Docs: SSO setup guide

Deliverables:
- Authentik SAML provider
- SAML metadata exchange
- Assertion validation logic
- Attribute mapping
- Group-based authorization
- SSO documentation
```

#### Scenario 4: Sync Groups Between Authentik and Applications
```
"Sync Authentik groups to application roles automatically"

Agents Deployed:
- Architect: Group sync architecture
- Authentik API Expert: Groups API integration
- Python Expert: Sync service
- DevOps: Deploy scheduler
- QA: Group sync testing

Deliverables:
- Group sync service
- Role mapping configuration
- Incremental sync support
- Conflict resolution
- Audit logging
```

---

## Coordination Patterns

### Pattern 1: Authentik Authentication Integration

**Scenario:** "Add Authentik authentication to our new web application"

**Agent Flow:**
1. **Architect** designs authentication flow
2. **Authentik API Expert** configures OAuth2/OIDC provider
3. **Python/Go Expert** implements authentication in the app
4. **QA** tests authentication flows
5. **Security** reviews security
6. **DevOps** deploys
7. **Credentials** manages OAuth credentials
8. **Docs** creates authentication documentation

---

### Pattern 2: Salesforce Data Integration

**Scenario:** "Build a dashboard that displays Salesforce data"

**Agent Flow:**
1. **Architect** designs dashboard architecture
2. **API Explorer** explores required Salesforce API endpoints
3. **Salesforce API Expert** implements data retrieval
4. **Python/Go Expert** builds the backend service
5. **Flutter Expert** builds mobile dashboard (if needed)
6. **QA** tests data accuracy
7. **Security** reviews security
8. **DevOps** deploys
9. **Credentials** manages Salesforce credentials
10. **Docs** creates documentation

---

### Pattern 3: Combined Authentik Auth + Salesforce Data

**Scenario:** "Build an internal app with Authentik authentication that displays Salesforce data"

**Agent Flow:**
1. **Architect** designs the complete application
2. **Authentik API Expert** sets up OAuth2 authentication
3. **Salesforce API Expert** implements data access
4. **Python/Go Expert** builds the application
5. **QA** tests both auth and data flows
6. **Security** comprehensive security review
7. **DevOps** deploys
8. **Credentials** manages both sets of credentials
9. **Docs** creates complete documentation

---

## Best Practices

### For API Explorer
- Always document rate limits and quotas
- Create reusable API client code
- Test error scenarios
- Monitor for API changes
- Version API documentation

### For Salesforce API Specialist
- Respect governor limits
- Use Bulk API for large operations
- Implement exponential backoff for retries
- Cache SOQL results where appropriate
- Use composite API for multiple operations
- Monitor API usage via Setup → System Overview

### For Authentik API Specialist
- Use service accounts for API access
- Implement token refresh logic
- Validate all user attributes
- Use Authentik's built-in rate limiting
- Monitor events for security anomalies
- Test MFA flows thoroughly
- Document all custom flows and stages

---

## Coordination Protocol

All integration agents follow this protocol:

### Before Integration Work
```bash
# Retrieve architecture decisions from Knowledge Manager
node ~/git/cc-orchestra/src/knowledge-manager.js search "architect decisions"
node ~/git/cc-orchestra/src/knowledge-manager.js search "architecture patterns"
```

### During Integration
```bash
# Store API exploration results in Knowledge Manager
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "API Schema: Salesforce objects (Contact, Lead) with endpoints documented" \
  --type implementation --agent salesforce-expert

# Store file edits
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Edit: salesforce_client.py - Implemented OAuth and API client" \
  --type edit --agent salesforce-expert

# Notify other agents of completion
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Status: Salesforce OAuth configured, ready for integration" \
  --type status --agent salesforce-expert
```

### After Integration
```bash
# Store final configuration in Knowledge Manager
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Integration complete: Salesforce API with endpoints and rate limits configured" \
  --type completion --agent salesforce-expert

# Document final status
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Task complete: Salesforce integration ready for production" \
  --type completion --agent salesforce-expert
```

---

## Example Prompts

### Salesforce Integration Examples

**Example 1: Data Sync**
```
"Sync Salesforce Opportunities to our internal database for reporting"

Orchestra deploys:
- Architect
- Salesforce API Specialist
- Python Expert (sync service)
- DevOps (deploy and schedule)
- QA (integration tests)
- Security (review OAuth)
- Credentials (manage credentials)
- Docs (integration documentation)
```

**Example 2: Event-Driven Workflows**
```
"Listen to Salesforce Platform Events and trigger internal workflows"

Orchestra deploys:
- Architect
- Salesforce API Specialist
- Go Expert (event handler)
- DevOps (deploy)
- QA (event testing)
- Docs (documentation)
```

### Authentik Integration Examples

**Example 1: OAuth2 Authentication**
```
"Add Authentik OAuth2 authentication to our Flutter mobile app"

Orchestra deploys:
- Architect
- Authentik API Specialist
- Flutter Expert (OAuth2 client)
- Security (security review)
- QA (auth flow testing)
- Credentials (manage OAuth credentials)
- Docs (authentication documentation)
```

**Example 2: SAML for Enterprise**
```
"Configure SAML SSO with Authentik for our enterprise SaaS platform"

Orchestra deploys:
- Architect
- Authentik API Specialist
- Go/Python Expert (SAML service provider)
- Security (SAML security review)
- QA (SSO testing)
- Docs (SSO setup guide)
```

**Example 3: User Provisioning**
```
"Automatically create Authentik users when they join via our registration flow"

Orchestra deploys:
- Architect
- Authentik API Specialist
- Python Expert (provisioning service)
- DevOps (deploy)
- QA (provisioning tests)
- Security (review)
- Credentials (API credentials)
- Docs (documentation)
```

### Independent Use Cases

**Authentik Only:**
- "Add SSO to our internal tools using Authentik"
- "Set up MFA for all Authentik users"
- "Create custom Authentik flow for contractor onboarding"

**Salesforce Only:**
- "Export all Opportunities to data warehouse"
- "Create automated lead scoring in Salesforce"
- "Build custom Salesforce dashboard integration"

**Both (but separate):**
- "Build an app that uses Authentik for auth AND displays Salesforce data"
  - Authentik handles: Authentication, authorization
  - Salesforce handles: Business data access

---

## Performance & Monitoring

### Salesforce API Monitoring
- Daily API usage via Salesforce Setup
- Track governor limit exceptions
- Monitor Bulk API job status
- Alert on API errors
- Track sync latency

### Authentik API Monitoring
- Monitor authentication success/failure rates
- Track API response times
- Alert on failed user provisioning
- Monitor event stream lag
- Track MFA bypass attempts

---

## Summary

With three specialized API integration agents, your Claude Orchestra can:

✅ **Explore any API** with the API Explorer
✅ **Deep Salesforce integration** with the Salesforce specialist
✅ **Centralized authentication** with the Authentik specialist
✅ **Coordinate complex integrations** across multiple systems
✅ **Handle authentication flows** from simple OAuth to enterprise SAML
✅ **Sync data bidirectionally** between Authentik, Salesforce, and your apps

The integration specialists work seamlessly with coding agents to build complete, production-ready integrations with proper security, testing, and documentation.
