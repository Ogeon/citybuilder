use std::io;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rand::{Rng, task_rng};
use std::mem::{swap, transmute};
use std::cmp::{min, max};
use std::iter;
use std::iter::FilterMap;
use std::slice::MutItems;

use rsfml;
use rsfml::window::VideoMode;
use rsfml::graphics::{RenderWindow, Color, Font};
use rsfml::graphics::rc::Sprite;
use rsfml::system::vector2::{Vector2f, Vector2i};

use gui;

use tile;
use tile::{Tile, TileType};

pub type TextureRc = Rc<RefCell<rsfml::graphics::Texture>>;

pub trait GameState {
    fn draw(&mut self, dt: f32, game: &mut Game);
    fn update(&mut self, dt: f32);
    fn handle_input(&mut self, game: &mut Game);
}

impl<'a> GameState for Rc<RefCell<Box<GameState + 'a>>> {
    fn draw(&mut self, dt: f32, game: &mut Game) {
        self.borrow_mut().draw(dt, game)
    }

    fn update(&mut self, dt: f32) {
        self.borrow_mut().update(dt)
    }

    fn handle_input(&mut self, game: &mut Game) {
        self.borrow_mut().handle_input(game)
    }
}

pub struct Game<'a> {
    states: Vec<Rc<RefCell<Box<GameState + 'a>>>>,
    textures: TextureManager,
    pub tile_size: uint,
    pub background: Sprite,
    pub window: RenderWindow,
    pub tile_atlas: HashMap<&'static str, Tile>,
    pub fonts: HashMap<&'static str, Rc<RefCell<Font>>>,
    pub stylesheets: HashMap<&'static str, gui::GuiStyle>
}

impl<'a> Game<'a> {
    pub fn new() -> Option<Game<'a>> {
        let maybe_window = RenderWindow::new(
            VideoMode::new_init(800, 600, 32),
            "Super Mega City Builder",
            rsfml::window::DefaultStyle,
            &rsfml::window::ContextSettings::default()
        );

        let tile_size = 8;

        maybe_window.map(|mut window| {
            let texture_manager = load_textures();
            let background = texture_manager.get_ref("background").expect("background texture was not loaded");
            let tiles = load_tiles(&texture_manager, tile_size);
            let fonts = load_fonts();
            window.set_framerate_limit(60);

            Game {
                states: Vec::new(),
                textures: texture_manager,
                tile_size: tile_size,
                background: Sprite::new_with_texture(background).expect("could not create background sprite"),
                window: window,
                tile_atlas: tiles,
                stylesheets: make_stylesheets(&fonts),
                fonts: fonts
            }
        })
    }

    pub fn push_state(&mut self, state: Box<GameState + 'a>) {
        self.states.push(Rc::new(RefCell::new(state)));
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    pub fn change_state(&mut self, state: Box<GameState + 'a>) {
        self.pop_state();
        self.push_state(state);
    }

    pub fn peek_state(&self) -> Option<Rc<RefCell<Box<GameState + 'a>>>> {
        self.states.last().map(|state| state.clone())
    }

    pub fn game_loop(&mut self) {
        let mut clock = rsfml::system::Clock::new();

        while self.window.is_open() {
            let elapsed = clock.restart();
            let dt = elapsed.as_seconds();

            match self.peek_state() {
                Some(mut state) => {
                    state.handle_input(self);
                    state.update(dt);
                    
                    self.window.clear(&rsfml::graphics::Color::black());
                    state.draw(dt, self);
                    self.window.display();
                },
                None => {}
            }
        }
    }
}

fn load_textures() -> TextureManager {
    let mut manager = TextureManager { textures: HashMap::new() };

    if !manager.load_texture("grass", "media/grass.png") {
        fail!("could not load texture: media/grass.png");
    }

    if !manager.load_texture("forest", "media/forest.png") {
        fail!("could not load texture: media/forest.png");
    }

    if !manager.load_texture("water", "media/water.png") {
        fail!("could not load texture: media/water.png");
    }

    if !manager.load_texture("residential", "media/residential.png") {
        fail!("could not load texture: media/residential.png");
    }

    if !manager.load_texture("commercial", "media/commercial.png") {
        fail!("could not load texture: media/commercial.png");
    }

    if !manager.load_texture("industrial", "media/industrial.png") {
        fail!("could not load texture: media/industrial.png");
    }

    if !manager.load_texture("road", "media/road.png") {
        fail!("could not load texture: media/road.png");
    }

    if !manager.load_texture("background", "media/background.png") {
        fail!("could not load texture: media/background.png");
    }

    manager
}

fn load_tiles(textures: &TextureManager, tile_size: uint) -> HashMap<&'static str, Tile> {
    let mut tiles = HashMap::new();

    tiles.insert("grass", Tile::new(
        tile_size, 1,
        textures.get_ref("grass").expect("grass texture not loaded"),
        vec![tile::Animation::new_static()],
        tile::Grass, 50
    ));

    tiles.insert("forest", Tile::new(
        tile_size, 1,
        textures.get_ref("forest").expect("forest texture not loaded"),
        vec![tile::Animation::new_static()],
        tile::Forest, 100
    ));

    tiles.insert("water", Tile::new(
        tile_size, 1,
        textures.get_ref("water").expect("water texture not loaded"),
        Vec::from_elem(3, tile::Animation::new(0, 3, 0.5)),
        tile::Water, 0
    ));

    tiles.insert("residential", Tile::new(
        tile_size, 2,
        textures.get_ref("residential").expect("residential texture not loaded"),
        Vec::from_elem(6, tile::Animation::new_static()),
        TileType::residential(50, 6), 300
    ));

    tiles.insert("commercial", Tile::new(
        tile_size, 2,
        textures.get_ref("commercial").expect("commercial texture not loaded"),
        Vec::from_elem(4, tile::Animation::new_static()),
        TileType::commercial(50, 4), 300
    ));

    tiles.insert("industrial", Tile::new(
        tile_size, 2,
        textures.get_ref("industrial").expect("industrial texture not loaded"),
        Vec::from_elem(4, tile::Animation::new_static()),
        TileType::industrial(50, 4), 300
    ));

    tiles.insert("road", Tile::new(
        tile_size, 1,
        textures.get_ref("road").expect("road texture not loaded"),
        Vec::from_elem(11, tile::Animation::new_static()),
        tile::Road, 100
    ));

    tiles
}

pub fn load_fonts() -> HashMap<&'static str, Rc<RefCell<Font>>> {
    let mut fonts = HashMap::new();

    fonts.insert("main_font", Rc::new(RefCell::new(Font::new_from_file("media/font.ttf").expect("could not load main font"))));

    fonts
}

pub fn make_stylesheets(fonts: &HashMap<&'static str, Rc<RefCell<Font>>>) -> HashMap<&'static str, gui::GuiStyle> {
    let mut stylesheets = HashMap::new();
    let font = fonts.find(&"main_font").expect("main font not loaded").clone();

    stylesheets.insert("button", gui::GuiStyle {
        font: font.clone(),
        border_size: 1.0,
        body_color: Color::new_RGB(0xc6, 0xc6, 0xc6),
        border_color: Color::new_RGB(0x94, 0x94, 0x94),
        text_color: Color::new_RGB(0x00, 0x00, 0x00),
        body_highlight_color: Color::new_RGB(0x61, 0x61, 0x61),
        border_highlight_color: Color::new_RGB(0x94, 0x94, 0x94),
        text_highlight_color: Color::new_RGB(0x00, 0x00, 0x00)
    });

    stylesheets.insert("text", gui::GuiStyle {
        font: font,
        border_size: 0.0,
        body_color: Color::new_RGBA(0x00, 0x00, 0x00, 0x00),
        border_color: Color::new_RGB(0x00, 0x00, 0x00),
        text_color: Color::new_RGB(0xff, 0xff, 0xff),
        body_highlight_color: Color::new_RGBA(0x00, 0x00, 0x00, 0x00),
        border_highlight_color: Color::new_RGB(0x00, 0x00, 0x00),
        text_highlight_color: Color::new_RGB(0xff, 0x00, 0x00)
    });

    stylesheets
}

pub struct TextureManager {
    textures: HashMap<&'static str, TextureRc>
}

impl TextureManager {
    fn load_texture(&mut self, name: &'static str, filename: &str) -> bool {
        match rsfml::graphics::Texture::new_from_file(filename) {
            Some(texture) => {
                self.textures.insert(name, Rc::new(RefCell::new(texture)));
                true
            },
            None => false
        }
    }

    fn get_ref(&self, name: &'static str) -> Option<TextureRc> {
        self.textures.find(&name).map(|rc| rc.clone())
    }
}

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

    fn depth_first_search(&mut self, whitelist: &Vec<TileType>, position: Vector2i, label: uint, region_type: uint) {
        if position.x < 0 || position.x >= self.width as i32 || position.y < 0 || position.y >= self.height as i32 {
            return
        }

        let found = {
            let &(ref mut tile, _, _) = self.tiles.get_mut(position.y as uint * self.width + position.x as uint);
            if tile.regions[region_type] != 0 {
                return
            }
            if whitelist.iter().find(|t| t.similar_to(&tile.tile_type)).is_some() {
                *tile.regions.get_mut(region_type) = label;
                true
            } else {
                false
            }
        };

        
        if found {
            self.depth_first_search(whitelist, position.add(&Vector2i::new(-1,  0)), label, region_type);
            self.depth_first_search(whitelist, position.add(&Vector2i::new( 0,  1)), label, region_type);
            self.depth_first_search(whitelist, position.add(&Vector2i::new( 1,  0)), label, region_type);
            self.depth_first_search(whitelist, position.add(&Vector2i::new( 0, -1)), label, region_type);
        }
    }

    pub fn find_connected_regions(&mut self, whitelist: Vec<TileType>, region_type: uint) {
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

                    whitelist.iter().find(|t| t.similar_to(&tile.tile_type)).is_some()
                };

                if found {
                    self.depth_first_search(&whitelist, Vector2i::new(x as i32, y as i32), regions, region_type);
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

    pub fn tile(&mut self, index: uint) -> &mut (Tile, uint, Selection) {
        self.tiles.get_mut(index)
    }

    pub fn tiles(&mut self) -> MutItems<(Tile, uint, Selection)> {
        self.tiles.mut_iter()
    }

    pub fn select(&mut self, start: Vector2i, end: Vector2i, blacklist: Vec<TileType>) {
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
                if blacklist.iter().find(|t| t.similar_to(&tile.tile_type)).is_some() {
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