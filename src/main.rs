use ::image as im;
use std::f64::consts::FRAC_PI_2;

mod bodies;
mod data;
use bodies::*;
use nalgebra::*;
use piston_window::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Rusty Planets", [1000, 1000])
        .exit_on_esc(true)
        .automatic_close(true)
        .build()
        .unwrap();

    let planets = data::get_planetdata();

    let mut world = World::new();
    let star: Box<Star> = Star::new();
    world.entities.push(star);
    for planet in planets {
        world.entities.push(Box::new(Planet::from_data(planet)));
    }
    //world.entities.push({
    //    let mut p: Planet = Planet::new();
    //    let pos = Point2::new(1.0, 0.0); // earth au from sun
    //    let vec_direction = Rotation2::new(FRAC_PI_2) * (Point2::new(0.0, 0.0) - pos).normalize();
    //    p.update(PhysicsState{
    //        position: pos,
    //        velocity: vec_direction * 1.9907e-7, // earth velocity in au/s
    //    });
    //    Box::new(p)
    //});

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut canvas = im::ImageBuffer::new(1000, 1000);
    let mut texture =
        Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();

    while let Some(event) = window.next() {
        println!("event: {:?}", event);
        match &event {
            Event::Input(input, _timestamp) => match handle_input(input) {
                Some(Action::Close) => return,
                None => {}
            },
            Event::Loop(Loop::Render(args)) => {
                if canvas.width() != args.draw_size[0] as u32
                    || canvas.height() != args.draw_size[1]
                {
                    canvas = im::ImageBuffer::new(args.draw_size[0], args.draw_size[1]);
                } else {
                    // otherwise clear the image buffer
                    let (prev_w, prev_h) = (canvas.width(), canvas.height());
                    let mut vec: Vec<_> = canvas.into_raw();
                    // memset 0, but safe. Should optimize to memset
                    for p in vec.iter_mut() {
                        *p = 0;
                    }
                    canvas = im::ImageBuffer::from_raw(prev_w, prev_h, vec).unwrap();
                }
                world.render(&mut canvas);
                texture.update(&mut texture_context, &canvas).unwrap();
                window.draw_2d(&event, |context, graphics, device| {
                    texture_context.encoder.flush(device);
                    // Set the background to hipster grey.
                    // Also clears anything previously drawn.
                    clear([0.1, 0.1, 0.1, 1.0], graphics);
                    image(&texture, context.transform, graphics);
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
