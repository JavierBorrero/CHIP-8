use chip8_core::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::fs::File;
use std::io::Read;

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
const TICKS_PER_FRAME: usize = 10;

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
     * the `Quit` event is detected, then the program breaks out of the loop.
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

    /*
     * Creating the `Emu` object needs to go somewhere prior to our main game loop, as that is
     * where the emulation drawing and key press handling will go.
     */
    let mut chip8 = Emu::new();

    /*
     * A few things to nothe here. In the event that Rust is unable to open the file from the path
     * the user gaves then the `expect` condition will fail and the program will exit with that
     * message.
     */
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    /*
     * At this point the game has been loaded into RAM and our main loop is running. Now we need to
     * tell our backend to begin processing its instructions, and to actually draw to the screen.
     * If you recall, the emulator runs through a clock cycle each time its `tick` function is
     * called, so let's add that to our loop.
     */
    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } => {
                    break 'gameloop;
                }
                /*
                 * Each event will check if the pressed key gives a `Some` value from our `key2btn`
                 * function, and if so pass it to the emulator via the public `keypress` function
                 * we defined earlier. The only difference between the two will be if it sets or
                 * clears
                 */
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, false);
                    }
                }
                _ => (),
            }
        }

        /*
         * The emulation `tick` speed should probably run faster than the canvas refresh rate. If
         * you watch your game run, it might feel a bit sluggish. Right now, we execute one
         * instruction, then draw to the screen, then repeat. As you're aware, it takes several
         * instructions to be able to do any meaningful changes to the screen. To get around this,
         * we will allow the emulator to tick several times before redrawing.
         *
         * The CHIP-8 specification says nothing about how quickly the system should actually run.
         * Even leaving it is now so it runs at 60Hz is a valid solution. We'll simply allow our
         * `tick` function to loop several times before moving on to drawing the screen.
         * Personally, I find that 10 ticks per frame is a nice sweet spot
         */
        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }

        /*
         * If you run again, you might notice that it doesn't get very far before pausing. This is
         * likely due to the fact that we never update our two timers, so the emulator has no
         * concept of how long time has passed for its games. I mentioned earlier that the timers
         * run once per frame, rather than at the clock speed, so we can modify the timers at the
         * same point as when we modify the screen.
         */
        chip8.tick_timers();

        draw_screen(&chip8, &mut canvas);
    }
}

/*
 * This function will take in a reference to our `Emu` object, as well as a mutable reference to
 * our SDL canvas. Drawing the screen requires a few steps. First, we clear the canvas to erase the
 * previous frame. Then, we iterate through the screen buffer, drawing a white rectangle anytime
 * the given value is true. Sinche CHIP-8 only supports black and white, if we clear the screen as
 * black, we only have to worry about drawing the white squares.
 */
fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>) {
    // Clear canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();

    // Now set draw color to white, iterate through each point and see if it should be drawn
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            // Convert our 1D array's index into a 2D (x, y) position
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_HEIGHT) as u32;

            // Draw a rectable at (x, y), scaled up by our SCALE value
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

/*
 * We can finally render our CHIP-8 game to the screen, but we can't get very far into playing it
 * as we have no way to control it. Fortunately, SDL supports reading in inputs to the keyboard
 * which we can translate and send to our backend emulation.
 *
 * The CHIP-8 system supports 16 different keys. These are typically organized in a 4x4 grid, with
 * keys 0-9 organized like a telephone with keys A-F surrounding. While you are welcome to organize
 * the keys in any configuration you like, some game devs assumed they're in the grid pattern when
 * choosing their games inputs, which means it can be awkward to play some games otherwise. For our
 * emulator, we'll use the left-hand keys of the QWERTY keyboard as our inputs, as shown below
 *
 * Let's create a function to convert SDL's key type into the values that we will send to the
 * emulator. We'll need to bring SDL keyboard support.
 */
fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
