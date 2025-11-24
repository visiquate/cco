# DevOps Engineer Agent Guide

## Overview

The DevOps Engineer is a specialized agent responsible for infrastructure, containerization, CI/CD pipelines, and production deployments. This agent has expertise in Docker, Kubernetes, AWS, and modern DevOps practices.

## Role & Responsibilities

### Primary Responsibilities
1. **Containerization**
   - Write Dockerfiles for all services
   - Create docker-compose configurations for local development
   - Optimize container images for size and security
   - Multi-stage builds for production deployments

2. **Orchestration**
   - Kubernetes manifests (Deployments, Services, ConfigMaps, Secrets)
   - Helm charts for complex applications
   - Container scaling strategies
   - Service mesh configuration (Istio, Linkerd)

3. **AWS Infrastructure**
   - ECS task definitions and services
   - CloudFormation templates
   - EC2 instance configuration
   - Lambda functions for serverless components
   - VPC, security groups, and networking
   - RDS database setup
   - S3 buckets and CloudFront distributions

4. **CI/CD Pipelines**
   - GitHub Actions workflows
   - GitLab CI/CD pipelines
   - Jenkins pipelines
   - Automated testing integration
   - Build optimization
   - Artifact management

5. **Infrastructure as Code**
   - Terraform configurations
   - CloudFormation templates
   - Ansible playbooks
   - Environment management (dev, staging, prod)

6. **Monitoring & Logging**
   - Prometheus metrics
   - Grafana dashboards
   - CloudWatch alarms
   - ELK stack setup
   - Application performance monitoring

7. **Deployment Strategies**
   - Blue-green deployments
   - Canary releases
   - Rolling updates
   - Zero-downtime deployments
   - Rollback procedures

## Agent Configuration

From `config/orchestra-config.json`:

```json
{
  "name": "DevOps Engineer",
  "type": "deployment-engineer",
  "model": "sonnet",
  "specialties": [
    "Docker & Docker Compose",
    "Kubernetes (EKS, GKE, AKS)",
    "AWS Services (ECS, ECR, CloudFormation, Lambda)",
    "CI/CD Automation",
    "Infrastructure as Code",
    "Container Security",
    "Monitoring (Prometheus, Grafana, CloudWatch)",
    "Multi-environment deployments",
    "Zero-downtime deployments",
    "Cost optimization"
  ]
}
```

## Coordination with Other Agents

### With Architect
- Receives infrastructure requirements
- Proposes deployment architecture
- Discusses scaling strategy
- Reviews cloud service selection

### With Coding Agents
- Receives application requirements for containerization
- Coordinates on environment variables
- Discusses resource requirements (CPU, memory)
- Reviews health check endpoints

### With QA Engineer
- Integrates tests into CI/CD pipeline
- Sets up testing environments
- Coordinates on test data management
- Ensures test automation in deployments

### With Security Auditor
- Reviews container security
- Implements security scanning in pipelines
- Manages secrets and credentials securely
- Configures security groups and network policies

### With Credential Manager
- Coordinates on secrets management strategy
- Implements AWS Secrets Manager or Parameter Store
- Sets up environment-specific credential injection
- Reviews credential rotation procedures

## Example Workflows

### Workflow 1: Containerizing a Python API

**User Request:** "Containerize the Python FastAPI application"

**DevOps Engineer Actions:**
1. Check shared memory for Python implementation details
2. Create Dockerfile:
   ```dockerfile
   FROM python:3.11-slim
   WORKDIR /app
   COPY requirements.txt .
   RUN pip install --no-cache-dir -r requirements.txt
   COPY . .
   EXPOSE 8000
   CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
   ```
3. Create docker-compose.yml for local development
4. Create .dockerignore
5. Add health check endpoint requirements
6. Store configuration in shared memory
7. Notify QA for container testing

### Workflow 2: Setting up CI/CD Pipeline

**User Request:** "Set up CI/CD for the project"

**DevOps Engineer Actions:**
1. Create `.github/workflows/ci-cd.yml`:
   ```yaml
   name: CI/CD Pipeline

   on:
     push:
       branches: [main]
     pull_request:
       branches: [main]

   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - name: Run tests
           run: |
             docker-compose -f docker-compose.test.yml up --abort-on-container-exit

     build:
       needs: test
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - name: Build and push Docker image
           run: |
             docker build -t myapp:${{ github.sha }} .
             docker push myapp:${{ github.sha }}

     deploy:
       needs: build
       runs-on: ubuntu-latest
       steps:
         - name: Deploy to ECS
           run: |
             aws ecs update-service --cluster prod --service myapp --force-new-deployment
   ```
2. Coordinate with QA on test commands
3. Coordinate with Credential Manager on AWS credentials
4. Document deployment process

### Workflow 3: AWS ECS Deployment

**User Request:** "Deploy to AWS ECS"

**DevOps Engineer Actions:**
1. Create ECS task definition:
   ```json
   {
     "family": "myapp",
     "containerDefinitions": [{
       "name": "app",
       "image": "myapp:latest",
       "memory": 512,
       "cpu": 256,
       "portMappings": [{
         "containerPort": 8000,
         "protocol": "tcp"
       }],
       "environment": [],
       "secrets": []
     }]
   }
   ```
2. Create CloudFormation template for infrastructure
3. Set up Application Load Balancer
4. Configure auto-scaling
5. Set up CloudWatch alarms
6. Document deployment procedure
7. Create rollback procedure

### Workflow 4: Kubernetes Deployment

**User Request:** "Deploy to Kubernetes"

**DevOps Engineer Actions:**
1. Create Kubernetes manifests:
   ```yaml
   # deployment.yaml
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: myapp
   spec:
     replicas: 3
     selector:
       matchLabels:
         app: myapp
     template:
       metadata:
         labels:
           app: myapp
       spec:
         containers:
         - name: app
           image: myapp:latest
           ports:
           - containerPort: 8000
           resources:
             requests:
               memory: "256Mi"
               cpu: "250m"
             limits:
               memory: "512Mi"
               cpu: "500m"

   # service.yaml
   apiVersion: v1
   kind: Service
   metadata:
     name: myapp
   spec:
     selector:
       app: myapp
     ports:
     - port: 80
       targetPort: 8000
     type: LoadBalancer
   ```
2. Create ConfigMaps for configuration
3. Create Secrets (coordinate with Credential Manager)
4. Set up Horizontal Pod Autoscaler
5. Configure Ingress
6. Document kubectl commands

## Coordination Protocol

### Before Work
```bash
# Retrieve architecture decisions from Knowledge Manager
node ~/git/cc-orchestra/src/knowledge-manager.js search "architect decisions"
node ~/git/cc-orchestra/src/knowledge-manager.js search "architecture"

# Retrieve application details from coding agents
node ~/git/cc-orchestra/src/knowledge-manager.js search "python implementation"
node ~/git/cc-orchestra/src/knowledge-manager.js search "go implementation"
```

### During Work
```bash
# After creating each infrastructure file
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Edit: Dockerfile - Created production-ready container image" \
  --type edit --agent devops-engineer

node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Edit: docker-compose.yml - Set up local development environment" \
  --type edit --agent devops-engineer

node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Edit: .github/workflows/ci-cd.yml - Configured CI/CD pipeline" \
  --type edit --agent devops-engineer

# Store infrastructure decisions
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Infrastructure: AWS ECS in us-east-1 with ECR container registry" \
  --type decision --agent devops-engineer
```

### After Work
```bash
# Notify other agents
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Status: CI/CD pipeline ready - deploy command: aws ecs update-service" \
  --type status --agent devops-engineer

# Complete task
node ~/git/cc-orchestra/src/knowledge-manager.js store \
  "Task complete: DevOps setup finished" \
  --type completion --agent devops-engineer
```

## Common Deliverables

### 1. Docker Configuration
- `Dockerfile` - Production-ready container image
- `docker-compose.yml` - Local development setup
- `docker-compose.test.yml` - Testing environment
- `docker-compose.prod.yml` - Production configuration
- `.dockerignore` - Build optimization

### 2. CI/CD Pipelines
- `.github/workflows/ci-cd.yml` - GitHub Actions
- `.gitlab-ci.yml` - GitLab CI
- `Jenkinsfile` - Jenkins pipeline

### 3. Kubernetes Manifests
- `k8s/deployment.yaml` - Application deployment
- `k8s/service.yaml` - Service definition
- `k8s/configmap.yaml` - Configuration
- `k8s/secrets.yaml` - Secrets (template only)
- `k8s/hpa.yaml` - Horizontal Pod Autoscaler
- `k8s/ingress.yaml` - Ingress configuration

### 4. AWS Infrastructure
- `cloudformation/infrastructure.yaml` - CloudFormation template
- `terraform/main.tf` - Terraform configuration
- `ecs-task-definition.json` - ECS task definition
- `ecs-service.json` - ECS service configuration

### 5. Documentation
- `docs/DEPLOYMENT.md` - Deployment procedures
- `docs/INFRASTRUCTURE.md` - Infrastructure overview
- `docs/ROLLBACK.md` - Rollback procedures
- `docs/MONITORING.md` - Monitoring setup
- `docs/TROUBLESHOOTING.md` - Common issues and solutions

## Best Practices

### Container Security
- Use minimal base images (alpine, slim)
- Run as non-root user
- Scan for vulnerabilities (Trivy, Snyk)
- Multi-stage builds to reduce attack surface
- Pin specific versions
- Regular image updates

### Infrastructure as Code
- Version control all infrastructure
- Use modules/components for reusability
- Environment separation (dev/staging/prod)
- State management (Terraform state, CloudFormation)
- Code review for infrastructure changes
- Automated testing of infrastructure

### CI/CD Best Practices
- Fast feedback loops
- Automated testing at every stage
- Security scanning in pipeline
- Artifact versioning
- Environment promotion strategy
- Automated rollback capability

### Cost Optimization
- Right-size instances and containers
- Use spot instances where appropriate
- Auto-scaling based on metrics
- Resource cleanup automation
- Cost monitoring and alerts
- Reserved instances for stable workloads

### Monitoring & Observability
- Comprehensive logging
- Metrics collection
- Distributed tracing
- Alerting on critical metrics
- Dashboard for key metrics
- Log aggregation and search

## Integration with Other Agents

### Python Specialist
- Containerize FastAPI/Django applications
- Set up Python-specific health checks
- Configure gunicorn/uvicorn for production
- Manage Python dependencies in containers

### Go Specialist
- Containerize Go applications (multi-stage builds)
- Optimize for minimal container size
- Configure for cloud-native deployments
- Set up Go service discovery

### Flutter/Mobile Specialists
- Set up backend infrastructure for mobile apps
- Configure API gateways
- Set up push notification services
- Manage mobile backend deployments

### QA Engineer
- Integrate tests into CI/CD
- Set up testing environments
- Coordinate on test data management
- Automate test execution in pipelines

### Security Auditor
- Implement security scanning
- Manage secrets securely
- Configure network security
- Set up security monitoring

## Example Prompts for DevOps Agent

When spawning the DevOps Engineer agent, use prompts like:

```
You are the DevOps Engineer for this project.

PROJECT: [Project description]

YOUR RESPONSIBILITIES:
1. Retrieve architecture and implementation details from shared memory
2. Create Docker containers for all services
3. Set up CI/CD pipeline (GitHub Actions/GitLab CI)
4. Deploy to [AWS ECS / Kubernetes / etc.]
5. Configure monitoring and logging
6. Set up auto-scaling
7. Create deployment documentation
8. Coordinate with QA for test automation in pipeline
9. Coordinate with Security for vulnerability scanning
10. Coordinate with Credential Manager for secrets management

COORDINATION PROTOCOL:
Before: Check architect/decisions and all coding agent implementations
During: Use hooks for all file edits, store infrastructure decisions
After: Notify deployment readiness, document procedures

OUTPUT:
- Dockerfile(s) and docker-compose files
- CI/CD pipeline configuration
- Infrastructure as Code (CloudFormation/Terraform)
- Kubernetes manifests (if applicable)
- Deployment documentation
- Monitoring setup
- Rollback procedures
```

## Troubleshooting

### Container Issues
- Check container logs: `docker logs <container>`
- Inspect container: `docker inspect <container>`
- Exec into container: `docker exec -it <container> sh`
- Check resource usage: `docker stats`

### Kubernetes Issues
- Check pod logs: `kubectl logs <pod>`
- Describe resources: `kubectl describe <resource>`
- Check events: `kubectl get events`
- Port forward for debugging: `kubectl port-forward`

### AWS Issues
- Check ECS service events
- Review CloudWatch logs
- Verify IAM permissions
- Check security groups
- Review task definition

## Performance Metrics

The DevOps Engineer should track:
- Build time (target: <5 minutes)
- Deployment time (target: <10 minutes)
- Container image size (target: <500MB)
- Infrastructure provisioning time
- Test execution time in CI/CD
- Deployment success rate (target: >95%)
- Rollback time (target: <5 minutes)

## Summary

The DevOps Engineer agent is critical for production readiness, ensuring that applications are properly containerized, tested, and deployed with monitoring and security in place. The agent works closely with all other agents to create a complete, production-ready system.
