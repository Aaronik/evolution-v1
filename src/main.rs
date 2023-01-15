use console_engine::crossterm::event::MouseEventKind;
use console_engine::events::Event;
use console_engine::pixel;
use console_engine::Color;
use console_engine::ConsoleEngine;
use console_engine::KeyCode;
use evolution::*;

// TODO
// * Make output neuron effects
// * Add physics for when lifeforms get down to so many, they auto reproduce

fn main() {
    let size = 50;
    let frame_rate = 1000;
    let num_inner_neurons = 3;

    let nnh = NeuralNetHelper::new(num_inner_neurons);

    let world_props = WorldProps {
        size,
        neural_net_helper: &nnh,
        num_initial_lifeforms: 20,
        genome_size: 25,
        mutation_rate: 0.001,
        food_density: 300,
        water_density: 30,
        num_inner_neurons,
        minimum_number_lifeforms: 15,
        // TODO Add num dangers
    };

    let mut world = World::new(world_props);

    // println!("lifeforms: {:#?}", world.lifeforms.values().map(|lf| &lf.genome).collect::<Vec<&Genome>>());

    let mut engine =
        console_engine::ConsoleEngine::init((size * 3) as u32, (size + 2) as u32, frame_rate)
            .unwrap();

    let mut paused = false;

    // TODO ATTOW there's an error in console_engine that disallows any value over 1000 for
    // target_fps. If this becomes a real issue we can switch back to the normal way instead
    // of engine.poll() way. However it'd also be really nice to add an escape hatch to run
    // the evolution and not show anything on the screen.
    loop {
        // Poll next event
        match engine.poll() {
            // A frame has passed
            Event::Frame => {
                if !paused {
                    step(size, &mut engine, &mut world);
                }
            }

            // A Key has been pressed
            Event::Key(keyevent) => {
                if keyevent.code == KeyCode::Char('q') {
                    break;
                }

                if keyevent.code == KeyCode::Char('p') {
                    paused = !paused;
                }

                if keyevent.code == KeyCode::Char('f') {
                    todo!();
                }

                if keyevent.code == KeyCode::Char('e') {
                    todo!();
                    // TODO here could pause this loop, call a fn that has another
                    // loop that just steps. In that fn though need to figure out
                    // how to capture key events.
                    // Alternatively, could have e mean do like 10,000 frames or something
                    // without UI. So like a quick jump into the future.
                }
            }

            // Mouse has been moved or clicked
            Event::Mouse(mouseevent) => {
                if let MouseEventKind::Down(_) = mouseevent.kind {
                    paused = true;
                    let loc = (mouseevent.column as usize, mouseevent.row as usize);
                    let lf = world.lifeform_at_location(&loc);
                    if let Some(lf) = lf {
                        let x = (size + 2) as i32;

                        for i in 0..engine.get_height() {
                            engine.print(x, i as i32, &format!("{: ^100}", " "));
                        }

                        engine.draw();

                        let x = (size + 2) as i32;
                        let y = 0 as i32;
                        engine.print(
                            x,
                            y,
                            &format!("LifeForm {} at {:?}", lf.id, lf.location),
                        );

                        let y = y + 1;

                        engine.print(x, y, "-------");

                        let y = y + 1;

                        for (idx, (neuron_type, neuron)) in
                            lf.neural_net.input_neurons.values().enumerate()
                        {
                            engine.print(
                                x,
                                y + idx as i32,
                                &format!("{:?}: {:?}", neuron_type, neuron.value),
                            );
                        }

                        let y = y + lf.neural_net.input_neurons.len() as i32;

                        engine.print(x, y, "-------");

                        let y = y + 1;

                        let probabilities = lf.run_neural_net(&nnh);

                        // engine.print((size + 2) as i32, ((size / 2) - 1) as i32, &format!("Input genes:"));
                        for (idx, (neuron_type, prob)) in probabilities.iter().enumerate() {
                            engine.print(
                                x,
                                y + idx as i32,
                                &format!("{:?}: {}", neuron_type, prob),
                            );
                        }

                        let y = y + probabilities.len() as i32;

                        engine.print(x, y, "-------");

                        let y = y + 1;

                        // engine.print(x, y, &format!("{:?}", lf.genome.ordered_genes.iter().map(|g| g.).join("-")))

                        engine.draw();
                    }
                }
            }

            // Window has been resized
            Event::Resize(_w, _h) => { /* ... */ }
        }
    }
}

fn step(size: usize, engine: &mut ConsoleEngine, world: &mut World) {
    engine.clear_screen(); // reset the screen

    world.step();

    for lifeform in world.lifeforms.values() {
        engine.set_pxl(
            lifeform.location.0 as i32,
            lifeform.location.1 as i32,
            pixel::pxl_fg('O', Color::White),
        );
    }

    for water in &world.water {
        engine.set_pxl(
            water.0 as i32,
            water.1 as i32,
            pixel::pxl_fg('O', Color::Blue),
        );
    }

    for food in &world.food {
        engine.set_pxl(
            food.0 as i32,
            food.1 as i32,
            pixel::pxl_fg('O', Color::Green),
        );
    }

    for danger in &world.danger {
        engine.set_pxl(
            danger.0 as i32,
            danger.1 as i32,
            pixel::pxl_fg('O', Color::Red),
        );
    }

    // Controls
    engine.print(
        0,
        (engine.get_height() - 1) as i32,
        format!(
            "controls: q = quit | p = pause | f = change frame rate | e = evolve without UI | frame {}",
            engine.frame_count
        )
        .as_str(),
    );

    let stats: Vec<(usize, usize, f32, f32, f32, (usize, usize))> = world
        .lifeforms
        .values()
        .map(|lf| {
            (
                lf.id,
                lf.lifespan,
                lf.health,
                lf.hunger,
                lf.thirst,
                lf.location,
            )
        })
        .collect();

    // let stats = format!("{:#?}", stats);

    // Stats
    engine.line(
        (size + 1) as i32,
        0,
        (size + 1) as i32,
        (engine.get_height() - 2) as i32,
        pixel::pxl('|'),
    );
    engine.print(
        (size + 2) as i32,
        0,
        "Stats: id, lifespan, health, hunger, thirst",
    );
    for (idx, stat) in stats.iter().enumerate() {
        engine.print(
            (size + 2) as i32,
            (idx + 1) as i32,
            &format!("{:.10?}", stat),
        );
    }
    // engine.print((size + 2) as i32, 1, &stats);

    engine.draw(); // draw the screen
}
