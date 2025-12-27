//! Performance Benchmarks
//!
//! Benchmarks for critical backend operations.
//! Run with: cargo bench --package backend-api

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::json;

/// Benchmark JSON serialization/deserialization
fn bench_json_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_operations");
    
    // Sample entity data
    let entity = json!({
        "id": Uuid::new_v4(),
        "name": "Test Contact",
        "email": "test@example.com",
        "phone": "+1234567890",
        "address": {
            "street": "123 Main St",
            "city": "New York",
            "zip": "10001"
        },
        "tags": ["hot-lead", "enterprise", "priority"],
        "metadata": {
            "source": "web",
            "campaign": "q4-2024",
            "score": 85
        }
    });
    
    // Serialize
    group.bench_function("serialize_entity", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(black_box(&entity));
        });
    });
    
    // Deserialize
    let json_str = serde_json::to_string(&entity).unwrap();
    group.bench_function("deserialize_entity", |b| {
        b.iter(|| {
            let _: serde_json::Value = serde_json::from_str(black_box(&json_str)).unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark UUID operations
fn bench_uuid_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid_operations");
    
    group.bench_function("uuid_v4_generate", |b| {
        b.iter(|| {
            black_box(Uuid::new_v4());
        });
    });
    
    group.bench_function("uuid_parse", |b| {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        b.iter(|| {
            black_box(Uuid::parse_str(uuid_str).unwrap());
        });
    });
    
    group.bench_function("uuid_to_string", |b| {
        let uuid = Uuid::new_v4();
        b.iter(|| {
            black_box(uuid.to_string());
        });
    });
    
    group.finish();
}

/// Benchmark HashMap operations (simulating in-memory cache)
fn bench_hashmap_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");
    
    // Pre-populate cache
    let mut cache: HashMap<Uuid, serde_json::Value> = HashMap::new();
    for _ in 0..10000 {
        cache.insert(Uuid::new_v4(), json!({"data": "cached_value"}));
    }
    
    let existing_key = *cache.keys().next().unwrap();
    
    group.bench_function("cache_lookup_hit", |b| {
        b.iter(|| {
            black_box(cache.get(black_box(&existing_key)));
        });
    });
    
    group.bench_function("cache_lookup_miss", |b| {
        let missing_key = Uuid::new_v4();
        b.iter(|| {
            black_box(cache.get(black_box(&missing_key)));
        });
    });
    
    group.bench_function("cache_insert", |b| {
        let mut cache_clone = cache.clone();
        b.iter(|| {
            cache_clone.insert(Uuid::new_v4(), json!({"data": "new_value"}));
        });
    });
    
    group.finish();
}

/// Benchmark event envelope processing
fn bench_event_processing(c: &mut Criterion) {
    use chrono::Utc;
    
    let mut group = c.benchmark_group("event_processing");
    
    // Create sample event
    let event_data = json!({
        "entity_id": Uuid::new_v4(),
        "aggregate_type": "contact",
        "event_type": "ContactCreated",
        "event_data": {
            "name": "New Contact",
            "email": "new@example.com"
        },
        "tenant_id": Uuid::new_v4(),
        "caused_by": Uuid::new_v4(),
        "occurred_at": Utc::now().to_rfc3339(),
        "version": 1
    });
    
    // Serialize event
    group.bench_function("event_serialize", |b| {
        b.iter(|| {
            let _ = serde_json::to_string(black_box(&event_data));
        });
    });
    
    // Deserialize event
    let event_str = serde_json::to_string(&event_data).unwrap();
    group.bench_function("event_deserialize", |b| {
        b.iter(|| {
            let _: serde_json::Value = serde_json::from_str(black_box(&event_str)).unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark base64 encoding (CRDT updates)
fn bench_base64_encoding(c: &mut Criterion) {
    use base64::{Engine, engine::general_purpose::STANDARD};
    
    let mut group = c.benchmark_group("base64_encoding");
    
    // Small payload (typical CRDT delta)
    let small_data = vec![0u8; 100];
    group.bench_with_input(BenchmarkId::new("encode", "100B"), &small_data, |b, data| {
        b.iter(|| {
            black_box(STANDARD.encode(data));
        });
    });
    
    // Medium payload
    let medium_data = vec![0u8; 10_000];
    group.bench_with_input(BenchmarkId::new("encode", "10KB"), &medium_data, |b, data| {
        b.iter(|| {
            black_box(STANDARD.encode(data));
        });
    });
    
    // Large payload (full document sync)
    let large_data = vec![0u8; 100_000];
    group.bench_with_input(BenchmarkId::new("encode", "100KB"), &large_data, |b, data| {
        b.iter(|| {
            black_box(STANDARD.encode(data));
        });
    });
    
    // Decode
    let encoded = STANDARD.encode(&medium_data);
    group.bench_function("decode_10KB", |b| {
        b.iter(|| {
            black_box(STANDARD.decode(black_box(&encoded)));
        });
    });
    
    group.finish();
}

/// Benchmark string operations for template processing
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");
    
    let template = "Hello {{name}}, your order {{order_id}} is ready for pickup at {{location}}.";
    let mut vars = HashMap::new();
    vars.insert("name", "John Doe");
    vars.insert("order_id", "ORD-12345");
    vars.insert("location", "Store #42");
    
    group.bench_function("template_replace", |b| {
        b.iter(|| {
            let mut result = template.to_string();
            for (key, value) in &vars {
                result = result.replace(&format!("{{{{{}}}}}", key), value);
            }
            black_box(result)
        });
    });
    
    group.bench_function("format_macro", |b| {
        b.iter(|| {
            black_box(format!(
                "Hello {}, your order {} is ready for pickup at {}.",
                vars["name"], vars["order_id"], vars["location"]
            ))
        });
    });
    
    group.finish();
}

/// Benchmark condition evaluation (workflow nodes)
fn bench_condition_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("condition_evaluation");
    
    let record = json!({
        "status": "active",
        "score": 85,
        "tags": ["premium", "enterprise"],
        "amount": 5000.00
    });
    
    // Simple field comparison
    group.bench_function("field_equals", |b| {
        b.iter(|| {
            let status = record.get("status").and_then(|v| v.as_str());
            black_box(status == Some("active"))
        });
    });
    
    // Numeric comparison
    group.bench_function("numeric_greater_than", |b| {
        b.iter(|| {
            let score = record.get("score").and_then(|v| v.as_i64()).unwrap_or(0);
            black_box(score > 80)
        });
    });
    
    // Array contains
    group.bench_function("array_contains", |b| {
        b.iter(|| {
            let tags = record.get("tags").and_then(|v| v.as_array());
            black_box(tags.map(|arr| arr.iter().any(|t| t.as_str() == Some("premium"))).unwrap_or(false))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_json_operations,
    bench_uuid_operations,
    bench_hashmap_cache,
    bench_event_processing,
    bench_base64_encoding,
    bench_string_operations,
    bench_condition_evaluation,
);

criterion_main!(benches);
