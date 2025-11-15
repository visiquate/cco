# CCO Test Suite Documentation

## Overview

This comprehensive test suite validates the Claude Cache Orchestrator (CCO) implementation across all layers: caching, routing, analytics, and HTTP proxy functionality. The test suite follows a TDD (Test-Driven Development) approach with clear separation of concerns.

## Test Files Structure

### 1. `cache_tests.rs` - Moka Cache Layer Tests
**Purpose**: Validate cache behavior, key generation, eviction, and metrics

**Test Coverage**:
- **Cache Hit/Miss Tests** (5 tests)
  - `test_cache_hit()` - Verify cache stores and retrieves responses
  - `test_cache_miss()` - Verify non-existent keys return None
  - `test_cache_hit_rate_calculation()` - Verify hit rate formula
  - `test_cache_hit_rate_100_percent()` - All hits scenario
  - `test_cache_hit_rate_0_percent()` - All misses scenario

- **Cache Key Generation Tests** (6 tests)
  - `test_cache_key_generation_consistency()` - Same inputs = same key
  - `test_cache_key_uniqueness()` - Different inputs = different keys
  - `test_cache_key_model_specificity()` - Model affects key
  - `test_cache_key_temperature_sensitivity()` - Temperature affects key
  - `test_cache_key_max_tokens_sensitivity()` - Max tokens affects key
  - `test_cache_key_length()` - SHA256 produces 64-char hex

- **Concurrent Access Tests** (2 tests)
  - `test_concurrent_cache_access()` - 100 concurrent reads
  - `test_concurrent_insert_and_read()` - Mix of inserts and reads

- **Model Isolation Tests** (1 test)
  - `test_cache_isolation_by_model()` - Different models cached separately

- **Eviction & Cleanup Tests** (3 tests)
  - `test_cache_fifo_behavior()` - FIFO ordering preserved
  - `test_cache_duplicate_keys()` - Latest value on duplicate key
  - `test_cache_clear()` - Clear operation works

- **Metrics Edge Cases** (3 tests)
  - `test_metrics_on_empty_cache()` - Empty cache hit rate = 0%
  - `test_metrics_all_hits()` - 100% hit rate validation
  - `test_metrics_all_misses()` - 0% hit rate validation

**Total Tests**: 21
**Success Criteria**: All tests pass, cache operations < 1ms latency

---

### 2. `router_tests.rs` - Multi-Model Routing Tests
**Purpose**: Validate routing rules, cost calculation, and multi-provider support

**Test Coverage**:
- **Model Routing Tests** (4 tests)
  - `test_route_claude_opus_model()` - Claude models → Anthropic
  - `test_route_claude_sonnet_model()` - Sonnet routing
  - `test_route_openai_gpt4_model()` - OpenAI models → OpenAI
  - `test_route_ollama_model()` - Ollama models → localhost:11434

- **Provider Endpoint Tests** (3 tests)
  - `test_anthropic_endpoint()` - Correct Anthropic URL
  - `test_openai_endpoint()` - Correct OpenAI URL
  - `test_ollama_endpoint()` - Correct Ollama URL

- **Cost Calculation Tests** (5 tests)
  - `test_cost_calculation_claude_opus()` - $52.50 for 1M input + 500K output
  - `test_cost_calculation_claude_sonnet()` - $10.50 for same tokens
  - `test_cost_calculation_openai_gpt4()` - $60.00 for same tokens
  - `test_cost_calculation_ollama_free()` - $0.00 for self-hosted
  - `test_cost_calculation_small_tokens()` - Fractional token costs

- **Cache Savings Tests** (2 tests)
  - `test_proxy_cache_savings_claude_opus()` - $52.50 savings on cache hit
  - `test_proxy_cache_savings_claude_sonnet()` - $10.50 savings on cache hit

- **Claude Prompt Cache Tests** (2 tests)
  - `test_claude_cache_savings_with_90_percent_cached()` - 90% cache hit
  - `test_claude_cache_savings_with_50_percent_cached()` - 50% cache hit

- **Self-Hosted Savings Tests** (3 tests)
  - `test_self_hosted_vs_claude_opus_savings()` - 100% savings with Ollama
  - `test_self_hosted_vs_claude_sonnet_savings()` - Sonnet comparison
  - `test_self_hosted_vs_openai_savings()` - GPT-4 comparison

- **Cumulative Cost Tests** (2 tests)
  - `test_monthly_cost_claude_opus_without_cache()` - ~$180K/month
  - `test_monthly_savings_with_50_percent_cache_hit_rate()` - ~$78.75K/month

- **Pricing Edge Cases** (2 tests)
  - `test_pricing_unknown_model()` - Unknown model has no pricing
  - `test_cost_zero_tokens()` - Zero tokens = $0 cost

**Total Tests**: 24
**Success Criteria**: All routing and cost calculations accurate

---

### 3. `analytics_tests.rs` - Analytics & Cost Tracking Tests
**Purpose**: Validate analytics recording, aggregation, and cost tracking

**Test Coverage**:
- **Basic Recording Tests** (3 tests)
  - `test_record_cache_miss()` - Record miss correctly
  - `test_record_cache_hit()` - Record hit with savings
  - `test_record_multiple_calls()` - Bulk recording

- **Hit Rate Tests** (5 tests)
  - `test_cache_hit_rate_calculation()` - 70% hit rate validation
  - `test_cache_hit_rate_100_percent()` - All hits = 100%
  - `test_cache_hit_rate_0_percent()` - All misses = 0%
  - `test_cache_hit_rate_empty()` - Empty analytics = 0%

- **Savings Tracking Tests** (4 tests)
  - `test_total_savings_single_cache_hit()` - $52.50 savings
  - `test_total_savings_multiple_cache_hits()` - $525 for 10 hits
  - `test_total_savings_mixed_hits_and_misses()` - $367.50 savings
  - `test_total_savings_zero()` - No savings without hits

- **Cost Tracking Tests** (3 tests)
  - `test_total_actual_cost_no_cache()` - $262.50 for 5 requests
  - `test_total_would_be_cost()` - Would-be cost calculation
  - `test_cost_savings_efficiency()` - Actual + Savings = Would-be

- **Per-Model Analytics Tests** (2 tests)
  - `test_savings_by_model_single_model()` - Single model breakdown
  - `test_savings_by_model_multiple_models()` - Multi-model comparison

- **Model Metrics Tests** (1 test)
  - `test_metrics_by_model()` - Per-model hit rates and costs

- **Concurrent Recording Tests** (1 test)
  - `test_concurrent_recording()` - 100 concurrent inserts

- **Reset Tests** (1 test)
  - `test_clear_analytics()` - Clear operation

**Total Tests**: 20
**Success Criteria**: Accurate cost tracking and aggregation

---

### 4. `proxy_tests.rs` - HTTP Proxy Tests
**Purpose**: Validate cache hit/miss paths and request handling

**Test Coverage**:
- **Cache Hit Path Tests** (3 tests)
  - `test_proxy_cache_hit_path()` - Hit returns from cache, no API call
  - `test_proxy_cache_miss_path()` - Miss triggers API call
  - `test_proxy_cache_isolation_by_model()` - Different models cached separately

- **Request Parameter Tests** (3 tests)
  - `test_proxy_cache_sensitivity_to_temperature()` - Temperature affects cache
  - `test_proxy_cache_sensitivity_to_max_tokens()` - Max tokens affects cache
  - `test_proxy_cache_sensitivity_to_prompt()` - Prompt content affects cache

- **Multiple Requests Tests** (2 tests)
  - `test_proxy_mixed_cache_hits_and_misses()` - Mixed workflow
  - `test_proxy_many_requests()` - 100 requests with 50 unique

- **Response Validity Tests** (2 tests)
  - `test_proxy_response_has_required_fields()` - Response structure valid
  - `test_proxy_cache_hit_response_matches_original()` - Cache match verification

- **Concurrent Request Tests** (1 test)
  - `test_proxy_concurrent_requests()` - 100 concurrent requests, 25 unique

- **Reset Tests** (1 test)
  - `test_proxy_cache_clear()` - Cache clear operation

**Total Tests**: 12
**Success Criteria**: Cache hit/miss paths work correctly

---

### 5. `integration_tests.rs` - End-to-End Integration Tests
**Purpose**: Validate complete system flow with cache, routing, and analytics

**Test Coverage**:
- **Full Request Flow Tests** (2 tests)
  - `test_full_request_flow_cache_miss_then_hit()` - Complete E2E flow
  - `test_full_request_flow_multiple_requests()` - Multiple unique/duplicate requests

- **Multi-Model Routing Tests** (4 tests)
  - `test_multi_model_routing_anthropic()` - Claude routing works
  - `test_multi_model_routing_openai()` - OpenAI routing works
  - `test_multi_model_routing_ollama()` - Ollama routing works
  - `test_multi_model_routing_unknown_model()` - Error handling

- **Multi-Model Cost Tests** (1 test)
  - `test_multi_model_different_costs()` - Cost differences validated

- **Analytics Persistence Tests** (5 tests)
  - `test_analytics_tracks_all_requests()` - All requests recorded
  - `test_analytics_tracks_cache_hits()` - Hits tracked correctly
  - `test_analytics_calculates_total_savings()` - Savings calculated
  - `test_analytics_tracks_cost()` - Costs tracked
  - `test_analytics_cost_breakdown_by_model()` - Per-model breakdown

- **Error Handling Tests** (1 test)
  - `test_error_unknown_model_handled_gracefully()` - Graceful errors

- **Concurrent Tests** (1 test)
  - `test_concurrent_multimodel_requests()` - 9 concurrent tasks, 3 models

- **Realistic Workflow Tests** (1 test)
  - `test_realistic_daily_workflow()` - 180 requests, multiple models

**Total Tests**: 15
**Success Criteria**: Complete E2E workflows validated

---

## Test Fixtures

### `mock_api_response.json`
Contains sample API responses from all providers:
- Claude Opus, Sonnet responses with token usage
- GPT-4, GPT-3.5 responses
- Ollama Llama3 responses
- Streaming response chunks
- Error responses (auth, rate limit)
- Cache hit responses with cache metadata

### `test_config.json`
Contains test configuration:
- **Router Configuration**: All routing rules, endpoints, timeouts
- **Pricing Configuration**: Per-model pricing for all providers
- **Cache Configuration**: Capacity, TTL, eviction policy
- **Test Scenarios**: 5 different usage scenarios
- **Test Data**: 10 sample prompts, 4 token profiles
- **Performance Targets**: Expected latencies and throughput

## Running Tests

### Run All Tests
```bash
cargo test --test cache_tests
cargo test --test router_tests
cargo test --test analytics_tests
cargo test --test proxy_tests
cargo test --test integration_tests
```

### Run Specific Test Category
```bash
cargo test --test cache_tests test_cache_hit
cargo test --test router_tests test_cost_calculation
cargo test --test analytics_tests test_total_savings
```

### Run with Output
```bash
cargo test -- --nocapture
```

### Run Specific Thread Count
```bash
cargo test -- --test-threads=1
```

### Generate Coverage Report
```bash
cargo tarpaulin --out Html
```

## Test Metrics & Performance

### Expected Test Execution Time
- Cache tests: 2-3 seconds (21 tests)
- Router tests: 1-2 seconds (24 tests)
- Analytics tests: 2-3 seconds (20 tests)
- Proxy tests: 3-4 seconds (12 tests)
- Integration tests: 5-6 seconds (15 tests)
- **Total**: ~15-20 seconds for full suite

### Expected Coverage
- Cache module: > 90%
- Router module: > 85%
- Analytics module: > 85%
- Proxy module: > 80%
- **Overall**: > 85% code coverage

### Performance Assertions
- Cache lookup: < 1ms
- Cache hit response: < 5ms
- Cache miss response: < 5000ms (API latency)
- Concurrent operations: 100+ operations without error

## Test Data Strategy

### Prompts
- 10 unique prompts covering different topics
- Each used multiple times to generate cache hits

### Token Profiles
- Small: 100 input, 50 output tokens
- Medium: 1,000 input, 500 output tokens
- Large: 10,000 input, 5,000 output tokens
- XLarge: 100,000 input, 50,000 output tokens

### Models Tested
- Claude: opus-4, sonnet-3.5 (Anthropic)
- OpenAI: gpt-4, gpt-3.5-turbo
- Ollama: llama3-70b, mistral (self-hosted)
- Unknown models: error handling validation

## Success Criteria

### Unit Tests (Cache, Router, Analytics, Proxy)
✅ All tests pass
✅ > 80% code coverage per module
✅ No flaky tests (100% deterministic)
✅ Fast execution (< 2s per test file)
✅ Clear assertions with descriptive messages

### Integration Tests
✅ All E2E scenarios pass
✅ Multi-model workflows validated
✅ Analytics correctly tracks all operations
✅ Cost calculations accurate across all providers
✅ Concurrent operations stable

### Performance
✅ Cache operations: < 1ms
✅ Hit rate calculations: < 10µs
✅ Cost calculations: < 100µs
✅ Concurrent requests: > 100 ops/sec

## Common Test Failures & Debugging

### Cache Hit Rate Calculation Off
- Check: Hits + Misses count is correct
- Check: Formula uses f64, not u64 division
- Check: Hit rate range is 0-100%

### Cost Calculation Mismatch
- Check: Token counts are in millions (divide by 1,000,000)
- Check: Model pricing is loaded correctly
- Check: Cache read costs only apply to Claude models

### Concurrency Issues
- Check: Use tokio::spawn for async tasks
- Check: Arc<Mutex<>> for shared state
- Check: Release locks before long operations

### Analytics Not Recording
- Check: Records are being inserted before retrieval
- Check: Cloning Arc references for concurrent access
- Check: Clear is called before running new tests

## Test Maintenance

### Adding New Tests
1. Identify test category (cache, router, analytics, proxy, integration)
2. Add to appropriate test file
3. Use descriptive test name: `test_[component]_[scenario]`
4. Include arrange-act-assert pattern
5. Add doc comments explaining test purpose

### Test Dependencies
- `tokio` for async runtime
- `sha2` for cache key generation
- `std::sync::Arc` for shared state
- `tokio::sync::Mutex` for concurrent access

## Future Test Enhancements

### Performance Profiling
- Benchmark cache operations
- Measure memory footprint
- Profile concurrent access patterns

### Chaos Testing
- Random failures at different stages
- Network latency injection
- Token limit variations

### Load Testing
- Sustained high-volume requests
- Memory leak detection
- Connection pool exhaustion scenarios

### Security Testing
- API key validation
- Input sanitization
- Token limit enforcement

## References

- Test Framework: Tokio + standard Rust testing
- Mocking: Custom mock implementations
- Fixtures: JSON configuration files
- Documentation: This file + inline comments

---

**Last Updated**: 2024-11-15
**Test Suite Version**: 1.0.0
**Total Tests**: 92
**Status**: Ready for Implementation
