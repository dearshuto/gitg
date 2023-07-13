pub trait ICommandBuffer {
    fn push_point(&mut self, x: f32, y: f32);

    fn push_line(&mut self, begin: [f32; 2], end: [f32; 2]);
}

pub struct Plotter {
    horizontal_offset: f32,
    vertical_offset: f32,
}

impl Default for Plotter {
    fn default() -> Self {
        Plotter::new(1.0, 1.0)
    }
}

impl Plotter {
    pub fn new(horizontal_offset: f32, vertical_offset: f32) -> Self {
        Self {
            horizontal_offset,
            vertical_offset,
        }
    }

    pub fn plot<TCommandBuffer: ICommandBuffer>(&self, command_buffer: &mut TCommandBuffer) {
        command_buffer.push_point(0.0 * self.horizontal_offset, 10.0 * self.vertical_offset);
        command_buffer.push_line([50.0, 50.0], [100.0, 50.0]);
    }
}
