use piston_window::*;

// NOTE: You do NOT need to use nalgebra. It's hairy but has great vector
// methods, including: ::norm() (2nd norm), ::normalize() (unit vector),
// and substraction to get a directional force vector.
//
// You can do all of this by hand with [f64; 2].
use nalgebra::{Point2, Vector2};

/// Gravitational Constant in m^3*kg^−1*s^−2
const G: f64 = 6.674e-11f64;

// How long to scale each second to
const TIME_SCALE: f64 = 1000.0 * 365.0 * 24.0 * 60.0 * 60.0;

/***********************
 * Trait and struct definitions.
 * At minimum, set and tick need modification to get something usable.
 *
 * Just fill in the empty tuples, the empty methods in "impl", and complete fn main().
 ***********************/
pub trait Renderable {
    fn render(&self, ctx: &Context, graphics: &mut G2d);
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

#[derive(Default)]
pub struct World {
    pub entities: Vec<Box<dyn Entity>>,
}

impl World {
    pub fn tick(&mut self, dt: f64) {
        let mut new_states = Vec::with_capacity(self.entities.len());
        for (i, entity) in self.entities.iter().enumerate() {
            let other_entities: Vec<_> = (&self.entities[0..i]).iter().chain((&self.entities[i+1..]).iter()).collect();
            new_states.push(entity.tick(dt, other_entities.as_slice()));
        }
        for (i, entity) in self.entities.iter_mut().enumerate() {
            entity.update(new_states[i]);
        }
    }
}

impl Renderable for World {
    fn render(&self, context: &Context, graphics: &mut G2d) {
        for entity in &self.entities {
            entity.render(context, graphics);
        }
    }
}

// some ideas
#[derive(Debug)]
pub struct Planet {
    state: PhysicsState,
    size: f64,
    mass: f64,
    color: [f32; 4],
}

/***********************
 * Trait definitions end
 ***********************/

impl Planet {
    // You will probably want one of these
    pub fn new() -> Self {
        Planet {
            state: PhysicsState{
                velocity: Vector2::from([0.0; 2]),
                position: Point2::from([0.0; 2]),
            },
            size: 1.0,
            mass: 1.0,
            color: [1.0; 4], // white, RGB and last is alpha.
        }
    }
}

impl Renderable for Planet {
    fn render(&self, context: &Context, graphics: &mut G2d) {
        let extents = ellipse::circle(self.state.position[0], self.state.position[1], self.size);

        // example:
        rectangle(self.color, extents, context.transform, graphics);
    }
}

impl PhysicsBody for Planet {
    // Need to return some data to calculate the next frame...
    // If you keep this method and feed its output into render, this is where the math is.
    //
    // Billy: math took me a bit to get 100% right; feel free to ask for a tip.
    fn tick(&self, dt: f64, other_entities: &[&Box<dyn Entity>]) -> PhysicsState {

        // The force due to gravity
        let force: Vector2<_> = other_entities.iter().map(|e| {
            let f = G * (self.mass * e.mass()) / (nalgebra::distance_squared(&self.state.position, &e.position()));
            f * (e.position() - self.state.position).normalize()
        }).sum();

        let new_velocity = self.state.velocity + force;

        PhysicsState {
            position: self.state.position + (self.state.velocity + force) * dt * TIME_SCALE,
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
    color: [f32; 4],
    mass: f64,
    size: f64,
}

impl Star {
    pub fn new(window_size: Size) -> Box<Star> {
        Box::new(Star {
            state: PhysicsState {
                position: Point2::from([window_size.width / 2.0, window_size.height / 2.0]),
                velocity: Vector2::new(0.0, 0.0),
            },
            color: [1.0, 1.0, 0.8, 1.0],
            mass: 1000.0,
            size: 15.0,
        })
    }
}

impl PhysicsBody for Star {
    // Let's pretend the star doesn't move to reduce nuttiness. You can just
    // return relevant phyiscs details here.
    fn tick(&self, dt: f64, world: &[&Box<dyn Entity>]) -> PhysicsState {
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
    fn render(&self, context: &Context, graphics: &mut G2d) {
        let extents = ellipse::circle(self.state.position[0], self.state.position[1], self.size);

        rectangle(self.color, extents, context.transform, graphics);
    }
}
