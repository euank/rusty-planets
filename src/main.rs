use ::image as im;

mod bodies;
mod data;
use bodies::*;
use piston_window::*;

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Rusty Planets", [1000, 1000])
        .exit_on_esc(true)
        .resizable(true)
        .graphics_api(OpenGL::V3_2)
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
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut canvas = im::ImageBuffer::new(640, 640);
    let mut texture =
        Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();

    while let Some(event) = window.next() {
        //println!("event: {:?}", event);
        match &event {
            Event::Input(input, _timestamp) => match handle_input(input) {
                Some(Action::Close) => {
                    window.set_should_close(true);
                    return
                },
                Some(Action::SpeedUp) => {
                    world.speed_up();
                },
                Some(Action::SlowDown) => {
                    world.slow_down();
                },
                Some(Action::ZoomOut) => {
                    world.zoom_out();
                },
                Some(Action::ZoomIn) => {
                    world.zoom_in();
                },
                None => {}
            },
            Event::Loop(Loop::Render(args)) => {
                // TODO: dpi only changes if the window switches screens, but the winit dpi
                // changed event isn't exposed in this eventloop, so I'm stuck with doing this
                // every iteration.
                // Upstream should probably expose that event.
                let dpi = window.window.ctx.window().get_hidpi_factor();
                if canvas.width() != args.draw_size[0]
                    || canvas.height() != args.draw_size[1]
                {
                    canvas = im::ImageBuffer::new(args.draw_size[0], args.draw_size[1]);
                    texture = Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();
                } else {
                    // otherwise clear the image buffer
                    let (prev_w, prev_h) = (canvas.width(), canvas.height());
                    let mut vec: Vec<_> = canvas.into_raw();
                    // memset 0, but safe. Should optimize to memset
                    for p in vec.iter_mut() {
                        *p = 10;
                    }
                    canvas = im::ImageBuffer::from_raw(prev_w, prev_h, vec).unwrap();
                }
                canvas = world.render(canvas);
                texture.update(&mut texture_context, &canvas).unwrap();
                window.draw_2d(&event, |c, graphics, device| {
                    let view = c.transform.scale(1.0 / dpi, 1.0 / dpi);
                    let transform = view.trans(0.0, 0.0);
                    texture_context.encoder.flush(device);
                    // Set the background to hipster grey.
                    // Also clears anything previously drawn.
                    clear([0.1, 0.1, 0.1, 1.0], graphics);
                    image(
                        &texture,
                        transform,
                        graphics,
                    );
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
    SpeedUp,
    SlowDown,
    ZoomOut,
    ZoomIn,
}

fn handle_input(input: &Input) -> Option<Action> {
    match input {
        Input::Text(s) => match s.as_ref() {
            "q" => Some(Action::Close),
            "+" => Some(Action::SpeedUp),
            "-" => Some(Action::SlowDown),
            "z" => Some(Action::ZoomIn),
            "x" => Some(Action::ZoomOut),
            _ => None,
        },
        _ => None,
    }
}
