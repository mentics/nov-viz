//! Draws edges (gpui-flow's own are hidden) along routes that run handle to
//! handle around every card (`route`), with type labels placed clear of cards.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use gpui::*;
use gpui_flow::*;

use crate::flow::edge_type_for_color;
use crate::route::{self, Occupied, Point, Rect};
use crate::scale::card_scale;

const RADIUS: f32 = 10.0;
const ARROW: f32 = 8.0;
const STROKE: f32 = 2.0;
const DEFAULT_SIZE: (f32, f32) = (176.0, 60.0);

pub struct EdgeLayer {
    state: Entity<FlowState>,
    /// Routes in zoom-scaled flow coordinates (no pan offset), kept while the
    /// nodes, the zoom and the edges are unchanged, so panning costs nothing.
    cache: Option<(u64, Vec<Vec<Point>>)>,
}

impl EdgeLayer {
    pub fn new(state: Entity<FlowState>) -> Self {
        Self { state, cache: None }
    }
}

/// Every card is laid out at this size (flow units); what is drawn is this
/// times `card_scale`. gpui-flow's measured size is zoomed pixels and lags.
fn layout_size() -> (f32, f32) {
    DEFAULT_SIZE
}

/// What the routes depend on: the zoom, every node's place, every edge.
fn routing_key(state: &FlowState) -> u64 {
    let mut h = DefaultHasher::new();
    state.viewport.zoom.to_bits().hash(&mut h);
    for n in &state.nodes {
        n.id.hash(&mut h);
        n.position.x.to_bits().hash(&mut h);
        n.position.y.to_bits().hash(&mut h);
    }
    for e in &state.edges {
        (&e.id, &e.source, &e.target).hash(&mut h);
    }
    h.finish()
}

/// Each edge's path, from its source's right-hand handle to its target's
/// left-hand one, around every card, in zoom-scaled flow coordinates. The
/// cards' drawn size is the layout size times `card_scale`.
fn compute_routes(state: &FlowState) -> Vec<Vec<Point>> {
    let zoom = state.viewport.zoom;
    let scale = card_scale(zoom).size;
    let (w, h) = layout_size();
    let (cw, ch) = (w * scale, h * scale);
    let rect_of = |n: &FlowNode| (n.position.x * zoom, n.position.y * zoom, cw, ch);
    let obstacles: Vec<Rect> = state.nodes.iter().map(rect_of).collect();
    let mut used = Occupied::default();
    state
        .edges
        .iter()
        .map(|edge| {
            let (Some(source), Some(target)) = (state.get_node(&edge.source), state.get_node(&edge.target))
            else {
                return Vec::new();
            };
            let (s, t) = (rect_of(source), rect_of(target));
            let from = (s.0 + cw, s.1 + ch / 2.0);
            let to = (t.0, t.1 + ch / 2.0);
            route::route(from, to, &obstacles, &mut used).unwrap_or_else(|| route::fallback(from, to))
        })
        .collect()
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.0 < b.0 + b.2 && b.0 < a.0 + a.2 && a.1 < b.1 + b.3 && b.1 < a.1 + a.3
}

/// A spot on the route for a label of `size` that touches no node and no
/// label already placed: segment midpoints, longest first. `None` when the
/// route has no clear spot, so the label is left off rather than drawn over
/// something.
fn label_anchor(points: &[(f32, f32)], size: (f32, f32), taken: &[Rect]) -> Option<(f32, f32)> {
    let mut segments: Vec<_> = points
        .windows(2)
        .map(|w| ((w[1].0 - w[0].0).hypot(w[1].1 - w[0].1), w[0], w[1]))
        .collect();
    segments.sort_by(|a, b| b.0.total_cmp(&a.0));
    let pad = |r: Rect| (r.0 - 4.0, r.1 - 4.0, r.2 + 8.0, r.3 + 8.0);
    let clear = |(x, y): (f32, f32)| {
        let rect = (x - size.0 / 2.0, y - size.1 / 2.0, size.0, size.1);
        !taken.iter().any(|&t| overlaps(pad(rect), t))
    };
    segments.iter().find_map(|&(_, a, b)| {
        let (x, y) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
        // On the segment; for a vertical one also beside it, in the gap.
        let mut spots = vec![(x, y)];
        if (a.0 - b.0).abs() < 0.5 {
            let off = size.0 / 2.0 + 6.0;
            spots.extend([(x + off, y), (x - off, y)]);
        }
        spots.into_iter().find(|&s| clear(s))
    })
}

impl Render for EdgeLayer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let viewport = state.viewport;
        let key = routing_key(state);
        if self.cache.as_ref().is_none_or(|(k, _)| *k != key) {
            self.cache = Some((key, compute_routes(state)));
        }
        let routes = &self.cache.as_ref().expect("just filled").1;
        let scale = card_scale(viewport.zoom);
        let (lw, lh) = layout_size();

        let mut taken: Vec<Rect> = state
            .nodes
            .iter()
            .map(|n| {
                let (x, y) = viewport.flow_to_screen(n.position);
                (x, y, lw * scale.size, lh * scale.size)
            })
            .collect();
        let mut paths: Vec<(Vec<(f32, f32)>, u32)> = Vec::new();
        let mut labels: Vec<AnyElement> = Vec::new();
        for (edge, route) in state.edges.iter().zip(routes) {
            if route.len() < 2 {
                continue;
            }
            // Cached routes carry no pan offset.
            let points: Vec<Point> = route.iter().map(|&(x, y)| (x + viewport.x, y + viewport.y)).collect();
            let color = edge.color.unwrap_or(0x71717a);
            // Labels are text, so they follow the cards': shown only while
            // the cards show theirs, and sized with them.
            if scale.text {
                let text = edge_type_for_color(edge.color);
                let font = 12.0 * scale.size;
                let (width, height) = (text.len() as f32 * font * 0.55 + 18.0, font + 10.0);
                if let Some((x, y)) = label_anchor(&points, (width, height), &taken) {
                    taken.push((x - width / 2.0, y - height / 2.0, width, height));
                    labels.push(
                        div()
                            .absolute()
                            .left(px(x - width / 2.0))
                            .top(px(y - height / 2.0))
                            .w(px(width))
                            .h(px(height))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(gpui::rgb(0x09090b))
                            .border_1()
                            .border_color(gpui::rgb(color))
                            .rounded_sm()
                            .text_size(px(font))
                            .text_color(gpui::rgb(color))
                            .child(text)
                            .into_any_element(),
                    );
                }
            }
            paths.push((points, color));
        }

        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .size_full()
            .child(
                canvas(
                    |_, _, _| {},
                    move |_, _: (), window, _| {
                        for (points, color) in &paths {
                            paint_route(window, points, *color, RADIUS * scale.size.min(1.0));
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .children(labels)
    }
}

fn paint_route(window: &mut Window, pts: &[(f32, f32)], color: u32, radius: f32) {
    if pts.len() < 2 {
        return;
    }
    let pt = |p: (f32, f32)| point(px(p.0), px(p.1));
    let unit = |a: (f32, f32), b: (f32, f32)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len = dx.hypot(dy).max(0.001);
        (dx / len, dy / len, len)
    };

    let mut builder = PathBuilder::stroke(px(STROKE));
    builder.move_to(pt(pts[0]));
    for i in 1..pts.len() - 1 {
        let (a, b, c) = (pts[i - 1], pts[i], pts[i + 1]);
        let (ux, uy, l1) = unit(b, a);
        let (vx, vy, l2) = unit(b, c);
        let r = radius.min(l1 / 2.0).min(l2 / 2.0);
        builder.line_to(pt((b.0 + ux * r, b.1 + uy * r)));
        builder.curve_to(pt((b.0 + vx * r, b.1 + vy * r)), pt(b));
    }
    let last = pts[pts.len() - 1];
    builder.line_to(pt(last));
    if let Ok(path) = builder.build() {
        window.paint_path(path, gpui::rgb(color));
    }

    let (ux, uy, _) = unit(pts[pts.len() - 2], last);
    let (nx, ny) = (-uy, ux);
    let base = (last.0 - ux * ARROW, last.1 - uy * ARROW);
    let mut tri = PathBuilder::fill();
    tri.move_to(pt(last));
    tri.line_to(pt((base.0 + nx * ARROW / 2.0, base.1 + ny * ARROW / 2.0)));
    tri.line_to(pt((base.0 - nx * ARROW / 2.0, base.1 - ny * ARROW / 2.0)));
    tri.line_to(pt(last));
    if let Ok(path) = tri.build() {
        window.paint_path(path, gpui::rgb(color));
    }
}
