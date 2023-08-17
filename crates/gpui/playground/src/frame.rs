use crate::{
    element::{
        AnyElement, Element, EventHandler, IntoElement, Layout, LayoutContext, NodeId, PaintContext,
    },
    style::{ElementStyle, Fill},
};
use anyhow::{anyhow, Result};
use gpui::LayoutNodeId;
use playground_macros::IntoElement;

#[derive(IntoElement)]
#[element_crate = "crate"]
pub struct Frame<V: 'static> {
    style: ElementStyle,
    handlers: Vec<EventHandler<V>>,
    children: Vec<AnyElement<V>>,
}

pub fn frame<V>() -> Frame<V> {
    Frame {
        style: ElementStyle::default(),
        handlers: Vec::new(),
        children: Vec::new(),
    }
}

impl<V: 'static> Element<V> for Frame<V> {
    type Layout = ();

    fn style_mut(&mut self) -> &mut ElementStyle {
        &mut self.style
    }

    fn handlers_mut(&mut self) -> &mut Vec<EventHandler<V>> {
        &mut self.handlers
    }

    fn layout(
        &mut self,
        view: &mut V,
        cx: &mut LayoutContext<V>,
    ) -> Result<(NodeId, Self::Layout)> {
        let child_layout_node_ids = self
            .children
            .iter_mut()
            .map(|child| child.layout(view, cx))
            .collect::<Result<Vec<LayoutNodeId>>>()?;

        let rem_size = cx.rem_pixels();
        let node_id = cx
            .layout_engine()
            .ok_or_else(|| anyhow!("no layout engine"))?
            .add_node(self.style.to_taffy(rem_size), child_layout_node_ids)?;

        Ok((node_id, ()))
    }

    fn paint(&mut self, layout: Layout<()>, view: &mut V, cx: &mut PaintContext<V>) -> Result<()> {
        cx.scene.push_quad(gpui::scene::Quad {
            bounds: layout.from_engine.bounds,
            background: self
                .style
                .fill
                .as_ref()
                .and_then(Fill::color)
                .map(Into::into),
            border: Default::default(),
            corner_radii: Default::default(),
        });

        for child in &mut self.children {
            child.paint(view, cx)?;
        }
        Ok(())
    }
}

impl<V: 'static> Frame<V> {
    pub fn child(mut self, child: impl IntoElement<V>) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children<I, E>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: IntoElement<V>,
    {
        self.children
            .extend(children.into_iter().map(|e| e.into_any_element()));
        self
    }
}