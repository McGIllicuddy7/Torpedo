use game::game_create_level;
use level::{default_setup, main_loop};

use math::Vector3;
use physics::{Col, create_box, create_box_stationary};
use raylib::{color::Color, ffi::GamepadAxis};
use renderer::ModelList;

pub mod arena;
pub mod game;
pub mod level;
pub mod math;
pub mod physics;
pub mod renderer;
pub mod ui;
pub mod draw_call;
#[allow(unused)]
pub fn make_test_level(
    thread: &raylib::RaylibThread,
    handle: &mut raylib::RaylibHandle,
) -> ModelList {
    let out = default_setup(thread, handle, 16384);
    let colors = [
        Color::WHITE,
        Color::HOTPINK,
        Color::GREEN,
        Color::RED,
        Color::BLUE,
        Color::PURPLE,
        Color::DARKBLUE,
        Color::DARKGREEN,
        Color::YELLOW,
        Color::BLUEVIOLET,
    ];
    let mut count = 0;
    let d = 5;
    let div = 2.;
    for x in 0..d {
        for y in 0..d {
            for z in 0..d {
                let dx = -d as f64 / div / 2.;
                let dy = -d as f64 / div / 2.;
                let dz = -d as f64 / div / 2.;
                let t = if rand::random::<u64>() % 100 < 10 || true {
                    create_box(
                        Vector3::new(
                            x as f64 / div + dx,
                            y as f64 / div + dy,
                            z as f64 / div + dz,
                        ),
                        Vector3::new(-x as f64 + dx, -y as f64 + dy, -z as f64 + dz) / 10.0,
                        colors[count % colors.len()],
                    )
                } else {
                    create_box_stationary(
                        Vector3::new(
                            x as f64 / div + dx,
                            y as f64 / div + dy,
                            z as f64 / div + dz,
                        ),
                        Vector3::new(-x as f64 + dx, -y as f64 + dy, -z as f64 + dz).normalized()
                            / 1.0,
                        colors[count % colors.len()],
                    )
                };
                count += 1;
            }
        }
    }

    out
}

fn main() {
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()
        .unwrap();
    main_loop(Box::new(game_create_level));
    if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg").unwrap();
        report.flamegraph(file).unwrap();
    };
}
