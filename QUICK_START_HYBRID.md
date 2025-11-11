# Quick Start: Hybrid Routing with Claude Orchestra

## ✅ Pre-flight Check

Your hybrid routing is deployed and ready! All 5 models are available:
- `claude-opus-4` → Anthropic API (Chief Architect ONLY)
- `claude-sonnet-4-5` → Local Ollama qwen-quality-128k (Normal Claude Code - FREE!)
- `claude-3-5-sonnet` → Local Ollama qwen2.5-coder (Coding agents)
- `claude-3-haiku` → Local Ollama qwen-fast (Credential manager)
- `gpt-4` → Local Ollama qwen-quality-128k (QA/Security/Docs)

## 🚀 Configure Claude Code (One-Time Setup)

### Step 1: Logout from claude.ai

Exit this Claude Code session and run:
```bash
claude /logout
```

### Step 2: Start Claude Code with API Key Mode

Start Claude Code again:
```bash
cd /Users/brent/git/cc-army
claude
```

When prompted:
1. **"Would you like to authenticate with claude.ai?"** → Say **"No"**
2. **"Would you like to use an API key?"** → Say **"Yes"**

Claude Code will automatically use these environment variables from your `.zshrc`:
```bash
ANTHROPIC_BASE_URL="https://coder.visiquate.com"
ANTHROPIC_API_KEY="da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c"
```

### Step 3: Verify Connection

Once Claude Code starts, test with:
```
"List the available models"
```

You should see all 5 models listed.

## 🎯 Test Hybrid Routing

### Simple Test (No Army)

Try a simple request to verify routing:
```
"What is 2+2? Use model claude-opus-4"
```

This should route to the real Anthropic API (costs ~$0.02).

### Claude Orchestra Test (Full Pipeline)

Now deploy the army with a simple task:
```
"Use the Claude Orchestra to create a simple Python function that adds two numbers with tests"
```

Expected behavior:
- **Chief Architect** (1 agent) uses `claude-opus-4` → Routes to **Anthropic API** (💰 ~$0.02)
- **TDD Coding Agent** (1 agent) uses `claude-3-5-sonnet` → Routes to **Local Ollama** (🆓 Free)
- **QA Engineer** (1 agent) uses `gpt-4` → Routes to **Local Ollama** (🆓 Free)

Result: ~$0.02 cost instead of ~$0.06 (67% savings!)

**BONUS**: Your normal Claude Code conversations (like this one) are now **100% FREE** using qwen-quality-128k!

## 📊 Monitor Savings

Watch your savings in real-time:
```
https://coder.visiquate.com/dashboard/
```

Or via command line:
```bash
watch -n 5 'curl -s https://coder.visiquate.com/dashboard/api/stats | jq ".total_saved, .savings_percent"'
```

## 🔍 Verify Routing

Check which models are being used:
```bash
# View recent requests
ssh brent@192.168.9.123 'tail -50 /Users/brent/ccproxy/logs/litellm.log | grep -E "model|anthropic|ollama"'

# Calculate current savings
/Users/brent/git/cc-orchestra/scripts/calculate-ccproxy-savings.sh
```

## ⚠️ Important Notes

1. **Chief Architect uses real Claude API** - This is intentional for best quality strategic decisions
2. **All coding agents use local Ollama** - Free, fast, and high quality (qwen2.5-coder 32B)
3. **Target savings: 90-95%** - You pay only for the Architect, everything else is free
4. **Dashboard updates every 30 seconds** - Refresh to see latest stats

## 🎨 Example Workflows

### Build a REST API
```
"Use the Claude Orchestra to build a Python FastAPI app with JWT authentication"
```

Expected:
- Architect: $0.02 (design)
- 6-8 coding agents: $0.00 (implementation)
- Total: ~$0.02 vs ~$0.12 all-Anthropic (83% savings)

### Full-Stack App
```
"Use the Claude Orchestra to build a Flutter app with Go backend"
```

Expected:
- Architect: $0.02 (design)
- 10-12 agents: $0.00 (Flutter, Go, DevOps, QA, Security, Docs)
- Total: ~$0.02 vs ~$0.20 all-Anthropic (90% savings)

### Integration Project
```
"Use the Claude Orchestra to integrate Salesforce API with our database"
```

Expected:
- Architect: $0.02 (design)
- 5-7 agents: $0.00 (Salesforce expert, Python, QA, Security, Docs)
- Total: ~$0.02 vs ~$0.10 all-Anthropic (80% savings)

## 🐛 Troubleshooting

### "Authentication failed"
Ensure `.zshrc` variables are loaded:
```bash
source ~/.zshrc
echo $ANTHROPIC_BASE_URL
echo $ANTHROPIC_API_KEY
```

### "No models available"
Test ccproxy endpoint:
```bash
curl -s https://coder.visiquate.com/v1/models \
  -H "Authorization: Bearer da69328552c59c6d38cf788d8ae54318f6cfb2f3dd5ff82d0376e5f62260cd6c" | jq
```

### "Dashboard shows $0.00"
This is normal until you make some requests. Run a test request with `claude-opus-4` model.

### VPN/SSH Issues
If you need to access Mac mini:
1. Ensure VPN is connected
2. Test: `ping 192.168.9.123`
3. SSH: `ssh brent@192.168.9.123`

## 📚 Additional Resources

- **Full Documentation**: `/Users/brent/git/cc-orchestra/docs/`
- **Army Roster**: `/Users/brent/git/cc-orchestra/docs/ORCHESTRA_ROSTER_TDD.md`
- **Dashboard Guide**: `/Users/brent/git/cc-orchestra/docs/COST_TRACKING_DASHBOARD.md`
- **Deployment Status**: `/Users/brent/git/cc-orchestra/DEPLOYMENT_STATUS.md`

## 🎉 You're Ready!

Your hybrid routing is configured and ready to deliver 90-95% cost savings while maintaining high quality through:
- Real Claude Opus 4 for strategic architecture
- Local qwen2.5-coder 32B for coding (comparable to Claude Sonnet 3.5)
- Complete observability via the dashboard

Happy coding with the Claude Orchestra! 🚀
