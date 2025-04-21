
use level::{default_setup, main_loop};

use math::Vector3;
use physics::{create_box, Col};
use raylib::color::Color;
use renderer::ModelList;

pub mod physics;
pub mod renderer;
pub mod game;
pub mod level;
pub mod math;
pub mod arena;

#[allow(unused)]
pub fn make_test_level(thread:&raylib::RaylibThread, handle:&mut raylib::RaylibHandle)->ModelList{
    let out = default_setup(thread, handle, 4096*8);
    let colors = [Color::WHITE, Color::HOTPINK, Color::GREEN, Color::RED, Color::BLUE, Color::PURPLE, Color::DARKBLUE, Color::DARKGREEN, Color::YELLOW, Color::BLUEVIOLET];
    let mut count = 0;
    let d =10;
    let div = 2.;
    for x in 0..d{
        for y in 0..d{
            for z in 0..d{
               /* if rand::random::<u64>()%100>50{
                    count += 1;
                    continue;
                }*/
                let dx =-d as f64/div/2.;
                let dy =- d as f64/div/2.;
                let dz =- d as f64/div/2.;
                let t = create_box(Vector3::new(x as f64/div+dx, y as f64/div+dy, z as f64/div +dz), Vector3::new(-x as f64+
                    dx, -y as f64+dy, -z as f64+dz)/10.0, colors[count%colors.len()]);
                    count += 1;
            }

        }
    }

    out
}


fn main() {
   let guard = pprof::ProfilerGuardBuilder::default()
    .frequency(10000)
    .blocklist(&["libc", "libgcc", "pthread", "vdso"])
    .build()
    .unwrap();
    main_loop(Box::new(make_test_level));
   if let Ok(report) = guard.report().build() {
        let file = std::fs::File::create("flamegraph.svg").unwrap();                                                       
        report.flamegraph(file).unwrap();}; 
}
