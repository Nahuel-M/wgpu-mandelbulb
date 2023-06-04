use wgpu_mandelbulb::run;

fn main() {
    pollster::block_on(run());
}
