use winit::dpi::PhysicalSize;

const MIN_ZOOM: f32 = 0.05;
const MAX_ZOOM: f32 = 200.0;
const ZOOM_PER_SCROLL_STEP: f32 = 1.15;
const TARGET_MAJOR_GRID_PX: f32 = 96.0;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub center_zoom: [f32; 4],
    pub viewport_grid: [f32; 4],
}

pub struct CameraController {
    center: [f32; 2],
    zoom: f32,
    viewport: [f32; 2],
    cursor: [f32; 2],
    panning: bool,
    major_grid: f32,
    minor_grid: f32,
}

impl CameraController {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        let mut camera = Self {
            center: [size.width as f32 * 0.5, size.height as f32 * 0.5],
            zoom: 1.0,
            viewport: [size.width.max(1) as f32, size.height.max(1) as f32],
            cursor: [size.width as f32 * 0.5, size.height as f32 * 0.5],
            panning: false,
            major_grid: 100.0,
            minor_grid: 20.0,
        };
        camera.update_grid_spacing();
        camera
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.viewport = [size.width.max(1) as f32, size.height.max(1) as f32];
        self.cursor = [
            self.cursor[0].clamp(0.0, self.viewport[0]),
            self.cursor[1].clamp(0.0, self.viewport[1]),
        ];
        self.update_grid_spacing();
    }

    pub fn set_pan_active(&mut self, active: bool) {
        self.panning = active;
    }

    pub fn on_cursor_moved(&mut self, x: f32, y: f32) {
        let new_cursor = [x, y];
        if self.panning {
            let dx = new_cursor[0] - self.cursor[0];
            let dy = new_cursor[1] - self.cursor[1];
            self.center[0] -= dx / self.zoom;
            self.center[1] -= dy / self.zoom;
        }
        self.cursor = new_cursor;
    }

    pub fn zoom_by_scroll(&mut self, scroll_steps: f32) {
        if scroll_steps.abs() <= f32::EPSILON {
            return;
        }

        let old_zoom = self.zoom;
        let new_zoom =
            (old_zoom * ZOOM_PER_SCROLL_STEP.powf(scroll_steps)).clamp(MIN_ZOOM, MAX_ZOOM);
        if (new_zoom - old_zoom).abs() <= f32::EPSILON {
            return;
        }

        let world_before = self.screen_to_world(self.cursor, old_zoom);
        self.zoom = new_zoom;
        let world_after = self.screen_to_world(self.cursor, self.zoom);
        self.center[0] += world_before[0] - world_after[0];
        self.center[1] += world_before[1] - world_after[1];
        self.update_grid_spacing();
    }

    pub fn uniform(&self) -> CameraUniform {
        CameraUniform {
            center_zoom: [self.center[0], self.center[1], self.zoom, 0.0],
            viewport_grid: [
                self.viewport[0],
                self.viewport[1],
                self.major_grid,
                self.minor_grid,
            ],
        }
    }

    pub fn center(&self) -> [f32; 2] {
        self.center
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn view_world_size(&self) -> [f32; 2] {
        [self.viewport[0] / self.zoom, self.viewport[1] / self.zoom]
    }

    fn screen_to_world(&self, screen: [f32; 2], zoom: f32) -> [f32; 2] {
        [
            self.center[0] + (screen[0] - self.viewport[0] * 0.5) / zoom,
            self.center[1] + (screen[1] - self.viewport[1] * 0.5) / zoom,
        ]
    }

    fn update_grid_spacing(&mut self) {
        let desired_world = TARGET_MAJOR_GRID_PX / self.zoom.max(1e-6);
        let pow10 = 10.0_f32.powf(desired_world.log10().floor());
        let norm = desired_world / pow10;
        let nice = if norm < 1.5 {
            1.0
        } else if norm < 3.0 {
            2.0
        } else if norm < 7.0 {
            5.0
        } else {
            10.0
        };
        self.major_grid = (nice * pow10).max(1e-6);
        self.minor_grid = (self.major_grid / 5.0).max(1e-6);
    }
}
