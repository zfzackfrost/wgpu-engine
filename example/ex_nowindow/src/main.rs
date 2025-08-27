use wgpu_engine::third_party::pollster;

fn main() {
    pollster::block_on(ex_nowindow::run());
}
