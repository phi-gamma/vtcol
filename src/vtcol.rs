#![feature(libc)]
#![feature(rustc_private)]
#![feature(collections)]
#![feature(convert)]

extern crate libc;
extern crate getopts;

use std::io::BufRead;

type Fd = libc::c_int;

static mut VERBOSITY : u8 = 0_u8;

macro_rules! vrb {
    ( $( $e:expr ),* ) => (
            if unsafe { VERBOSITY } > 0_u8 { println!( $( $e ),* ) }
        )
}

const PALETTE_SIZE  : usize = 16_usize;
const PALETTE_BYTES : usize = PALETTE_SIZE * 3_usize; // 16 * sizeof(int)

const RAW_COLEXPR_SIZE : usize = 6_usize; // e. g. 0xBADF00

type RawPalette<'a> = [&'a str; PALETTE_SIZE];

const KDGKBTYPE     : libc::c_int  = 0x4b33;     /* kd.h */
const PIO_CMAP      : libc::c_int  = 0x00004B71; /* kd.h */
const KB_101        : libc::c_char = 0x0002;     /* kd.h */
const O_NOCTTY      : libc::c_int  = 0o0400;     /* fcntl.h */

#[derive(Debug)]
enum Color {
    Black(bool), Red(bool),     Green(bool), Yellow(bool),
    Blue(bool),  Magenta(bool), Cyan(bool),  White(bool),
}

impl Color {

    fn
    of_value (val : u8)
        -> Color
    {
        match val
        {
            0x00_u8 => Color::Black   (false),
            0x01_u8 => Color::Red     (false),
            0x02_u8 => Color::Green   (false),
            0x03_u8 => Color::Yellow  (false),
            0x04_u8 => Color::Blue    (false),
            0x05_u8 => Color::Magenta (false),
            0x06_u8 => Color::Cyan    (false),
            0x07_u8 => Color::White   (false),

            0x08_u8 => Color::Black   (true),
            0x09_u8 => Color::Red     (true),
            0x0a_u8 => Color::Green   (true),
            0x0b_u8 => Color::Yellow  (true),
            0x0c_u8 => Color::Blue    (true),
            0x0d_u8 => Color::Magenta (true),
            0x0e_u8 => Color::Cyan    (true),
            0x0f_u8 => Color::White   (true),

            _ => panic!("invalid color value: {}", val)
        }
    }

    fn
    format_brightness
        (b : bool,
         s : &str)
        -> String
    {
        if b {
            return String::from_str("bright ") + s;
        }
        String::from_str(s)
    }

    fn
    to_string
        (&self)
        -> String
    {
        match *self
        {
            Color::Black  (b) => { Color::format_brightness(b, "black"  ) },
            Color::Red    (b) => { Color::format_brightness(b, "red"    ) },
            Color::Green  (b) => { Color::format_brightness(b, "green"  ) },
            Color::Yellow (b) => { Color::format_brightness(b, "yellow" ) },
            Color::Blue   (b) => { Color::format_brightness(b, "blue"   ) },
            Color::Magenta(b) => { Color::format_brightness(b, "magenta") },
            Color::Cyan   (b) => { Color::format_brightness(b, "cyan"   ) },
            Color::White  (b) => { Color::format_brightness(b, "white"  ) },
        }
    }

} /* [impl Color] */

#[derive(Debug)]
enum Scheme {
    Default,
    SolarizedDark,
    SolarizedLight,
    Custom (Option<String>)
}

impl<'a> std::fmt::Display for Scheme {

    fn
    fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let id : &str = match *self
        {
            Scheme::Default           => "default",
            Scheme::SolarizedDark     => "solarized_dark",
            Scheme::SolarizedLight    => "solarized_light",
            Scheme::Custom(ref kind) => {
                let tmp : &str = match *kind {
                    Some(ref fname) => fname.as_str(),
                    None            => "<read stdin>"
                };
                tmp
            }
        };
        write!(f, "{}", id)
    }

} /* [impl std::fmt::String for Scheme] */

extern { fn exit (code : libc::c_int) -> !; }

/* struct Job -- Runtime parameters.
 */
#[derive(Debug)]
struct Job {
    this   : String, /* argv[0] */
    scheme : Scheme, /* The color scheme to switch to. */
}

impl<'a> Job {

    pub fn
    new ()
        -> Job
    {
        let argv : Vec<String> = std::env::args().collect();
        let this = argv[0].clone();
        let opts = &[
            getopts::optopt("s", "scheme", "predefined color scheme", "NAME"),
            getopts::optopt("d", "dump", "dump predefined scheme", "NAME"),
            getopts::optopt("f", "file", "apply scheme from file", "PATH"),
            getopts::optflag("v", "verbose", "enable verbose messages"),
            getopts::optflag("l", "list", "list available color schemes"),
            getopts::optflag("h", "help", "print this message")
        ];

        let matches = match getopts::getopts(argv.tail(), opts)
        {
            Ok(m) => m,
            Err(f) => panic!(f.to_string())
        };

        if matches.opt_present("v") { unsafe { VERBOSITY = 1_u8; } };

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

        let scheme =
            if matches.opt_present("f")
            {
                match matches.opt_str("f")
                {
                    None => {
                        Job::usage(&this, opts);
                        panic!("no file name specified, aborting")
                    },
                    Some (fname) =>
                        if fname == "-" { Job::scheme_from_stdin()            }
                        else            { Scheme::Custom(Some(fname.clone())) }
                }
            } else {
                match matches.opt_str("s")
                {
                    None        => Job::scheme_from_stdin(),
                    Some (name) =>
                        if name == "-" { Job::scheme_from_stdin() }
                        else           { Job::pick_scheme(&name)  }
                }
            }; /* [let scheme] */

        Job {
            this   : this,
            scheme : scheme
        }
    }

    fn
    pick_scheme <'b> (name : &String)
        -> Scheme
    {
        match name.as_str() {
            "solarized" | "solarized_dark" | "sd"
                => Scheme::SolarizedDark,
            "solarized_light" | "sl"
                => Scheme::SolarizedLight,
            "default" | "normal"
                => Scheme::Default,
            _any => Scheme::Custom (Some(name.clone()))
        }
    }

    fn
    scheme_from_stdin ()
        -> Scheme
    {
        Scheme::Custom(None)
    }

    fn
    usage (this : &String, opts: &[getopts::OptGroup])
    {
        let brief = format!("usage: {} [options]", this);
        print!("{}", getopts::usage(brief.as_str(), opts));
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
        vrb!("Dumping color scheme {}", scm);
        match scm {
            Scheme::Default        => Job::dump_scheme(&DEFAULT_COLORS),
            Scheme::SolarizedDark  => Job::dump_scheme(&SOLARIZED_COLORS_DARK),
            Scheme::SolarizedLight => Job::dump_scheme(&SOLARIZED_COLORS_LIGHT),
            Scheme::Custom(kind)  =>
                match kind {
                    Some(fname) => Job::dump_palette(Palette::from_file(&fname)),
                    None        => Job::dump_palette(Palette::from_stdin())
                }
        }
    }

    fn
    dump_scheme (colors : &[&str; PALETTE_SIZE])
    {
        let pal : Palette = Palette::new(colors);
        pal.dump()
    }

    fn
    dump_palette (pal : Palette)
    {
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

static CONSOLE_PATHS : [&'static str; 6] = [
    "/proc/self/fd/0",
    "/dev/tty",
    "/dev/tty0",
    "/dev/vc/0",
    "/dev/systty",
    "/dev/console",
];

static DEFAULT_COLORS : RawPalette<'static> = [
    "000000", "aa0000", "00aa00", "aa5500",
    "0000aa", "aa00aa", "00aaaa", "aaaaaa",
    "555555", "ff5555", "55ff55", "ffff55",
    "5555ff", "ff55ff", "55ffff", "ffffff"
];

static SOLARIZED_COLORS_DARK : RawPalette<'static> = [
    "002b36", "dc322f", "859900", "b58900",
    "268bd2", "d33682", "2aa198", "eee8d5",
    "002b36", "cb4b16", "586e75", "657b83",
    "839496", "6c71c4", "93a1a1", "fdf6e3",
];

static SOLARIZED_COLORS_LIGHT : RawPalette<'static> = [
    "eee8d5", "dc322f", "859900", "b58900",
    "268bd2", "d33682", "2aa198", "073642",
    "fdf6e3", "cb4b16", "93a1a1", "839496",
    "657b83", "6c71c4", "586e75", "002b36",
];

static DUMMY_COLORS : RawPalette<'static> = [
    "000000", "ffffff", "000000", "ffffff",
    "000000", "ffffff", "000000", "ffffff",
    "000000", "ffffff", "000000", "ffffff",
    "000000", "ffffff", "000000", "ffffff",
];

pub struct Palette {
    colors : [u8; PALETTE_BYTES]
}

impl Palette
{

    fn
    dump (&self)
    {
        let mut i : usize = 0_usize;
        let mut buf : [u8; 3] = [ 0u8, 0u8, 0u8 ];
        for col in self.colors.iter()
        {
            let idx : usize = i % 3;
            buf[idx] = *col;
            if idx == 2_usize {
                println!("{:>15} => 0x{:02.X}{:02.X}{:02.X}",
                         Color::of_value((i / 3) as u8).to_string(),
                         buf[0_usize], buf[1_usize], buf[2_usize]);
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
         | nibble_of_char($ar[$off + 1_usize]) as u8
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
        let mut idx : usize = 0_usize;
        let mut pal : [u8; PALETTE_BYTES] = unsafe { std::mem::zeroed() };

        for def in colors.iter() {
            let (r, g, b) = rgb_of_hex_triplet(*def);
            pal[idx + 0_usize] = r;
            pal[idx + 1_usize] = g;
            pal[idx + 2_usize] = b;
            //println!(">> {} -> {:X} {:X} {:X}", def, r, g, b);
            idx = idx + 3_usize;
        }

        Palette {
            colors : pal
        }
    } /* [Palette::new] */

    pub fn
    dummy ()
        -> Palette
    {
        Palette::new(&DUMMY_COLORS)
    } /* [Palette::dummy] */

    pub fn
    from_buffered_reader
    (reader : &mut std::io::BufRead)
        -> Palette
    {
        let mut pal_idx : usize = 0_usize;
        let mut pal     : [u8; PALETTE_BYTES] = unsafe { std::mem::zeroed() };
        let mut line    : String = String::new();

        while reader.read_line(&mut line).is_ok() {
            let len = line.len();
            if len == 0_usize { break; }
            else if len >= 8_usize {
                if let Some(off) = line.find('#') {
                    if off != 0_usize {
                        /* Palette index specified, number prepended */
                        let str_idx = unsafe { line.slice_unchecked(0_usize, off) };
                        let parse_res : Result<usize, _>
                            = std::str::FromStr::from_str(str_idx);
                        match parse_res {
                            Ok(new_idx) => {
                                if new_idx < PALETTE_SIZE { pal_idx = new_idx * 3_usize; }
                            },
                            _ => ()
                        }
                    }
                    let off = off + 1_usize;
                    if off > len - 6_usize { /* no room left for color definition after '#' char */
                        panic!("invalid color definition: {}", line);
                    }
                    let col = line.slice_chars(off, off + RAW_COLEXPR_SIZE);

                    let (r, g, b) = rgb_of_hex_triplet(col);
                    pal[pal_idx + 0_usize] = r;
                    pal[pal_idx + 1_usize] = g;
                    pal[pal_idx + 2_usize] = b;
                    pal_idx = (pal_idx + 3_usize) % PALETTE_BYTES;
                }
            }
            line.truncate(0);
        };

        Palette { colors : pal }
    } /* [Palette::from_buffered_reader] */

    pub fn
    from_file (fname : &String)
        -> Palette
    {
        /* Check if file exists
         */
        let path = std::path::Path::new(fname);
        let file = match std::fs::File::open(&path)
        {
            Err(e) => panic!("failed to open {} as file ({})", fname, e),
            Ok(f) => f
        };
        let mut reader = std::io::BufReader::new(file);

        /* Parse scheme file
         */
        Palette::from_buffered_reader (&mut reader)
    } /* [Palette::from_file] */

    pub fn
    from_stdin ()
        -> Palette
    {
        vrb!("Go ahead, type your color scheme …");
        vrb!("vtcol>");
        let mut reader = std::io::BufReader::new(std::io::stdin());

        /* Parse scheme file
         */
        Palette::from_buffered_reader (&mut reader)
    } /* [Palette::from_stdin] */

} /* [impl Palette] */

impl std::fmt::Display for Palette {

    fn
    fmt (&self,
         f : &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        let mut i : usize = 0_usize;
        while i < PALETTE_BYTES {
            let _ = write!(f, "{}", if i == 0 { "(" } else { "\n " });
            let r = self.colors[i + 0_usize];
            let g = self.colors[i + 1_usize];
            let b = self.colors[i + 2_usize];
            let _ = write!(f, "((r 0x{:02.X}) (g 0x{:02.X}) (b 0x{:02.x}))", r, g, b);
            i = i + 3_usize;
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
        let mut i : u8 = 0_u8;
        while (i as usize) < PALETTE_BYTES {
            let r = self.colors[i as usize + 0_usize];
            let g = self.colors[i as usize + 1_usize];
            let b = self.colors[i as usize + 2_usize];
            let _ = write!(f, "{} => 0x{:02.X}{:02.X}{:02.X}\n",
                           Color::of_value(i).to_string(), r, g, b);
            i = i + 3_u8;
        }
        std::result::Result::Ok(())
    }

} /* [impl std::fmt::Debug for Palette] */


fn
fd_of_path
    (path : &std::path::Path)
    -> Option<Fd>
{
    let p = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
    match unsafe { libc::open(p.as_ptr(), libc::O_RDWR | O_NOCTTY, 0) }
    {
        -1 => return None,
        fd =>
        {
            vrb!("  *> got fd");
            if unsafe { libc::isatty(fd) } == 0 {
                vrb!("  *> not a tty");
                return None
            }

            let mut tty_type : libc::c_char = 0;

            let res = unsafe { ioctl(fd,
                                     KDGKBTYPE as libc::c_int,
                                     std::mem::transmute(&mut tty_type)) };
            if res < 0 {
                vrb!("  *> ioctl failed");
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
            //let path = std::path::Path::new(std::ffi::CString::new(path.as_bytes()).unwrap());
            let path = std::path::Path::new(path);
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
                vrb!("trying path: {:?}", path);
                let path = std::path::Path::new(path);
                if let Some (fd) = fd_of_path(&path) {
                    vrb!(" * Success!");
                    return Some (fd)
                }
            }
            vrb!("could not retrieve fd for any of the search paths");
            None
        }
    }
}

fn
write_to_term (fd : Fd, buf : &str)
{
    let len = buf.len() as libc::size_t;
    let raw = std::ffi::CString::new(buf.as_bytes()).unwrap();
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
    vrb!("job parms: {:?}", job);
    let mut pal : Palette = {
        match job.scheme {
            Scheme::Default            => Palette::new(&DEFAULT_COLORS),
            Scheme::SolarizedDark      => Palette::new(&SOLARIZED_COLORS_DARK),
            Scheme::SolarizedLight     => Palette::new(&SOLARIZED_COLORS_LIGHT),
            Scheme::Custom(ref kind) =>
                match *kind {
                    Some(ref fname) => Palette::from_file(fname),
                    None            => Palette::from_stdin()
                }
        }
    };
    vrb!("Using palette:");
    vrb!("{}", pal);
    let fd = get_console_fd(None).unwrap();
    vrb!("fd: {}", fd);

    if unsafe { ioctl(fd, PIO_CMAP, std::mem::transmute(&mut pal)) } < 0 {
        panic!("PIO_CMAP, ioctl failed to insert new palette")
    }
    clear_term(fd);
    vrb!("terminated from job {:?}", job);
}

