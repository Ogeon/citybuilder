use std::io;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rand::{Rng, task_rng};
use std::mem::swap;
use std::cmp::{min, max};

use rsfml;
use rsfml::window::VideoMode;
use rsfml::graphics::{RenderWindow, IntRect, Color};
use rsfml::graphics::rc::Sprite;
use rsfml::system::vector2::{Vector2f, Vector2i};

pub static RESIDENTIAL: TileType = Residential {
    population: 0.0,
    max_pop_per_level: 0,
    max_levels: 0
};

pub static COMMERCIAL: TileType = Commercial {
    population: 0.0,
    max_pop_per_level: 0,
    max_levels: 0
};

pub static INDUSTRIAL: TileType = Industrial {
    population: 0.0,
    max_pop_per_level: 0,
    max_levels: 0
};

pub type TextureRc = Rc<RefCell<rsfml::graphics::Texture>>;

pub trait GameState {
    fn draw(&mut self, dt: f32, game: &mut Game);
    fn update(&mut self, dt: f32);
    fn handle_input(&mut self, game: &mut Game);
}

impl GameState for Rc<RefCell<Box<GameState>>> {
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

pub struct Game {
    states: Vec<Rc<RefCell<Box<GameState>>>>,
    textures: TextureManager,
    pub tile_size: uint,
    pub background: Sprite,
    pub window: RenderWindow,
    pub tile_atlas: HashMap<&'static str, Tile>
}

impl Game {
    pub fn new() -> Option<Game> {
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
            window.set_framerate_limit(60);

            Game {
                states: Vec::new(),
                textures: texture_manager,
                tile_size: tile_size,
                background: Sprite::new_with_texture(background).expect("could not create background sprite"),
                window: window,
                tile_atlas: tiles
            }
        })
    }

    pub fn push_state(&mut self, state: Box<GameState>) {
        self.states.push(Rc::new(RefCell::new(state)));
    }

    pub fn pop_state(&mut self) {
        self.states.pop();
    }

    pub fn change_state(&mut self, state: Box<GameState>) {
        self.pop_state();
        self.push_state(state);
    }

    pub fn peek_state(&self) -> Option<Rc<RefCell<Box<GameState>>>> {
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
        vec![Animation::new_static()],
        Grass, 50
    ));

    tiles.insert("forest", Tile::new(
        tile_size, 1,
        textures.get_ref("forest").expect("forest texture not loaded"),
        vec![Animation::new_static()],
        Forest, 100
    ));

    tiles.insert("water", Tile::new(
        tile_size, 1,
        textures.get_ref("water").expect("water texture not loaded"),
        Vec::from_elem(3, Animation::new(0, 3, 0.5)),
        Water, 0
    ));

    tiles.insert("residential", Tile::new(
        tile_size, 2,
        textures.get_ref("residential").expect("residential texture not loaded"),
        Vec::from_elem(6, Animation::new_static()),
        TileType::residential(50, 6), 300
    ));

    tiles.insert("commercial", Tile::new(
        tile_size, 2,
        textures.get_ref("commercial").expect("commercial texture not loaded"),
        Vec::from_elem(6, Animation::new_static()),
        TileType::commercial(50, 6), 300
    ));

    tiles.insert("industrial", Tile::new(
        tile_size, 2,
        textures.get_ref("industrial").expect("industrial texture not loaded"),
        Vec::from_elem(6, Animation::new_static()),
        TileType::industrial(50, 6), 300
    ));

    tiles.insert("road", Tile::new(
        tile_size, 1,
        textures.get_ref("road").expect("road texture not loaded"),
        Vec::from_elem(11, Animation::new_static()),
        Road, 100
    ));

    tiles
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

#[deriving(Clone)]
pub struct Animation {
    pub start_frame: uint,
    pub end_frame: uint,
    pub duration: f32
}

impl Animation {
    pub fn new(start_frame: uint, end_frame: uint, duration: f32) -> Animation {
        Animation {
            start_frame: start_frame,
            end_frame: end_frame,
            duration: duration
        }
    }

    pub fn new_static() -> Animation {
        Animation {
            start_frame: 0,
            end_frame: 0,
            duration: 1.0
        }
    }

    pub fn get_length(&self) -> uint {
        self.end_frame - self.start_frame + 1
    }
}

#[deriving(Clone)]
pub struct AnimationHandler {
    animations: Vec<Animation>,
    time: f32,
    current_anim: uint,
    pub bounds: IntRect,
    pub frame_size: (uint, uint)
}

impl AnimationHandler {
    pub fn new() -> AnimationHandler {
        AnimationHandler::new_with_size(0, 0)
    }

    pub fn new_with_size(width: uint, height: uint) -> AnimationHandler {
        AnimationHandler {
            animations: Vec::new(),
            time: 0.0,
            current_anim: 0,
            bounds: IntRect::new(0, 0, 0, 0),
            frame_size: (width, height)
        }
    }

    pub fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation)
    }

    pub fn update(&mut self, dt: f32) {
        if self.current_anim >= self.animations.len() {
            return
        }

        let duration = self.animations[self.current_anim].duration;

        let frame = ((self.time + dt) / duration) as i32;
        if frame > (self.time / duration) as i32 {
            let frame = frame % self.animations[self.current_anim].get_length() as i32;
            let (width, height) = self.frame_size;
            let width = width as i32;
            let height = height as i32;
            self.bounds = IntRect::new(width * frame as i32, height * self.current_anim as i32, width, height);
        }

        self.time += dt;

        if dt > duration * self.animations[self.current_anim].get_length() as f32 {
            self.time = 0.0
        }
    }

    pub fn change_animation(&mut self, new_animation: uint) {
        if new_animation != self.current_anim && new_animation < self.animations.len() {
            self.current_anim = new_animation;
            let (width, height) = self.frame_size;
            self.bounds = IntRect::new(0, (height * new_animation) as i32, width as i32, height as i32);
            self.time = 0.0;
        }
    }
}

#[deriving(Clone)]
pub enum TileType {
    Void,
    Grass,
    Forest,
    Water,
    Residential {
        population: f32,
        max_pop_per_level: uint,
        max_levels: uint
    },
    Commercial {
        population: f32,
        max_pop_per_level: uint,
        max_levels: uint
    },
    Industrial {
        population: f32,
        max_pop_per_level: uint,
        max_levels: uint
    },
    Road
}

impl TileType {
    fn residential(max_pop_per_level: uint, max_levels: uint) -> TileType {
        Residential {
            population: 0.0,
            max_pop_per_level: max_pop_per_level,
            max_levels: max_levels
        }
    }

    fn commercial(max_pop_per_level: uint, max_levels: uint) -> TileType {
        Commercial {
            population: 0.0,
            max_pop_per_level: max_pop_per_level,
            max_levels: max_levels
        }
    }

    fn industrial(max_pop_per_level: uint, max_levels: uint) -> TileType {
        Industrial {
            population: 0.0,
            max_pop_per_level: max_pop_per_level,
            max_levels: max_levels
        }
    }

    pub fn similar_to(&self, other: &TileType) -> bool {
        match (self, other) {
            (&Void, &Void) => true,
            (&Grass, &Grass) => true,
            (&Forest, &Forest) => true,
            (&Water, &Water) => true,
            (&Residential {..}, &Residential {..}) => true,
            (&Commercial {..}, &Commercial {..}) => true,
            (&Industrial {..}, &Industrial {..}) => true,
            (&Road, &Road) => true,
            _ => false
        }
    }
}

impl fmt::Show for TileType {
    fn fmt(&self, buf: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Void => write!(buf, "Void"),
            Grass => write!(buf, "Grass"),
            Forest => write!(buf, "Forest"),
            Water => write!(buf, "Water"),
            Residential {..} => write!(buf, "Residential Zone"),
            Commercial {..} => write!(buf, "Commercial Zone"),
            Industrial {..} => write!(buf, "Industrial Zone"),
            Road => write!(buf, "Road")
        }
    }
}

#[deriving(Clone)]
pub struct Tile {
    sprite: Sprite,
    pub tile_type: TileType,
    variant: uint,
    regions: Vec<uint>,
    pub cost: uint,
    production: f32,
    stored_goods: f32,
    animation_handler: AnimationHandler
}

impl Tile {
    pub fn new(tile_size: uint, height: uint, texture: TextureRc, animations: Vec<Animation>, tile_type: TileType, cost: uint) -> Tile {
        let mut animation_handler = AnimationHandler::new_with_size(tile_size * 2, tile_size * height);
        for animation in animations.move_iter() {
            animation_handler.add_animation(animation);
        }
        animation_handler.update(0.0);

        let mut sprite = Sprite::new_with_texture(texture).unwrap();
        sprite.set_origin(&Vector2f::new(0.0, (tile_size * (height - 1)) as f32));

        Tile {
            sprite: sprite,
            tile_type: tile_type,
            variant: 0,
            regions: vec![0],
            cost: cost,
            production: 0.0,
            stored_goods: 0.0,
            animation_handler: animation_handler
        }
    }

    pub fn draw(&mut self, window: &mut RenderWindow, dt: f32) {
        self.animation_handler.change_animation(self.variant);
        self.animation_handler.update(dt);
        self.sprite.set_texture_rect(&self.animation_handler.bounds);
        window.draw(&self.sprite);
    }

    pub fn update(&mut self) {
        match self.tile_type {
            Residential {mut population, max_pop_per_level, max_levels} |
            Commercial {mut population, max_pop_per_level, max_levels} |
            Industrial {mut population, max_pop_per_level, max_levels}
            => if population as uint == max_pop_per_level * (self.variant + 1) && self.variant < max_levels {
                if (0.01f32 / (self.variant + 1) as f32) > task_rng().gen() {
                    self.variant += 1;
                }
            },
            _ => {}
        }
    }

    pub fn set_population(&mut self, new_population: f32) {
        match self.tile_type {
            Residential {mut population, ..} |
            Commercial {mut population, ..} |
            Industrial {mut population, ..}
            => population = new_population,
            _ => {}
        }
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
    num_selected: uint,
    num_regions: Vec<uint>
}

impl Map {
    pub fn new_generated(tile_size: uint, tile_atlas: &HashMap<&'static str, Tile>) -> Map {
        let width = 50;
        let height = 50;

        let mut tiles = Vec::new();

        for _ in range(0u, width * height) {
            let tile = match task_rng().gen_range(0u8, 8) {
                0 | 1 => tile_atlas.find(&"grass").expect("grass tile was not loaded").clone(),
                2 => tile_atlas.find(&"forest").expect("forest tile was not loaded").clone(),
                3 => tile_atlas.find(&"water").expect("water tile was not loaded").clone(),
                4 => tile_atlas.find(&"residential").expect("residential tile was not loaded").clone(),
                5 => tile_atlas.find(&"commercial").expect("commercial tile was not loaded").clone(),
                6 => tile_atlas.find(&"industrial").expect("industrial tile was not loaded").clone(),
                7 => tile_atlas.find(&"road").expect("road tile was not loaded").clone(),
                _ => unreachable!()
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
                    tile.set_population(try!(file.read_be_f32()));
                    tile
                },
                5 => {
                    let mut tile = tile_atlas.find(&"commercial").unwrap().clone();
                    tile.set_population(try!(file.read_be_f32()));
                    tile
                },
                6 => {
                    let mut tile = tile_atlas.find(&"industrial").unwrap().clone();
                    tile.set_population(try!(file.read_be_f32()));
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

            tile.stored_goods = try!(file.read_be_f32());

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
                Void => try!(file.write_u8(0)),
                Grass => try!(file.write_u8(1)),
                Forest => try!(file.write_u8(2)),
                Water => try!(file.write_u8(3)),
                Residential {population, ..} => {
                    try!(file.write_u8(4));
                    try!(file.write_be_f32(population));
                },
                Commercial {population, ..} => {
                    try!(file.write_u8(5));
                    try!(file.write_be_f32(population));
                },
                Industrial {population, ..} => {
                    try!(file.write_u8(6));
                    try!(file.write_be_f32(population));
                },
                Road => try!(file.write_u8(7))
            }

            try!(file.write_be_u32(tile.variant as u32));
            try!(file.write_be_u32(tile.regions.len() as u32));
            for &region in tile.regions.iter() {
                try!(file.write_be_u32(region as u32));
            }

            try!(file.write_be_f32(tile.stored_goods));
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

    fn update_direction(&mut self, tile_type: TileType) {
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
                    adjecent[0][1] = tile.tile_type.similar_to(&tile_type);

                    if y < self.height - 1 {
                        let (ref tile, _, _) = self.tiles[(y + 1) * self.width + x - 1];
                        adjecent[0][2] = tile.tile_type.similar_to(&tile_type);
                    }
                }

                if y > 0 {
                    let (ref tile, _, _) = self.tiles[(y - 1) * self.width + x];
                    adjecent[1][0] = tile.tile_type.similar_to(&tile_type);
                }

                if y < self.height - 1 {
                    let (ref tile, _, _) = self.tiles[(y + 1) * self.width + x];
                    adjecent[1][2] = tile.tile_type.similar_to(&tile_type);
                }

                if x < self.width - 1 {
                    if y > 0 {
                        let (ref tile, _, _) = self.tiles[(y - 1) * self.width + x + 1];
                        adjecent[2][0] = tile.tile_type.similar_to(&tile_type);
                    }

                    let (ref tile, _, _) = self.tiles[y* self.width + x + 1];
                    adjecent[2][1] = tile.tile_type.similar_to(&tile_type);

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
}