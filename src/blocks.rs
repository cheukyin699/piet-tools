use image::io::Reader;
use image::ImageRgb8;
use std::collections::{HashSet, HashMap};
use std::io;
use crate::utils::Coord;

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
#[repr(u8)]
pub enum Hue {
    Red = 0, Yellow, Green, Cyan, Blue, Magenta
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
#[repr(u8)]
pub enum Lightness {
    Light = 0, Normal, Dark
}

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Color(Lightness, Hue),
    Black,
    White
}

pub fn to_blocktype(color: &[u8; 3]) -> Type {
    match color {
        [0xff, 0xc0, 0xc0] => Type::Color(Lightness::Light, Hue::Red),
        [0xff, 0x00, 0x00] => Type::Color(Lightness::Normal, Hue::Red),
        [0xc0, 0x00, 0x00] => Type::Color(Lightness::Dark, Hue::Red),
        [0xff, 0xff, 0xc0] => Type::Color(Lightness::Light, Hue::Yellow),
        [0xff, 0xff, 0x00] => Type::Color(Lightness::Normal, Hue::Yellow),
        [0xc0, 0xc0, 0x00] => Type::Color(Lightness::Dark, Hue::Yellow),
        [0xc0, 0xff, 0xc0] => Type::Color(Lightness::Light, Hue::Green),
        [0x00, 0xff, 0x00] => Type::Color(Lightness::Normal, Hue::Green),
        [0x00, 0xc0, 0x00] => Type::Color(Lightness::Dark, Hue::Green),
        [0xc0, 0xff, 0xff] => Type::Color(Lightness::Light, Hue::Cyan),
        [0x00, 0xff, 0xff] => Type::Color(Lightness::Normal, Hue::Cyan),
        [0x00, 0xc0, 0xc0] => Type::Color(Lightness::Dark, Hue::Cyan),
        [0xc0, 0xc0, 0xff] => Type::Color(Lightness::Light, Hue::Blue),
        [0x00, 0x00, 0xff] => Type::Color(Lightness::Normal, Hue::Blue),
        [0x00, 0x00, 0xc0] => Type::Color(Lightness::Dark, Hue::Blue),
        [0xff, 0xc0, 0xff] => Type::Color(Lightness::Light, Hue::Magenta),
        [0xff, 0x00, 0xff] => Type::Color(Lightness::Normal, Hue::Magenta),
        [0xc0, 0x00, 0xc0] => Type::Color(Lightness::Dark, Hue::Magenta),

        [0x00, 0x00, 0x00] => Type::Black,
        [0xff, 0xff, 0xff] => Type::White,
        _ => panic!("Invalid color type #{:?}", color)
    }
}

#[derive(Debug)]
pub struct Block {
    pub t: Type,
    pub coords: HashSet<Coord>
}

impl Block {
    pub fn is_next_to(&self, other: Coord) -> bool {
        let (x, y) = other;
        let left = &(x - 1, y);
        let right = &(x + 1, y);
        let up = &(x, y - 1);
        let down = &(x, y + 1);

        self.coords.contains(left) || self.coords.contains(right) ||
            self.coords.contains(up) || self.coords.contains(down)
    }
}

pub struct Blocks {
    blocks: Vec<Block>,
    blkLookup: HashMap<Coord, usize>
}

impl <'a> Blocks {
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn from_file(filename: &str, codel_size: usize) -> Result<Blocks, io::Error> {
        let img = Reader::open(filename)?.decode().unwrap();
        let mut blocks: Vec<Block> = vec![];
        let mut lookup: HashMap<Coord, usize> = HashMap::new();

        let img = match img {
            ImageRgb8(rgb8) => rgb8,
            _ => panic!("Invalid image type")
        };

        let (w, h) = img.dimensions();

        // Add coordinates to the blocks
        for x in (0..w).step_by(codel_size) {
            for y in (0..h).step_by(codel_size) {
                let p = img.get_pixel(x, y);
                let t = to_blocktype(&[p[0], p[1], p[2]]);
                let coord: Coord = (x as i32, y as i32);
                let mut found = false;
                for i in 0..blocks.len() {
                    if blocks[i].t == t && blocks[i].is_next_to(coord) {
                        blocks[i].coords.insert(coord);
                        lookup.insert(coord, i);
                        found = true;
                        break;
                    }
                }

                if !found {
                    let mut hs: HashSet<Coord> = HashSet::new();
                    hs.insert(coord);
                    blocks.push(Block {t: t, coords: hs});
                    lookup.insert(coord, blocks.len() - 1);
                }
            }
        }

        Ok(Blocks {blocks: blocks, blkLookup: lookup})
    }

    pub fn find_block_from_index(&'a self, crd: &Coord) -> &'a Block {
        self.blocks.get(self.blkLookup[crd]).unwrap()
    }
}
