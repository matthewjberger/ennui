use crate::components::Plot;
use crate::data::{Mark, Series};
use crate::theme::{PLOT_BAR_GAP, PLOT_DIVISIONS, PLOT_GRID_ALPHA, PLOT_INSET, PLOT_LINE};
use ennui_quads::prelude::Quad;
use ennui_ui::prelude::{Rect, Theme};

use nalgebra_glm::{Vec2, Vec4};
use std::hash::{DefaultHasher, Hash, Hasher};

pub(crate) fn plot_key(plot: &Plot, rect: &Rect) -> u64 {
    let mut hasher = DefaultHasher::new();
    for value in [
        rect.size.x,
        rect.size.y,
        plot.low[0],
        plot.low[1],
        plot.high[0],
        plot.high[1],
    ] {
        value.to_bits().hash(&mut hasher);
    }
    for series in &plot.series {
        (series.mark == Mark::Bars).hash(&mut hasher);
        for value in series.color.iter() {
            value.to_bits().hash(&mut hasher);
        }
        for point in &series.points {
            point[0].to_bits().hash(&mut hasher);
            point[1].to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn solid(center: Vec2, half: Vec2, color: Vec4) -> Quad {
    Quad::over([center.x, center.y], [half.x, half.y], color.into())
}

struct Field {
    low: Vec2,
    span: Vec2,
    least: Vec2,
    room: Vec2,
}

fn place(field: &Field, point: [f32; 2]) -> Vec2 {
    let along = (Vec2::from(point) - field.low).component_div(&field.span);
    let along = Vec2::new(along.x.clamp(0.0, 1.0), along.y.clamp(0.0, 1.0));
    field.least + along.component_mul(&field.room)
}

fn grid(field: &Field, color: Vec4, line: f32, quads: &mut Vec<Quad>) {
    for step in 0..=PLOT_DIVISIONS {
        let share = step as f32 / PLOT_DIVISIONS as f32;
        let across = field.least + Vec2::new(field.room.x * share, field.room.y * 0.5);
        let down = field.least + Vec2::new(field.room.x * 0.5, field.room.y * share);
        quads.push(solid(across, Vec2::new(line, field.room.y * 0.5), color));
        quads.push(solid(down, Vec2::new(field.room.x * 0.5, line), color));
    }
}

fn strokes(field: &Field, series: &Series, (step, line): (f32, f32), quads: &mut Vec<Quad>) {
    for pair in series.points.windows(2) {
        let (mut start, mut end) = (place(field, pair[0]), place(field, pair[1]));
        if end.x < start.x {
            std::mem::swap(&mut start, &mut end);
        }
        let run = (end.x - start.x).max(step);
        let mut across = start.x;
        while across < end.x || across == start.x {
            let next = (across + step).min(end.x.max(start.x + step));
            let first = start.y + (end.y - start.y) * ((across - start.x) / run).clamp(0.0, 1.0);
            let second = start.y + (end.y - start.y) * ((next - start.x) / run).clamp(0.0, 1.0);
            let center = Vec2::new((across + next) * 0.5, (first + second) * 0.5);
            let half = Vec2::new(
                (next - across) * 0.5 + line,
                (second - first).abs() * 0.5 + line,
            );
            quads.push(solid(center, half, series.color));
            across = next;
        }
    }
}

fn bars(field: &Field, series: &Series, (slot, offset): (f32, f32), quads: &mut Vec<Quad>) {
    let base = place(field, [field.low.x, field.low.y]).y;
    for point in &series.points {
        let top = place(field, *point);
        let center = Vec2::new(top.x + offset, (top.y + base) * 0.5);
        let half = Vec2::new(slot * 0.5, (top.y - base).abs() * 0.5);
        quads.push(solid(center, half, series.color));
    }
}

pub(crate) fn plot_quads(plot: &Plot, rect: &Rect, look: &Theme) -> Vec<Quad> {
    let inset = PLOT_INSET;
    let half = rect.size * 0.5 - Vec2::repeat(inset);
    let span = Vec2::new(
        (plot.high[0] - plot.low[0]).max(f32::EPSILON),
        (plot.high[1] - plot.low[1]).max(f32::EPSILON),
    );
    let field = Field {
        low: Vec2::from(plot.low),
        span,
        least: -half,
        room: half * 2.0,
    };
    let mut quads = Vec::new();
    let faint = Vec4::new(look.edge.x, look.edge.y, look.edge.z, PLOT_GRID_ALPHA);
    let line = PLOT_LINE * 0.5;
    grid(&field, faint, line, &mut quads);
    let barred = plot
        .series
        .iter()
        .filter(|series| series.mark == Mark::Bars)
        .count()
        .max(1) as f32;
    let mut lane = 0.0;
    for series in &plot.series {
        match series.mark {
            Mark::Line => strokes(&field, series, (1.0, line), &mut quads),
            Mark::Bars => {
                let count = series.points.len().max(1) as f32;
                let slot = field.room.x / count * (1.0 - PLOT_BAR_GAP) / barred;
                let offset = (lane - (barred - 1.0) * 0.5) * slot;
                bars(&field, series, (slot, offset), &mut quads);
                lane += 1.0;
            }
        }
    }
    quads
}
