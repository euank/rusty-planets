use std::f64::consts::FRAC_PI_4;

mod bodies;
use bodies::*;
use nalgebra::*;
use piston_window::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Rusty Planets", [640, 480])
        .exit_on_esc(true)
        .automatic_close(true)
        .build()
        .unwrap();

    let mut world = World::default();
    let star = Star::new(window.size());
    let star_pos = star.position();
    world.entities.push(Star::new(window.size()));
    world.entities.push({
        let mut p = Planet::new();
        let wsize = window.size();
        let pos = Point2::new(wsize.width / 2.0 + 200.0, wsize.height / 2.0 + 200.0);
        let vec_to_sun: Vector2<f64> = (star_pos - pos).normalize();
        //let vec_perp_to_sun: Vector2<f64> = Rotation2::new(FRAC_PI_4) * vec_to_sun;
        let vec_perp_to_sun: Vector2<f64> = vec_to_sun * Rotation2::new(FRAC_PI_4);
        p.update(PhysicsState{
            position: pos,
            velocity: vec_perp_to_sun * 0.2,
        });
        Box::new(p)
    });

    while let Some(event) = window.next() {
        println!("event: {:?}", event);
        match &event {
            Event::Input(input, _timestamp) => match handle_input(input) {
                Some(Action::Close) => return,
                None => {}
            },
            Event::Loop(Loop::Render(args)) => {
                let window_size = args.window_size;
                // TODO, use window_size
                window.draw_2d(&event, |context, graphics, _device| {
                    // Set the background to hipster grey.
                    // Also clears anything previously drawn.
                    clear([0.1, 0.1, 0.1, 1.0], graphics);
                    world.render(&context, graphics);
                });
            }
            Event::Loop(Loop::AfterRender(_args)) => {
                // Update planets immediately after rendering them
                // Do nothing
            }
            Event::Loop(Loop::Update(args)) => {
                world.tick(args.dt);
            }
            Event::Loop(Loop::Idle(_args)) => {
                // Do nothing
            }
            Event::Custom(id, ev, _timestamp) => {
                unimplemented!(
                    "no custom events expected or handled: id {:?}, ev {:?}",
                    id,
                    ev
                );
            }
        }
    }
}

enum Action {
    Close,
}

fn handle_input(input: &Input) -> Option<Action> {
    match input {
        Input::Text(s) => match s.as_ref() {
            "q" => Some(Action::Close),
            _ => None,
        },
        _ => None,
    }
}
