extern crate libc;

/* Rust appears to come with two wrappers for ``ioctl(2)``, but neither can be utilized for our
 * purposes. The one in ``sys`` is part of a private (seriously‽) whereas the one in the
 * ``libc`` module is defined as taking variable arguments and therefore cannot be called from
 * Rust. Wrapping C is still a bit awkward, as it seems.
 */
extern {
    pub fn
    ioctl(d       :      libc::c_int,
          request :      libc::c_int,
          data    : *mut libc::c_void)
        -> libc::c_int;
}

type fd_t = libc::c_int;

const PALETTE_SIZE  : usize = 16_us;
const PALETTE_BYTES : usize = PALETTE_SIZE * 3_us; // 16 * sizeof(int)

const KDGKBTYPE     : libc::c_int  = 0x4b33;     /* kd.h */
const PIO_CMAP      : libc::c_int  = 0x00004B71; /* kd.h */
const KB_101        : libc::c_char = 0x0002;     /* kd.h */
const O_NOCTTY      : libc::c_int  = 0o0400;     /* fcntl.h */

static CONSOLE_PATHS : [&'static str; 6] = [
    "/proc/self/fd/0",
    "/dev/tty",
    "/dev/tty0",
    "/dev/vc/0",
    "/dev/systty",
    "/dev/console",
];

static DEFAULT_COLORS : [&'static str; PALETTE_SIZE] = [
    "000000", "aa0000", "00aa00", "aa5500",
    "0000aa", "aa00aa", "00aaaa", "aaaaaa",
    "555555", "ff5555", "55ff55", "ffff55",
    "5555ff", "ff55ff", "55ffff", "ffffff"
];

static SOLARIZED_COLORS : [&'static str; PALETTE_SIZE] = [
    "002b36", "dc322f", "859900", "b58900",
    "268bd2", "d33682", "2aa198", "eee8d5",
    "002b36", "cb4b16", "586e75", "657b83",
    "839496", "6c71c4", "93a1a1", "fdf6e3",
];

/*
 * The palette struct is the type expected by ioctl PIO_CMAP
 */
//struct palette { unsigned char colors[PALETTE_SIZE * 7]; };

pub struct Palette {
    colors : [u8; PALETTE_BYTES]
}

fn
nibble_of_char
    (chr : u8)
    -> u8
{
    match chr as char {
        '0' ... '9' => { chr - '0' as u8 },
        'a' ... 'f' => { chr - 'a' as u8 + 10 },
        'A' ... 'F' => { chr - 'A' as u8 + 10 },
        _ => 0
    }
}

macro_rules! byte_of_hex {
    ($ar:ident, $off:expr) => (
        (nibble_of_char($ar[$off])) << 4
         | nibble_of_char($ar[$off + 1_us]) as u8
    )
}

fn
rgb_of_hex_triplet
    (def : &str)
    -> (u8, u8, u8)
{
    let bytes = def.as_bytes();
    let r : u8 = byte_of_hex!(bytes, 0);
    let g : u8 = byte_of_hex!(bytes, 2);
    let b : u8 = byte_of_hex!(bytes, 3);
    (r, g, b)
}


impl Palette {

    pub fn
    new (colors : &[&str; PALETTE_SIZE])
        -> Palette
    {
        let mut red   : u32 = 0_u32;
        let mut green : u32 = 0_u32;
        let mut blue  : u32 = 0_u32;

        let mut idx : usize = 0_us;
        let mut pal : [u8; PALETTE_BYTES] = unsafe { std::mem::zeroed() };
            ;

        for def in colors.iter() {
            let (r, g, b) = rgb_of_hex_triplet(*def);
            pal[idx + 0_us] = r;
            pal[idx + 1_us] = g;
            pal[idx + 2_us] = b;
            //println!(">> {} -> {:X} {:X} {:X}", def, r, g, b);
            idx = idx + 3_us;
        }

        Palette {
            colors : pal
        }
    }

} /* [impl Palette] */

impl std::fmt::Display for Palette {

    fn
    fmt (&self,
         f : &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        let mut i : usize = 0_us;
        while i < PALETTE_BYTES {
            write!(f, "{}", if i == 0 { "(" } else { "\n " });
            let r = self.colors[i + 0_us];
            let g = self.colors[i + 1_us];
            let b = self.colors[i + 2_us];
            write!(f, "((r 0x{:02.X}) (g 0x{:02.X}) (b 0x{:02.x}))", r, g, b);
            i = i + 3_us;
        }
        write!(f, ")\n")
    }

} /* [impl std::fmt::Display for Palette] */

impl std::fmt::Debug for Palette {

    fn
    fmt (&self,
         f : &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        let mut i : usize = 0_us;
        while i < PALETTE_BYTES {
            let r = self.colors[i + 0_us];
            let g = self.colors[i + 1_us];
            let b = self.colors[i + 2_us];
            write!(f, "{:02} => 0x{:02.X}{:02.X}{:02.X}\n", i, r, g, b);
            i = i + 3_us;
        }
        std::result::Result::Ok(())
    }

} /* [impl std::fmt::Debug for Palette] */


fn
fd_of_path
    (path : &std::path::Path)
    -> Option<fd_t>
{
    let p = std::ffi::CString::from_slice(path.as_vec());
    match unsafe { libc::open(p.as_ptr(), libc::O_RDWR | O_NOCTTY, 0) }
    {
        -1 => return None,
        fd =>
        {
            println!("  *> got fd");
            if unsafe { libc::isatty(fd) } == 0 {
                println!("  *> not a tty");
                return None
            }

            let mut tty_type : libc::c_char = 0;

            let res = unsafe { ioctl(fd,
                                     KDGKBTYPE as libc::c_int,
                                     std::mem::transmute(&mut tty_type)) };
            if res < 0 {
                println!("  *> ioctl failed");
                return None
            }

            if tty_type != KB_101 { return None }

            return Some(fd)
        }
    }
}

fn
get_console_fd
    (path : Option<&str>)
    -> Option<fd_t>
{
    match path
    {
        Some (path) =>
        {
            let path = std::path::Path::new(std::ffi::CString::from_slice(path.as_bytes()));
            match fd_of_path(&path)
            {
                Some (fd) => Some (fd),
                None    => panic!("cannot open {:?} as a tty", path)
            }
        },
        None =>
        {
            let mut it = CONSOLE_PATHS.iter();
            while let Some (&path) = it.next()
            {
                println!("trying path: {:?}", path);
                let path = std::path::Path::new(path);
                if let Some (fd) = fd_of_path(&path) {
                    println!(" * Success!");
                    return Some (fd)
                }
            }
            println!("could not retrieve fd for any of the search paths");
            None
        }
    }
}

fn
main ()
{
    let color_set : [[&str; 7]; PALETTE_SIZE];
    //let mut pal : Palette = Palette::new(&DEFAULT_COLORS);
    let mut pal : Palette = Palette::new(&SOLARIZED_COLORS);
    println!("{}", pal);
    //println!("{:?}", pal);
    let fd = get_console_fd(None).unwrap();
    println!("fd: {}", fd);

    if unsafe { ioctl(fd, PIO_CMAP, std::mem::transmute(&mut pal)) } < 0 {
        panic!("PIO_CMAP, ioctl failed to insert new palette")
    }
}

