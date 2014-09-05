use std::rand::{Rng, task_rng};
use std::iter::AdditiveIterator;
use std::collections::HashMap;

use map;
use tile;

pub struct City {
    current_time: f32,
    time_per_day: f32,

    population_pool: f64,
    prop_can_work: f64,

    birth_rate: f64,
    death_rate: f64,

    pub map: map::Map,

    pub population: f64,
    pub employable: f64,

    pub residential_tax: f64,
    pub commercial_tax: f64,
    pub industrial_tax: f64,

    pub earnings: f64,
    pub funds: f64,

    pub day: uint
}

impl City {
    pub fn new(map: map::Map) -> City {
        City {
            current_time: 0.0,
            time_per_day: 1.0,

            population_pool: 0.0,
            prop_can_work: 0.5,
            
            birth_rate: 0.00055,
            death_rate: 0.00023,

            map: map,

            population: 0.0,
            employable: 0.0,

            residential_tax: 0.05,
            commercial_tax: 0.05,
            industrial_tax: 0.05,

            earnings: 0.0,
            funds: 0.0,

            day: 0
        }
    }

    pub fn bulldoze(&mut self, new_tile: &tile::Tile) {
        let mut removed_jobs = Vec::new();
        let mut removed_workers = HashMap::new();

        for (index, mut tile, _) in self.map.selected() {
            match tile.tile_type {
                tile::Residential {population, ref employees, ..} => {
                    self.population_pool += population;
                    for (&workplace, &people) in employees.iter() {
                        removed_workers.insert_or_update_with(workplace, people, |_k, v| *v += people);
                    }
                },
                tile::Commercial {..} | tile::Industrial {..} => removed_jobs.push(index),
                _ => {}
            }

            *tile = new_tile.clone()
        }

        for (index, &(ref mut tile, _, _)) in self.map.mut_tiles().enumerate() {
            match &mut tile.tile_type {
                &tile::Residential {ref mut employees, ..} => for index in removed_jobs.iter() {
                    employees.remove(index);
                },
                &tile::Commercial {ref mut population, ..} | &tile::Industrial {ref mut population, ..} => {
                    match removed_workers.find(&index) {
                        Some(&workers) => *population -= workers,
                        None => {}
                    }
                },
                _ => {}
            }
        }
    }

    pub fn tiles_changed(&mut self) {
        self.map.update_direction(tile::Road);
        self.map.find_connected_regions(
            |tile| match tile {
                &tile::Road | &tile::Residential {..} | &tile::Commercial {..} | &tile::Industrial {..} => true,
                _ => false
            },
            0
        );
    }

    pub fn update(&mut self, dt: f32) {
        let mut pop_total = 0.0;
        let mut commercial_revenue = 0.0;
        let mut industrial_revenue = 0.0;

        let mut empty_homes = 0.0;
        let mut free_jobs = 0.0;

        let mut stores = 0u;
        let mut industries = 0u;

        self.current_time += dt;
        if self.current_time < self.time_per_day {
            return;
        }

        self.day += 1;
        self.current_time = 0.0;

        if self.day % 30 == 0 {
            self.funds += self.earnings;
            self.earnings = 0.0;
        }

        let shuffled_indices = self.map.shuffled_indices();

        //population distribution pass
        for &index in shuffled_indices.iter() {
            let &(ref mut tile, _, _) = self.map.mut_tile(index);
            match &mut tile.tile_type {
                &tile::Residential {ref mut population, max_pop_per_level, ref employees, ..} => {
                    let max_pop = (max_pop_per_level * (tile.variant + 1)) as f64;

                    //Unemployed losing their homes
                    let employees = employees.iter().map(|(_, &employees)| employees).sum();
                    let unemployed = *population - employees / self.prop_can_work;
                    let new_homeless = if 0.01f32 > task_rng().gen() { unemployed * 0.1 } else { 0.0 };

                    let (pool, new_population) = distribute_pool(
                        self.population_pool + new_homeless,
                        *population - new_homeless,
                        max_pop,
                        self.birth_rate - self.death_rate
                    );

                    empty_homes += max_pop - new_population;

                    self.population_pool = pool;
                    *population = new_population;
                    pop_total += *population;
                },
                _ => {}
            }
        }

        let mut unemployed = get_all_unemployed(&shuffled_indices, &self.map, self.prop_can_work);
        let mut num_unemployed = unemployed.iter().map(|&(people, _)| people).sum();

        //employment distribution pass
        for &index in shuffled_indices.iter() {
            let &(ref mut tile, ref mut resources, _) = self.map.mut_tile(index);
            match &mut tile.tile_type {
                &tile::Commercial {ref mut population, max_pop_per_level, ..} => {
                    let max_pop = (max_pop_per_level * (tile.variant + 1)) as f64;

                    for &(ref mut people, home_index) in unemployed.mut_iter() {
                        if (*people / num_unemployed) * (1.0 - self.commercial_tax) * 0.15 > task_rng().gen() {
                            let (pool, new_population) = distribute_pool(
                                *people,
                                *population,
                                max_pop,
                                0.0
                            );

                            //TODO: add new employees in residence

                            num_unemployed -= *people - pool;
                            *people = pool;
                            *population = new_population;
                        }
                    }

                    stores += 1;
                    free_jobs += max_pop - *population;
                },
                &tile::Industrial {ref mut production, ref mut population, max_pop_per_level, ..} => {
                    if *resources > 0 && *population * 0.01 > task_rng().gen() {
                        *production += 1;
                        *resources -= 1;
                    }

                    let max_pop = (max_pop_per_level * (tile.variant + 1)) as f64;

                    for &(ref mut people, home_index) in unemployed.mut_iter() {
                        if (*people / num_unemployed) * (1.0 - self.industrial_tax) * 0.15 > task_rng().gen() {
                            let (pool, new_population) = distribute_pool(
                                *people,
                                *population,
                                max_pop,
                                0.0
                            );

                            //TODO: add new employees in residence

                            num_unemployed -= *people - pool;
                            *people = pool;
                            *population = new_population;
                        }
                    }

                    industries += 1;
                    free_jobs += max_pop - *population;
                },
                _ => {}
            }

            tile.update();
        }

        assert!(num_unemployed >= 0.0);

        //manufacture pass
        for &index in shuffled_indices.iter() {
            let (region, level) = match self.map.tile(index) {
                &(
                    tile::Tile {
                        tile_type: tile::Industrial {..},
                        ref regions,
                        variant,
                        ..
                    },
                    _resources,
                    _selection
                ) => (regions[0], variant as u32 + 1),
                _ => continue
            };

            let mut received_resources = 0;
            
            for &(ref mut tile2, _, _) in self.map.mut_tiles() {
                if tile2.regions[0] == region {
                    match tile2.tile_type {
                        tile::Industrial {ref mut production, ..} => {
                            if *production > 0 {
                                received_resources += 1;
                                *production -= 1;
                            }

                            if received_resources >= level {
                                break;
                            }
                        },
                        _ => {}
                    }
                }
            }

            let &(ref mut tile, _, _) = self.map.mut_tile(index);
            match tile.tile_type {
                tile::Industrial {ref mut stored_goods, production, ..} => *stored_goods += (received_resources + production) * level,
                _ => unreachable!()
            }
        }

        //goods distribution pass
        for &index in shuffled_indices.iter() {
            let (region, level, population) = match self.map.tile(index) {
                &(
                    tile::Tile {
                        tile_type: tile::Commercial {population, ..},
                        ref regions,
                        variant,
                        ..
                    },
                    _resources,
                    _selection
                ) => (regions[0], variant as u32 + 1, population),
                _ => continue
            };

            let mut received_goods = 0;
            let mut max_customers = 0.0;

            for &(ref mut tile2, _, _) in self.map.mut_tiles() {
                if tile2.regions[0] == region {
                    match tile2.tile_type {
                        tile::Industrial {ref mut stored_goods, ..} => {
                            while *stored_goods > 0 && received_goods < level {
                                *stored_goods -= 1;
                                received_goods += 1;
                                industrial_revenue += 100.0 * (1.0 - self.industrial_tax);
                            }
                        },
                        tile::Residential {population, ..} => {
                            max_customers += population;
                        }
                        _ => {}
                    }

                    if received_goods >= level {
                        break;
                    }
                }
            }

            let production = (received_goods as f64 * 100.0 + 20.0 * task_rng().gen()) * (1.0 - self.commercial_tax);
            commercial_revenue += production * max_customers * population / 100.0;
        }

        self.population_pool += self.population_pool * (self.birth_rate - self.death_rate);

        let imigrants = 1.0 + (empty_homes - self.population_pool).max(0.0) * (free_jobs - num_unemployed).max(0.0) * (1.0 - self.residential_tax) * 0.0001;
        let prob = (empty_homes - self.population_pool).max(0.0) * (free_jobs - num_unemployed).max(0.0) * (1.0 - self.residential_tax) * 0.00001;
        
        //people moving to the city
        if stores > 0 && industries > 0 && prob > task_rng().gen() {
            self.population_pool += imigrants;
        }

        //people moving from the city
        if (self.population_pool > empty_homes || num_unemployed > free_jobs) && (self.population_pool + num_unemployed) * 0.01 > task_rng().gen() {
            self.population_pool -= (self.population_pool + num_unemployed) * 0.05 + 1.0;
        }

        pop_total += self.population_pool;

        let new_workers = (pop_total - self.population).abs() * self.prop_can_work;
        self.employable += new_workers;
        
        if self.employable < 0.0 {
            self.employable = 0.0;
        }

        self.population = pop_total;

        self.earnings += (self.population - self.population_pool) * 15.0 * self.residential_tax;
        self.earnings += commercial_revenue * self.commercial_tax;
        self.earnings += industrial_revenue * self.industrial_tax;
    }

    pub fn get_homeless(&self) -> f64  {
        self.population_pool
    }

    pub fn get_unemployed(&self) -> f64  {
        self.map.tiles().filter_map(|&(ref tile, _, _)| get_unemployed(tile, self.prop_can_work)).sum()
    }
}

fn distribute_pool(pool: f64, population: f64, max_pop: f64, change_rate: f64) -> (f64, f64) {

    let (pool, population) = if pool > 0.0 {
        let moving = (max_pop - population).min(4.0).min(pool);
        (pool - moving, population + moving)
    } else {
        (pool, population)
    };

    let population = population + population * change_rate;

    if population > max_pop {
        (pool + (population - max_pop), max_pop)
    } else {
        (pool, population)
    }
}

fn get_all_unemployed(indices: &Vec<uint>, map: &map::Map, prop_can_work: f64) -> Vec<(f64, uint)> {
    indices.iter().filter_map(|&index| {
        let &(ref tile, _, _) = map.tile(index);
        get_unemployed(tile, prop_can_work).map(|people| (people, index))
    }).collect()
}

fn get_unemployed(tile: &tile::Tile, prop_can_work: f64) -> Option<f64> {
    match tile.tile_type {
        tile::Residential {population, ref employees, ..} => {
            let employees = employees.iter().map(|(_, &employees)| employees).sum();
            let unemployed = population * prop_can_work - employees;
            if unemployed > 0.0 {
                Some(unemployed)
            } else {
                None
            }
        },
        _ => None
    }
}