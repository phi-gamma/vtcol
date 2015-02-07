#![allow(unstable)]

extern crate libc;
extern crate getopts;

#[derive(Show)]
enum Scheme { Default, SolarizedDark, SolarizedLight }

impl std::fmt::String for Scheme {

    fn
    fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let id : &str = match (*self)
        {
            Scheme::Default         => "default",
            Scheme::SolarizedDark   => "solarized_dark",
            Scheme::SolarizedLight  => "solarized_light"
        };
        write!(f, "{}", id)
    }

} /* [impl String for Scheme] */

extern { fn exit (code : libc::c_int) -> !; }

/* struct Job -- Runtime parameters.
 */
#[derive(Show)]
struct Job {
    this   : String, /* argv[0] */
    scheme : Scheme, /* The color scheme to switch to. */
}

impl Job {

    pub fn
    new ()
        -> Job
    {
        let argv = std::os::args();
        let this = argv[0].clone();
        let opts = &[
            getopts::optopt("s", "scheme", "predefined color scheme", "NAME"),
            getopts::optopt("d", "dump", "dump predefined scheme", "NAME"),
            getopts::optflag("l", "list", "list available color schemes"),
            getopts::optflag("h", "help", "print this message")
        ];

        let matches = match getopts::getopts(argv.tail(), opts)
        {
            Ok(m) => m,
            Err(f) => panic!(f.to_string())
        };

        if matches.opt_present("l") {
            Job::schemes();
            unsafe { exit(0) };
        };

        if matches.opt_present("d")
        {
            match matches.opt_str("d") {
                None =>
                {
                    Job::usage(&this, opts);
                    panic!("no color scheme given, aborting")
                },
                Some (name) =>
                {
                    let scm = Job::pick_scheme(&name);
                    Job::dump(scm);
                    unsafe { exit(0) };
                }
            };
        }

        let scheme = match matches.opt_str("s")
        {
            None => {
                Job::usage(&this, opts);
                panic!("no color scheme given, aborting")
            },
            Some (name) => Job::pick_scheme(&name)
        };

        Job {
            this   : this,
            scheme : scheme
        }
    }

    fn
    pick_scheme (name : &String)
        -> Scheme
    {
        match name.as_slice() {
            "solarized" | "solarized_dark" | "sd"
                => Scheme::SolarizedDark,
            "solarized_light" | "sl"
                => Scheme::SolarizedLight,
            "default" | "normal"
                => Scheme::Default,
            garbage => {
                panic!("unknown color scheme “{}”, aborting", garbage);
            }
        }
    }

    fn
    usage (this : &String, opts: &[getopts::OptGroup])
    {
        let brief = format!("usage: {} [options]", this);
        print!("{}", getopts::usage(brief.as_slice(), opts));
    }

    fn
    schemes ()
    {
        println!("Available color schemes:");
        println!("      · solarized_dark");
        println!("      · solarized_light");
        println!("      · default");
    }

    fn
    dump (scm : Scheme)
    {
        println!("Dumping color scheme {}", scm);
        match scm {
            Scheme::Default        => Job::dump_scheme(&DEFAULT_COLORS),
            Scheme::SolarizedDark  => Job::dump_scheme(&SOLARIZED_COLORS_DARK),
            Scheme::SolarizedLight => Job::dump_scheme(&SOLARIZED_COLORS_LIGHT)
        }
    }

    fn
    dump_scheme (colors : &[&str; PALETTE_SIZE])
    {
        let pal : Palette = Palette::new(colors);
        pal.dump()
    }

} /* [impl Job] */

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

type Fd = libc::c_int;

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

static SOLARIZED_COLORS_DARK : [&'static str; PALETTE_SIZE] = [
    "002b36", "dc322f", "859900", "b58900",
    "268bd2", "d33682", "2aa198", "eee8d5",
    "002b36", "cb4b16", "586e75", "657b83",
    "839496", "6c71c4", "93a1a1", "fdf6e3",
];

static SOLARIZED_COLORS_LIGHT : [&'static str; PALETTE_SIZE] = [
    "eee8d5", "dc322f", "859900", "b58900",
    "268bd2", "d33682", "2aa198", "073642",
    "fdf6e3", "cb4b16", "93a1a1", "839496",
    "657b83", "6c71c4", "586e75", "002b36",
];

pub struct Palette {
    colors : [u8; PALETTE_BYTES]
}

impl Palette
{

    fn
    dump (&self)
    {
        let mut i : usize = 0;
        let mut buf : [u8; 3] = [ 0u8, 0u8, 0u8 ];
        for col in self.colors.iter()
        {
            let idx : usize = i % 3;
            buf[idx] = *col;
            if idx == 2us {
                println!("[{:02.x}] 0x{:02.X}{:02.X}{:02.X}",
                         i / 3, buf[0us], buf[1us], buf[2us]);
            }
            i = i + 1;
        }
    }

} /* [impl Palette] */

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
    let b : u8 = byte_of_hex!(bytes, 4);
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
    -> Option<Fd>
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
    -> Option<Fd>
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
write_to_term (fd : Fd, buf : &str)
{
    let len = buf.len() as u32;
    let raw = std::ffi::CString::from_slice(buf.as_bytes());
    unsafe { libc::write(fd, raw.as_ptr() as *const libc::c_void, len) };
}

fn
clear_term (fd : Fd)
{
    let clear  : &str = "\x1b[2J";
    let cursor : &str = "\x1b[1;1H";
    write_to_term(fd, clear);
    write_to_term(fd, cursor);
}

fn
main ()
{
    let job = Job::new();
    println!("job parms: {:?}", job);
    let color_set : [[&str; 7]; PALETTE_SIZE];
    let mut pal : Palette = {
        match job.scheme {
            Scheme::Default        => Palette::new(&DEFAULT_COLORS),
            Scheme::SolarizedDark  => Palette::new(&SOLARIZED_COLORS_DARK),
            Scheme::SolarizedLight => Palette::new(&SOLARIZED_COLORS_LIGHT),
        }
    };
    println!("{}", pal);
    //println!("{:?}", pal);
    let fd = get_console_fd(None).unwrap();
    println!("fd: {}", fd);

    if unsafe { ioctl(fd, PIO_CMAP, std::mem::transmute(&mut pal)) } < 0 {
        panic!("PIO_CMAP, ioctl failed to insert new palette")
    }
    clear_term(fd);
    println!("terminated from job {:?}", job);
}

