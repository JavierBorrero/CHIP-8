use std::usize;

use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONT_SIZE: usize = 80;

/*
 * The CHIP-8 screen display renders sprites which are stored in memory to the
 * screen, one line at a time. It is up to the gamedev to correctly load their sprites
 * before copying them over. Wouldn't it be nice if the system automatically had sprites
 * for commonly used things, such as numbers?. The PC starts at 0x200, leaving the
 * first 512 intentionally empty. Modern emulators will use that space to store the sprite
 * data for hex digits: 0-9 and A-F. We could store this data at any fixed position in RAM,
 * but this space is already defined as empty anyway.
 *
 * Each character is made up of five rows of eight pixels, with each row using a byte
 * of data, meaning that each letter altogether takes up five bytes of data. The following
 * diagram illustrates how a character is stored as bytes:
 *
 * 00100000
 * 01100000
 * 00100000
 * 00100000
 * 01110000
 *
 * Each pixel is assigned a bit, which corresponds to wether that pixel will be white or black.
 * Every sprite in CHIP-8 is eight pixels wide, which means a pixel row requires 8-bits (1 byte).
 * The above diagram shows the layour of the "1" character sprite
 */

const FONTSET: [u8; FONT_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emu {
    pc: u16, // program counter
    /*
     * [u8; RAM_SIZE] esto es un array
     *
     * u8 -> es el tipo de cada elemento (unsigned int 8bit)
     * RAM_SIZE -> numero de elementos del array (4096)
     */
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS], // v registers
    i_reg: u16,            // index register
    sp: u16,               // stack pointer
    stack: [u16; STACK_SIZE],
    // array de booleans para trackear el estado de los botones pulsados
    keys: [bool; NUM_KEYS],
    dt: u8, // delay timer
    st: u8, // sound timer
}

/*
 * `impl` significa implementacion y se usa para definir
 * metodos (funciones asociadas) o agregar funcionalidades
 * a una estructura (struct), enumeracion (enum) o trait.
 *
 * `pub fn new() -> Self` es la funcion new de la struct Emu.
 * Self se refiere al mismo tipo de la impl, en lugar de escribir
 * otra vez Emu se pone Self, que queda mas claro y si algun dia cambia
 * Emu por otro nombre es un lugar menos en el que lo tendrias que cambiar
 *
 * Esta funcion se usaria de la siguiente manera en main.rs:
 *
 * let mut emulator = Emu::new()
 *
 */

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        /*
         * new_emu.ram[..FONT_SIZE] desde el principio del array (0) hasta FONT_SIZE (80)
         * Copiar el contenido de &FONTSET a ese rango de RAM
         *
         * [..] es un operador de rango. Se usa para intervalos de valores
         *
         * start..end -> Desde start hasta (sin incluir) end
         * start..=end -> Desde start hasta e incluyendo end
         * ..end -> Desde el principio hasta SIN INCLUIR end
         * ..=end -> Desde el principio hasta e incluyendo end
         * start.. -> Desde start hasta el final (cuando no se especifica final)
         * .. -> Todo el rango disponible
         */
        new_emu.ram[..FONT_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    /*
     * Resetear el estado de la CPU, todos los valores de vuelta al inicial
     *
     * PC en 0x200
     * Poner la RAM todo en 0
     * Todos los valores del screen en false
     * Poner todos los V Registers en 0
     * Poner el Index Register en 0
     * Poner el SP en 0
     * Vaciar el stack
     * Cambiar todo el estado de las keys a false
     * Delay Timer y Sound Timer en 0
     * Volver a poner el FONT_SIZE y el FONTSET en el espacio especificado
     *
     */
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONT_SIZE].copy_from_slice(&FONTSET);
    }

    // Pass a pointer to our screen buffer array up to the frontend, where it can be used to render
    // to the display
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    /*
     * We need to handle key presses. We already have a `keys` array, but it never actually gets
     * written to. Our frontend will handle actually reading keyboard presses, but we'll need to
     * expose a function that allows it to interface and set elements in our `keys` array.
     *
     * This function takes the index of the key that has been pressed and sets the value. Keep in
     * mind that `idx` needs to be under 16 or else the program will panic. You can add that
     * restriction here, but instead we'll handle it in the frontend and assume that it's been done
     * correctly in the backend, rather than chhecking it twice.
     */
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    /*
     * We need some way to load the game code from a file into our RAM so it can be executed. We'll
     * dive into this more deeply when we begin reading from a file in our frontend, but for now we
     * need to take in a list of bytes and copy them into RAM
     *
     * This function copies all the values from our input `data` slice into RAM beginning at 0x200.
     * Remember that the first 512 bytes of RAM aren't to contain game data, and are empty for the
     * character sprite data we store there.
     */
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    /*
     * How the CPU will process each instruction and move through the game:
     *
     * 1. Fetch the value from our game (loaded into RAM) at the memory address stored
     * in our Program Counter
     *
     * 2. Decode this instruction
     *
     * 3. Execute, which will possibly involve modifying our CPU registers or RAM
     *
     * 4. Move the PC to the next instruction and repeat
     *
     */
    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();

        // Decode & execute
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        /*
         * Not the cleanest code, but we need each hex digit separately. From here, we can create a
         * match statement where we can specify the patterns for all of our opcodes:
         */
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        /*
         * Rust `match` statement demands that all possible options be taken into account which is
         * done with the `_` variable, which captures "everything else". Inside, we'll use
         * `uninplemented!` macro to cause the program to panic if it reaches that point.
         *
         * While a long `match` statement would certainly work for other architectures, it is
         * usually more common to implement instructions in their own functions, and either use a
         * lookup table or programmatically determine which function is correct. CHIP-8 is somewhat
         * unusual because it stores instruction parameters into the opcode itself, meaning we need
         * a lot of wild cards to match the instructions. Since there are a relatively small number
         * of them, a `match` statement works well here.
         *
         */
        match (digit1, digit2, digit3, digit4) {
            /*
             * // 0000 // | NOP
             *
             * No opcode. Do nothing. This may seem a silly one, but sometimes it's needed for
             * timing or alignment purposes.
             */
            (0, 0, 0, 0) => return,

            /*
             * 00E0 - Clear Screen
             *
             * Clear the screen, which means we need to reset our screen buffer to be empty again
             */
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            /*
             * // 00EE // | Return from Subroutine
             *
             * We haven`t yet spoken about subroutines (aka functions) and how they work. Entering
             * into a subroutine works in the same way as a plain jump: We move the PC to the
             * specified address and resume execution from there. Unlike a jump, a subroutine is
             * expected to complete at some point, and we will need to return back to the point
             * where we entered. This is where our stack comes in. When we enter a subroutine, we
             * simply push our address onto the stack, run the routine's code, and when we're ready
             * to return we pop that value off our stack and execute from that point again. A stack
             * also allows us to maintain return address for nested subroutines while ensuring they
             * are returned in the correct order.
             */
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }

            /*
             * // 1NNN // | Jump
             *
             * The jump instruction is easy to add, simply move the PC to the given address. The
             * main thing to notice here is that this opcode is defined by `0x1` being the most
             * significant digit. The other digits are used as paremeters for this operation, hence
             * the `_` placeholder in our match statement, here we want anything starting with a 1,
             * but ending in any three digits to enter this statement.
             *
             * Explicacion español:
             *
             * El statement: let nnn = op & 0xFFF, significa:
             *
             * De los 16 bits del opcode, quedate solo con los ultimos 12 bits (los menos
             * significativos), porque 0xFFF en binario es: 0000 1111 1111 1111
             */
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            /*
             * // 2NNN // | Call Subroutine
             *
             * The opposite of our 'Return from Subroutine' operation, we are going to add our
             * current PC to the stack, and then jump to the given address.
             */
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }

            /*
             * // 3XNN // | Skip next if VX == NN
             *
             * This opcode is first of a few that follow a similar pattern. For those who are
             * unfamiliar with assembly, being able to skip a line gives similar functionallity to
             * an if-else block. We can make a comparison, and if true go to one instruction, and
             * if false go somewhere else. This is also the first opcode which will use one of our
             * `V` registers. In this case, the second digit tells us which register to use, while
             * the last two digits provide the raw value.
             *
             * The implementation works like this: Since we already have the second digit saved to
             * a variable, we will reuse it for our 'X' index, although cast to a `usize`, as Rust
             * requires all array index to be done with a `usize` variable. If that value stored in
             * that register equals `nn`, then we skip the next opcode, which is the same as
             * skipping our PC ahead by two bytes
             */
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            /*
             * // 4XNN // | Skip next if VX != NN
             *
             * This opcode is exactly the same as the previous, except we skip if the compared
             * values are not equal
             */
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2
                }
            }

            /*
             * // 5XY0 // | Skip next if VX == VY
             *
             * A similar operation again, however we now use the third digit to index into another
             * V Register. You will also notice that the last significant digit is not used in the
             * operation. This opcode requires it to be 0
             */
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            /*
             * // 6XNN // | VX = NN
             *
             * Set the V Register specified by the second digit to the value given
             */
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }

            /*
             * // 7XNN // | VX += NN
             *
             * This operation adds the given value to the VX Register. In the event of an overflow,
             * Rust will panic, so we need to use a different method than the typical addition
             * operator. Note also that while CHIP-8 has a carry flag, it is not modified by this
             * operation.
             */
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            /*
             * // 8XY1 // | Bitwise operation OR
             */
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }

            /*
             * // 8XY2 // | Bitwise operation AND
             */
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }

            /*
             * // 8XY3 // | Bitwise operation XOR
             */
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }

            /*
             * // 8XY4 // | VX += VY
             *
             * This operation has two aspects to make note of. Firstly, this operation has the
             * potential to overflow, which will cause a panic in Rust if not handled correctly.
             * Secondly, this operation is the first to utilize `VF` flag register. I've touched
             * upon it previously, but while the first 15V registers are general usage, the final
             * 16th (0xF) register doubles as the flag register. Flag registers are common in many
             * CPU processors. In the case of CHIP-8 it also stores the carry flag, basically a
             * special variable that notes if the last application operation resulted in an
             * overflow/underflow. Here, if an overflow were to happen, we need to set the `VF` to
             * be 1, or 0 if not. With these two aspects in mind, we will use Rust's
             * `overflowing_add` attribute, which will return a tuple of both the wrapped sum, as
             * well as a boolean of wether an overflow occured.
             */
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            /*
             * // 8XY5 // | VX -= VY
             *
             * Same operation as the previous op, but with subtraction rather than addition. The
             * key distinction is that the `VF` carry flag works in the opposite fashion. The
             * addition operation would set the flag to 1 if an overflow occurred, here if an
             * underflow occurs, it is set to 0, and vice versa. The `overflowing_sub` method will
             * be of use to us here.
             */
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            /*
             *  // 8XY6 // | VX >>= 1
             *
             *  This operation performs a single right shift on the value in VX, with the bit that
             *  was dropped off being stored into the `VF` register. Unfortunately, there isn't a
             *  built-in Rust `u8` operator to catch the dropped bit, so we will have to do it
             *  ourself.
             *
             *  lsb = AND bit a bit con 1, que da 1 si el ultimo bit es 1, y 0 si el ultimo bit es
             *  0.
             *
             *  self.v_reg[x] >>= 1 -> todos los bits se mueven una posicion a la derecha, y el bit
             *  mas significativo se rellena con 0.
             */
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            }

            /*
             *  // 8XY7 // | VX = VY - VX
             *
             *  This operation works the same as the previous VX -= VY, but with the operands in
             *  the opposite direction
             */
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            /*
             *  // 8XYE // | VX <<= 1
             *
             *  Similar to the right shift operation, but we store the value that is overflowed in
             *  the flag register.
             *
             *  msb:
             *  (>> 7) mueve el bit mas significativo hasta la posicion 0
             *  (& 1) asegura que solo nos quedamos con ese bit, descartando los demas
             *
             *  ejemplo: self.v_reg[x] = 1010_0001:
             *
             *  self.v_reg[x] >> 7 = 0000_0001. & 1 da 1, que es lo que queremos almacenar como
             *  flag en `VF`
             *
             *  self.v_reg[x] <<= 1: Mueve todos los bits una posicion a la izquierda, y el bit
             *  menos significativo se convierte en 0. El bit mas significativo se pierde (que es
             *  el que guardamos antes en msb)
             */
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            }

            /*
             * // 8XY0 // | VX = VY
             *
             * Like the `VX = NN` operation, but the source value is from the VY register
             *
             * Si esta instruccion va primero salta un warning de unreachable code para el resto de
             * opcodes
             */
            (8, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }

            /*
             *  // 9XY0 // | Skip if VX != VY
             *
             *  Skip the next line if VX != VY. This is the same code as the 5XY0 operation, but
             *  with an inequality
             */
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            /*
             * // ANNN // | I = NNN
             *
             * This is the first instruction to utilize the I Register, which will be used in
             * several additional instructions, primarily as an address pointer to RAM. In this
             * case, we are simply setting it to the 0xNNN value encodede in this opcode
             */
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }

            /*
             *  // BNNN // | Jump to V0 + NNN
             *
             *  While previous instructions have used the V Register specified within the opcode,
             *  this instruction always uses the first V0 Register. This operations moves the PC to
             *  the sum of the value stored in V0 and the raw value 0xNNN supplied in the opcode.
             */
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            /*
             *  // CXNN // | VX = rand() & NN
             *
             *  This opcode is CHIP-8's random number generation, with a slight twist, in that the
             *  random number is then AND'd with the lower 8-bits of the opcode. Install rand crate
             *  in your project to implement this opcode.
             */
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }

            /*
             *  // DXYN // | Draw Sprite
             *
             *  This is probably the single most complicated opcode. Rather than drawing individual
             *  pixels or rectangles to the screen at a time, the CHIP-8 display works by drawing
             *  sprites, images stored in memory that are copied to the screen at a specified (x,
             *  y) coordinates from. So far so good.
             *
             *  CHIP8's sprites are always 8 pixels wide, but can be a variable number of pixels
             *  tall, from 1 to 16. This is specified in the final digit of our opcode. I mentioned
             *  earlier that the `I Register` is used frequently to store an address in memory, and
             *  this is the case here. Our sprites are stored row by row beginning at the address
             *  stored in `I`.
             *
             *  So if we are told to draw a 3px tall sprite, the first row's data is stored at *I
             *  followed by *I + 1, then *I + 2. This explains why all sprites are 8 pixels wide,
             *  each row is assigned a byte, which is 8-bits, one for each pixel, black or white.
             *  The last detail to note if that is any is flipped from white to black or vice
             *  versa, the `VF` is set (and cleared otherwise). With these things in mind, let's
             *  begin.
             *
             */
            (0xD, _, _, _) => {
                // Get the (x,y) coords for our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;

                // The last digit determines how many rows high our sprite is
                let num_rows = digit4;

                // Keep track if any pixel is flipped
                let mut flipped = false;

                // Iterate over each ROW of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    // Iterate over each COLUMN in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }

            /*
             *  // EX9E // | Skip if Key Pressed
             *
             *  Time at last to introduce user input. When setting up our emulator object, I
             *  mentioned that there are 16 possible keys numbered 0 to 0xF. This instruction
             *  checks if the index stored in VX is pressed, and if so, skips the next instruction.
             */
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }

            /*
             *  // EXA1 // | Skip if Key Not Pressed
             *
             *  Same as the previous instruction, however this time the next instruction is skipped
             *  if the key in question is not being pressed
             */
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }

            /*
             *  // FX07 // | VX = DT
             *
             * The Delay Timer ticks down every frame until reaching zero. However, that operation
             * happens automatically, it would be really useful to be able to actually see what's
             * in the Delay Timer for our game's timing purposes. This instruction does just that,
             * and stores the current value into one of the `V` Registers for us to use
             */
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }

            /*
             *  // FX0A // | Wait for Key Press
             *
             *  While we already had instructions to check if keys are either pressed or released,
             *  this instruction does something very different. Unlike those, which checked the key
             *  state and then moved on, this instruction is blocking, meaning the whole game will
             *  pause and wait for as long as it needs to until the player presses a key. That
             *  means it needs to loop endlessly until something in our `keys` arrays turns true.
             *  Once a key is found, is stored into VX. If more than one key is currently being
             *  pressed, it takes the lowest indexed one.
             *
             *  "Why are we resetting the opcode and going through the entire fetch sequence again,
             *  rather than simply doing this in a loop?". Simply put, while we want this
             *  instruction to block future instructions from running, we do not want to block any
             *  new keypresses from being registered. By remaining in a loop, we would prevent our
             *  key press code from ever running, causing this loop to never end. Perharps
             *  inefficient, but much simpler  than some sort of asynchronous checking.
             */
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // Redo opcode
                    self.pc -= 2;
                }
            }

            /*
             *  // FX15 // | DT = VX
             *
             *  This operation works the other direction from our previous Delay Timer instruction.
             *  We need someway to reset the Delay Timer to a value, and this instrucction allows
             *  us to copy over a value from a `V Register` of our choosing.
             */
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }

            /*
             *  // FX18 // | ST = VX
             *
             *  Almost the same exact same instruction as the previous, however this time we are
             *  going to store the value from VX into our `Sound Timer`.
             */
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }

            /*
             *  // FX1E // | I += VX
             *
             *  Instruction ANNN sets I to the encoded 0xNNN value, but sometimes it is useful to
             *  be able to simply increment the value. This instruction takes the value stored in
             *  VX and adds it to the `I Register`. In the case of an overflow, the register should
             *  simply roll over back to 0, which we can accomplish with Rust's `wrapping_add`
             *  method.
             */
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }

            /*
             *  // FX29 // | Set I to Font Address
             *
             *  This is another tricky instruction where it may not be clear how to progress at
             *  first. If you recall, we stored an array of font data at the very beginning of RAM
             *  when initializing the emulator. This instruction wants us to take from VX a number
             *  to print on the screen (from 0 to 0xF), and store the RAM address of that sprite
             *  into the I Register. We are actually free to store those sprites anywhere we
             *  wanted, so long as we are consistent and point to them correctly here. However, we
             *  stored them in a very convenient location, at the beginning of RAM. Let me show you
             *  what I mean by printing out some of the sprites and their RAM locations.
             *
             *  Character | RAM Address
             *  0           0
             *  1           5
             *  2           10
             *  3           15
             *  ...         ...
             *  E(14)       70
             *  F(15)       75
             *
             *  You'll notice that since all of our font sprites take up five bytes each, their RAM
             *  address is simply their value times 5. If we happened to store the fonts in a
             *  different RAM address, we could still follow this rule, however we'd have to apply
             *  an offset to where the block begins
             */
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            }

            /*
             *  // FX33 // | BCD of VX
             *
             *  Most of the instructions for CHIP-8 are rather self-explanitory, and can be
             *  implemented quite easily just by hearing a vague description. However, there are a
             *  few that are quite tricky, such as drawing to a screen and this one, storing the
             *  Binary-Coded Decimal of a number stored in VX into I Register.
             *
             *  In this tutorial we've been using hexadecimal quite a bit, which works by
             *  converting our normal decimal numbers into base-16, which is more easily understood
             *  by computers. For example, the decimal number 100 would become 0x64. This is good
             *  for computers, but not very accesible to humans, and certainly not to the general
             *  audience who are going to play our games.
             *
             *  The main purpose of BCD is to convert a hexadecimal number back into a
             *  pseudo-decimal number to print out for the user, such as for your points or high
             *  scores. So while CHIP-8 might store 0x64 internally, fetching its BCD would give us
             *  `0x1, 0x0, 0x0`, which we could print to the screen has "100".
             *
             *  You'll notice that we've gone from one byte to three in order to store all three
             *  digits of our number, which is why we are going to store the BCD into RAM,
             *  beginning at the address currently in the I Register and moving along. Given that
             *  VX stores 8-bit numbers, which are range from 0 to 255, we are always going to end
             *  up with three bytes, even if some are zero
             */
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                // Fetch the hundreds digit by divigind by 100 and tossing the decimal
                let hundreds = (vx / 100.0) as u8;

                // Fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;

                // Fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            /*
             *  // FX55 // | Store V0 - VX into I
             *
             *  These final two instructions populate our V Registers V0 through the specified VX
             *  (inclusive) with the same range of values from RAM, beginning with the address in
             *  the I Register. This first one stores the values into RAM, while the next one will
             *  load them the opposite way
             */
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            }

            /*
             *  // FX65 // | Load I into V0 - VX
             */
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            }

            (_, _, _, _) => unimplemented!("Uninplemented opcode: {}", op),
        }
    }

    /*
     * The fetch function will only be called internaly as part of our tick
     * loop, so it doesn't need to be public. The purpose of this function is to grab
     * the instruction we are about to execute (opcode) for use in the next steps of this cycle.
     *
     * Fortunately, CHIP-8 is easier than many systems. For one, there's only 35 opcodes to deal
     * with as opposed to the hundreds that many processors support. In addition, many systems
     * store additional parameters for each opcode in subsequent bytes (such as operands for
     * addition), CHIP-8 encodes these into the opcode itself. Due to this, all opcodes are
     * exactly 2 bytes, which is larger than some other systems, but the entire instruction
     * is stored in those two bytes, while other contemporary systems might consume between 1 and 3
     * bytes per cycle.
     *
     * Each opcode is encoded differently, but fortunately since all instructions consume two
     * bytes, the fetch operation is the same for all of them, and implemented as such:
     *
     * == Explicacion de fetch ==
     *
     * Si pc = 0x200 y en ram[0x200] = 0xA2, entonces -> higher_byte = 0xA2
     *
     * Siguiente instruccion para obtener el siguiente byte que viene justo despues
     *
     * Si ram[0x201] = 0xF0, entonces -> lower_byte = 0xF0
     *
     * Combinar los 2 bytes en un mismo opcode
     *
     * << 8 ---> Shift left
     *
     * 0000 0000 1010 0010 --> 0x00A2
     * 1010 0010 0000 0000 --> despues de << 8 (0xA200)
     *
     *  | ---> bitwise OR (combina dos numeros bit a bit. El resultado tiene un 1 en cada posicion
     *  donde alguno de los dos operandos tiene un 1)
     *
     * let a: u16 = 0xA200 --> 1010 0010 0000 0000
     * let b: u16 = 0x00F0 --> 0000 0000 1111 0000
     *
     * let result = a | b --> 1010 0010 1111 0000 = 0xA2F0
     */
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    /*
     * While the `tick` function operates once every CPU cycle, these timers are modified instead
     * once every frame, and thus need to be in a separate function. Their behavior is rather
     * simple, every frame both decrease by one. If the Sound Timer is set to one, the system will
     * emit a 'beep' noise. If the timers ever hit zero, they do not automatically reset, they will
     * remain at zero until the game manually resets them to some value.
     *
     */
    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
        }
    }

    /*
     * Push al stack:
     *
     * Pasamos nuestro CPU, y el valor que queremos almacenar.
     *
     * En nuestro array del stack, en el indice del array donde apunta nuestro SP
     * vamos a añadir el valor que queremos almacenar. Al principio el SP esta en 0,
     * asique para el primer valor seria lo siguiente:
     *
     * Añadir al stack (self.stack) en el valor al que esta apuntando el sp (al principio
     * es 0 --> [self.sp as usize] = [0]) el valor que pasemos como parametro (val)
     *
     * Despues de este paso tenemos que incrementar en uno el SP, para que a la hora
     * de guardar el siguiente valor en el stack no se sobreescriba el anterior
     * valor guardado
     *
     */
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    /*
     * Pop del stack:
     *
     * Pasamos nuestro CPU, en este caso el pop es para eliminar del stack,
     * asique no necesitamos pasar ningun valor.
     *
     * El pop hace la operacion a la inversa, retrocede en uno el SP y devuelve
     * lo que se encuentre en esa posicion. Intentar hacer un pop a la pila
     * sin valores almacenados da un `underflow`.
     *
     * Devolvemos el u16. Importante a la hora de devolver valores en
     * Rust no ponemos `;` en el ultimo statement de la funcion
     */
    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
