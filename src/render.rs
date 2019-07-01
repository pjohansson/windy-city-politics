use amethyst::{
    ecs::{ReadExpect, Resources, SystemData},
    renderer::{
        pass::{DrawDebugLinesDesc, DrawFlat2DTransparentDesc},
        types::DefaultBackend,
        Factory, Format, GraphBuilder, GraphCreator, Kind, RenderGroupDesc, SubpassBuilder,
    },
    ui::DrawUiDesc,
    window::{ScreenDimensions, Window},
};

// This graph structure is used for creating a proper `RenderGraph` for rendering.
// A `RenderGraph` can be thought of as the stages during a render pass. In our case,
// we are only executing one subpass (DrawFlat2D, or the sprite pass). This graph
// also needs to be rebuilt whenever the window is resized, so the boilerplate code
// for that operation is also here.
#[derive(Default)]
pub struct ExampleGraph {
    dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for ExampleGraph {
    // This trait method reports to the renderer if the graph must be rebuilt, usually
    // because the window has been resized. This implementation checks the screen size
    // and returns `true` if it has changed.
    fn rebuild(&mut self, res: &Resources) -> bool {
        use std::ops::Deref;

        // Rebuild when dimensions change, but wait until at least two frames have the same size.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();

        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }

        return self.dirty;
    }

    // This is the core of a `RenderGraph`, which is building the actual graph with the subpass
    // images.
    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;

        // Retrieve a referece to the target window, which is created by the `WindowBundle`.
        let window = <ReadExpect<'_, Window>>::fetch(res);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        // Create a new drawing surface in our window.
        let surface = factory.create_surface(&window);
        let surface_format = factory.get_surface_format(&surface);

        // Begin building our `RenderGraph`.
        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(ClearValue::Color([0., 0., 0., 1.].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1., 0))),
        );

        // Create our single `Subpass`, which is the `DrawFlat2D` pass.
        // We pass the subpass builder a description of our pass for construction.
        let pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DTransparentDesc::new().builder())
                .with_group(DrawUiDesc::default().builder())
                // DEBUG
                .with_group(DrawDebugLinesDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        // Finally, add the pass to the graph.
        let _present = graph_builder
            .add_node(PresentNode::builder(factory, surface, color).with_dependency(pass));

        graph_builder
    }
}
