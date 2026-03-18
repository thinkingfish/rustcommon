// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use clocksource::precise::Duration;
use rand::RngExt;
use rand_distr::*;
use std::time::Instant;
use waterfall::*;

fn main() {
    println!("Welcome to the simulator!");

    for shape in &[
        Shape::Cauchy,
        Shape::Normal,
        Shape::Uniform,
        Shape::Triangular,
        Shape::Gamma,
    ] {
        simulate(*shape);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Shape {
    Cauchy,
    Normal,
    Uniform,
    Triangular,
    Gamma,
}

pub fn simulate(shape: Shape) {
    let run_duration = std::time::Duration::from_secs(10);
    println!(
        "Simulating for {:?} distribution for {:?} seconds",
        shape,
        run_duration.as_secs_f64()
    );

    let mut heatmap =
        Heatmap::new(0, 30, Duration::from_secs(10), Duration::from_millis(250)).unwrap();

    let cauchy = Cauchy::new(500_000.0, 2_000.00).unwrap();
    let normal = Normal::new(200_000.0, 100_000.0).unwrap();
    let uniform = rand_distr::Uniform::new(10_000.0, 200_000.0).unwrap();
    let triangular = Triangular::new(1.0, 200_000.0, 50_000.0).unwrap();
    let gamma = Gamma::new(2.0, 2.0).unwrap();

    let mut rng = rand::rng();
    let start = Instant::now();
    loop {
        if start.elapsed() >= run_duration {
            break;
        }
        let value: f64 = match shape {
            Shape::Cauchy => rng.sample(cauchy),
            Shape::Normal => rng.sample(normal),
            Shape::Uniform => rng.sample(uniform),
            Shape::Triangular => rng.sample(triangular),
            Shape::Gamma => rng.sample(gamma) * 100_000.0,
        };
        let value = value.floor() as u64;
        if value != 0 {
            let _ = heatmap.increment(clocksource::precise::Instant::now(), value, 1);
        }
    }

    let shape_name = match shape {
        Shape::Cauchy => "cauchy",
        Shape::Normal => "normal",
        Shape::Uniform => "uniform",
        Shape::Triangular => "triangular",
        Shape::Gamma => "gamma",
    };

    for scale in [Scale::Linear, Scale::Logarithmic].iter() {
        for palette in [Palette::Classic, Palette::Ironbow].iter() {
            let scale_name = match scale {
                Scale::Linear => "linear",
                Scale::Logarithmic => "logarithmic",
            };

            let palette_name = match palette {
                Palette::Classic => "classic",
                Palette::Ironbow => "ironbow",
            };

            let filename = format!("{}_{}_{}.png", shape_name, palette_name, scale_name);

            WaterfallBuilder::new(&filename)
                .label(100, "100")
                .label(1000, "1000")
                .label(10000, "10000")
                .label(100000, "100000")
                .scale(*scale)
                .palette(*palette)
                .build(&heatmap);

            let filename = format!("{}_{}_{}_smooth.png", shape_name, palette_name, scale_name);

            WaterfallBuilder::new(&filename)
                .label(100, "100")
                .label(1000, "1000")
                .label(10000, "10000")
                .label(100000, "100000")
                .scale(*scale)
                .palette(*palette)
                .smooth(Some(1.0))
                .build(&heatmap);
        }
    }
}
