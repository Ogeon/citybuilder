use std::io;
use std::mem::{swap, transmute};
use std::iter;
use std::iter::FilterMap;
use std::slice::MutItems;
use std::rand::{Rng, task_rng};
use std::cmp::{min, max};
use std::collections::HashMap;

use rsfml::system::vector2::{Vector2f, Vector2i};
use rsfml::graphics::{RenderWindow, Color};

use tile;
use tile::{Tile, TileType};

pub enum Selection {
    Deselected,
    Selected,
    Invalid
}

pub struct Map {
    width: uint,
    height: uint,
    tiles: Vec<(Tile, uint, Selection)>,
    tile_size: uint,
    pub num_selected: uint,
    num_regions: Vec<uint>
}

impl Map {
    pub fn new_generated(tile_size: uint, tile_atlas: &HashMap<&'static str, Tile>) -> Map {
        let width = 50;
        let height = 50;

        let mut tiles = Vec::new();

        for _ in range(0u, width * height) {
            let tile = if 0.2f32 > task_rng().gen() {
                tile_atlas.find(&"forest").expect("forest tile was not loaded").clone()
            } else if 0.02f32 > task_rng().gen() {
                tile_atlas.find(&"water").expect("water tile was not loaded").clone()
            } else {
                tile_atlas.find(&"grass").expect("grass tile was not loaded").clone()
            };

            tiles.push((tile, 255, Deselected));
        }

        Map {
            width: width,
            height: height,
            tiles: tiles,
            tile_size: tile_size,
            num_selected: 0,
            num_regions: vec![0]
        }
    }

    pub fn load(&mut self, path: &Path, tile_atlas: &HashMap<&'static str, Tile>) -> io::IoResult<()> {
        let mut file = try!(io::File::open(path));
        self.width = try!(file.read_be_u32()) as uint;
        self.height = try!(file.read_be_u32()) as uint;

        let mut tiles = Vec::new();

        for _ in range(0u, self.width * self.height) {
            let mut tile = match try!(file.read_u8()) {
                0 | 1 => tile_atlas.find(&"grass").unwrap().clone(),
                2 => tile_atlas.find(&"forest").unwrap().clone(),
                3 => tile_atlas.find(&"water").unwrap().clone(),
                4 => {
                    let mut tile = tile_atlas.find(&"residential").unwrap().clone();
                    tile.set_population(try!(file.read_be_f64()));
                    tile
                },
                5 => {
                    let mut tile = tile_atlas.find(&"commercial").unwrap().clone();
                    tile.set_population(try!(file.read_be_f64()));
                    tile
                },
                6 => {
                    let mut tile = tile_atlas.find(&"industrial").unwrap().clone();
                    tile.set_population(try!(file.read_be_f64()));
                    tile.set_production(try!(file.read_be_u32()));
                    tile.set_stored_goods(try!(file.read_be_u32()));
                    tile
                },
                7 => tile_atlas.find(&"road").unwrap().clone(),
                n => return Err(io::IoError {
                    kind: io::OtherIoError,
                    desc: "invalid tile type in map file",
                    detail: Some(format!("found type number {}", n))
                })
            };

            tile.variant = try!(file.read_be_u32()) as uint;

            let num_regions = try!(file.read_be_u32()) as uint;
            let mut regions = Vec::new();
            for _ in range(0u, num_regions) {
                regions.push(try!(file.read_be_u32()) as uint);
            }
            tile.regions = regions;

            tiles.push((tile, 255, Deselected));
        }

        self.tiles = tiles;

        Ok(())
    }

    pub fn save(&self, path: &Path) -> io::IoResult<()> {
        let mut file = try!(io::File::create(path));

        try!(file.write_be_u32(self.width as u32));
        try!(file.write_be_u32(self.height as u32));

        for &(ref tile, _resources, _) in self.tiles.iter() {
            match tile.tile_type {
                tile::Void => try!(file.write_u8(0)),
                tile::Grass => try!(file.write_u8(1)),
                tile::Forest => try!(file.write_u8(2)),
                tile::Water => try!(file.write_u8(3)),
                tile::Residential {population, ..} => {
                    try!(file.write_u8(4));
                    try!(file.write_be_f64(population));
                },
                tile::Commercial {population, ..} => {
                    try!(file.write_u8(5));
                    try!(file.write_be_f64(population));
                },
                tile::Industrial {population, production, stored_goods, ..} => {
                    try!(file.write_u8(6));
                    try!(file.write_be_f64(population));
                    try!(file.write_be_u32(production));
                    try!(file.write_be_u32(stored_goods));
                },
                tile::Road => try!(file.write_u8(7))
            }

            try!(file.write_be_u32(tile.variant as u32));
            try!(file.write_be_u32(tile.regions.len() as u32));
            for &region in tile.regions.iter() {
                try!(file.write_be_u32(region as u32));
            }
        }

        Ok(())
    }

    pub fn size(&self) -> (uint, uint) {
        (self.width, self.height)
    }

    pub fn draw(&mut self, window: &mut RenderWindow, dt: f32) {
        for y in range(0, self.height) {
            for x in range(0, self.width) {
                let pos = Vector2f::new(
                    ((x - y) * self.tile_size + self.width * self.tile_size) as f32,
                    ((x + y) * self.tile_size) as f32 * 0.5
                );
                let &(ref mut tile, _, ref selection) = self.tiles.get_mut(y * self.width + x);

                match selection {
                    &Selected | &Invalid => tile.sprite.set_color(&Color::new_RGB(0x7d, 0x7d, 0x7d)),
                    _ => tile.sprite.set_color(&Color::new_RGB(0xff, 0xff, 0xff))
                }

                tile.sprite.set_position(&pos);
                tile.draw(window, dt);
            }
        }
    }

    pub fn update_direction(&mut self, tile_type: TileType) {
        for y in range(0, self.height) {
            for x in range(0, self.width) {
                {
                    let (ref tile, _, _) = self.tiles[y * self.width + x];
                    if !tile.tile_type.similar_to(&tile_type) {
                        continue;
                    }
                }

                let mut adjecent = [[false, ..3], ..3];

                if x > 0 {
                    if y > 0 {
                        let (ref tile, _, _) = self.tiles[(y - 1) * self.width + x - 1];
                        adjecent[0][0] = tile.tile_type.similar_to(&tile_type);
                    }

                    let (ref tile, _, _) = self.tiles[y* self.width + x - 1];
                    adjecent[1][0] = tile.tile_type.similar_to(&tile_type);

                    if y < self.height - 1 {
                        let (ref tile, _, _) = self.tiles[(y + 1) * self.width + x - 1];
                        adjecent[2][0] = tile.tile_type.similar_to(&tile_type);
                    }
                }

                if y > 0 {
                    let (ref tile, _, _) = self.tiles[(y - 1) * self.width + x];
                    adjecent[0][1] = tile.tile_type.similar_to(&tile_type);
                }

                if y < self.height - 1 {
                    let (ref tile, _, _) = self.tiles[(y + 1) * self.width + x];
                    adjecent[2][1] = tile.tile_type.similar_to(&tile_type);
                }

                if x < self.width - 1 {
                    if y > 0 {
                        let (ref tile, _, _) = self.tiles[(y - 1) * self.width + x + 1];
                        adjecent[0][2] = tile.tile_type.similar_to(&tile_type);
                    }

                    let (ref tile, _, _) = self.tiles[y* self.width + x + 1];
                    adjecent[1][2] = tile.tile_type.similar_to(&tile_type);

                    if y < self.height - 1 {
                        let (ref tile, _, _) = self.tiles[(y + 1) * self.width + x + 1];
                        adjecent[2][2] = tile.tile_type.similar_to(&tile_type);
                    }
                }

                let &(ref mut tile, _, _) = self.tiles.get_mut(y * self.width + x);

                if adjecent[1][0] && adjecent[1][2] && adjecent[0][1] && adjecent[2][1] {
                    tile.variant = 2;
                } else if adjecent[1][0] && adjecent[1][2] && adjecent[0][1] {
                    tile.variant = 7;
                } else if adjecent[1][0] && adjecent[1][2] && adjecent[2][1] {
                    tile.variant = 8;
                } else if adjecent[0][1] && adjecent[2][1] && adjecent[1][0] {
                    tile.variant = 9;
                } else if adjecent[0][1] && adjecent[2][1] && adjecent[1][2] {
                    tile.variant = 10;
                } else if adjecent[1][0] && adjecent[1][2] {
                    tile.variant = 0;
                } else if adjecent[0][1] && adjecent[2][1] {
                    tile.variant = 1;
                } else if adjecent[2][1] && adjecent[1][0] {
                    tile.variant = 3;
                } else if adjecent[0][1] && adjecent[1][2] {
                    tile.variant = 4;
                } else if adjecent[1][0] && adjecent[0][1] {
                    tile.variant = 5;
                } else if adjecent[2][1] && adjecent[1][2] {
                    tile.variant = 6;
                } else if adjecent[1][0] {
                    tile.variant = 0;
                } else if adjecent[1][2] {
                    tile.variant = 0;
                } else if adjecent[0][1] {
                    tile.variant = 1;
                } else if adjecent[2][1] {
                    tile.variant = 1;
                }
            }
        }
    }

    fn depth_first_search(&mut self, whitelisted: &mut |&TileType| -> bool, position: Vector2i, label: uint, region_type: uint) {
        if position.x < 0 || position.x >= self.width as i32 || position.y < 0 || position.y >= self.height as i32 {
            return
        }

        let found = {
            let &(ref mut tile, _, _) = self.tiles.get_mut(position.y as uint * self.width + position.x as uint);
            if tile.regions[region_type] != 0 {
                return
            }
            if (*whitelisted)(&tile.tile_type) {
                *tile.regions.get_mut(region_type) = label;
                true
            } else {
                false
            }
        };

        
        if found {
            self.depth_first_search(whitelisted, position.add(&Vector2i::new(-1,  0)), label, region_type);
            self.depth_first_search(whitelisted, position.add(&Vector2i::new( 0,  1)), label, region_type);
            self.depth_first_search(whitelisted, position.add(&Vector2i::new( 1,  0)), label, region_type);
            self.depth_first_search(whitelisted, position.add(&Vector2i::new( 0, -1)), label, region_type);
        }
    }

    pub fn find_connected_regions(&mut self, whitelisted: |&TileType| -> bool, region_type: uint) {
        let mut regions = 1;

        for &(ref mut tile, _, _) in self.tiles.mut_iter() {
            *tile.regions.get_mut(region_type) = 0;
        }

        for y in range(0, self.height) {
            for x in range(0, self.width) {
                let found = {
                    let &(ref tile, _, _) = self.tiles.get_mut(y * self.width + x);

                    if tile.regions[region_type] != 0 {
                        continue;
                    }

                    whitelisted(&tile.tile_type)
                };

                if found {
                    self.depth_first_search(&mut |tile| whitelisted(tile), Vector2i::new(x as i32, y as i32), regions, region_type);
                    regions += 1;
                }
            }
        }

        *self.num_regions.get_mut(region_type) = regions;
    }

    pub fn clear_selected(&mut self) {
        for &(_, _, ref mut selection) in self.tiles.mut_iter() {
            *selection = Deselected;
        }

        self.num_selected = 0;
    }

    pub fn tile(&self, index: uint) -> &(Tile, uint, Selection) {
        &self.tiles[index]
    }

    pub fn mut_tile(&mut self, index: uint) -> &mut (Tile, uint, Selection) {
        self.tiles.get_mut(index)
    }

    pub fn tile_at(&mut self, pos: &Vector2i) -> Option<&(Tile, uint, Selection)> {
        if pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32 {
            Some(&self.tiles[pos.x as uint + pos.y as uint * self.width])
        } else {
            None
        }
    }

    pub fn tiles(&mut self) -> MutItems<(Tile, uint, Selection)> {
        self.tiles.mut_iter()
    }

    pub fn select(&mut self, start: Vector2i, end: Vector2i, blacklisted: |&TileType| -> bool) {
        let mut start = start;
        let mut end = end;

        if end.x < start.x {
            swap(&mut start.x, &mut end.x)
        }

        if end.y < start.y {
            swap(&mut start.y, &mut end.y)
        }

        start.x = min(max(start.x, 0), self.width as i32 - 1);
        start.y = min(max(start.y, 0), self.height as i32 - 1);
        end.x = min(max(end.x, 0), self.width as i32 - 1);
        end.y = min(max(end.y, 0), self.height as i32 - 1);


        for y in range(start.y as uint, end.y as uint + 1) {
            for x in range(start.x as uint, end.x as uint + 1) {
                let &(ref tile, _, ref mut selection) = self.tiles.get_mut(y * self.width + x);
                if blacklisted(&tile.tile_type) {
                    *selection = Invalid;
                } else {
                    *selection = Selected;
                    self.num_selected += 1;
                }
            }
        }
    }

    pub fn selected(&mut self) -> FilterMap<&mut (Tile, uint, Selection), (&mut Tile, &mut uint), MutItems<(Tile, uint, Selection)>> {
        self.tiles.mut_iter().filter_map(|&(ref mut tile, ref mut resources, selection)| match selection {
            Selected => Some((tile, resources)),
            _ => None
        })
    }

    pub fn shuffled(&mut self) -> ShuffledItems<(Tile, uint, Selection)> {
        ShuffledItems::new(&mut self.tiles)
    }
}

struct ShuffledItems<'a, T: 'a> {
    items: &'a mut Vec<T>,
    indices: Vec<uint>,
    counter: uint
}

impl<'a, T: 'a> ShuffledItems<'a, T> {
    pub fn new(items: &'a mut Vec<T>) -> ShuffledItems<'a, T> {
        let mut indices: Vec<uint> = range(0, items.len()).collect();
        task_rng().shuffle(indices.as_mut_slice());
        ShuffledItems {
            items: items,
            indices: indices,
            counter: 0
        }
    }

    pub fn into_indices(self) -> Vec<uint> {
        self.indices
    }
}

impl<'a, T: 'a> iter::Iterator<&'a mut T> for ShuffledItems<'a, T> {
    fn next(&mut self) -> Option<&'a mut T> {
        if self.counter < self.items.len() {
            let index = self.indices[self.counter];
            self.counter += 1;
            unsafe {
                //less nice...
                Some(transmute(self.items.get_mut(index)))
            }
        } else {
            None
        }
    }
}