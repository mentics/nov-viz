use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_flow::*;

use crate::keyboard::Depth;
use crate::layout::{LaidOutEdge, LaidOutNode, LayoutResult};
use crate::nav::NavVisual;

const ACCENT_BLUE: u32 = 0x3b82f6;
const ACCENT_AMBER: u32 = 0xf59e0b;
const ACCENT_ROSE: u32 = 0xf43f5e;
const ACCENT_VIOLET: u32 = 0xa78bfa;
const ACCENT_MUTED: u32 = 0x71717a;
const CURSOR: u32 = 0xfafafa;
const INTERIOR: u32 = 0x22d3ee;
const EDITING: u32 = 0xe8a87c;

/// Edge types and their colours, for the legend (edges carry no inline labels:
/// gpui-flow places them at the straight-line midpoint, over the nodes).
pub const EDGE_LEGEND: [(&str, u32); 4] = [
    ("depends_on", ACCENT_BLUE),
    ("blocks", ACCENT_ROSE),
    ("affects", ACCENT_AMBER),
    ("replaces", ACCENT_VIOLET),
];

/// The edge type is carried by the edge colour (gpui-flow's label is not used),
/// so each type needs its own; this is the inverse of [`edge_color`].
pub fn edge_type_for_color(color: Option<u32>) -> &'static str {
    match color {
        Some(ACCENT_BLUE) => "depends_on",
        Some(ACCENT_ROSE) => "blocks",
        Some(ACCENT_AMBER) => "affects",
        Some(ACCENT_VIOLET) => "replaces",
        _ => "related",
    }
}

pub fn to_flow_nodes(laid_out: &[LaidOutNode]) -> Vec<FlowNode> {
    laid_out
        .iter()
        .map(|node| {
            FlowNode::new(&node.id, node.x as f32, node.y as f32)
                .label(node.label.clone())
                .node_type("nov-card")
                .size(node.width as f32, node.height as f32)
                .handles(vec![
                    HandleDef::target(HandlePosition::Left),
                    HandleDef::source(HandlePosition::Right),
                ])
        })
        .collect()
}

pub fn to_flow_edges(laid_out: &[LaidOutEdge]) -> Vec<FlowEdge> {
    laid_out
        .iter()
        .map(|edge| {
            // Hidden: gpui-flow keeps the edge in its state (nav reads it) but
            // does not paint it; `EdgeLayer` draws it along ELK's route.
            let mut flow_edge = FlowEdge::new(&edge.id, &edge.source, &edge.target)
                .edge_type(EdgeType::SmoothStep {
                    border_radius: 10.0,
                    offset: 24.0,
                })
                .color(edge_color(&edge.edge_type))
                .stroke_width(2.0);
            flow_edge.hidden = true;
            flow_edge
        })
        .collect()
}

fn edge_color(edge_type: &str) -> u32 {
    match edge_type {
        "depends_on" => ACCENT_BLUE,
        "blocks" => ACCENT_ROSE,
        "affects" => ACCENT_AMBER,
        "replaces" => ACCENT_VIOLET,
        _ => ACCENT_MUTED,
    }
}

pub fn build_flow_graph(
    layout: &LayoutResult,
    visual: Entity<NavVisual>,
    cx: &mut App,
) -> (Entity<FlowState>, Entity<FlowGraph>) {
    let nodes = to_flow_nodes(&layout.nodes);
    let edges = to_flow_edges(&layout.edges);
    let state = cx.new(|_| {
        let mut state = FlowState::new(nodes, edges);
        state.min_zoom = 0.25;
        state.max_zoom = 4.0;
        state
    });
    let sizes: std::rc::Rc<std::collections::HashMap<String, (f32, f32)>> = std::rc::Rc::new(
        layout
            .nodes
            .iter()
            .map(|n| (n.id.clone(), (n.width as f32, n.height as f32)))
            .collect(),
    );
    let zoom_state = state.clone();
    let flow = cx.new(|cx| {
        FlowGraph::new(state.clone(), cx)
            .bg_color(0x09090b)
            .grid_color(0x18181b)
            .bg_pattern(BackgroundPattern::Cross)
            .no_node_chrome()
            .node_renderer("nov-card", {
                let visual = visual.clone();
                move |node, window, cx| {
                    let scale = crate::scale::card_scale(zoom_state.read(cx).viewport.zoom);
                    let (w, h) = sizes.get(node.id.as_ref()).copied().unwrap_or((176.0, 60.0));
                    render_nov_card(node, &visual, (w * scale.size, h * scale.size), scale, window, cx)
                }
            })
    });
    (state, flow)
}

fn render_nov_card(
    node: &FlowNode,
    visual: &Entity<NavVisual>,
    (width, height): (f32, f32),
    scale: crate::scale::CardScale,
    _window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    const TEXT: u32 = 0xfafafa;
    const TEXT_MUTED: u32 = 0xa1a1aa;
    // 30% blue over the canvas so cards read against the black background.
    const CARD_FILL: u32 = 0x3b82f64d;

    let vis = visual.read(cx);
    let id = node.id.to_string();
    let is_cursor = vis.cursor.as_deref() == Some(id.as_str());
    let is_editing = vis.editing.as_deref() == Some(id.as_str());
    let is_target = vis.connecting && is_cursor;
    let interior = vis.depth == Depth::Node && is_cursor;

    let border = if is_editing {
        EDITING
    } else if is_target {
        ACCENT_AMBER
    } else if interior {
        INTERIOR
    } else if is_cursor {
        CURSOR
    } else if node.selected {
        ACCENT_BLUE
    } else {
        0x27272a
    };

    let tag = node.id.to_string();
    let label = if node.label.is_empty() {
        node.id.to_string()
    } else {
        node.label.to_string()
    };

    // gpui-flow does not scale node content, so the card is drawn at its
    // layout size times `card_scale` (text held legible, see `scale`); edges
    // are routed against that size.
    div()
        .flex()
        .flex_col()
        .justify_center()
        .w(px(width))
        .h(px(height))
        .overflow_hidden()
        .px(px(12.0 * scale.size))
        .bg(gpui::rgba(CARD_FILL))
        .border_1()
        .border_color(gpui::rgb(border))
        .rounded_md()
        .when(scale.text, |card| {
            card.child(
                div()
                    .text_size(px(12.0 * scale.size))
                    .text_color(gpui::rgb(TEXT_MUTED))
                    .font_weight(FontWeight::MEDIUM)
                    .child(tag),
            )
            .child(
                div()
                    .text_size(px(14.0 * scale.size))
                    .text_color(gpui::rgb(TEXT))
                    .font_weight(FontWeight::MEDIUM)
                    .child(label),
            )
        })
        .into_any_element()
}
