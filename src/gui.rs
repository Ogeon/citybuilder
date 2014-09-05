use std::rc::Rc;
use std::cell::RefCell;
use std::str::StrAllocating;

use rsfml::graphics::{Color, Font, RectangleShape, Transformable, RenderWindow, RenderTexture};
use rsfml::graphics::rc::{Text};
use rsfml::system::vector2::Vector2f;
use rsfml::traits::Drawable;

#[deriving(Clone)]
pub struct GuiStyle {
    pub body_color: Color,
    pub body_highlight_color: Color,
    pub border_color: Color,
    pub border_highlight_color: Color,
    pub text_color: Color,
    pub text_highlight_color: Color,
    pub font: Rc<RefCell<Font>>,
    pub border_size: f32
}

pub struct GuiEntry<'s> {
    pub shape: RectangleShape<'s>,
    pub message: String,
    pub text: Text
}

pub struct Gui<'s> {
    horizontal: bool,
    style: GuiStyle,
    dimensions: Vector2f,
    padding: i32,
    visible: bool,
    rect: RectangleShape<'s>,
    pub transform: Transformable,
    pub entries: Vec<GuiEntry<'s>>
}

impl<'s> Gui<'s> {
    pub fn new<Txt: StrAllocating>(dimensions: Vector2f, padding: i32, horizontal: bool, style: GuiStyle, entries: Vec<(Txt, String)>) -> Gui<'s> {
        let mut rect = RectangleShape::new().expect("unable to create new rectangle shape");
        rect.set_size(&dimensions);
        rect.set_fill_color(&style.body_color);
        rect.set_outline_thickness(style.border_size);
        rect.set_outline_color(&style.border_color);

        Gui {
            horizontal: horizontal,
            dimensions: dimensions,
            padding: padding,
            visible: false,
            transform: Transformable::new().unwrap(),
            entries: entries.move_iter().map(|(text_str, message_str)| {
                let mut text = Text::new_init(text_str.as_slice(), style.font.clone(), (dimensions.y - style.border_size - padding as f32) as uint).unwrap();
                text.set_color(&style.text_color);
                GuiEntry {
                    shape: rect.clone(),
                    message: message_str,
                    text:text
                }
            }).collect(),
            rect: rect,
            style: style
        }
    }

    pub fn get_size(&self) -> Vector2f {
        Vector2f::new(self.dimensions.x, self.dimensions.y * self.entries.len() as f32)
    }

    pub fn get_entry(&self, mouse_pos: &Vector2f) -> Option<uint> {
        if self.entries.len() == 0 || !self.visible {
            return None
        }

        for (index, entry) in self.entries.iter().enumerate() {
            let point = mouse_pos.add(&entry.shape.get_origin()).sub(&entry.shape.get_position());
            if point.x < 0.0 || point.x > entry.shape.get_scale().x * self.dimensions.x {
                continue;
            }
            if point.y < 0.0 || point.y > entry.shape.get_scale().y * self.dimensions.y {
                continue;
            }

            return Some(index);
        }

        None
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_entry_text<Txt: StrAllocating>(&mut self, entry: uint, text: Txt) {
        if entry < self.entries.len() {
            self.entries.get_mut(entry).text.set_string(text.as_slice());
        }
    }

    pub fn set_entries<Txt: StrAllocating>(&mut self, entries: Vec<(Txt, String)>) {
        self.entries = entries.move_iter().map(|(text_str, message_str)| {
            let mut text = Text::new_init(text_str.as_slice(), self.style.font.clone(), (self.dimensions.y - self.style.border_size - self.padding as f32) as uint).unwrap();
            text.set_color(&self.style.text_color);
            GuiEntry {
                shape: self.rect.clone(),
                message: message_str,
                text:text
            }
        }).collect()
    }

    pub fn set_dimensions(&mut self, dimensions: &Vector2f) {
        for entry in self.entries.mut_iter() {
            entry.shape.set_size(dimensions);
            entry.text.set_character_size((dimensions.y - self.style.border_size - self.padding as f32) as uint)
        }

        self.rect.set_size(dimensions);
        self.dimensions = dimensions.clone();
    }

    pub fn show(&mut self) {
        self.visible = true;

        let origin = self.transform.get_origin();
        let position = self.transform.get_position();
        for (index, entry) in self.entries.mut_iter().enumerate() {
            let offset = if self.horizontal {
                Vector2f::new(origin.x - index as f32 * self.dimensions.x, origin.y)
            } else {
                Vector2f::new(origin.x, origin.y - index as f32 * self.dimensions.y)
            };

            entry.shape.set_origin(&offset);
            entry.text.set_origin(&offset);

            entry.shape.set_position(&position);
            entry.text.set_position(&position);
        }
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn highlight(&mut self, index: Option<uint>) {
        let index = index.unwrap_or(self.entries.len());
        for (i, entry) in self.entries.mut_iter().enumerate() {
            if i == index {
                entry.shape.set_fill_color(&self.style.body_highlight_color);
                entry.shape.set_outline_color(&self.style.border_highlight_color);
                entry.text.set_color(&self.style.text_highlight_color);
            } else {
                entry.shape.set_fill_color(&self.style.body_color);
                entry.shape.set_outline_color(&self.style.border_color);
                entry.text.set_color(&self.style.text_color);
            }
        }
    }

    pub fn activate(&self, index: uint) -> Option<&str> {
        if index >= self.entries.len() {
            return None;
        }

        Some(self.entries[index].message.as_slice())
    }

    pub fn activate_at(&self, mouse_pos: &Vector2f) -> Option<&str> {
        self.get_entry(mouse_pos).and_then(|index| self.activate(index))
    }
}

impl<'s> Drawable for Gui<'s> {
    fn draw_in_render_window(&self, render_window: &mut RenderWindow) {
        if self.visible {
            for entry in self.entries.iter() {
                render_window.draw(&entry.shape);
                render_window.draw(&entry.text);
            }
        }
    }

    fn draw_in_render_texture(&self, render_texture: &mut RenderTexture) {
        if self.visible {
            for entry in self.entries.iter() {
                render_texture.draw(&entry.shape);
                render_texture.draw(&entry.text);
            }
        }
    }
}