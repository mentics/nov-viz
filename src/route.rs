//! Orthogonal edge routing around node rectangles.
//!
//! An edge runs from the right-hand handle of its source to the left-hand
//! handle of its target, leaves and enters horizontally, and never passes
//! through a node. It is found by A* over a grid whose lines are the nodes'
//! (padded) edges plus a few lines in each gap, with a cost for every bend and
//! for running along a segment an earlier edge already uses.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

pub type Point = (f32, f32);
/// x, y, width, height.
pub type Rect = (f32, f32, f32, f32);

/// Clearance kept between an edge and any node it passes.
pub const MARGIN: f32 = 10.0;
/// How far an edge runs straight out of / into a handle before it may turn.
pub const STUB: f32 = 18.0;
const BEND_COST: f32 = 40.0;
const SHARED_COST: f32 = 2.5;
/// Extra grid lines per gap between two node edges.
const SUBDIVISIONS: usize = 3;

/// Segments already used by routed edges, so later edges prefer their own.
#[derive(Default)]
pub struct Occupied(HashSet<(usize, usize, bool)>);

/// Route from `from` (a handle on the right of a node) to `to` (a handle on
/// the left of one) around `obstacles`. `None` when no route exists.
pub fn route(from: Point, to: Point, obstacles: &[Rect], used: &mut Occupied) -> Option<Vec<Point>> {
    let start = (from.0 + STUB, from.1);
    let goal = (to.0 - STUB, to.1);
    let padded: Vec<Rect> = obstacles
        .iter()
        .map(|r| (r.0 - MARGIN, r.1 - MARGIN, r.2 + 2.0 * MARGIN, r.3 + 2.0 * MARGIN))
        .collect();

    let mut xs = vec![start.0, goal.0];
    let mut ys = vec![start.1, goal.1];
    for r in &padded {
        xs.extend([r.0, r.0 + r.2]);
        ys.extend([r.1, r.1 + r.3]);
    }
    let xs = with_subdivisions(xs);
    let ys = with_subdivisions(ys);
    let (nx, ny) = (xs.len(), ys.len());
    let at = |v: &[f32], c: f32| v.iter().position(|&x| (x - c).abs() < 0.01);
    let (sx, sy) = (at(&xs, start.0)?, at(&ys, start.1)?);
    let (gx, gy) = (at(&xs, goal.0)?, at(&ys, goal.1)?);

    // A vertex inside a padded node is unusable; a step is blocked if it
    // crosses the inside of one.
    let inside = |x: f32, y: f32| {
        padded
            .iter()
            .any(|r| x > r.0 + 0.01 && x < r.0 + r.2 - 0.01 && y > r.1 + 0.01 && y < r.1 + r.3 - 0.01)
    };
    let blocked = |ax: f32, ay: f32, bx: f32, by: f32| {
        let (lx, hx, ly, hy) = (ax.min(bx), ax.max(bx), ay.min(by), ay.max(by));
        padded.iter().any(|r| {
            hx > r.0 + 0.01 && lx < r.0 + r.2 - 0.01 && hy > r.1 + 0.01 && ly < r.1 + r.3 - 0.01
        })
    };
    if inside(start.0, start.1) || inside(goal.0, goal.1) {
        return None;
    }

    // Directions: 0 right, 1 down, 2 left, 3 up.
    const STEP: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    type State = (usize, usize, usize);
    let key = |s: State| (s.0 * ny + s.1) * 4 + s.2;
    let mut best: HashMap<usize, f32> = HashMap::new();
    let mut from_of: HashMap<usize, State> = HashMap::new();
    let mut heap = BinaryHeap::new();
    let h = |x: usize, y: usize| (xs[x] - xs[gx]).abs() + (ys[y] - ys[gy]).abs();
    let origin: State = (sx, sy, 0);
    best.insert(key(origin), 0.0);
    heap.push((Reverse(OrdF(h(sx, sy))), origin));

    let mut end = None;
    while let Some((_, state)) = heap.pop() {
        let cost = best[&key(state)];
        let (x, y, dir) = state;
        if (x, y) == (gx, gy) {
            end = Some(state);
            break;
        }
        for d in 0..4 {
            if d == (dir + 2) % 4 {
                continue;
            }
            let (nxi, nyi) = (x as i32 + STEP[d].0, y as i32 + STEP[d].1);
            if nxi < 0 || nyi < 0 || nxi as usize >= nx || nyi as usize >= ny {
                continue;
            }
            let (nxi, nyi) = (nxi as usize, nyi as usize);
            if inside(xs[nxi], ys[nyi]) || blocked(xs[x], ys[y], xs[nxi], ys[nyi]) {
                continue;
            }
            let len = (xs[nxi] - xs[x]).abs() + (ys[nyi] - ys[y]).abs();
            let horizontal = d % 2 == 0;
            let seg = (x.min(nxi) * ny + y.min(nyi), x.max(nxi) * ny + y.max(nyi), horizontal);
            let mut step = len;
            if d != dir {
                step += BEND_COST;
            }
            if used.0.contains(&seg) {
                step += len * SHARED_COST;
            }
            let next: State = (nxi, nyi, d);
            let total = cost + step;
            if best.get(&key(next)).is_none_or(|&c| total < c - 1e-3) {
                best.insert(key(next), total);
                from_of.insert(key(next), state);
                heap.push((Reverse(OrdF(total + h(nxi, nyi))), next));
            }
        }
    }
    let end = end?;

    let mut cells = vec![end];
    let mut cur = end;
    while let Some(&prev) = from_of.get(&key(cur)) {
        cells.push(prev);
        cur = prev;
    }
    cells.reverse();
    for w in cells.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = (
            a.0.min(b.0) * ny + a.1.min(b.1),
            a.0.max(b.0) * ny + a.1.max(b.1),
            a.1 == b.1,
        );
        used.0.insert(seg);
    }

    let mut points = vec![from];
    points.extend(cells.iter().map(|&(x, y, _)| (xs[x], ys[y])));
    points.push(to);
    Some(simplify(points))
}

/// A route with no path found: handle, a vertical step halfway, handle.
pub fn fallback(from: Point, to: Point) -> Vec<Point> {
    let mid = (from.0 + to.0) / 2.0;
    simplify(vec![from, (mid, from.1), (mid, to.1), to])
}

fn with_subdivisions(mut v: Vec<f32>) -> Vec<f32> {
    v.sort_by(f32::total_cmp);
    v.dedup_by(|a, b| (*a - *b).abs() < 0.01);
    let mut out = Vec::with_capacity(v.len() * SUBDIVISIONS);
    for w in v.windows(2) {
        for i in 0..SUBDIVISIONS {
            out.push(w[0] + (w[1] - w[0]) * i as f32 / SUBDIVISIONS as f32);
        }
    }
    out.extend(v.last());
    out
}

/// Drop points that lie on a straight run.
fn simplify(points: Vec<Point>) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(points.len());
    for p in points {
        if out.last().is_some_and(|&q| (q.0 - p.0).abs() < 0.01 && (q.1 - p.1).abs() < 0.01) {
            continue;
        }
        while out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            let straight_x = (a.0 - b.0).abs() < 0.01 && (b.0 - p.0).abs() < 0.01;
            let straight_y = (a.1 - b.1).abs() < 0.01 && (b.1 - p.1).abs() < 0.01;
            if straight_x || straight_y {
                out.pop();
            } else {
                break;
            }
        }
        out.push(p);
    }
    out
}

#[derive(PartialEq)]
struct OrdF(f32);
impl Eq for OrdF {}
impl PartialOrd for OrdF {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OrdF {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crosses(a: Point, b: Point, r: Rect) -> bool {
        let (lx, hx, ly, hy) = (a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1));
        hx > r.0 && lx < r.0 + r.2 && hy > r.1 && ly < r.1 + r.3
    }

    #[test]
    fn goes_around_a_node_in_the_way() {
        let blocker: Rect = (200.0, 80.0, 100.0, 60.0);
        let (from, to) = ((100.0, 110.0), (400.0, 110.0));
        let pts = route(from, to, &[blocker], &mut Occupied::default()).expect("route");
        assert_eq!(pts.first(), Some(&from));
        assert_eq!(pts.last(), Some(&to));
        for w in pts.windows(2) {
            assert!(!crosses(w[0], w[1], blocker), "{w:?} enters the node");
            assert!(w[0].0 == w[1].0 || w[0].1 == w[1].1, "{w:?} is not orthogonal");
        }
    }

    #[test]
    fn straight_when_nothing_is_in_the_way() {
        let pts = route((0.0, 50.0), (300.0, 50.0), &[], &mut Occupied::default()).unwrap();
        assert_eq!(pts.len(), 2);
    }

    #[test]
    fn a_second_edge_prefers_its_own_channel() {
        let blocker: Rect = (200.0, 80.0, 100.0, 60.0);
        let mut used = Occupied::default();
        let a = route((100.0, 110.0), (400.0, 110.0), &[blocker], &mut used).unwrap();
        let b = route((100.0, 110.0), (400.0, 110.0), &[blocker], &mut used).unwrap();
        assert_ne!(a, b);
    }
}
