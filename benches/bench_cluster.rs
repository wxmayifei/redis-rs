#![cfg(feature = "cluster")]
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use redis::cluster::cluster_pipe;

use support::*;

#[path = "../tests/support/mod.rs"]
mod support;

const PIPELINE_QUERIES: usize = 100;

fn bench_set_get_and_del(c: &mut Criterion, con: &mut redis::cluster::ClusterConnection) {
    let key = "test_key";

    let mut group = c.benchmark_group("cluster_basic");

    group.bench_function("set", |b| {
        b.iter(|| black_box(redis::cmd("SET").arg(key).arg(42).execute(con)))
    });

    group.bench_function("get", |b| {
        b.iter(|| black_box(redis::cmd("GET").arg(key).query::<isize>(con).unwrap()))
    });

    let mut set_and_del = || {
        redis::cmd("SET").arg(key).arg(42).execute(con);
        redis::cmd("DEL").arg(key).execute(con);
    };
    group.bench_function("set_and_del", |b| b.iter(|| black_box(set_and_del())));

    group.finish();
}

fn bench_pipeline(c: &mut Criterion, con: &mut redis::cluster::ClusterConnection) {
    let mut group = c.benchmark_group("cluster_pipeline");
    group.throughput(Throughput::Elements(PIPELINE_QUERIES as u64));

    let mut queries = Vec::new();
    for i in 0..PIPELINE_QUERIES {
        queries.push(format!("foo{}", i));
    }

    let build_pipeline = || {
        let mut pipe = cluster_pipe();
        for q in &queries {
            pipe.set(q, "bar").ignore();
        }
    };
    group.bench_function("build_pipeline", |b| b.iter(|| black_box(build_pipeline())));

    let mut pipe = cluster_pipe();
    for q in &queries {
        pipe.set(q, "bar").ignore();
    }
    group.bench_function("query_pipeline", |b| {
        b.iter(|| black_box(pipe.query::<()>(con).unwrap()))
    });

    group.finish();
}

fn bench_cluster_setup(c: &mut Criterion) {
    let cluster = TestClusterContext::new(6, 1);
    cluster.wait_for_cluster_up();

    let mut con = cluster.connection();
    bench_set_get_and_del(c, &mut con);
    bench_pipeline(c, &mut con);
}

criterion_group!(cluster_bench, bench_cluster_setup);
criterion_main!(cluster_bench);