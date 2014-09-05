use std::rc::Rc;
use std::cell::RefCell;

use rsfml;
use rsfml::window::event::{
    Closed,
    Resized,
    MouseMoved,
    MouseWheelMoved,
    MouseButtonPressed,
    MouseButtonReleased,
    NoEvent
};
use rsfml::window::mouse;
use rsfml::system::vector2::{ToVec, Vector2f, Vector2i};

use game;
use tile;
use map;
use city;
use gui;

enum ActionState {
    Nothing,
    Panning(Vector2f),
    Selecting(Vector2i, Vector2i)
}

pub struct EditState<'s> {
    game_view: Rc<RefCell<rsfml::graphics::View>>,
    gui_view: Rc<RefCell<rsfml::graphics::View>>,
    city: city::City,
    action_state: ActionState,
    zoom_level: f32,
    current_tile: Option<tile::Tile>,

    right_click_menu: gui::Gui<'s, 'static, &'static str>,
    selection_cost_text: gui::Gui<'s, 'static, ()>,
    info_text: gui::Gui<'s, 'static, ()>,
    info_bar: gui::Gui<'s, 'static, ()>
}

impl<'s> EditState<'s> {
    pub fn new(game: &game::Game) -> Option<EditState<'s>> {
        let size = game.window.get_size().to_vector2f();
        let center = size.mul(&0.5f32);

        let gui_view = match rsfml::graphics::View::new_init(&center, &size) {
            Some(view) => view,
            None => return None
        };

        let map = map::Map::new_generated(game.tile_size, &game.tile_atlas);

        let (width, height) = map.size();

        let mut city = city::City::new(map);
        city.funds = 10_000.0;

        let center = Vector2f::new(
            (width * game.tile_size) as f32,
            (height * game.tile_size) as f32 * 0.5
        );

        let game_view = match rsfml::graphics::View::new_init(&center, &size) {
            Some(view) => view,
            None => return None
        };

        let right_click_menu = gui::Gui::new(
            Vector2f::new(196.0, 16.0), 2, false,
            game.stylesheets.find(&"button").unwrap().clone(),
            vec![
                ("Inspect".to_string(), "inspect"),
                (format!("Flatten ${}", game.tile_atlas.find(&"grass").expect("grass tile was not loaded").cost), "grass"),
                (format!("Forest ${}", game.tile_atlas.find(&"forest").expect("forest tile was not loaded").cost), "forest"),
                (format!("Residential Zone ${}", game.tile_atlas.find(&"residential").expect("residential tile was not loaded").cost), "residential"),
                (format!("Commercial Zone ${}", game.tile_atlas.find(&"commercial").expect("commercial tile was not loaded").cost), "commercial"),
                (format!("Industrial Zone ${}", game.tile_atlas.find(&"industrial").expect("industrial tile was not loaded").cost), "industrial"),
                (format!("Road ${}", game.tile_atlas.find(&"road").expect("road tile was not loaded").cost), "road")
            ]
        );

        let selection_cost_text = gui::Gui::new(
            Vector2f::new(196.0, 16.0), 0, false,
            game.stylesheets.find(&"text").unwrap().clone(),
            vec![("", ())]
        );

        let mut info_bar = gui::Gui::new(
            Vector2f::new(game.window.get_size().x as f32 / 5.0, 16.0), 2, true,
            game.stylesheets.find(&"button").unwrap().clone(),
            vec![
                ("time", ()),
                ("funds", ()),
                ("population", ()),
                ("employment", ()),
                ("current tile", ())
            ]
        );
        let info_bar_pos = game.window.map_pixel_to_coords(&Vector2i::new(0, size.y as i32 - 16), &gui_view);
        info_bar.transform.set_position(&info_bar_pos);
        info_bar.show();

        let info_text = gui::Gui::new::<String>(
            Vector2f::new(196.0, 16.0), 2, false,
            game.stylesheets.find(&"button").unwrap().clone(),
            Vec::new()
        );

        Some(EditState {
            game_view: Rc::new(RefCell::new(game_view)),
            gui_view: Rc::new(RefCell::new(gui_view)),
            city: city,
            action_state: Nothing,
            zoom_level: 1.0,
            current_tile: None,

            right_click_menu: right_click_menu,
            selection_cost_text: selection_cost_text,
            info_bar: info_bar,
            info_text: info_text
        })
    }
}

impl<'s> game::GameState for EditState<'s> {
    fn draw(&mut self, dt: f32, game: &mut game::Game) {
        game.window.clear(&rsfml::graphics::Color::black());
        game.window.set_view(self.gui_view.clone());
        game.window.draw(&game.background);

        game.window.set_view(self.game_view.clone());
        self.city.map.draw(&mut game.window, dt);

        game.window.set_view(self.gui_view.clone());
        game.window.draw(&self.info_bar);
        game.window.draw(&self.right_click_menu);
        game.window.draw(&self.selection_cost_text);
        game.window.draw(&self.info_text);
    }

    fn update(&mut self, dt: f32) {
        self.city.update(dt);
        self.info_bar.set_entry_text(0, format!("Day: {}", self.city.day));
        self.info_bar.set_entry_text(1, format!("${:.0}", self.city.funds));
        self.info_bar.set_entry_text(2, format!("{:.0} ({:.0})", self.city.population, self.city.get_homeless()));
        self.info_bar.set_entry_text(3, format!("{:.0} ({:.0})", self.city.employable, self.city.get_unemployed()));
        let action_name = self.current_tile.as_ref().map(|tile| tile.tile_type.to_string()).unwrap_or_else(|| "Inspect".to_string());
        self.info_bar.set_entry_text(4, action_name);
    }

    fn handle_input(&mut self, game: &mut game::Game) {
        let game_pos = game.window.map_pixel_to_coords(&game.window.get_mouse_position(), self.game_view.borrow().deref());
        let gui_pos = game.window.map_pixel_to_coords(&game.window.get_mouse_position(), self.gui_view.borrow().deref());

        let index = self.right_click_menu.get_entry(&gui_pos);
        self.right_click_menu.highlight(index);

        loop {
            match game.window.poll_event() {
                Closed => game.window.close(),
                Resized {width, height} => {
                    let size = Vector2f::new(width as f32, height as f32);
                    self.game_view.borrow_mut().set_size(&size);
                    self.game_view.borrow_mut().zoom(self.zoom_level);
                    self.gui_view.borrow_mut().set_size(&size);

                    let info_bar_pos = game.window.map_pixel_to_coords(&Vector2i::new(0, height as i32 - 16), self.gui_view.borrow().deref());
                    let info_bar_len = self.info_bar.entries.len() as f32;
                    self.info_bar.set_dimensions(&Vector2f::new(width as f32 / info_bar_len, 16.0));
                    self.info_bar.transform.set_position(&info_bar_pos);
                    self.info_bar.show();

                    let background_size = game.background.get_texture().unwrap().borrow().get_size();
                    let gui_origin = game.window.map_pixel_to_coords(&Vector2i::new(0, 0), self.gui_view.borrow().deref());
                    game.background.set_position(&gui_origin);
                    game.background.set_scale(&Vector2f::new(width as f32 / background_size.x as f32, height as f32 / background_size.y as f32));
                },
                MouseMoved {x, y} => match self.action_state {
                    Panning(ref mut anchor) => {
                        let pos = Vector2f::new(anchor.x - x as f32, anchor.y - y as f32);
                        self.game_view.borrow_mut().move(&pos.mul(&self.zoom_level));
                        *anchor = Vector2f::new(x as f32, y as f32);
                    },
                    Selecting(ref selection_start, ref mut selection_end) => {
                        match self.current_tile {
                            Some(ref current_tile) => {
                                let (width, _) = self.city.map.size();
                                selection_end.x = (game_pos.y / game.tile_size as f32 + game_pos.x / (2.0 * game.tile_size as f32) - width as f32 * 0.5 - 0.5) as i32;
                                selection_end.y = (game_pos.y / game.tile_size as f32 - game_pos.x / (2.0 * game.tile_size as f32) + width as f32 * 0.5 + 0.5) as i32;

                                self.city.map.clear_selected();
                                if current_tile.tile_type.similar_to(&tile::Grass) {
                                    self.city.map.select(selection_start.clone(), selection_end.clone(), vec![tile::Water]);
                                } else {
                                    self.city.map.select(selection_start.clone(), selection_end.clone(),
                                        vec![
                                            current_tile.tile_type.clone(),
                                            tile::Water,
                                            tile::Forest,
                                            tile::Road,
                                            tile::RESIDENTIAL,
                                            tile::COMMERCIAL,
                                            tile::INDUSTRIAL
                                        ]
                                    );
                                }

                                let total_cost = current_tile.cost as f64 * self.city.map.num_selected as f64;
                                self.selection_cost_text.set_entry_text(0, format!("${}", total_cost));
                                if self.city.funds < total_cost {
                                    self.selection_cost_text.highlight(Some(0));
                                } else {
                                    self.selection_cost_text.highlight(None);
                                }

                                let pos = Vector2f::new(
                                    if gui_pos.x + 16.0 > game.window.get_size().x as f32 - self.selection_cost_text.get_size().x {
                                        gui_pos.x - self.selection_cost_text.get_size().x - 16.0
                                    } else {
                                        gui_pos.x + 16.0
                                    },
                                    if gui_pos.y - 16.0 > game.window.get_size().y as f32 - self.selection_cost_text.get_size().y {
                                        gui_pos.y - self.selection_cost_text.get_size().y
                                    } else {
                                        gui_pos.y - 16.0
                                    }
                                );
                                self.selection_cost_text.transform.set_position(&pos);
                                self.selection_cost_text.show();
                            },
                            None => {}
                        }
                    },
                    _ => {}
                },
                MouseButtonPressed {x, y, button: mouse::MouseMiddle} => match self.action_state {
                    Panning(_) => {}
                    _ => {
                        self.action_state = Panning(Vector2f::new(x as f32, y as f32));
                        self.right_click_menu.hide();
                        self.selection_cost_text.hide();
                        self.info_text.hide();
                    },
                },
                MouseButtonPressed {button: mouse::MouseLeft, ..} => {
                    if self.right_click_menu.visible() {
                        match self.right_click_menu.activate_at(&gui_pos) {
                            Some(&tile_name) if tile_name == "inspect" => self.current_tile = None,
                            Some(tile_name) => self.current_tile = Some(game.tile_atlas.find_equiv(tile_name).expect("unknown tile").clone()),
                            _ => {}
                        }
                        self.right_click_menu.hide();
                    } else {
                        match self.action_state {
                            Selecting(..) => {},
                            _ => {
                                let (width, _) = self.city.map.size();
                                let pos = Vector2i::new(
                                    (game_pos.y / game.tile_size as f32 + game_pos.x / (2.0 * game.tile_size as f32) - width as f32 * 0.5 - 0.5) as i32,
                                    (game_pos.y / game.tile_size as f32 - game_pos.x / (2.0 * game.tile_size as f32) + width as f32 * 0.5 + 0.5) as i32
                                );
                                match self.current_tile {
                                    Some(_) => {
                                        self.action_state = Selecting(pos.clone(), pos);
                                    },
                                    None => {
                                        match self.city.map.tile_at(&pos) {
                                            Some(&(ref tile, resources, _)) => {
                                                let mut entries = vec![(tile.tile_type.to_string(), ()), (format!("Resources: {}", resources), ())];

                                                match tile.tile_type {
                                                    tile::Residential {population, ..} => {
                                                        entries.push((format!("Level: {}", tile.variant + 1), ()));
                                                        entries.push((format!("Residents: {:.0}", population), ()));
                                                    },
                                                    tile::Commercial {population, ..} => {
                                                        entries.push((format!("Level: {}", tile.variant + 1), ()));
                                                        entries.push((format!("Employees: {:.0}", population), ()));
                                                    },
                                                    tile::Industrial {population, ..} => {
                                                        entries.push((format!("Level: {}", tile.variant + 1), ()));
                                                        entries.push((format!("Employees: {:.0}", population), ()));
                                                    },
                                                    _ => {}
                                                }

                                                self.info_text.set_entries(entries);

                                                let pos = Vector2f::new(
                                                    if gui_pos.x + 16.0 > game.window.get_size().x as f32 - self.info_text.get_size().x {
                                                        gui_pos.x - self.info_text.get_size().x - 16.0
                                                    } else {
                                                        gui_pos.x + 16.0
                                                    },
                                                    if gui_pos.y - 16.0 > game.window.get_size().y as f32 - self.info_text.get_size().y {
                                                        gui_pos.y - self.info_text.get_size().y
                                                    } else {
                                                        gui_pos.y - 16.0
                                                    }
                                                );

                                                self.info_text.transform.set_position(&pos);

                                                self.info_text.show();
                                            },
                                            None => {
                                                self.info_text.hide();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                MouseButtonPressed {button: mouse::MouseRight, ..} => match self.action_state {
                    Selecting(..) => {
                        self.action_state = Nothing;
                        self.city.map.clear_selected();
                        self.selection_cost_text.hide();
                    },
                    _ => {
                        if !self.info_text.visible() {
                            let pos = Vector2f::new(
                                if gui_pos.x > game.window.get_size().x as f32 - self.right_click_menu.get_size().x {
                                    gui_pos.x - self.right_click_menu.get_size().x
                                } else {
                                    gui_pos.x
                                },
                                if gui_pos.y > game.window.get_size().y as f32 - self.right_click_menu.get_size().y {
                                    gui_pos.y - self.right_click_menu.get_size().y
                                } else {
                                    gui_pos.y
                                }
                            );

                            self.right_click_menu.transform.set_position(&pos);
                            self.right_click_menu.show();
                        } else {
                            self.info_text.hide();
                        }
                    }
                },
                MouseButtonReleased {button: mouse::MouseMiddle, ..} => self.action_state = Nothing,
                MouseButtonReleased {button: mouse::MouseLeft, ..} => match self.action_state {
                    Selecting(..) => {
                        match self.current_tile {
                            Some(ref current_tile) => {
                                let total_cost = current_tile.cost as f64 * self.city.map.num_selected as f64;
                                if self.city.funds >= total_cost {
                                    self.city.bulldoze(current_tile);
                                    self.city.funds -= total_cost;
                                    self.city.tiles_changed();
                                }

                                self.action_state = Nothing;
                                self.city.map.clear_selected();
                                self.selection_cost_text.hide();
                            },
                            None => {}
                        }
                    },
                    _ => {}
                },
                MouseWheelMoved {delta, ..} if delta > 0 => {
                    self.game_view.borrow_mut().zoom(2.0);
                    self.zoom_level *= 2.0;
                },
                MouseWheelMoved {delta, ..} if delta < 0 => {
                    self.game_view.borrow_mut().zoom(0.5);
                    self.zoom_level *= 0.5;
                },
                NoEvent => break,
                _ => {}
            }
        }
    }
}