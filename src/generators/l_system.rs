use kurbo::{BezPath, PathEl, Point};
use std::collections::HashMap;

use super::{Generator, GeneratorKind, GeneratorParams};
use crate::core::PlotPath;

#[derive(Clone, Debug)]
pub struct LSystemParams {
    pub axiom: String,
    #[allow(dead_code)]
    pub rules: HashMap<char, String>,
    pub rules_text: String,
    pub angle: f64,
    pub segment_length: f64,
    pub iterations: u32,
}

impl Default for LSystemParams {
    fn default() -> Self {
        let mut rules = HashMap::new();
        rules.insert('F', "F[+F]F[-F]F".into());
        Self {
            axiom: "F".into(),
            rules: rules.clone(),
            rules_text: rules.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("\n"),
            angle: 25.0_f64.to_radians(),
            segment_length: 12.0,
            iterations: 3,
        }
    }
}

fn parse_rules_text(text: &str) -> HashMap<char, String> {
    let mut rules = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().chars().next();
            let value = line[eq_pos + 1..].trim().to_string();
            if let Some(k) = key {
                rules.insert(k, value);
            }
        }
    }
    rules
}

pub struct LSystemGenerator;

impl Generator for LSystemGenerator {
    fn generate(&self, params: &GeneratorParams) -> Vec<PlotPath> {
        let p = &params.l_system;
        let rules = parse_rules_text(&p.rules_text);
        let mut current = p.axiom.clone();

        for _ in 0..p.iterations {
            let mut next = String::new();
            for ch in current.chars() {
                match rules.get(&ch) {
                    Some(replacement) => next.push_str(replacement),
                    None => next.push(ch),
                }
            }
            current = next;
        }

        let mut path = BezPath::new();
        let mut pos = Point::new(0.0, 0.0);
        let mut angle = -90.0_f64.to_radians(); // start pointing up
        let mut stack: Vec<(Point, f64)> = Vec::new();
        let mut first = true;

        path.push(PathEl::MoveTo(pos));

        for ch in current.chars() {
            match ch {
                'F' | 'G' => {
                    let dx = pos.x + p.segment_length * angle.cos();
                    let dy = pos.y + p.segment_length * angle.sin();
                    pos = Point::new(dx, dy);
                    path.push(PathEl::LineTo(pos));
                }
                'f' => {
                    let dx = pos.x + p.segment_length * angle.cos();
                    let dy = pos.y + p.segment_length * angle.sin();
                    pos = Point::new(dx, dy);
                    if first {
                        path.push(PathEl::MoveTo(pos));
                        first = false;
                    } else {
                        path.push(PathEl::MoveTo(pos));
                    }
                }
                '+' => {
                    angle += p.angle;
                }
                '-' => {
                    angle -= p.angle;
                }
                '[' => {
                    stack.push((pos, angle));
                }
                ']' => {
                    if let Some((saved_pos, saved_angle)) = stack.pop() {
                        pos = saved_pos;
                        angle = saved_angle;
                        path.push(PathEl::MoveTo(pos));
                        first = false;
                    }
                }
                _ => {}
            }
        }

        vec![PlotPath::new(path, false).with_name("L-System")]
    }

    fn kind(&self) -> GeneratorKind {
        GeneratorKind::LSystem
    }

    fn name(&self) -> &'static str {
        "L-System"
    }
}
