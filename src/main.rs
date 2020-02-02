use fallible_iterator::FallibleIterator;
use image::{GrayAlphaImage, LumaA};
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, SeekFrom};

fn process(
    buffered: impl BufRead,
    txt: &mut impl Write,
    image: &mut GrayAlphaImage,
    mut i: usize,
    upper: bool,
    wide: bool,
) -> io::Result<usize> {
    for line in buffered
        .lines()
        .flatten()
        .filter(|x| x.len() == if wide { 64 } else { 32 } + if upper { 7 } else { 5 })
    {
        if upper && &line[0..5] == "0020:" {
            continue;
        }

        let chr_x = (i % 16) * if wide { 16 } else { 8 };
        let chr_y = (i / 16) * 16;
        let chrstr = if upper { &line[0..6] } else { &line[0..4] };
        let chr32 = u32::from_str_radix(chrstr, 16).unwrap();
        let mut utf8_buf = [0; 4];
        let chr = char::try_from(chr32).unwrap();
        txt.write_all(chr.encode_utf8(&mut utf8_buf).as_ref())?;
        let len = if wide { 32 } else { 16 };
        for i in 0..len {
            let y = if wide { i / 2 } else { i };
            let startidx = (i * 2) + if upper { 7 } else { 5 };
            let endidx = (i * 2) + if upper { 7 } else { 5 } + 2;
            let slice = &line[startidx..endidx];
            let byte = u8::from_str_radix(slice, 16).unwrap();
            for x in 0..8 {
                let bit = (byte >> (7 - x)) & 1;
                let px = if wide {
                    chr_x + x + (i % 2) * 8
                } else {
                    chr_x + x
                };
                let py = chr_y + y;
                if bit == 1 {
                    image.put_pixel(px as _, py as _, LumaA([0, 255]));
                }
            }
        }
        i += 1;
    }
    Ok(i)
}

fn main() -> io::Result<()> {
    let hex = File::open(env::args().nth(1).unwrap())?;
    let upper_hex = File::open(env::args().nth(2).unwrap())?;
    let mut buffered = BufReader::new(hex);
    let mut upper_buffered = BufReader::new(upper_hex);
    let mut txt = File::create(env::args().nth(3).unwrap())?;
    let mut wide_txt = File::create(env::args().nth(5).unwrap())?;
    let lower_lines = fallible_iterator::convert(buffered.by_ref().lines())
        .filter(|x| Ok(x.len() == 32 + 5))
        .count()?;
    let upper_lines = fallible_iterator::convert(upper_buffered.by_ref().lines())
        .filter(|x| Ok(x.len() == 32 + 7))
        .count()?;
    let lines = lower_lines + upper_lines;
    buffered.seek(SeekFrom::Start(0))?;
    upper_buffered.seek(SeekFrom::Start(0))?;
    let lower_wide_lines = fallible_iterator::convert(buffered.by_ref().lines())
        .filter(|x| Ok(x.len() == 64 + 5))
        .count()?;
    let upper_wide_lines = fallible_iterator::convert(upper_buffered.by_ref().lines())
        .filter(|x| Ok(x.len() == 64 + 7))
        .count()?;
    let wide_lines = lower_wide_lines + upper_wide_lines;
    buffered.seek(SeekFrom::Start(0))?;
    upper_buffered.seek(SeekFrom::Start(0))?;
    let height = ((lines + 15) / 16) * 16;
    let wide_height = ((wide_lines + 15) / 16) * 16;
    let mut image = GrayAlphaImage::new(128, height as _);
    let mut wide_image = GrayAlphaImage::new(256, wide_height as _);
    let i = process(buffered.by_ref(), &mut txt, &mut image, 0, false, false)?;
    process(
        upper_buffered.by_ref(),
        &mut txt,
        &mut image,
        i,
        true,
        false,
    )?;
    image.save(env::args().nth(4).unwrap())?;

    buffered.seek(SeekFrom::Start(0))?;
    upper_buffered.seek(SeekFrom::Start(0))?;

    let i = process(
        buffered.by_ref(),
        &mut wide_txt,
        &mut wide_image,
        0,
        false,
        true,
    )?;
    process(
        upper_buffered.by_ref(),
        &mut wide_txt,
        &mut wide_image,
        i,
        true,
        true,
    )?;
    wide_image.save(env::args().nth(6).unwrap())?;

    Ok(())
}
