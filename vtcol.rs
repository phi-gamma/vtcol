#![allow(unstable)]

extern crate libc;
extern crate getopts;

type Fd = libc::c_int;

const PALETTE_SIZE  : usize = 16_us;
const PALETTE_BYTES : usize = PALETTE_SIZE * 3_us; // 16 * sizeof(int)

const RAW_COLEXPR_SIZE : usize = 6_us; // e. g. 0xBADF00

type RawPalette<'a> = [&'a str; PALETTE_SIZE];

const KDGKBTYPE     : libc::c_int  = 0x4b33;     /* kd.h */
const PIO_CMAP      : libc::c_int  = 0x00004B71; /* kd.h */
const KB_101        : libc::c_char = 0x0002;     /* kd.h */
const O_NOCTTY      : libc::c_int  = 0o0400;     /* fcntl.h */

#[derive(Show)]
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

#[derive(Show)]
enum Scheme<'a> {
    Default,
    SolarizedDark,
    SolarizedLight,
    Custom (String)
}

impl<'a> std::fmt::Display for Scheme<'a> {

    fn
    fmt (&self, f : &mut std::fmt::Formatter) -> std::fmt::Result
    {
        let id : &str = match *self
        {
            Scheme::Default           => "default",
            Scheme::SolarizedDark     => "solarized_dark",
            Scheme::SolarizedLight    => "solarized_light",
            Scheme::Custom(ref fname) => fname.as_slice()
        };
        write!(f, "{}", id)
    }

} /* [impl std::fmt::String for Scheme] */

extern { fn exit (code : libc::c_int) -> !; }

/* struct Job -- Runtime parameters.
 */
#[derive(Show)]
struct Job<'a> {
    this   : String,     /* argv[0] */
    scheme : Scheme<'a>, /* The color scheme to switch to. */
}

impl<'a> Job<'a> {

    pub fn
    new ()
        -> Job<'a>
    {
        let argv = std::os::args();
        let this = argv[0].clone();
        let opts = &[
            getopts::optopt("s", "scheme", "predefined color scheme", "NAME"),
            getopts::optopt("d", "dump", "dump predefined scheme", "NAME"),
            getopts::optopt("f", "file", "apply scheme from file", "PATH"),
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

        let scheme =
            if matches.opt_present("f")
            {
                match matches.opt_str("f")
                {
                    None => {
                        Job::usage(&this, opts);
                        panic!("no file name specified, aborting")
                    },
                    Some (fname) => Scheme::Custom(fname.clone())
                }
            } else {
                match matches.opt_str("s")
                {
                    None => {
                        Job::usage(&this, opts);
                        panic!("no color scheme given, aborting")
                    },
                    Some (name) => Job::pick_scheme(&name)
                }
            }; /* [let scheme] */

        Job {
            this   : this,
            scheme : scheme
        }
    }

    fn
    pick_scheme <'b> (name : &String)
        -> Scheme<'b>
    {
        match name.as_slice() {
            "solarized" | "solarized_dark" | "sd"
                => Scheme::SolarizedDark,
            "solarized_light" | "sl"
                => Scheme::SolarizedLight,
            "default" | "normal"
                => Scheme::Default,
            _any => Scheme::Custom (name.clone())
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
            Scheme::SolarizedLight => Job::dump_scheme(&SOLARIZED_COLORS_LIGHT),
            Scheme::Custom(_fname) => panic!("cannot dump custom palette, yet")
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

#[derive(Copy)]
pub struct Palette {
    colors : [u8; PALETTE_BYTES]
}

impl Palette
{

    fn
    dump (&self)
    {
        let mut i : usize = 0_us;
        let mut buf : [u8; 3] = [ 0u8, 0u8, 0u8 ];
        for col in self.colors.iter()
        {
            let idx : usize = i % 3;
            buf[idx] = *col;
            if idx == 2us {
                println!("{:>15} => 0x{:02.X}{:02.X}{:02.X}",
                         Color::of_value((i / 3) as u8).to_string(),
                         buf[0us], buf[1us], buf[2us]);
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
        let mut idx : usize = 0_us;
        let mut pal : [u8; PALETTE_BYTES] = unsafe { std::mem::zeroed() };

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
    } /* [Palette::new] */

    pub fn
    dummy ()
        -> Palette
    {
        Palette::new(&DUMMY_COLORS)
    } /* [Palette::dummy] */

    pub fn
    from_buffered_reader
    (reader : &mut std::io::BufferedReader<std::io::File>)
        -> Palette
    {
        let mut pal_idx : usize = 0_us;
        let mut pal     : [u8; PALETTE_BYTES] = unsafe { std::mem::zeroed() };

        while  let Ok(line) = reader.read_line() {
            let len = line.len();
            if len < 8_us { panic!("invalid line in string: {}", line); };
            if let Some(idx) = line.find_str("#") {
                let idx = idx + 1_us;
                if idx > len - 6_us { /* no room left for color definition after '#' char */
                    panic!("invalid color definition: {}", line);
                }
                let col = line.slice_chars(idx, idx + RAW_COLEXPR_SIZE);
                println!("raw color: {}", col);

                let (r, g, b) = rgb_of_hex_triplet(col);
                pal[pal_idx + 0_us] = r;
                pal[pal_idx + 1_us] = g;
                pal[pal_idx + 2_us] = b;
                pal_idx = pal_idx + 3_us;
            }
        };

        Palette { colors : pal }
    } /* [Palette::from_buffered_reader] */

    pub fn
    from_file (fname : &String)
        -> Palette
    {
        /* Check if file exists
         */
        let path = Path::new(fname.as_bytes());
        let file = match std::io::File::open(&path)
        {
            Err(e) => panic!("failed to open {} as file ({})", fname, e),
            Ok(f) => f
        };
        let mut reader = std::io::BufferedReader::new(file);

        /* Parse scheme file
         */
        Palette::from_buffered_reader (&mut reader)
    } /* [Palette::from_file] */

} /* [impl Palette] */

impl std::fmt::Display for Palette {

    fn
    fmt (&self,
         f : &mut std::fmt::Formatter)
        -> std::fmt::Result
    {
        let mut i : usize = 0_us;
        while i < PALETTE_BYTES {
            let _ = write!(f, "{}", if i == 0 { "(" } else { "\n " });
            let r = self.colors[i + 0_us];
            let g = self.colors[i + 1_us];
            let b = self.colors[i + 2_us];
            let _ = write!(f, "((r 0x{:02.X}) (g 0x{:02.X}) (b 0x{:02.x}))", r, g, b);
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
        let mut i : u8 = 0_u8;
        while (i as usize) < PALETTE_BYTES {
            let r = self.colors[i as usize + 0_us];
            let g = self.colors[i as usize + 1_us];
            let b = self.colors[i as usize + 2_us];
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
    let mut pal : Palette = {
        match job.scheme {
            Scheme::Default            => Palette::new(&DEFAULT_COLORS),
            Scheme::SolarizedDark      => Palette::new(&SOLARIZED_COLORS_DARK),
            Scheme::SolarizedLight     => Palette::new(&SOLARIZED_COLORS_LIGHT),
            Scheme::Custom (ref fname) => Palette::from_file(fname)
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

