use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rand::{Rng, task_rng};

use rsfml;
use rsfml::window::VideoMode;
use rsfml::graphics::{RenderWindow, IntRect};
use rsfml::graphics::rc::Sprite;
use rsfml::system::vector2::Vector2f;

pub type TextureRc = Rc<RefCell<rsfml::graphics::Texture>>;

pub trait GameState {
    fn draw(&self, dt: f32, game: &mut Game);
    fn update(&mut self, dt: f32);
    fn handle_input(&mut self, game: &mut Game);
}

impl GameState for Rc<RefCell<Box<GameState>>> {
    fn draw(&self, dt: f32, game: &mut Game) {
        self.borrow().draw(dt, game)
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
    tile_size: uint,
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
        tile_size, 1,
        textures.get_ref("residential").expect("residential texture not loaded"),
        Vec::from_elem(6, Animation::new_static()),
        TileType::residential(50, 6), 300
    ));

    tiles.insert("commercial", Tile::new(
        tile_size, 1,
        textures.get_ref("commercial").expect("commercial texture not loaded"),
        Vec::from_elem(6, Animation::new_static()),
        TileType::commercial(50, 6), 300
    ));

    tiles.insert("industrial", Tile::new(
        tile_size, 1,
        textures.get_ref("industrial").expect("industrial texture not loaded"),
        Vec::from_elem(6, Animation::new_static()),
        TileType::industrial(50, 6), 300
    ));

    tiles.insert("road", Tile::new(
        tile_size, 1,
        textures.get_ref("road").expect("road texture not loaded"),
        Vec::from_elem(11, Animation::new(0, 3, 0.5)),
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

pub struct Tile {
    sprite: Sprite,
    tile_type: TileType,
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
}