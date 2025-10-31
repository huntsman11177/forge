use criterion::{criterion_group, criterion_main, Criterion};
use forge_engine::{
    read_graph, ReactRenderer, RenderContext, RenderDialect, RenderOptions, RendererAdapter,
    RiverpodAdapter, ScreenGraph,
};
use std::path::Path;

fn load_fixture(name: &str) -> ScreenGraph {
    let path = Path::new("fixtures/ui").join(format!("{name}.json"));
    read_graph(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()))
}

fn bench_react(c: &mut Criterion) {
    let graph = load_fixture("react_basic");
    let renderer = ReactRenderer;
    let adapter = RiverpodAdapter::new();
    let options = RenderOptions {
        pretty: true,
        include_comments: false,
        dialect: RenderDialect::Jsx,
    };
    let ctx = RenderContext::new(0, &adapter, &options);

    c.bench_function("react_render_basic", |b| {
        b.iter(|| renderer.render_tree(&graph.root, &ctx))
    });
}

criterion_group!(benches, bench_react);
criterion_main!(benches);
