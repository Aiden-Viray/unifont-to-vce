use fallible_iterator::FallibleIterator;
use image::{GrayAlphaImage, LumaA};
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, SeekFrom};

fn main() -> io::Result<()> {
    let hex = File::open(env::args().nth(1).unwrap())?;
    let mut buffered = BufReader::new(hex);
    let mut txt = File::create(env::args().nth(2).unwrap())?;
    let mut wide_txt = File::create(env::args().nth(4).unwrap())?;
    let lines = fallible_iterator::convert(buffered.by_ref().lines())
        .filter(|x| Ok(x.len() == 32 + 5))
        .count()?;
    buffered.seek(SeekFrom::Start(0))?;
    let wide_lines = fallible_iterator::convert(buffered.by_ref().lines())
        .filter(|x| Ok(x.len() == 64 + 5))
        .count()?;
    buffered.seek(SeekFrom::Start(0))?;
    let height = ((lines + 15) / 16) * 16;
    let wide_height = ((wide_lines + 15) / 16) * 16;
    let mut image = GrayAlphaImage::new(128, height as _);
    let mut wide_image = GrayAlphaImage::new(256, wide_height as _);
    for (i, line) in buffered
        .by_ref()
        .lines()
        .flatten()
        .filter(|x| x.len() == 32 + 5)
        .enumerate()
    {
        let chr_x = (i % 16) * 8;
        let chr_y = (i / 16) * 16;
        let chr16 = u16::from_str_radix(&line[0..4], 16).unwrap();
        let mut utf8_buf = [0; 4];
        let chr = char::try_from(chr16 as u32).unwrap();
        txt.write_all(chr.encode_utf8(&mut utf8_buf).as_ref())?;
        for y in 0..16 {
            let startidx = (y * 2) + 5;
            let endidx = (y * 2) + 5 + 2;
            let slice = &line[startidx..endidx];
            let byte = u8::from_str_radix(slice, 16).unwrap();
            for x in 0..8 {
                let bit = (byte >> (7 - x)) & 1;
                let px = chr_x + x;
                let py = chr_y + y;
                if bit == 1 {
                    image.put_pixel(px as _, py as _, LumaA([0, 255]));
                }
            }
        }
    }

    image.save(env::args().nth(3).unwrap())?;

    buffered.seek(SeekFrom::Start(0))?;

    for (i, line) in buffered
        .by_ref()
        .lines()
        .flatten()
        .filter(|x| x.len() == 64 + 5)
        .enumerate()
    {
        let chr_x = (i % 16) * 16;
        let chr_y = (i / 16) * 16;
        let chr16 = u16::from_str_radix(&line[0..4], 16).unwrap();
        let mut utf8_buf = [0; 4];
        let chr = char::try_from(chr16 as u32).unwrap();
        wide_txt.write_all(chr.encode_utf8(&mut utf8_buf).as_ref())?;
        for i in 0..32 {
            let y = i / 2;
            let startidx = (i * 2) + 5;
            let endidx = (i * 2) + 5 + 2;
            let slice = &line[startidx..endidx];
            let byte = u8::from_str_radix(slice, 16).unwrap();
            for x in 0..8 {
                let bit = (byte >> (7 - x)) & 1;
                let px = chr_x + x + (i % 2) * 8;
                let py = chr_y + y;
                if bit == 1 {
                    wide_image.put_pixel(px as _, py as _, LumaA([0, 255]));
                }
            }
        }
    }

    wide_image.save(env::args().nth(5).unwrap())?;

    Ok(())
}
