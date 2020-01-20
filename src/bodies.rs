use ::image::ImageBuffer;
use ::image::Rgba;

// NOTE: You do NOT need to use nalgebra. It's hairy but has great vector
// methods, including: ::norm() (2nd norm), ::normalize() (unit vector),
// and substraction to get a directional force vector.
//
// You can do all of this by hand with [f64; 2].
use nalgebra::{Point2, Vector2};

/// Gravitational Constant in km^3/(kg*s^2)
const G: f64 = 6.67430e-20;

const MIN_PIXEL_SIZE: u32 = 2;

type Buffer = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub struct ScaledBuffer {
    inner: Buffer,
    width: f64,
}

impl ScaledBuffer {
    fn new(buf: Buffer, width: f64) -> Self {
        ScaledBuffer{
            inner: buf,
            width,
        }
    }
    fn into_buffer(self) -> Buffer {
        self.inner
    }

    fn map_point(&self, pos: Point2<f64>) -> Point2<u32> {
        // 0.0 is in the center in universe coordinates, but upper-left in image coordinates. Do
        // that shifting too.
        let width = self.inner.width() as f64;
        let height = self.inner.height() as f64;

        let universe_width = self.width * 2.0;
        let universe_height = universe_width * height / width;


        // what percent x and y were are at relative to 0;
        let x_fraction = 0.5 + (pos.x / universe_width / 2.0);
        let y_fraction = 0.5 + (pos.y / universe_height / 2.0);

        Point2::new((x_fraction * width) as u32, (y_fraction * height) as u32)
    }

    fn map_length(&self, len: f64) -> u32 {
        let width = self.inner.width() as f64;
        let universe_width = self.width * 2.0;
        return (len * width / universe_width) as u32;
    }

    fn draw_circle(&mut self, center: Point2<f64>, radius: f64, color: Rgba<u8>) {
        let mcenter = self.map_point(center);
        let mut radius = self.map_length(radius) as i32;
        if radius * 2 < MIN_PIXEL_SIZE as i32 {
            radius = MIN_PIXEL_SIZE as i32;
        }

        for x in (-radius)..=radius {
            for y in (-radius)..=radius {
                if (x * x + y * y) < (radius * radius) {
                    let ix = mcenter.x as i32 + x;
                    let iy = mcenter.y as i32 + y;
                    if ix < 0 || iy < 0 || ix >= self.inner.width() as i32 || iy >= self.inner.height() as i32 {
                        continue;
                    }
                    self.inner.put_pixel(ix as u32, iy as u32, color);
                }
            }
        }
    }
}

/***********************
 * Trait and struct definitions.
 * At minimum, set and tick need modification to get something usable.
 *
 * Just fill in the empty tuples, the empty methods in "impl", and complete fn main().
 ***********************/
pub trait Renderable {
    fn render(&self, image: &mut ScaledBuffer);
}

pub trait PhysicsBody {
    fn mass(&self) -> f64;
    fn position(&self) -> Point2<f64>;

    fn tick(&self, dt: f64, w: &[&Box<dyn Entity>]) -> PhysicsState;
    fn update(&mut self, to: PhysicsState);
}

// These are the parts of an entities physics that are able to change over time
#[derive(Copy, Clone, Debug)]
pub struct PhysicsState {
    pub position: Point2<f64>,
    pub velocity: Vector2<f64>,
}

pub trait Entity: PhysicsBody + Renderable {}

impl<T> Entity for T where T: PhysicsBody + Renderable {}

pub struct World {
    pub entities: Vec<Box<dyn Entity>>,

    // How long to scale each second to
    time_scale: f64,
    width: f64,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            time_scale: 100.0 * 24.0 * 60.0 * 60.0,
            width: 150_000_000.0,
        }
    }

    pub fn zoom_out(&mut self) {
        self.width += 1_000_000.0;
    }

    pub fn zoom_in(&mut self) {
        self.width -= 1_000_000.0;
        if self.width < 1_000.0 {
            self.width = 1000.0;
        }
    }

    pub fn speed_up(&mut self) {
        self.time_scale += 10.0 * 24.0 * 60.0 * 60.0;
    }

    pub fn slow_down(&mut self) {
        self.time_scale -= 10.0 * 24.0 * 60.0 * 60.0;
        if self.time_scale <= 0.00 {
            self.time_scale = 0.0;
        }
    }

    pub fn tick(&mut self, dt: f64) {
        let dt = dt * self.time_scale;
        let mut new_states = Vec::with_capacity(self.entities.len());
        for (i, entity) in self.entities.iter().enumerate() {
            let other_entities: Vec<_> = (&self.entities[0..i])
                .iter()
                .chain((&self.entities[i + 1..]).iter())
                .collect();
            new_states.push(entity.tick(dt, other_entities.as_slice()));
        }
        for (i, entity) in self.entities.iter_mut().enumerate() {
            entity.update(new_states[i]);
        }
    }

    pub fn render(&self, canvas: Buffer) -> Buffer {
        let mut buf = ScaledBuffer::new(canvas, self.width);
        for entity in &self.entities {
            entity.render(&mut buf);
        }
        buf.into_buffer()
    }
}

// some ideas
#[derive(Debug)]
pub struct Planet {
    state: PhysicsState,
    size: f64,
    mass: f64,
    color: [u8; 4],
}

/***********************
 * Trait definitions end
 ***********************/

impl Planet {
    pub fn from_data(d: super::data::PlanetData) -> Self {
        Planet {
            size: 0.0,               // TODO
            mass: d.mass * 10e24f64, // kg
            color: [244; 4],         // white
            state: PhysicsState {
                velocity: Vector2::new(0.0, d.orbital_velocity),
                position: Point2::new(d.distance_from_sun * 1_000_000.0, 0.0),
            },
        }
    }
}

impl Renderable for Planet {
    fn render(&self, canvas: &mut ScaledBuffer) {
        canvas.draw_circle(self.state.position, self.size, Rgba(self.color));
    }
}

impl PhysicsBody for Planet {
    // Need to return some data to calculate the next frame...
    // If you keep this method and feed its output into render, this is where the math is.
    //
    // Billy: math took me a bit to get 100% right; feel free to ask for a tip.
    fn tick(&self, dt: f64, other_entities: &[&Box<dyn Entity>]) -> PhysicsState {
        // The force due to gravity
        let force: Vector2<_> = other_entities
            .iter()
            .map(|e| {
                let f = G * (self.mass * e.mass())
                    / (nalgebra::distance_squared(&self.state.position, &e.position()));
                let force = f * (e.position() - self.state.position).normalize();
                force
            })
            .sum();

        let new_velocity = self.state.velocity + (force * dt / self.mass);

        PhysicsState {
            position: self.state.position + new_velocity * dt,
            velocity: new_velocity,
        }
    }

    fn update(&mut self, to: PhysicsState) {
        self.state = to
    }

    fn mass(&self) -> f64 {
        self.mass
    }
    fn position(&self) -> Point2<f64> {
        self.state.position
    }
}

pub struct Star {
    state: PhysicsState,
    color: [u8; 4],
    mass: f64,
    size: f64,
}

impl Star {
    pub fn new() -> Box<Star> {
        Box::new(Star {
            state: PhysicsState {
                position: Point2::new(0.0, 0.0),
                velocity: Vector2::new(0.0, 0.0),
            },
            color: [255, 255, 200, 255],
            mass: 1.98850e30, // kg
            size: 0.0,        // fake width
        })
    }
}

impl PhysicsBody for Star {
    fn tick(&self, _dt: f64, _other_entities: &[&Box<dyn Entity>]) -> PhysicsState {
        // pretend the sun doesn't move
        self.state
    }

    fn update(&mut self, to: PhysicsState) {
        self.state = to;
    }

    fn mass(&self) -> f64 {
        self.mass
    }
    fn position(&self) -> Point2<f64> {
        self.state.position
    }
}

impl Renderable for Star {
    fn render(&self, image: &mut ScaledBuffer) {
        image.draw_circle(self.state.position, self.size, Rgba(self.color));
    }
}
