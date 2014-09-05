use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use rsfml;
use rsfml::window::VideoMode;
use rsfml::graphics::{RenderWindow, Color, Font};
use rsfml::graphics::rc::Sprite;

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