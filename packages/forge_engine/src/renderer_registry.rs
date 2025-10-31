use crate::renderer_adapter::{RenderDialect, RendererAdapter};
use crate::{
    angular_renderer::AngularRenderer, flutter_renderer::FlutterRenderer, manifest::ManifestKind,
    react_renderer::ReactRenderer,
};

/// Metadata describing a renderer implementation available to the engine.
pub struct RendererDescriptor {
    pub name: &'static str,
    pub dialect: RenderDialect,
    pub file_extension: &'static str,
    pub manifest_kind: Option<ManifestKind>,
    factory: fn() -> Box<dyn RendererAdapter>,
}

impl RendererDescriptor {
    /// Creates a new renderer instance from this descriptor.
    pub fn instantiate(&self) -> Box<dyn RendererAdapter> {
        (self.factory)()
    }
}

fn make_flutter_renderer() -> Box<dyn RendererAdapter> {
    Box::new(FlutterRenderer)
}

fn make_react_renderer() -> Box<dyn RendererAdapter> {
    Box::new(ReactRenderer)
}

fn make_angular_renderer() -> Box<dyn RendererAdapter> {
    Box::new(AngularRenderer)
}

static RENDERERS: &[RendererDescriptor] = &[
    RendererDescriptor {
        name: "flutter",
        dialect: RenderDialect::Dart,
        file_extension: "dart",
        manifest_kind: Some(ManifestKind::PubspecYaml),
        factory: make_flutter_renderer,
    },
    RendererDescriptor {
        name: "react",
        dialect: RenderDialect::Jsx,
        file_extension: "jsx",
        manifest_kind: Some(ManifestKind::PackageJson),
        factory: make_react_renderer,
    },
    RendererDescriptor {
        name: "angular",
        dialect: RenderDialect::Html,
        file_extension: "html",
        manifest_kind: Some(ManifestKind::PackageJson),
        factory: make_angular_renderer,
    },
];

/// Returns all renderer descriptors registered with the engine.
pub fn all_renderers() -> &'static [RendererDescriptor] {
    RENDERERS
}

/// Attempts to resolve a renderer descriptor by framework name.
pub fn get_renderer(name: &str) -> Option<&'static RendererDescriptor> {
    let needle = name.trim().to_lowercase();
    RENDERERS
        .iter()
        .find(|descriptor| descriptor.name == needle)
}

/// Returns the canonical list of framework names supported by the engine.
pub fn renderer_names() -> Vec<&'static str> {
    RENDERERS.iter().map(|descriptor| descriptor.name).collect()
}
