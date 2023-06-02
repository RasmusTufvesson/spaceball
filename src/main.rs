extern crate image as im;

use piston_window::*;
use glam::{
    DVec2,
};
use opengl_graphics::{OpenGL};
use rand::{self, Rng, distributions::Standard, prelude::Distribution};
use imageproc::{drawing::{draw_polygon_mut, draw_filled_rect_mut, draw_filled_circle_mut}, rect::Rect, point::Point};

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const WHITE_IM: im::Rgba<u8> = im::Rgba([255, 255, 255, 255]);
const BLACK_IM: im::Rgba<u8> = im::Rgba([0, 0, 0, 255]);
const TRIANGLE: [DVec2; 3] = [DVec2 { x: -0.5, y: -0.5 }, DVec2 { x: 0.5, y: -0.5 }, DVec2 { x: 0.0, y: 0.36602540378443864676372317075294 }];

enum ShapeType {
    Circle,
    Square,
    Triangle,
    Hexagon,
}

impl Distribution<ShapeType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ShapeType {
        match rng.gen_range(0..=2) {
            0 => ShapeType::Circle,
            1 => ShapeType::Triangle,
            _ => ShapeType::Square,
        }
    }
}

struct Shape {
    pub position: DVec2,
    pub shape_type: ShapeType,
    pub speed: DVec2,
    pub radius: f64,
    pub image: im::ImageBuffer<im::Rgba<u8>, Vec<u8>>,
}

fn draw_shape(shape_type: &ShapeType, radius: f64) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let mut img: im::ImageBuffer<im::Rgba<u8>, Vec<_>> = im::ImageBuffer::new((radius * 2.0).round() as u32, (radius * 2.0).round() as u32);
    match shape_type {
        ShapeType::Circle => {
            draw_filled_circle_mut(&mut img, (
                radius.round() as i32, radius.round() as i32
            ), radius.round() as i32, WHITE_IM);
        }
        ShapeType::Square => {
            draw_filled_rect_mut(&mut img,
                Rect::at(
                    0,
                    0,
                ).of_size(
                    (radius * 2.0).round() as u32,
                    (radius * 2.0).round() as u32,
                )
            .into(), WHITE_IM)
        }
        ShapeType::Triangle => {
            let world_pos = DVec2::new(radius, radius);
            let triangle = [
                to_point(from_relative(TRIANGLE[0] * radius, world_pos)),
                to_point(from_relative(TRIANGLE[1] * radius, world_pos)),
                to_point(from_relative(TRIANGLE[2] * radius, world_pos)),
            ];
            draw_polygon_mut(&mut img, &triangle, WHITE_IM);
        }
        _ => {}
    }
    return img;
}

struct World {
    pub shapes: Vec<Shape>,
}

pub struct App {
    world: World,
    half_size: DVec2,
    rng: rand::rngs::ThreadRng,
    size: DVec2,
}

impl App {
    fn update(&mut self, args: &UpdateArgs) {
        // args.dt
        let mut to_remove: Vec<usize> = vec![];
        for n in 0..self.world.shapes.len() {
            let speed = self.world.shapes[n].speed;
            self.world.shapes[n].position += speed * args.dt;
            let position = self.world.shapes[n].position;
            let radius = self.world.shapes[n].radius;
            if position.x < -self.half_size.x - radius {
                self.world.shapes[n].position.x = self.half_size.x + radius
            } else if position.x > self.half_size.x + radius {
                self.world.shapes[n].position.x = -self.half_size.x - radius
            }
            if position.y < -self.half_size.y - radius {
                self.world.shapes[n].position.y = self.half_size.y + radius
            } else if position.y > self.half_size.y + radius {
                self.world.shapes[n].position.y = -self.half_size.y - radius
            }
            self.world.shapes[n].radius -= 0.1 * args.dt;
            if self.world.shapes[n].radius.round() <= 1.0 {
                to_remove.push(n);
            } else if self.world.shapes[n].radius.round() != radius.round() {
                self.world.shapes[n].image = draw_shape(&self.world.shapes[n].shape_type, self.world.shapes[n].radius);
            }
        }
        for index in to_remove.into_iter().rev() {
            self.world.shapes.remove(index);
        }
    }

    fn keydown(&mut self, button: &Button) {
        match button {
            Button::Keyboard(_key) => {
                if self.world.shapes.len() <= 50 {
                    let position = DVec2::new((self.rng.gen::<f64>() * self.size.x - self.half_size.y) * 0.75, (self.rng.gen::<f64>() * self.size.y - self.half_size.y) * 0.75);
                    let speed = DVec2::new(self.rng.gen::<f64>() * 450.0 - 225.0, self.rng.gen::<f64>() * 450.0 - 225.0);
                    let radius = self.rng.gen::<f64>() * 17.0 + 3.0;
                    let shape_type = self.rng.gen::<ShapeType>();
                    let image = draw_shape(&shape_type, radius);
                    let shape = Shape { position, shape_type, speed, radius, image };
                    self.world.shapes.push(shape);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;
    let (width, height) = (700, 500);
    let size = DVec2::new(width as f64, height as f64);
    let half_size = size / 2.0;
    let mut window: PistonWindow =
        WindowSettings::new("Spaceballs", (width, height))
        .resizable(false)
        .graphics_api(opengl)
        .build()
        .unwrap();

    let mut canvas: im::ImageBuffer<im::Rgba<u8>, Vec<_>> = im::ImageBuffer::new(width, height);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &canvas,
        &TextureSettings::new()
    ).unwrap();

    let mut app = App {
        world: World { shapes: vec![] },
        half_size: half_size.clone(),
        rng: rand::thread_rng(),
        size: size.clone(),
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(_args) = e.render_args() {
            window.draw_2d(&e, |c: Context, g, device| {
                clear(BLACK, g);
                for p in canvas.pixels_mut() {
                    *p = BLACK_IM;
                }
                for shape in &app.world.shapes {
                    let coords = [
                        (half_size.x + shape.position.x - shape.radius).round() as i64,
                        (half_size.y - shape.position.y - shape.radius).round() as i64,
                    ];
                    im::imageops::overlay(&mut canvas, &shape.image, coords[0], coords[1]);
                }
                texture.update(&mut texture_context, &canvas).unwrap();
                texture_context.encoder.flush(device);
                image(&texture, c.transform, g);
            });
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(button) = e.press_args() {
            app.keydown(&button);
        }
    }
}

fn to_point(vec: DVec2) -> Point<i32> {
    Point { x: vec.x.round() as i32, y: vec.y.round() as i32 }
}

fn from_relative(vec: DVec2, position: DVec2) -> DVec2 {
    DVec2 { x: position.x + vec.x, y: position.y + vec.y }
}