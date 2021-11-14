#![allow(unused)]

use std::cmp::min;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use object::{Object, ObjectSection, ReadRef, Section, SectionFlags};
use atty::Stream;
use std::io::{Write, Stdout, Seek, stdin, stdout, Read, BufReader, SeekFrom, Stdin, StdinLock};
use super::utils::*;

macro_rules! write_or_panic {
    ($dst:expr, $($arg:tt)*) => ({
        write!($dst, $($arg)*).expect("Couldn't write data");
    })
}

// region Options

#[derive(Copy, Clone)]
pub enum UnicodeDisplayKind {
    Default,
    Show,
    Escape,
    Hex,
    Highlight,
    Invalid,
}

#[derive(Copy, Clone)]
pub enum EncodingKind {
    Bit7,
    Bit8,
    BigEndian16,
    LittleEndian16,
    BigEndian32,
    LittleEndian32,
}

impl EncodingKind {
    const fn num_bytes(&self) -> u8 {
        return match self {
            EncodingKind::Bit7 | EncodingKind::Bit8 => 1,
            EncodingKind::BigEndian16 | EncodingKind::LittleEndian16 => 2,
            EncodingKind::BigEndian32 | EncodingKind::LittleEndian32 => 4
        };
    }
}

#[derive(Copy, Clone)]
pub enum RadixKind {
    Oct,
    Dec,
    Hex,
}

pub struct Options {
    pub datasection_only: bool,
    pub print_filenames: bool,
    pub min_length: u16,
    pub include_all_whitespace: bool,
    pub print_addresses: bool,
    pub address_radix: RadixKind,
    pub encoding: EncodingKind,
    pub output_separator: Option<String>,
    pub unicode_display: UnicodeDisplayKind,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            datasection_only: false,
            print_filenames: false,
            min_length: 4,
            include_all_whitespace: false,
            print_addresses: false,
            address_radix: RadixKind::Hex,
            output_separator: None,
            encoding: EncodingKind::Bit7,
            unicode_display: UnicodeDisplayKind::Default,
        }
    }
}

// endregion

const SEC_ALLOC: u64 = 0x1;
const SEC_LOAD: u64 = 0x2;
const SEC_HAS_CONTENTS: u64 = 0x100;

const MAX_KEEP_BACK_SIZE: usize = 4;

const DATA_FLAGS: u64 = SEC_ALLOC | SEC_LOAD | SEC_HAS_CONTENTS;

// region internal data structures

trait DataSource {
    fn read_byte(&mut self) -> Option<u8>;
    fn read_symbol(&mut self, encoding: &EncodingKind) -> Option<(u32, u8)>;
    fn seek_back(&mut self, num_bytes: u8);
}

struct ByteArrayHolder<'a> {
    inner: &'a [u8],
    position: usize,
}

impl DataSource for ByteArrayHolder<'_> {
    fn read_byte(&mut self) -> Option<u8> {
        return match self.read_symbol(&EncodingKind::Bit8) {
            Some(x) => Some(x.0 as u8),
            None => None
        };
    }

    fn read_symbol(&mut self, encoding: &EncodingKind) -> Option<(u32, u8)> {
        let mut current = 0u8;
        let mut num_read = 0u8;
        let mut result = 0u32;

        if self.inner.is_empty() {
            return None;
        }

        while num_read < encoding.num_bytes() {
            if self.position + num_read as usize >= self.inner.len() {
                break;
            }
            current = self.inner[self.position + num_read as usize];
            result = (result << 8) | (current as u32 & 0xff);
            num_read += 1;
        }

        if num_read == 0 {
            return None;
        }

        match encoding {
            EncodingKind::LittleEndian16 => {
                result = to_little_endian_16(result);
            }
            EncodingKind::LittleEndian32 => {
                result = to_little_endian_32(result);
            }
            _ => {
                // not interested
            }
        }

        self.position += num_read as usize;

        return Some((result, num_read));
    }

    fn seek_back(&mut self, num_bytes: u8) {
        self.position -= num_bytes as usize;
    }
}

struct ReaderWithSeek<'a> {
    inner: Box<(dyn Read + 'a)>,
    back_buf: VecDeque<u8>,
    back_pos: usize,
}

impl<'a> Into<ReaderWithSeek<'a>> for BufReader<File> {
    fn into(self) -> ReaderWithSeek<'a> {
        return ReaderWithSeek {
            inner: Box::new(self),
            back_buf: VecDeque::with_capacity(MAX_KEEP_BACK_SIZE),
            back_pos: 0,
        };
    }
}

impl<'a> Into<ReaderWithSeek<'a>> for BufReader<StdinLock<'a>> {
    fn into(self) -> ReaderWithSeek<'a> {
        return ReaderWithSeek {
            inner: Box::new(self),
            back_buf: VecDeque::with_capacity(MAX_KEEP_BACK_SIZE),
            back_pos: 0,
        };
    }
}

impl DataSource for ReaderWithSeek<'_> {
    fn read_byte(&mut self) -> Option<u8> {
        return match self.read_symbol(&EncodingKind::Bit8) {
            Some(x) => Some(x.0 as u8),
            None => None
        };
    }

    fn read_symbol(&mut self, encoding: &EncodingKind) -> Option<(u32, u8)> {
        let mut current = 0u8;
        let mut num_read = 0u8;
        let mut result = 0u32;

        let mut buf = [0u8; 1];
        while num_read < encoding.num_bytes() {
            if self.back_pos > 0 {
                current = self.back_buf[self.back_buf.len() - self.back_pos];
                self.back_pos -= 1;
            } else {
                current = match self.inner.read_exact(&mut buf) {
                    Ok(_) => {
                        buf[0]
                    }
                    Err(_) => {
                        break;
                    }
                };
                self.back_buf.push_back(current as u8);
            }

            while self.back_buf.len() > MAX_KEEP_BACK_SIZE {
                self.back_buf.pop_front();
            }

            result = (result << 8) | (current as u32 & 0xff);
            num_read += 1;
        }

        if num_read == 0 {
            return None;
        }

        match encoding {
            EncodingKind::LittleEndian16 => {
                result = to_little_endian_16(result);
            }
            EncodingKind::LittleEndian32 => {
                result = to_little_endian_32(result);
            }
            _ => {
                // not interested
            }
        }

        return Some((result, num_read));
    }

    fn seek_back(&mut self, num_bytes: u8) {
        self.back_pos += num_bytes as usize;
        if self.back_pos > self.back_buf.len() {
            panic!("Cannot seek back more than {} bytes", MAX_KEEP_BACK_SIZE)
        }
    }
}

// endregion

pub fn print_strings_for_file(file_path_str: &OsStr, options: &Options) -> bool {
    let file_path = Path::new(file_path_str);

    if !file_path.exists() {
        eprintln!("{:?}: No such file", file_path_str);
        return false;
    }

    if file_path.is_dir() {
        eprintln!("Warning: '{:?}' is a directory", file_path_str);
        return false;
    }

    if !options.datasection_only || !print_strings_for_object_file(file_path, options) {
        let stdout = stdout();
        let mut writer = stdout.lock();

        let mut reader: ReaderWithSeek = BufReader::new(
            File::open(file_path).expect("Couldn't open the file.")
        ).into();

        print_strings(file_path_str.to_str().expect("Couldn't convert file path to string"),
                      0, &mut reader, options, &mut writer);

        writer.flush();
        return true;
    }
    return true;
}

pub fn print_strings_for_stdin(options: &Options) {
    let stdin = stdin();
    let stdout = stdout();
    let mut writer = stdout.lock();
    let mut reader: ReaderWithSeek = BufReader::new(stdin.lock()).into();
    print_strings("<stdin>", 0, &mut reader, options, &mut writer);
    writer.flush();
}

fn print_strings_for_object_file(file_path: &Path, options: &Options) -> bool {
    return match std::fs::read(file_path) {
        Ok(data) => {
            if let Ok(object) = object::File::parse(&*data) {
                let mut got_section = false;
                for section in object.sections() {
                    got_section |= print_strings_for_object_section(
                        file_path.as_os_str(), &section, options,
                    );
                }
                got_section
            } else {
                println!("File is not an object");
                false
            }
        }
        Err(err) => {
            println!("Warning: could not open '{:?}'.  reason: {}", file_path, err);
            false
        }
    };
}

fn print_strings_for_object_section(
    filename: &OsStr,
    section: &Section,
    options: &Options,
) -> bool {
    if !is_data_section(section) || section.size() == 0 {
        return false;
    }

    if let Ok(compressed_data) = section.compressed_data() {
        let stdout = stdout();
        let mut writer = stdout.lock();
        let mut byte_holder = ByteArrayHolder {
            inner: compressed_data.data,
            position: 0,
        };
        print_strings(
            filename.to_str().unwrap(),
            section.address(),
            &mut byte_holder, options,
            &mut writer,
        );
        writer.flush();
        return true;
    }

    return false;
}

fn is_data_section(section: &Section) -> bool {
    let flags = match section.flags() {
        SectionFlags::Elf { sh_flags } => {
            sh_flags
        }
        SectionFlags::MachO { flags } => {
            flags as u64
        }
        SectionFlags::Coff { characteristics } => {
            characteristics as u64
        }
        _ => 0
    };

    if flags == 0 {
        return false;
    }

    // TODO check here, use flags maybe? Elf() type? is it complete match?
    return matches!(section.kind(), object::SectionKind::Metadata)
        || matches!(section.kind(), object::SectionKind::ReadOnlyData)
        || matches!(section.kind(), object::SectionKind::Text);
}

fn print_strings(
    filename: &str,
    address: u64,
    data: &mut dyn DataSource,
    options: &Options,
    writer: &mut dyn Write,
) {
    if !matches!(options.unicode_display, UnicodeDisplayKind::Default) {
        // TODO downcast? support buffer vs stream case?
        //print_unicode_buffer(filename, address, data, options, writer);
        print_unicode_stream(filename, address, data, options, writer);
        return;
    }

    let mut search_start_address = address;
    let mut buffer = Vec::<u8>::new();

    // TODO split this giant method.
    // current logic of this big loop:
    // * Search for a matching sequence. Once found, we will have a sequence (content
    // + start address + end address).
    // * Print sequence start address
    // * Print sequence content and continue to scan until wrong char found.
    loop {
        let mut current_address = search_start_address;

        if let Some(address) = find_matching_ascii_sequence(
            search_start_address, data, &mut buffer, options,
        ) {
            search_start_address = address;
            current_address = address + buffer.len() as u64;
        } else {
            return;
        }

        /* We found a run of `string_min' graphic characters.  Print up
         to the next non-graphic character.  */
        print_filename_and_address(filename, search_start_address, options, writer);

        // continue until we find non-valid char
        loop {
            let (character, read) = match data.read_symbol(&options.encoding) {
                Some(x) => x,
                None => break
            };
            current_address += read as u64;
            if character > 255 || !char_is_printable(character as u8 as char,
                                                     options.encoding,
                                                     options.include_all_whitespace) {
                current_address -= read as u64;
                data.seek_back(read);
                break;
            }
            buffer.push(character as u8);
        }

        if let Some(separator) = &options.output_separator {
            buffer.extend_from_slice(separator.as_bytes());
        } else {
            buffer.push('\n' as u8);
        }

        std::io::copy(&mut buffer.as_slice(), writer);
        buffer.clear();

        search_start_address = current_address;
    }
}

/*
 Finds an ASCII sequence which is matching the min length criteria. It will be written to
 the buffer and start address will be returned.
 */
fn find_matching_ascii_sequence(
    start_address: u64,
    data: &mut dyn DataSource,
    buffer: &mut Vec<u8>,
    options: &Options,
) -> Option<u64> {
    let mut search_start_address = start_address;
    let mut current_address = start_address;
    let mut symbol = 0u32;

    /* See if the next `string_min' chars are all graphic chars.  */
    let mut should_retry = true;

    while should_retry {
        current_address = search_start_address;
        should_retry = false;

        if !buffer.is_empty() {
            buffer.clear();
        }

        let mut i = 0u16;
        while i < options.min_length {
            let (character, read) = data.read_symbol(&options.encoding)?;
            symbol = character;
            current_address += read as u64;

            if symbol > 255 || !char_is_printable(symbol as u8 as char, options.encoding,
                                                  options.include_all_whitespace) {
                /* Found a non-graphic.  Try again starting with next byte.  */
                search_start_address =
                    current_address - (options.encoding.num_bytes() as u64 - 1);
                data.seek_back(read - 1);
                should_retry = true;
                break;
            }

            // TODO wrong cast, symbol can be up to 4 bytes
            buffer.push(symbol as u8);

            i += 1;
        }
    }

    return Some(current_address - buffer.len() as u64);
}

fn print_unicode_buffer(
    filename: &str,
    address: u64,
    buffer: &[u8],
    options: &Options,
    writer: &mut dyn Write,
) {
    if !matches!(options.encoding, EncodingKind::Bit8) {
        eprintln!("ICE: bad arguments to print_unicode_buffer");
        return;
    }

    let mut current_buffer = buffer;
    let mut current_address = address;

    loop {
        if current_buffer.len() == 0 {
            return;
        }

        /* We must only display strings that are at least string_min *characters*
       long.  So we scan the buffer in two stages.  First we locate the start
       of a potential string.  Then we walk along it until we have found
       string_min characters.  Then we go back to the start point and start
       displaying characters according to the unicode_display setting.  */

        let mut sequence_start_address_offset = 0usize;
        let mut address_offset = 0usize;
        let mut char_len = 1u8;
        let mut num_found = 0u16;

        loop {
            if address_offset == current_buffer.len() {
                break;
            }

            let c = current_buffer[address_offset];

            char_len = 1;

            /* Find the first potential character of a string.  */
            if !char_is_printable(c as char, options.encoding, options.include_all_whitespace) {
                num_found = 0;
                address_offset += char_len as usize;
                continue;
            }

            if c > 126 {
                if c < 0xc0 {
                    num_found = 0;
                    address_offset += char_len as usize;
                    continue;
                }

                char_len = is_valid_utf8(&current_buffer[address_offset..]);
                if char_len == 0 {
                    char_len = 1;
                    num_found = 0;
                    address_offset += 1;
                    continue;
                }

                if matches!(options.unicode_display, UnicodeDisplayKind::Invalid) {
                    /* We have found a valid UTF-8 character, but we treat it as non-graphic.  */
                    num_found = 0;
                    address_offset += char_len as usize;
                    continue;
                }
            }

            if num_found == 0 {
                /* We have found a potential starting point for a string.  */
                sequence_start_address_offset = address_offset;
            }

            num_found += 1;

            if num_found >= options.min_length {
                break;
            }

            address_offset += char_len as usize;
        }

        if num_found < options.min_length {
            return;
        }

        print_filename_and_address(
            filename,
            current_address + sequence_start_address_offset as u64,
            options,
            writer,
        );

        /* We have found string_min characters.  Display them and any
       more that follow.  */
        let mut offset = sequence_start_address_offset;
        while offset < current_buffer.len() {
            let c = current_buffer[offset];

            char_len = 1;

            if !char_is_printable(c as char, options.encoding, options.include_all_whitespace) {
                break;
            } else if c < 127 {
                write_or_panic!(writer, "{}", c as char);
            } else if is_valid_utf8(&current_buffer[offset..]) == 0 {
                break;
            } else if matches!(options.unicode_display, UnicodeDisplayKind::Invalid) {
                break;
            } else {
                // TODO max slice length?
                char_len = display_utf8_char(
                    &current_buffer[offset..offset + 4],
                    options.unicode_display,
                    writer,
                );
            }
            offset += char_len as usize;
        }

        if let Some(separator) = &options.output_separator {
            write_or_panic!(writer, "{}", separator.as_str());
        } else {
            write_or_panic!(writer, "\n");
        }

        current_address += offset as u64;
        current_buffer = &current_buffer[offset..];
    }
}

/*
UTF-8 structure

First code point 	Last code point 	Byte 1 	    Byte 2 	    Byte 3 	    Byte 4
U+0000 	            U+007F 	            0xxxxxxx
U+0080 	            U+07FF 	            110xxxxx 	10xxxxxx
U+0800 	            U+FFFF 	            1110xxxx 	10xxxxxx 	10xxxxxx
U+10000             U+10FFFF 	        11110xxx 	10xxxxxx 	10xxxxxx 	10xxxxxx
 */
fn print_unicode_stream(
    filename: &str,
    address: u64,
    data: &mut dyn DataSource,
    options: &Options,
    writer: &mut dyn Write,
) {
    if !matches!(options.encoding, EncodingKind::Bit8) {
        eprintln!("ICE: bad arguments to print_unicode_stream");
        return;
    }

    /* It would be nice if we could just read the stream into a buffer
         and then process if with print_unicode_buffer.  But the input
         might be huge or it might time-locked (eg stdin).  So instead
         we go one byte at a time...  */
    // TODO rewrite this big chunk of code
    loop {
        let mut start_point = 0usize;
        let mut num_read = 0u16;
        // number of unicode chars
        let mut num_chars = 0u16;
        // number of bytes
        let mut num_print = 0u16;
        let mut c = 0u8;
        let mut print_buf = vec![0u8; (options.min_length * 4) as usize];

        /* Find a series of string_min characters.  Put them into print_buf.  */
        loop {
            if num_chars >= options.min_length {
                break;
            }

            c = match data.read_byte() {
                Some(value) => {
                    value
                }
                None => return
            };
            num_read += 1;

            if !char_is_printable(c as char, options.encoding, options.include_all_whitespace) {
                num_chars = 0;
                num_print = 0;
                continue;
            }

            if num_chars == 0 {
                start_point = num_read as usize - 1;
            }

            if c < 127 {
                print_buf[num_print as usize] = c;
                num_chars += 1;
                num_print += 1;
                continue;
            }

            if c < 0xc0 {
                num_chars = 0;
                num_print = 0;
                continue;
            }

            /* We *might* have a UTF-8 sequence.  Time to start peeking.  */
            let mut utf8 = [0u8; 4];

            utf8[0] = c;
            c = match data.read_byte() {
                Some(value) => {
                    value
                }
                None => return
            };
            num_read += 1;
            utf8[1] = c;

            if (utf8[1] & 0xc0) != 0x80 {
                /* Invalid UTF-8.  */
                data.seek_back(1);
                num_chars = 0;
                num_print = 0;
                continue;
            } else if (utf8[0] & 0x20) == 0 {
                /* A valid 2-byte UTF-8 encoding.  */
                if matches!(options.unicode_display, UnicodeDisplayKind::Invalid) {
                    data.seek_back(1);
                    num_chars = 0;
                    num_print = 0;
                } else {
                    print_buf[num_print as usize] = utf8[0];
                    num_print += 1;
                    print_buf[num_print as usize] = utf8[1];
                    num_print += 1;
                    num_chars += 1;
                }
                continue;
            }

            c = match data.read_byte() {
                Some(value) => {
                    value
                }
                None => return
            };
            num_read += 1;
            utf8[2] = c;

            if (utf8[2] & 0xc0) != 0x80 {
                /* Invalid UTF-8.  */
                data.seek_back(2);
                num_chars = 0;
                num_print = 0;
                continue;
            } else if (utf8[0] & 0x10) == 0 {
                /* A valid 3-byte UTF-8 encoding.  */
                if matches!(options.unicode_display, UnicodeDisplayKind::Invalid) {
                    data.seek_back(2);
                    num_chars = 0;
                    num_print = 0;
                } else {
                    print_buf[num_print as usize] = utf8[0];
                    num_print += 1;
                    print_buf[num_print as usize] = utf8[1];
                    num_print += 1;
                    print_buf[num_print as usize] = utf8[2];
                    num_print += 1;
                    num_chars += 1;
                }
                continue;
            }

            c = match data.read_byte() {
                Some(value) => {
                    value
                }
                None => return
            };
            num_read += 1;
            utf8[3] = c;

            if (utf8[3] & 0xc0) != 0x80 {
                /* Invalid UTF-8.  */
                data.seek_back(3);
                num_chars = 0;
                num_print = 0;
            } else if matches!(options.unicode_display, UnicodeDisplayKind::Invalid) {
                /* We have a valid 4-byte UTF-8 encoding.  */
                data.seek_back(3);
                num_chars = 0;
                num_print = 0;
            } else {
                print_buf[num_print as usize] = utf8[0];
                num_print += 1;
                print_buf[num_print as usize] = utf8[1];
                num_print += 1;
                print_buf[num_print as usize] = utf8[2];
                num_print += 1;
                print_buf[num_print as usize] = utf8[3];
                num_print += 1;
                num_chars += 1;
            }
        }

        if num_chars >= options.min_length {
            /* We know that we have string_min valid characters in print_buf,
         and there may be more to come in the stream.  Start displaying
         them.  */

            print_filename_and_address(filename, address + start_point as u64, options, writer);

            let mut i = 0usize;
            loop {
                if i >= num_print as usize {
                    break;
                }
                if print_buf[i] < 127 {
                    write_or_panic!(writer, "{}", print_buf[i] as char);
                    i += 1;
                } else {
                    i += display_utf8_char(
                        &print_buf[i..i + 4],
                        options.unicode_display,
                        writer,
                    ) as usize;
                }
            }

            /* OK so now we have to start read unchecked bytes.  */
            loop {
                c = match data.read_byte() {
                    Some(value) => {
                        value
                    }
                    None => return
                };
                num_read += 1;

                if !char_is_printable(c as char, options.encoding, options.include_all_whitespace) {
                    break;
                }

                if c < 127 {
                    write_or_panic!(writer, "{}", c as char);
                    continue;
                }

                if c < 0xc0 {
                    break;
                }

                /* We *might* have a UTF-8 sequence.  Time to start peeking.  */
                let mut utf8 = [0u8; 4];

                utf8[0] = c;
                c = match data.read_byte() {
                    Some(value) => {
                        value
                    }
                    None => return
                };
                num_read += 1;
                utf8[1] = c;

                if (utf8[1] & 0xc0) != 0x80 {
                    /* Invalid UTF-8.  */
                    data.seek_back(1);
                    break;
                } else if (utf8[0] & 0x20) == 0 {
                    /* Valid 2-byte UTF-8.  */
                    if matches!(options.unicode_display, UnicodeDisplayKind::Invalid) {
                        data.seek_back(1);
                        break;
                    } else {
                        display_utf8_char(&utf8, options.unicode_display, writer);
                        continue;
                    }
                }

                c = match data.read_byte() {
                    Some(value) => {
                        value
                    }
                    None => return
                };
                num_read += 1;
                utf8[2] = c;

                if (utf8[2] & 0xc0) != 0x80 {
                    /* Invalid UTF-8.  */
                    data.seek_back(1);
                    break;
                } else if (utf8[0] & 0x10) == 0 {
                    /* Valid 3-byte UTF-8.  */
                    if matches!(options.unicode_display, UnicodeDisplayKind::Invalid) {
                        data.seek_back(1);
                        break;
                    } else {
                        display_utf8_char(&utf8, options.unicode_display, writer);
                        continue;
                    }
                }

                c = match data.read_byte() {
                    Some(value) => {
                        value
                    }
                    None => return
                };
                num_read += 1;
                utf8[3] = c;

                if (utf8[3] & 0xc0) != 0x80 {
                    /* Invalid UTF-8.  */
                    data.seek_back(3);
                    break;
                } else if matches!(options.unicode_display, UnicodeDisplayKind::Invalid) {
                    data.seek_back(3);
                    break;
                } else {
                    /* A valid 4-byte UTF-8 encoding.  */
                    display_utf8_char(&utf8, options.unicode_display, writer);
                }
            }

            if let Some(separator) = &options.output_separator {
                write_or_panic!(writer, "{}", separator.as_str());
            } else {
                write_or_panic!(writer, "\n");
            }

            start_point += num_read as usize;
        }
    }
}

fn print_filename_and_address(
    filename: &str,
    address: u64,
    options: &Options,
    writer: &mut dyn Write,
) {
    if options.print_filenames {
        write_or_panic!(writer, "{}: ", filename);
    }

    if !options.print_addresses {
        return;
    }

    // TODO should support longer addresses?
    match options.address_radix {
        RadixKind::Oct => {
            write_or_panic!(writer, "{:7o} ", address);
        }
        RadixKind::Dec => {
            write_or_panic!(writer, "{:7} ", address);
        }
        RadixKind::Hex => {
            write_or_panic!(writer, "{:7x} ", address);
        }
    }
}

fn display_utf8_char(buffer: &[u8], display: UnicodeDisplayKind, writer: &mut dyn Write) -> u8 {
    let utf8_len = match buffer[0] & 0x30 {
        0x00 | 0x10 => 2u8,
        0x20 => 3u8,
        _ => 4u8
    };

    match display {
        UnicodeDisplayKind::Escape | UnicodeDisplayKind::Highlight => {
            if matches!(display, UnicodeDisplayKind::Highlight) && atty::is(Stream::Stdout) {
                write_or_panic!(writer, "\x1B[31;47m"); /* Red.  */
            }
            match utf8_len {
                2 => {
                    write_or_panic!(
                        writer,
                        "\\u{:02x}{:02x}",
                        ((buffer[0] & 0x1c) >> 2),
                        ((buffer[0] & 0x03) << 6) | (buffer[1] & 0x3f));
                }

                3 => {
                    write_or_panic!(
                        writer,
                        "\\u{:02x}{:02x}",
                        ((buffer[0] & 0x0f) << 4) | ((buffer[1] & 0x3c) >> 2),
                        ((buffer[1] & 0x03) << 6) | ((buffer[2] & 0x3f)));
                }

                4 => {
                    write_or_panic!(
                        writer,
                        "\\u{:02x}{:02x}{:02x}",
                        ((buffer[0] & 0x07) << 6) | ((buffer[1] & 0x3c) >> 2),
                        ((buffer[1] & 0x03) << 6) | ((buffer[2] & 0x3c) >> 2),
                        ((buffer[2] & 0x03) << 6) | ((buffer[3] & 0x3f)));
                }
                _ => {
                    panic!("Unknown utf8_len")
                }
            }

            if matches!(display, UnicodeDisplayKind::Highlight) && atty::is(Stream::Stdout) {
                write_or_panic!(writer, "\033[0m"); /* Default colour.  */
            }
        }
        UnicodeDisplayKind::Hex => {
            write_or_panic!(writer, "<");
            write_or_panic!(writer, "0x");
            for j in 0usize..utf8_len as usize {
                write_or_panic!(writer, "{:02x}", buffer[j]);
            }
            write_or_panic!(writer, ">");
        }
        UnicodeDisplayKind::Show => {
            write_or_panic!(writer, "{:01?}", buffer);
        }
        _ => {
            eprintln!("ICE: unexpected unicode display type");
        }
    }

    return utf8_len;
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_OBJECT_FILE_PATH: &str = "test-resources/a.out";

    #[test]
    fn test_display_utf8_char_escape_2bytes() {
        let mut output = Vec::new();
        display_utf8_char("¢".as_bytes(), UnicodeDisplayKind::Escape, &mut output);

        assert_eq!("\\u00a2", String::from_utf8(output).expect("Not valid UTF8"))
    }

    #[test]
    fn test_display_utf8_char_escape_3bytes() {
        let mut output = Vec::new();
        display_utf8_char("ह".as_bytes(), UnicodeDisplayKind::Escape, &mut output);

        assert_eq!("\\u0939", String::from_utf8(output).expect("Not valid UTF8"))
    }

    #[test]
    fn test_display_utf8_char_escape_4bytes() {
        let mut output = Vec::new();
        display_utf8_char("𐍈".as_bytes(), UnicodeDisplayKind::Escape, &mut output);

        // should be 10348, but strings.c produces the same
        assert_eq!("\\u040348", String::from_utf8(output).expect("Not valid UTF8"))
    }

    #[test]
    fn test_display_utf8_char_hex() {
        let mut output = Vec::new();
        display_utf8_char("𐍈".as_bytes(), UnicodeDisplayKind::Hex, &mut output);

        assert_eq!("<0xf0908d88>", String::from_utf8(output).expect("Not valid UTF8"))
    }

    #[test]
    fn test_display_utf8_char_show() {
        let mut output = Vec::new();
        display_utf8_char("𐍈".as_bytes(), UnicodeDisplayKind::Show, &mut output);

        // TODO recheck this
        assert_eq!("[240, 144, 141, 136]", String::from_utf8(output).expect("Not valid UTF8"))
    }

    #[test]
    fn test_print_strings_default_params() {
        let mut data: ReaderWithSeek = BufReader::new(
            File::open(TEST_OBJECT_FILE_PATH).unwrap()
        ).into();
        let mut output = Vec::new();

        let expected = String::from_utf8(
            std::fs::read("test-resources/default-output.txt").unwrap()
        ).unwrap();

        print_strings(TEST_OBJECT_FILE_PATH, 0, &mut data, &Options::default(), &mut output);
        assert_eq!(expected, String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_print_strings_with_address_hex() {
        let mut data: ReaderWithSeek = BufReader::new(
            File::open(TEST_OBJECT_FILE_PATH).unwrap()
        ).into();
        let mut output = Vec::new();

        let expected = String::from_utf8(
            std::fs::read("test-resources/output-with-address-hex.txt").unwrap()
        ).unwrap();

        let mut options = Options::default();
        options.print_addresses = true;
        options.address_radix = RadixKind::Hex;

        print_strings(TEST_OBJECT_FILE_PATH, 0, &mut data, &options, &mut output);
        assert_eq!(expected, String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_print_strings_with_address_octal() {
        let mut data: ReaderWithSeek = BufReader::new(
            File::open(TEST_OBJECT_FILE_PATH).unwrap()
        ).into();
        let mut output = Vec::new();

        let expected = String::from_utf8(
            std::fs::read("test-resources/output-with-address-octal.txt").unwrap()
        ).unwrap();

        let mut options = Options::default();
        options.print_addresses = true;
        options.address_radix = RadixKind::Oct;

        print_strings(TEST_OBJECT_FILE_PATH, 0, &mut data, &options, &mut output);
        assert_eq!(expected, String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_print_strings_with_separator() {
        let mut data: ReaderWithSeek = BufReader::new(
            File::open(TEST_OBJECT_FILE_PATH).unwrap()
        ).into();
        let mut output = Vec::new();

        let expected = String::from_utf8(
            std::fs::read("test-resources/output-with-separator.txt").unwrap()
        ).unwrap();

        let mut options = Options::default();
        options.output_separator = Some("\n\n".to_string());

        print_strings(TEST_OBJECT_FILE_PATH, 0, &mut data, &options, &mut output);
        assert_eq!(expected, String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_print_strings_num_bytes_8() {
        let mut data: ReaderWithSeek = BufReader::new(
            File::open(TEST_OBJECT_FILE_PATH).unwrap()
        ).into();
        let mut output = Vec::new();

        let expected = String::from_utf8(
            std::fs::read("test-resources/output-with-num-bytes-8.txt").unwrap()
        ).unwrap();

        let mut options = Options::default();
        options.min_length = 8;

        print_strings(TEST_OBJECT_FILE_PATH, 0, &mut data, &options, &mut output);
        assert_eq!(expected, String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_print_strings_encoding_8_bits() {
        let mut data: ReaderWithSeek = BufReader::new(
            File::open(TEST_OBJECT_FILE_PATH).unwrap()
        ).into();
        let mut output = Vec::<u8>::new();

        let expected = std::fs::read("test-resources/output-with-encoding-8-bits.txt")
            .unwrap();

        let mut options = Options::default();
        options.encoding = EncodingKind::Bit8;

        print_strings(TEST_OBJECT_FILE_PATH, 0, &mut data, &options, &mut output);
        assert_eq!(expected, output)
    }

    #[test]
    fn test_print_strings_with_filenames() {
        let mut data: ReaderWithSeek = BufReader::new(
            File::open(TEST_OBJECT_FILE_PATH).unwrap()
        ).into();
        let mut output = Vec::<u8>::new();

        let expected = String::from_utf8(
            std::fs::read("test-resources/output-with-filenames.txt").unwrap()
        ).unwrap();

        let mut options = Options::default();
        options.print_filenames = true;

        print_strings(TEST_OBJECT_FILE_PATH, 0, &mut data, &options, &mut output);
        assert_eq!(expected, String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_print_strings_with_unicode_escape() {
        let mut data: ReaderWithSeek = BufReader::new(
            File::open(TEST_OBJECT_FILE_PATH).unwrap()
        ).into();
        let mut output = Vec::<u8>::new();

        let expected = String::from_utf8(
            std::fs::read("test-resources/output-with-unicode-escape.txt").unwrap()
        ).unwrap();

        let mut options = Options::default();
        options.unicode_display = UnicodeDisplayKind::Escape;
        options.encoding = EncodingKind::Bit8;

        print_strings(TEST_OBJECT_FILE_PATH, 0, &mut data, &options, &mut output);
        assert_eq!(expected, String::from_utf8(output).unwrap())
    }

    #[test]
    fn test_data_source_backed_by_array() {
        let buffer = [0x12u8, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, 0x89, 0xFF, 0xAA];

        let mut source = ByteArrayHolder {
            inner: &buffer,
            position: 0,
        };

        assert_eq!(0x12, source.read_byte().unwrap());

        let (char, read) = source.read_symbol(&EncodingKind::Bit7).unwrap();
        assert_eq!(0x23, char);
        assert_eq!(1, read);

        let (char, read) = source.read_symbol(&EncodingKind::BigEndian32).unwrap();
        assert_eq!(0x34 << 24 | 0x45 << 16 | 0x56 << 8 | 0x67, char);
        assert_eq!(4, read);

        source.seek_back(3);

        let (char, read) = source.read_symbol(&EncodingKind::BigEndian16).unwrap();
        assert_eq!(0x45 << 8 | 0x56, char);
        assert_eq!(2, read);

        let (char, read) = source.read_symbol(&EncodingKind::BigEndian32).unwrap();
        assert_eq!(0x67 << 24 | 0x78 << 16 | 0x89 << 8 | 0xFF, char);
        assert_eq!(4, read);

        let (char, read) = source.read_symbol(&EncodingKind::BigEndian32).unwrap();
        assert_eq!(0xAA, char);
        assert_eq!(1, read);

        assert_eq!(None, source.read_byte());
    }

    #[test]
    fn test_data_source_backed_by_reader_with_seek() {
        let buffer = [0x12u8, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, 0x89, 0xFF, 0xAA];

        let mut source = ReaderWithSeek {
            inner: Box::new(&buffer[..]),
            back_buf: VecDeque::with_capacity(MAX_KEEP_BACK_SIZE),
            back_pos: 0,
        };

        assert_eq!(0x12, source.read_byte().unwrap());

        let (char, read) = source.read_symbol(&EncodingKind::Bit7).unwrap();
        assert_eq!(0x23, char);
        assert_eq!(1, read);

        let (char, read) = source.read_symbol(&EncodingKind::BigEndian32).unwrap();
        assert_eq!(0x34 << 24 | 0x45 << 16 | 0x56 << 8 | 0x67, char);
        assert_eq!(4, read);

        source.seek_back(3);

        let (char, read) = source.read_symbol(&EncodingKind::BigEndian16).unwrap();
        assert_eq!(0x45 << 8 | 0x56, char);
        assert_eq!(2, read);

        let (char, read) = source.read_symbol(&EncodingKind::BigEndian32).unwrap();
        assert_eq!(0x67 << 24 | 0x78 << 16 | 0x89 << 8 | 0xFF, char);
        assert_eq!(4, read);

        let (char, read) = source.read_symbol(&EncodingKind::BigEndian32).unwrap();
        assert_eq!(0xAA, char);
        assert_eq!(1, read);

        assert_eq!(None, source.read_byte());
    }
}
