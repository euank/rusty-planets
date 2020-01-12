mod bodies;
use bodies::{Planet, Renderable, Star, World};
use nalgebra::*;
use piston_window::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Rusty Planets", [640, 480])
        .exit_on_esc(true)
        .automatic_close(true)
        .build()
        .unwrap();

    let mut world = World::default();
    world.entities.push(Star::new(window.size()));

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
                // Do nothing
            }
            Event::Loop(Loop::Update(_args)) => {
                // TODO
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
