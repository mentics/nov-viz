//! Minimap that maps clicks relative to its own bounds (gpui-flow's takes
//! absolute window coordinates, so a click pans the view off into empty space).

use std::cell::Cell;
use std::rc::Rc;

use gpui::*;
use gpui_flow::*;

const WIDTH: f32 = 200.0;
const HEIGHT: f32 = 140.0;
const PAD: f32 = 8.0;

pub struct NovMinimap {
    state: Entity<FlowState>,
    container: (f32, f32),
    bounds: Rc<Cell<Bounds<Pixels>>>,
    dragging: Rc<Cell<bool>>,
}

impl NovMinimap {
    pub fn new(state: Entity<FlowState>) -> Self {
        Self {
            state,
            container: (1200.0, 800.0),
            bounds: Rc::new(Cell::new(Bounds::default())),
            dragging: Rc::new(Cell::new(false)),
        }
    }

    pub fn container_bounds(mut self, width: f32, height: f32) -> Self {
        self.container = (width, height);
        self
    }

    pub fn set_container(&mut self, width: f32, height: f32) {
        self.container = (width, height);
    }
}

/// (min_x, min_y, width, height) of all visible nodes, padded.
fn graph_bounds(state: &FlowState) -> Option<(f32, f32, f32, f32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let mut any = false;
    let zoom = state.viewport.zoom;
    for n in state.nodes.iter().filter(|n| !n.hidden) {
        let (w, h) = node_size(n, zoom);
        x0 = x0.min(n.position.x);
        y0 = y0.min(n.position.y);
        x1 = x1.max(n.position.x + w);
        y1 = y1.max(n.position.y + h);
        any = true;
    }
    any.then(|| (x0 - 50.0, y0 - 50.0, x1 - x0 + 100.0, y1 - y0 + 100.0))
}

/// Cards are all laid out at this size (flow units); see `edge_layer`.
fn node_size(_: &FlowNode, _zoom: f32) -> (f32, f32) {
    (176.0, 60.0)
}

/// (scale, offset_x, offset_y) mapping flow coords to minimap-local coords.
fn transform(g: (f32, f32, f32, f32)) -> (f32, f32, f32) {
    let (iw, ih) = (WIDTH - PAD * 2.0, HEIGHT - PAD * 2.0);
    let scale = (iw / g.2).min(ih / g.3);
    (
        scale,
        (iw - g.2 * scale) / 2.0 + PAD,
        (ih - g.3 * scale) / 2.0 + PAD,
    )
}

impl Render for NovMinimap {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_paint = self.state.clone();
        let container = self.container;
        let cell = self.bounds.clone();
        let entity_id = cx.entity_id();

        let pan = {
            let state = self.state.clone();
            let cell = self.bounds.clone();
            move |pos: Point<Pixels>, window: &mut Window, cx: &mut App| {
                let origin = cell.get().origin;
                let (lx, ly) = ((pos.x - origin.x).as_f32(), (pos.y - origin.y).as_f32());
                state.update(cx, |state, _| {
                    let Some(g) = graph_bounds(state) else { return };
                    let (scale, ox, oy) = transform(g);
                    let fx = (lx - ox) / scale + g.0;
                    let fy = (ly - oy) / scale + g.1;
                    state.viewport.x = container.0 / 2.0 - fx * state.viewport.zoom;
                    state.viewport.y = container.1 / 2.0 - fy * state.viewport.zoom;
                });
                cx.notify(entity_id);
                window.refresh();
            }
        };
        let pan_paint = pan.clone();
        let dragging = self.dragging.clone();
        let dragging_down = self.dragging.clone();

        div()
            .id("nov-minimap")
            .w(px(WIDTH))
            .h(px(HEIGHT))
            .bg(gpui::rgba(0x1a1a1acc))
            .rounded_md()
            .border_1()
            .border_color(gpui::rgba(0xffffff33))
            .overflow_hidden()
            .child(
                canvas(
                    move |bounds, _, _| cell.set(bounds),
                    move |bounds, _: (), window, cx| {
                        paint(&bounds, state_paint.read(cx), container, window);
                        // While a drag is on, follow the cursor anywhere in the
                        // window, not only over the minimap.
                        if dragging.get() {
                            let pan = pan_paint.clone();
                            let dragging_move = dragging.clone();
                            window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, cx| {
                                if phase.capture() && dragging_move.get() {
                                    if e.pressed_button == Some(MouseButton::Left) {
                                        pan(e.position, window, cx);
                                    } else {
                                        dragging_move.set(false);
                                    }
                                    cx.stop_propagation();
                                }
                            });
                            let dragging_up = dragging.clone();
                            window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                                if phase.capture() && dragging_up.get() {
                                    dragging_up.set(false);
                                    cx.stop_propagation();
                                }
                            });
                        }
                    },
                )
                .size_full(),
            )
            .occlude()
            .on_mouse_down(MouseButton::Left, move |e, window, cx| {
                dragging_down.set(true);
                pan(e.position, window, cx);
                cx.stop_propagation();
            })
    }
}

fn paint(bounds: &Bounds<Pixels>, state: &FlowState, container: (f32, f32), window: &mut Window) {
    let Some(g) = graph_bounds(state) else { return };
    let (scale, ox, oy) = transform(g);
    let (bx, by) = (bounds.origin.x.as_f32() + ox, bounds.origin.y.as_f32() + oy);
    let rect = |x: f32, y: f32, w: f32, h: f32| Bounds::new(point(px(x), px(y)), size(px(w), px(h)));

    let zoom = state.viewport.zoom;
    for n in state.nodes.iter().filter(|n| !n.hidden) {
        let (w, h) = node_size(n, zoom);
        let color = if n.selected {
            gpui::rgba(0x3b82f6cc)
        } else {
            gpui::rgba(0x3b82f688)
        };
        window.paint_quad(fill(
            rect(
                bx + (n.position.x - g.0) * scale,
                by + (n.position.y - g.1) * scale,
                w * scale,
                h * scale,
            ),
            color,
        ));
    }

    let vp = &state.viewport;
    let (vx, vy) = (
        bx + (-vp.x / vp.zoom - g.0) * scale,
        by + (-vp.y / vp.zoom - g.1) * scale,
    );
    let (vw, vh) = (container.0 / vp.zoom * scale, container.1 / vp.zoom * scale);
    window.paint_quad(fill(rect(vx, vy, vw, vh), gpui::rgba(0x3b82f620)));
    let edge: Background = gpui::rgba(0x3b82f6aa).into();
    for r in [
        rect(vx, vy, vw, 1.0),
        rect(vx, vy + vh, vw, 1.0),
        rect(vx, vy, 1.0, vh),
        rect(vx + vw, vy, 1.0, vh),
    ] {
        window.paint_quad(fill(r, edge.clone()));
    }
}
