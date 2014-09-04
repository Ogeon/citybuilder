use std::rand::{Rng, task_rng};
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use rsfml;
use rsfml::graphics::{RenderWindow, IntRect};
use rsfml::graphics::rc::Sprite;
use rsfml::system::vector2::Vector2f;

pub type TextureRc = Rc<RefCell<rsfml::graphics::Texture>>;

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
    max_levels: 0,
    production: 0,
    stored_goods: 0
};

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
            bounds: IntRect::new(0, 0, width as i32, height as i32),
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
        pub population: f64,
        pub max_pop_per_level: uint,
        max_levels: uint
    },
    Commercial {
        pub population: f64,
        pub max_pop_per_level: uint,
        max_levels: uint
    },
    Industrial {
        pub population: f64,
        pub max_pop_per_level: uint,
        pub production: u32,
        pub stored_goods: u32,
        max_levels: uint
    },
    Road
}

impl TileType {
    pub fn residential(max_pop_per_level: uint, max_levels: uint) -> TileType {
        Residential {
            population: 0.0,
            max_pop_per_level: max_pop_per_level,
            max_levels: max_levels
        }
    }

    pub fn commercial(max_pop_per_level: uint, max_levels: uint) -> TileType {
        Commercial {
            population: 0.0,
            max_pop_per_level: max_pop_per_level,
            max_levels: max_levels
        }
    }

    pub fn industrial(max_pop_per_level: uint, max_levels: uint) -> TileType {
        Industrial {
            population: 0.0,
            max_pop_per_level: max_pop_per_level,
            max_levels: max_levels,
            production: 0,
            stored_goods: 0
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
    pub sprite: Sprite,
    pub tile_type: TileType,
    pub variant: uint,
    pub regions: Vec<uint>,
    pub cost: uint,
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
            Residential {population, max_pop_per_level, max_levels} |
            Commercial {population, max_pop_per_level, max_levels} |
            Industrial {population, max_pop_per_level, max_levels, ..}
            => if population as uint == max_pop_per_level * (self.variant + 1) && self.variant < max_levels {
                if (0.01f32 / (self.variant + 1) as f32) > task_rng().gen() {
                    self.variant += 1;
                }
            },
            _ => {}
        }
    }

    pub fn set_population(&mut self, new_population: f64) {
        match self.tile_type {
            Residential {ref mut population, ..} |
            Commercial {ref mut population, ..} |
            Industrial {ref mut population, ..}
            => *population = new_population,
            _ => {}
        }
    }

    pub fn set_production(&mut self, new_production: u32) {
        match self.tile_type {
            Industrial {ref mut production, ..} => *production = new_production,
            _ => {}
        }
    }

    pub fn set_stored_goods(&mut self, new_stored_goods: u32) {
        match self.tile_type {
            Industrial {ref mut stored_goods, ..} => *stored_goods = new_stored_goods,
            _ => {}
        }
    }
}