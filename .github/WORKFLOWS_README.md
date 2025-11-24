# GitHub Actions Workflows Documentation

This directory contains automated CI/CD workflows for the Claude Orchestra repository.

## Available Workflows

### 1. Linting & Testing (`test.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

**What it does:**
- Tests on Node.js 18.x, 20.x, and 21.x
- Validates JavaScript syntax
- Runs ESLint (if configured)
- Executes npm test suite
- Generates code coverage reports (if nyc is installed)
- Validates package.json structure
- Runs npm audit for security vulnerabilities

**Status Badge:**
```markdown
[![Linting & Testing](https://github.com/yourusername/claude-orchestra/actions/workflows/test.yml/badge.svg)](https://github.com/yourusername/claude-orchestra/actions/workflows/test.yml)
```

### 2. Documentation Validation (`docs-validation.yml`)

**Triggers:**
- Push to `main` or `develop` branches (when docs/**/*.md or *.md files change)
- Pull requests to `main` or `develop` branches (when docs/**/*.md or *.md files change)

**What it does:**
- Checks for broken markdown links
- Validates markdown syntax with markdownlint
- Extracts and validates Mermaid diagrams
- Checks for ASCII diagrams (suggests conversion to Mermaid)
- Validates internal links
- Verifies required documentation files exist

**Status Badge:**
```markdown
[![Documentation Validation](https://github.com/yourusername/claude-orchestra/actions/workflows/docs-validation.yml/badge.svg)](https://github.com/yourusername/claude-orchestra/actions/workflows/docs-validation.yml)
```

### 3. Security Scanning (`security.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches
- Scheduled: Weekly on Monday at 9 AM UTC

**What it does:**
- **Secret Detection**: Scans entire git history for exposed secrets using TruffleHog
- **Dependency Security**: Runs npm audit and checks for vulnerable dependencies
- **Code Security**: Performs CodeQL analysis for security vulnerabilities
- **File Security**: Checks file permissions, validates .gitignore, ensures no .env files committed

**Status Badge:**
```markdown
[![Security Scanning](https://github.com/yourusername/claude-orchestra/actions/workflows/security.yml/badge.svg)](https://github.com/yourusername/claude-orchestra/actions/workflows/security.yml)
```

### 4. Dependency Updates (`dependabot.yml`)

**Configuration file** (not a workflow, but automated by GitHub)

**Triggers:**
- Weekly on Monday at 9:00 AM EST
- Immediately on security vulnerability detection

**What it does:**
- Automatically checks for npm dependency updates
- Groups development and production dependencies
- Creates pull requests for updates
- Monitors GitHub Actions versions
- Follows conventional commit format

**Features:**
- Maximum 10 open PRs at once
- Labels: `dependencies`, `automated`
- Prefix: `chore(deps)` or `chore(deps-dev)`
- Automatic security updates

## Configuration Files

### `markdown-link-check-config.json`
Configuration for markdown link validation:
- Ignores localhost and example.com links
- Ignores GitHub blob URLs with commit hashes
- 20-second timeout for link checks
- Retry on 429 (rate limit) responses
- Accepts common redirect status codes

## Workflow Features

### Concurrency Control
All workflows use concurrency groups to automatically cancel outdated workflow runs when new commits are pushed. This saves CI/CD minutes and provides faster feedback.

### Permissions
Workflows follow the principle of least privilege:
- `contents: read` - Read repository contents
- `pull-requests: write` - Comment on PRs with results
- `security-events: write` - Upload security scan results (security workflow only)

### Summary Reports
All workflows generate GitHub Actions summary reports that appear in the workflow run UI, making it easy to see results at a glance.

## Setting Up in Your Repository

### Step 1: Update Repository URL
Replace `yourusername/claude-orchestra` in the badge URLs with your actual GitHub username and repository name.

### Step 2: Optional Secrets
Some features require GitHub secrets to be configured:

- `CODECOV_TOKEN` (optional): For uploading test coverage to Codecov
  - Sign up at https://codecov.io
  - Add your repository
  - Copy the token to GitHub Settings > Secrets > Actions

### Step 3: Enable GitHub Features
1. Go to repository Settings > Code security and analysis
2. Enable:
   - Dependabot alerts
   - Dependabot security updates
   - CodeQL analysis (if desired)

### Step 4: Add Status Badges to README
Copy the badge markdown from this file and add to your README.md:

```markdown
## Build Status

[![Linting & Testing](https://github.com/yourusername/claude-orchestra/actions/workflows/test.yml/badge.svg)](https://github.com/yourusername/claude-orchestra/actions/workflows/test.yml)
[![Documentation Validation](https://github.com/yourusername/claude-orchestra/actions/workflows/docs-validation.yml/badge.svg)](https://github.com/yourusername/claude-orchestra/actions/workflows/docs-validation.yml)
[![Security Scanning](https://github.com/yourusername/claude-orchestra/actions/workflows/security.yml/badge.svg)](https://github.com/yourusername/claude-orchestra/actions/workflows/security.yml)
```

## Customization

### Adding ESLint
The test workflow will automatically detect and run ESLint if you add a configuration file:

```bash
# Install ESLint
npm install --save-dev eslint

# Initialize configuration
npx eslint --init

# Or create .eslintrc.json manually
```

### Adding Test Coverage
Install nyc for code coverage:

```bash
npm install --save-dev nyc

# Add to package.json scripts:
"test:coverage": "nyc npm test"
```

### Customizing Dependabot
Edit `.github/dependabot.yml` to:
- Change update schedule
- Add reviewers
- Ignore specific packages
- Adjust grouping strategy

### Workflow Triggers
Modify the `on:` section in each workflow to customize when they run:

```yaml
on:
  push:
    branches: [ main, develop, staging ]  # Add more branches
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight
```

## Troubleshooting

### Workflow Not Running
- Check that the workflow file is in `.github/workflows/`
- Verify YAML syntax is valid
- Check repository permissions allow Actions

### TruffleHog Blocking Commits
If TruffleHog detects secrets:
1. Remove the secret from the commit history
2. Rotate the exposed credential
3. Use git-filter-repo or BFG Repo-Cleaner to clean history

### npm audit Failures
If critical vulnerabilities are found:
1. Run `npm audit fix` locally
2. For breaking changes: `npm audit fix --force` (carefully)
3. Create an issue if no fix is available
4. Consider temporarily allowing the vulnerability (not recommended)

### Mermaid Validation Failures
If Mermaid diagrams fail validation:
1. Test diagrams at https://mermaid.live
2. Check for syntax errors
3. Ensure proper code block formatting:
   ````markdown
   ```mermaid
   graph TD
       A --> B
   ```
   ````

## Monitoring Workflows

### Viewing Results
- Go to repository > Actions tab
- Click on a workflow run to see details
- Check the Summary tab for quick overview
- Review individual job logs for details

### Email Notifications
Configure in GitHub Settings > Notifications to receive emails when:
- Workflows fail on your commits
- Workflows fail on default branch

## Best Practices

1. **Review Dependabot PRs**: Don't auto-merge without testing
2. **Monitor Security Alerts**: Fix critical issues immediately
3. **Keep Workflows Updated**: GitHub Actions versions change frequently
4. **Test Locally First**: Run linters and tests before pushing
5. **Use Branch Protection**: Require workflow success before merging

## Contributing

When adding new workflows:
1. Test in a feature branch first
2. Use concurrency groups
3. Add proper permissions blocks
4. Generate summary reports
5. Document in this file

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Dependabot Documentation](https://docs.github.com/en/code-security/dependabot)
- [TruffleHog Documentation](https://github.com/trufflesecurity/trufflehog)
- [Markdown Link Check](https://github.com/tcort/markdown-link-check)
- [Markdownlint](https://github.com/DavidAnson/markdownlint)
