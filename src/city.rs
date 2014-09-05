use std::rand::{Rng, task_rng};

use map;
use tile;

pub struct City {
    current_time: f32,
    time_per_day: f32,

    population_pool: f64,
    employment_pool: f64,
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
            employment_pool: 0.0,
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
        for (mut tile, _) in self.map.selected() {
            match tile.tile_type {
                tile::Residential {population, ..} => self.population_pool += population,
                tile::Commercial {population, ..} | tile::Industrial {population, ..} => self.employment_pool += population,
                _ => {}
            }

            *tile = new_tile.clone()
        }
    }

    pub fn tiles_changed(&mut self) {
        self.map.update_direction(tile::Road);
        self.map.find_connected_regions(
            vec![tile::Road, tile::RESIDENTIAL, tile::COMMERCIAL, tile::INDUSTRIAL],
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

        
        let shuffled_indices = {
            let mut shuffled_tiles = self.map.shuffled();

            //population and employment distribution pass
            for &(ref mut tile, ref mut resources, _) in shuffled_tiles {
                match &mut tile.tile_type {
                    &tile::Residential {ref mut population, max_pop_per_level, ..} => {
                        let max_pop = (max_pop_per_level * (tile.variant + 1)) as f64;

                        let (pool, new_population) = distribute_pool(
                            self.population_pool,
                            *population,
                            max_pop,
                            self.birth_rate - self.death_rate
                        );

                        empty_homes += max_pop - new_population;

                        self.population_pool = pool;
                        *population = new_population;
                        pop_total += *population;
                    },
                    &tile::Commercial {ref mut population, max_pop_per_level, ..} => {
                        let max_pop = (max_pop_per_level * (tile.variant + 1)) as f64;

                        if (1.0 - self.commercial_tax) * 0.15 > task_rng().gen() {
                            let (pool, new_population) = distribute_pool(
                                self.employment_pool,
                                *population,
                                max_pop,
                                0.0
                            );

                            self.employment_pool = pool;
                            *population = new_population;
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

                        if (1.0 - self.industrial_tax) * 0.15 > task_rng().gen() {
                            let (pool, new_population) = distribute_pool(
                                self.employment_pool,
                                *population,
                                max_pop,
                                0.0
                            );

                            self.employment_pool = pool;
                            *population = new_population;
                        }

                        industries += 1;
                        free_jobs += max_pop - *population;
                    },
                    _ => {}
                }

                tile.update();
            }

            shuffled_tiles.into_indices()
        };

        //manufacture pass
        for &index in shuffled_indices.iter() {
            let (region, level) = {
                let &(ref tile, _, _) = self.map.tile(index);
                if !tile.tile_type.similar_to(&tile::INDUSTRIAL) {
                    continue;
                }
                (tile.regions[0], tile.variant as u32 + 1)
            };

            let mut received_resources = 0;
            
            for &(ref mut tile2, _, _) in self.map.tiles() {
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

            let &(ref mut tile, _, _) = self.map.tile(index);
            match tile.tile_type {
                tile::Industrial {ref mut stored_goods, production, ..} => *stored_goods += (received_resources + production) * level,
                _ => unreachable!()
            }
        }

        //goods distribution pass
        for &index in shuffled_indices.iter() {
            let (region, level, population) = {
                let &(ref tile, _, _) = self.map.tile(index);
                let population = match tile.tile_type {
                    tile::Commercial {population, ..} => population,
                    _ => continue
                };
                (tile.regions[0], tile.variant as u32 + 1, population)
            };

            let mut received_goods = 0;
            let mut max_customers = 0.0;

            for &(ref mut tile2, _, _) in self.map.tiles() {
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

        let imigrants = 1.0 + (empty_homes - self.population_pool).max(0.0) * (free_jobs - self.employment_pool).max(0.0) * (1.0 - self.residential_tax) * 0.0001;
        let prob = (empty_homes - self.population_pool).max(0.0) * (free_jobs - self.employment_pool).max(0.0) * (1.0 - self.residential_tax) * 0.00001;
        
        //people moving to the city
        if stores > 0 && industries > 0 && prob > task_rng().gen() {
            self.population_pool += imigrants;
        }

        //people moving from the city
        if (self.population_pool > empty_homes || self.employment_pool > free_jobs) && (self.population_pool + self.employment_pool) * 0.01 > task_rng().gen() {
            self.population_pool -= (self.population_pool + self.employment_pool) * 0.05 + 1.0;
        }

        pop_total += self.population_pool;

        let new_workers = (pop_total - self.population).abs() * self.prop_can_work;
        self.employment_pool += new_workers;
        self.employable += new_workers;

        if self.employment_pool < 0.0 {
            self.employment_pool = 0.0;
        }
        
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
        self.employment_pool
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