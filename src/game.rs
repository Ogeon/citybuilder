use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use rsfml;
use rsfml::window::VideoMode;
use rsfml::graphics::{RenderWindow, IntRect};

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
    pub background: rsfml::graphics::rc::Sprite,
    pub window: RenderWindow
}

impl Game {
    pub fn new() -> Option<Game> {
        let maybe_window = RenderWindow::new(
            VideoMode::new_init(800, 600, 32),
            "Super Mega City Builder",
            rsfml::window::DefaultStyle,
            &rsfml::window::ContextSettings::default()
        );

        maybe_window.map(|window| {
            let texture_manager = load_textures();
            let background = texture_manager.get_ref("background").expect("background texture was not loaded");

            Game {
                states: Vec::new(),
                textures: texture_manager,
                background: rsfml::graphics::rc::Sprite::new_with_texture(background).expect("could not create background sprite"),
                window: window
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

    if !manager.load_texture("background", "media/background.png") {
        fail!("could not load texture: media/background.png");
    }

    manager
}

pub struct TextureManager {
    textures: HashMap<&'static str, Rc<RefCell<rsfml::graphics::Texture>>>
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

    fn get_ref(&self, name: &'static str) -> Option<Rc<RefCell<rsfml::graphics::Texture>>> {
        self.textures.find(&name).map(|rc| rc.clone())
    }
}

pub struct Animation {
    pub start_frame: uint,
    pub end_frame: uint,
    pub duration: f32
}

impl Animation {
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
        AnimationHandler::new_with_size((0, 0))
    }

    pub fn new_with_size(frame_size: (uint, uint)) -> AnimationHandler {
        AnimationHandler {
            animations: Vec::new(),
            time: 0.0,
            current_anim: 0,
            bounds: IntRect::new(0, 0, 0, 0),
            frame_size: frame_size
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