use chip8_core::*;
use sdl2::event::Event;
use std::env;

/*
 * Inside `chip8_core`, we created public constants to hold the screen size, which we are now
 * importing. However, a 64x32 game window is really small on today's monitors, so let's go ahead
 * and scale it up a bit.
 *
 * Recall that `SCREEN_WIDTH` and `SCREEN_HEIGHT` were public constants we defined in our backend
 * and are now included into this create via the `use chip8_core::*` statement. SDL will require
 * screen size to be `u32` rather than `usize` so we'll cast them here.
 */
const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {
    /*
     * We need to read the command line args to receive the path to our game ROM file. We could
     * create several flags for additional configuration, but we'll keep it simple and say that
     * we'll require exactly one argument - the path to the argument. Any other number and we'll
     * exit out with an error.
     *
     * This grabs all of the passed command line parameters into a vector, and if there isn't two
     * (the name of the program is always stored in args[0]), then we print out the correct input
     * and exit. The path passed in by the user is now stored in args[1]. We'll have to make sure
     * that's a valid file once we attempt to open it, but first, we have some other stuff to
     * setup.
     */
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    /*
     * It's time to create our SDL window! The following code simply creates a new SDL context,
     * then makes the window itself and the canvas that we'll draw upon.
     *
     * We'll initialize SDL and tell it to create a new window of our scaled up size. We'll also
     * have it be created in the middle of the user's screen. We'll then get the canvas object
     * we'll actually draw to, with VSYNC on. Then go ahead and clear it and show it to the user.
     *
     * If you attempt to run ti now, you'll see a window pop up for a brief moment before closing.
     * This is because the SDL window is created briefly, but then the program ends and the window
     * closes. We'll need to create our main game loop so that our program doesn't end immediately,
     * and while we're at it, let's add some handling to quit the program if we try to exit out of
     * the window.
     *
     * SDL uses something called an event pump to poll for events every loop. By checking this, we
     * can cause differente things to happen for given events, such as attempting to close the
     * window or pressin a key. For now. we'll just have the program break out of the main game
     * loop if it need the window to close.
     *
     * This addition sets up our main game loop, which checks if any events have been triggered. If
     * the `Quit` event is detected, then the program breaks out of the loop, causing it to end.
     */

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("CHIP-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                _ => (),
            }
        }
    }
}
