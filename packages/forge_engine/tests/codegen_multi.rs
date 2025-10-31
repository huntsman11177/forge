use forge_engine::{
    react_renderer::ReactRenderer,
    read_graph,
    renderer_adapter::{RenderContext, RenderDialect, RenderOptions},
    state_adapter::RiverpodAdapter,
    ScreenGraph,
};
use std::path::Path;

fn load_ui_fixture(file_stem: &str) -> ScreenGraph {
    let path = Path::new("fixtures/ui").join(format!("{file_stem}.json"));
    read_graph(&path)
        .unwrap_or_else(|err| panic!("failed to load fixture {}: {err}", path.display()))
}

#[test]
fn react_renderer_emits_jsx() {
    let graph = load_ui_fixture("react_basic");
    let renderer = ReactRenderer;
    let adapter = RiverpodAdapter::new();
    let options = RenderOptions {
        pretty: true,
        include_comments: false,
        dialect: RenderDialect::Jsx,
    };
    let ctx = RenderContext::new(0, &adapter, &options);

    let unit = renderer
        .render_tree(&graph.root, &ctx)
        .expect("render react tree");

    assert!(unit.code.contains("<Button"));
    assert!(unit.code.contains("text=\"Click Me\""));
    assert!(unit.code.contains("</Button>"));
}
