mod strings;
mod utils;

use std::ffi::{OsString};
use clap::{Parser};
use strings::{Options, UnicodeDisplayKind, EncodingKind, RadixKind};

impl Options {
    fn new(args: &CliArgs) -> Options {
        // defaults
        let mut datasection_only = false;
        let mut print_filenames = false;
        let min_length = args.min_bytes;
        let mut include_all_whitespace = false;
        let mut print_addresses = false;
        let mut address_radix: RadixKind = RadixKind::Hex;
        let mut output_separator: Option<String> = None;
        let mut encoding: EncodingKind = EncodingKind::Bit7;
        let mut unicode_display = UnicodeDisplayKind::Default;

        if args.all {
            datasection_only = false;
        }

        if args.data {
            datasection_only = true;
        }

        if args.print_file_name {
            print_filenames = true;
        }

        if args.include_all_whitespace {
            include_all_whitespace = true;
        }

        if args.octal_radix {
            print_addresses = true;
            address_radix = RadixKind::Oct;
        }

        if let Some(radix) = args.radix.as_deref() {
            print_addresses = true;
            match radix {
                "o" => { address_radix = RadixKind::Oct; }
                "d" => { address_radix = RadixKind::Dec; }
                "x" => { address_radix = RadixKind::Hex; }
                wrong => {
                    panic!("Wrong value of radix argument: {}", wrong)
                }
            }
        }

        if let Some(enc) = args.encoding.as_deref() {
            encoding = EncodingKind::from(enc.parse().expect(
                &format!("invalid char argument {}", enc)
            ))
        }

        if let Some(separator) = args.output_separator.as_deref() {
            output_separator = Some(separator.to_string())
        }

        if let Some(unicode) = args.unicode.as_deref() {
            unicode_display = UnicodeDisplayKind::from(unicode);
        }

        if !matches!(unicode_display, UnicodeDisplayKind::Default) {
            encoding = EncodingKind::Bit8;
        }

        Options {
            datasection_only,
            print_filenames,
            min_length,
            include_all_whitespace,
            print_addresses,
            address_radix,
            output_separator,
            encoding,
            unicode_display,
        }
    }
}

impl UnicodeDisplayKind {
    fn from(kind: &str) -> UnicodeDisplayKind {
        return match kind {
            "default" | "d" => UnicodeDisplayKind::Default,
            "locale" | "l" => UnicodeDisplayKind::Show,
            "escape" | "e" => UnicodeDisplayKind::Escape,
            "invalid" | "i" => UnicodeDisplayKind::Invalid,
            "hex" | "x" => UnicodeDisplayKind::Hex,
            "highlight" | "h" => UnicodeDisplayKind::Highlight,
            wrong => {
                panic!("invalid argument to -u/--unicode: {}", wrong);
            }
        };
    }
}

impl EncodingKind {
    fn from(kind: char) -> EncodingKind {
        return match kind {
            's' => EncodingKind::Bit7,
            'S' => EncodingKind::Bit8,
            'b' => EncodingKind::BigEndian16,
            'l' => EncodingKind::LittleEndian16,
            'B' => EncodingKind::BigEndian32,
            'L' => EncodingKind::LittleEndian32,
            wrong => {
                panic!("invalid argument to -e/--encoding: {}", wrong);
            }
        };
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {

    /// Sets the input file(s) to scan (stdin by default)
    #[clap()]
    files: Vec<OsString>,

    /// Scan the entire file, not just the data section [default].
    #[clap(short, long)]
    all: bool,

    /// Scan only the initialized data section(s) of object files.
    #[clap(short, long)]
    data: bool,

    /// Print the name of the file before each string.
    #[clap(short = 'f', long = "print-file-name")]
    print_file_name: bool,

    /// Print graphic char sequences, MIN-LEN or more bytes long, that are followed by a NUL or
    /// a newline.  Default is 4.
    #[clap(short = 'n', long="bytes", default_value = "4")]
    min_bytes: u16,

    /// Print the offset within the file before each string, in octal/hex/decimal.
    /// Values are {o,x,d}.
    #[clap(short = 't', long)]
    radix: Option<String>,

    /// Like -to. (Some other implementations have -o like -to, others like -td.
    /// We chose one arbitrarily.)
    #[clap(short = 'o')]
    octal_radix: bool,

    /// By default tab and space are the only whitespace included in graphic char sequences.
    /// This option considers all of isspace() valid.
    #[clap(short = 'w', long="include-all-whitespace")]
    include_all_whitespace: bool,

    /// Select character encoding: 7-bit-character, 8-bit-character, bigendian 16-bit,
    /// littleendian 16-bit, bigendian 32-bit,  littleendian 32-bit. Values are {s,S,b,l,B,L}.
    #[clap(short, long)]
    encoding: Option<String>,

    /// Determine how to handle UTF-8 unicode characters.  The default  is no special treatment.
    /// All other versions of this option  only apply if the encoding is valid and enabling the
    /// option implies --encoding=S.  The 'show' option displays the characters according to
    /// the current locale.  The 'invalid' option treats them as non-string characters.
    /// The 'hex' option displays them as hex byte sequences.  The 'escape' option displays
    /// them as escape sequences and the 'highlight' option displays them as coloured escape
    /// sequences. Values are {default|show|invalid|hex|escape|highlight}.
    #[clap(short, long)]
    unicode: Option<String>,

    /// String used to separate parsed strings in output.  Default is newline.
    #[clap(short='s', long="output-separator")]
    output_separator: Option<String>
}

fn main() {
    let cli_args = CliArgs::parse();

    let run_options = Options::new(&cli_args);

    let mut success = true;

    if !cli_args.files.is_empty() {
        for file in cli_args.files {
            success &= strings::print_strings_for_file(file.as_os_str(), &run_options);
        }
    } else {
        strings::print_strings_for_stdin(&run_options);
    }

    std::process::exit((!success).into())
}
