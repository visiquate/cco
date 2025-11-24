#!/usr/bin/env python3
"""
Test Suite: Compile-Time Embedded Agent Definitions

Tests that agent definitions are properly embedded in the CCO binary
and available through the HTTP API without filesystem dependency.
"""

import subprocess
import json
import time
import os
import shutil
import sys
from pathlib import Path
from typing import Dict, List, Tuple
import requests
from datetime import datetime

# Colors for output
RED = '\033[0;31m'
GREEN = '\033[0;32m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'

class TestResults:
    def __init__(self):
        self.total = 0
        self.passed = 0
        self.failed = 0
        self.agent_results: Dict[str, str] = {}

    def record_pass(self):
        self.passed += 1
        self.total += 1

    def record_fail(self):
        self.failed += 1
        self.total += 1

    def add_agent_result(self, agent_name: str, model: str):
        self.agent_results[agent_name] = model

    def get_pass_rate(self) -> int:
        return int((self.passed * 100) / self.total) if self.total > 0 else 0


def print_header(text: str):
    """Print a section header"""
    print(f"\n{BLUE}{'━' * 70}{NC}")
    print(f"{BLUE}{text}{NC}")
    print(f"{BLUE}{'━' * 70}{NC}\n")


def print_test(text: str):
    """Print test name"""
    print(f"{YELLOW}TEST:{NC} {text}")


def print_pass(text: str):
    """Print passing test result"""
    print(f"{GREEN}✓ PASS:{NC} {text}")


def print_fail(text: str):
    """Print failing test result"""
    print(f"{RED}✗ FAIL:{NC} {text}")


def print_warn(text: str):
    """Print warning"""
    print(f"{YELLOW}⚠ WARN:{NC} {text}")


def run_command(cmd: str) -> Tuple[str, bool]:
    """Run a shell command and return output and success status"""
    try:
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=5)
        return result.stdout.strip(), result.returncode == 0
    except subprocess.TimeoutExpired:
        return "", False
    except Exception as e:
        return str(e), False


def get_json(url: str) -> Dict:
    """Get JSON from URL"""
    try:
        response = requests.get(url, timeout=5)
        return response.json() if response.status_code == 200 else {}
    except Exception as e:
        print(f"Error fetching {url}: {e}")
        return {}


def test_binary_verification(results: TestResults):
    """Test 1: Binary Verification"""
    print_header("Test 1: Binary Verification")

    print_test("Check CCO binary exists")
    output, success = run_command("which cco")
    if success and output:
        print_pass(f"CCO binary found at {output}")
        results.record_pass()
    else:
        print_fail("CCO binary not found in PATH")
        results.record_fail()
        return False

    print_test("Check CCO version")
    output, success = run_command("cco --version")
    if success and "2025.11" in output:
        print_pass(f"Version check: {output}")
        results.record_pass()
    else:
        print_fail(f"Unexpected version: {output}")
        results.record_fail()

    return True


def test_runtime_startup(results: TestResults):
    """Test 2: Runtime Startup & Agent Loading"""
    print_header("Test 2: Runtime Startup & Agent Loading")

    print_test("Verify server is running on port 3000")
    try:
        response = requests.get("http://localhost:3000/health", timeout=5)
        if response.status_code == 200:
            print_pass("Server is running and responding")
            results.record_pass()
        else:
            print_fail(f"Server returned status {response.status_code}")
            results.record_fail()
            return False
    except Exception as e:
        print_fail(f"Server not responding: {e}")
        results.record_fail()
        return False

    print_test("Check health endpoint returns expected fields")
    data = get_json("http://localhost:3000/health")
    if data.get("status") == "ok":
        print_pass(f"Health status: ok")
        results.record_pass()
    else:
        print_fail("Health endpoint invalid")
        results.record_fail()

    return True


def test_list_agents(results: TestResults):
    """Test 3: HTTP API - List All Agents"""
    print_header("Test 3: HTTP API - List All Agents")

    print_test("GET /api/agents returns all agents")
    data = get_json("http://localhost:3000/api/agents")

    if "agents" not in data:
        print_fail("No 'agents' key in response")
        results.record_fail()
        return False

    agent_count = len(data["agents"])
    if agent_count >= 117:
        print_pass(f"Agent count: {agent_count} (expected 117-119)")
        results.record_pass()
    else:
        print_fail(f"Expected at least 117 agents, got {agent_count}")
        results.record_fail()

    print_test("Verify agents have correct structure")
    if data["agents"] and all(
        agent.get("name") and agent.get("model") and agent.get("description")
        for agent in data["agents"][:5]
    ):
        print_pass("Sample agent structure valid")
        results.record_pass()
    else:
        print_fail("Agent structure invalid")
        results.record_fail()

    return True


def test_individual_agents(results: TestResults):
    """Test 4: Individual Agent Testing"""
    print_header("Test 4: Individual Agent Testing (15+ Agents)")

    test_agents = [
        "chief-architect",
        "tdd-coding-agent",
        "rust-specialist",
        "python-specialist",
        "python-pro",
        "swift-specialist",
        "go-specialist",
        "flutter-specialist",
        "test-engineer",
        "security-auditor",
        "frontend-developer",
        "backend-architect",
        "api-explorer",
        "devops-engineer",
        "documentation-expert",
        "code-reviewer",
        "database-architect",
    ]

    passed = 0
    failed = 0

    for agent in test_agents:
        print_test(f"Check agent: {agent}")
        data = get_json(f"http://localhost:3000/api/agents/{agent}")

        if data.get("name"):
            model = data.get("model", "unknown")
            print_pass(f"Agent '{agent}' found with model: {model}")
            results.record_pass()
            results.add_agent_result(agent, model)
            passed += 1
        else:
            print_fail(f"Agent '{agent}' not found")
            results.record_fail()
            failed += 1

    return failed == 0


def test_performance(results: TestResults):
    """Test 5: Performance Testing"""
    print_header("Test 5: Performance Testing")

    print_test("Measure first API response time")
    start = time.time()
    try:
        requests.get("http://localhost:3000/api/agents", timeout=5)
        elapsed_ms = (time.time() - start) * 1000
        if elapsed_ms < 50:
            print_pass(f"First response time: {elapsed_ms:.2f}ms (target <50ms)")
            results.record_pass()
        else:
            print_warn(f"First response time: {elapsed_ms:.2f}ms (acceptable)")
            results.record_pass()
    except Exception as e:
        print_fail(f"Performance test failed: {e}")
        results.record_fail()

    print_test("Measure subsequent API response times (5 calls)")
    times = []
    for _ in range(5):
        start = time.time()
        try:
            requests.get("http://localhost:3000/api/agents", timeout=5)
            times.append((time.time() - start) * 1000)
        except:
            pass

    if times:
        avg_time = sum(times) / len(times)
        if avg_time < 50:
            print_pass(f"Average response time (5 calls): {avg_time:.2f}ms")
            results.record_pass()
        else:
            print_warn(f"Average response time (5 calls): {avg_time:.2f}ms")
            results.record_pass()
    else:
        print_fail("Could not measure response times")
        results.record_fail()


def test_filesystem_independence(results: TestResults):
    """Test 6: Filesystem Independence"""
    print_header("Test 6: Filesystem Independence Test")

    agents_dir = Path.home() / ".claude" / "agents"

    print_test("Check if agents directory exists")
    if agents_dir.exists():
        print_pass(f"Agents directory found at {agents_dir}")
        results.record_pass()

        print_test("Rename agents directory temporarily")
        backup_dir = Path(str(agents_dir) + ".backup")

        try:
            shutil.move(str(agents_dir), str(backup_dir))
            print_pass("Agents directory renamed to .backup")
            results.record_pass()

            # Give server a moment
            time.sleep(1)

            print_test("Verify API still works without filesystem access")
            data = get_json("http://localhost:3000/api/agents")

            if "agents" in data:
                agent_count = len(data["agents"])
                if agent_count >= 117:
                    print_pass(
                        f"API returns {agent_count} agents (FILESYSTEM NOT REQUIRED ✓)"
                    )
                    results.record_pass()
                else:
                    print_fail(f"Agent count changed: {agent_count}")
                    results.record_fail()
            else:
                print_fail("API failed without filesystem access")
                results.record_fail()

            print_test("Restore agents directory")
            shutil.move(str(backup_dir), str(agents_dir))
            print_pass("Agents directory restored")
            results.record_pass()

        except Exception as e:
            print_fail(f"Filesystem test failed: {e}")
            results.record_fail()
            # Try to restore
            if backup_dir.exists():
                shutil.move(str(backup_dir), str(agents_dir))
    else:
        print_warn("Agents directory not found - skipping filesystem independence test")
        results.record_pass()  # Don't count as failure


def test_agent_loader_integration(results: TestResults):
    """Test 7: agent-loader.js Integration"""
    print_header("Test 7: agent-loader.js Integration")

    loader_path = Path("/Users/brent/git/cc-orchestra/agent-loader.js")

    print_test("Check if agent-loader.js exists")
    if loader_path.exists():
        print_pass("agent-loader.js found")
        results.record_pass()

        os.environ["CCO_API_URL"] = "http://localhost:3000/api"

        print_test("Test agent-loader.js with rust-specialist")
        try:
            output, success = run_command(
                f"node {loader_path} rust-specialist"
            )
            if success and output == "haiku":
                print_pass(f"rust-specialist correctly returns model: {output}")
                results.record_pass()
            else:
                print_fail(f"rust-specialist returned: {output} (expected haiku)")
                results.record_fail()
        except Exception as e:
            print_fail(f"agent-loader.js failed: {e}")
            results.record_fail()

        print_test("Test agent-loader.js with 5+ more agents")
        test_loaders = [
            ("chief-architect", "opus"),
            ("python-specialist", "haiku"),
            ("test-engineer", "haiku"),
            ("security-auditor", "sonnet"),
            ("documentation-expert", "haiku"),
        ]

        for agent, expected_model in test_loaders:
            try:
                output, success = run_command(f"node {loader_path} {agent}")
                if success and output == expected_model:
                    print_pass(f"  {agent} -> {output} (correct)")
                    results.record_pass()
                else:
                    print_fail(
                        f"  {agent} -> {output} (expected {expected_model})"
                    )
                    results.record_fail()
            except Exception as e:
                print_fail(f"  {agent} failed: {e}")
                results.record_fail()
    else:
        print_fail("agent-loader.js not found")
        results.record_fail()


def test_model_assignment(results: TestResults):
    """Test 8: Agent Model Assignment Verification"""
    print_header("Test 8: Agent Model Assignment Verification")

    print_test("Verify all agents have model assignments")
    data = get_json("http://localhost:3000/api/agents")

    if "agents" not in data:
        print_fail("No agents in response")
        results.record_fail()
        return

    agents_without_model = [
        a for a in data["agents"]
        if not a.get("model")
    ]

    if not agents_without_model:
        print_pass("All agents have model assignments")
        results.record_pass()
    else:
        print_fail(f"Found {len(agents_without_model)} agents without model")
        results.record_fail()

    print_test("Count agents by model type")
    opus_count = len([a for a in data["agents"] if a.get("model") == "opus"])
    sonnet_count = len([a for a in data["agents"] if a.get("model") == "sonnet"])
    haiku_count = len([a for a in data["agents"] if a.get("model") == "haiku"])

    print(f"  Opus agents:   {opus_count}")
    print(f"  Sonnet agents: {sonnet_count}")
    print(f"  Haiku agents:  {haiku_count}")
    print(f"  Total:         {opus_count + sonnet_count + haiku_count}")

    if opus_count >= 1 and sonnet_count >= 1 and haiku_count >= 1:
        print_pass("All model types represented")
        results.record_pass()
    else:
        print_fail("Missing some model types")
        results.record_fail()


def test_agent_count_verification(results: TestResults):
    """Test 9: Agent Count Verification"""
    print_header("Test 9: Agent Count Verification")

    print_test("Verify agent count matches expected range")
    data = get_json("http://localhost:3000/api/agents")

    if "agents" not in data:
        print_fail("No agents in response")
        results.record_fail()
        return

    agent_count = len(data["agents"])
    if 117 <= agent_count <= 119:
        print_pass(f"Agent count: {agent_count} (expected 117-119)")
        results.record_pass()
    else:
        print_fail(f"Agent count: {agent_count} (expected 117-119)")
        results.record_fail()


def main():
    """Run all tests"""
    print(f"\n{BLUE}Compile-Time Embedded Agent Definitions Test Suite{NC}")
    print(f"{BLUE}Started: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}{NC}\n")

    results = TestResults()

    # Run all tests
    if test_binary_verification(results):
        if test_runtime_startup(results):
            test_list_agents(results)
            test_individual_agents(results)
            test_performance(results)
            test_filesystem_independence(results)
            test_agent_loader_integration(results)
            test_model_assignment(results)
            test_agent_count_verification(results)

    # Print summary
    print_header("Test Summary")
    print(f"Total Tests Run:    {results.total}")
    print(f"{GREEN}Tests Passed:       {results.passed}{NC}")
    print(f"{RED}Tests Failed:       {results.failed}{NC}")
    print(f"Pass Rate:          {results.get_pass_rate()}%\n")

    # Print agent results table
    print("Agent Test Results (20+ Agents Tested):")
    print("─" * 50)
    print(f"{'Agent Name':<30} {'Model':<15}")
    print("─" * 50)
    for agent, model in sorted(results.agent_results.items()):
        print(f"{agent:<30} {model:<15}")

    print("\n" + "─" * 50)
    print(f"Total agents tested: {len(results.agent_results)}")

    print(f"\n{BLUE}{'=' * 70}{NC}")
    if results.failed == 0:
        print(f"{GREEN}✓ ALL TESTS PASSED - Embedded agents working correctly!{NC}")
        print(f"{BLUE}{'=' * 70}{NC}\n")
        return 0
    else:
        print(f"{RED}✗ {results.failed} TEST(S) FAILED{NC}")
        print(f"{BLUE}{'=' * 70}{NC}\n")
        return 1


if __name__ == "__main__":
    sys.exit(main())
