use forge_engine::{
    read_graph, AngularRenderer, ReactRenderer, RenderContext, RenderDialect, RenderOptions,
    RendererAdapter, RiverpodAdapter, ScreenGraph,
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
    assert!(unit.code.contains("/>") || unit.code.contains("</Button>"));
}

#[test]
fn angular_renderer_emits_html() {
    let graph = load_ui_fixture("angular_basic");
    let renderer = AngularRenderer;
    let adapter = RiverpodAdapter::new();
    let options = RenderOptions {
        pretty: true,
        include_comments: false,
        dialect: RenderDialect::Html,
    };
    let ctx = RenderContext::new(0, &adapter, &options);

    let unit = renderer
        .render_tree(&graph.root, &ctx)
        .expect("render angular tree");

    assert!(unit.code.contains("<button"));
    assert!(unit.code.contains("ngClass=\"primary\""));
    assert!(unit.code.contains("[disabled]=\"isDisabled\""));
    assert!(unit.code.contains("ariaLabel=\"Submit\""));
}
