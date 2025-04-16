pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONT_SIZE: usize = 80;

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
             * 0000 - NOP
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
             * 00EE - Return from Subroutine
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
             * 1NNN - Jump
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
             * 2NNN - Call Subroutine
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
             * 3XNN - Skip next if VX == NN
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
            },

            /*
             * 4XNN - Skip next if VX != NN
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
            },

            /*
             * 5XY0 - Skip next if VX == VY
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
            },

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
