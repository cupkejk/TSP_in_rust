use macroquad::prelude::*;
use ::rand::{rng, RngExt, rngs::ThreadRng};

const BORDER: f32 = 50.0;

enum Alg {
    SA,
    CLASSICAL,
    TWO_OPT,
}

struct State {
    cities: Vec<[f32; 2]>,
    tour: Vec<usize>,
    n: usize,
    temp: f64,
    dist: f32,
    rng: ThreadRng,
    running: bool,
    alg: Alg,
}

impl State {
    fn new(n: usize) -> Self {
        Self {
            cities: Vec::new(),
            tour: Vec::new(),
            n,
            temp: 1000.0,
            dist: 0.0,
            rng: rng(),
            running: false,
            alg: Alg::SA,
        }
    }

    fn randomize(&mut self) {
        self.cities.clear();
        self.tour.clear();

        let w = screen_width();
        let h = screen_height();

        let mut available_cities = Vec::new();

        // Initializing the cities
        for i in 0..self.n {
            let city: [f32; 2] = [
                self.rng.random_range(BORDER..(w - BORDER)),
                self.rng.random_range(BORDER..(h - BORDER)),
            ];
            self.cities.push(city);
            available_cities.push(i);
        }

        // Initializing the tour
        while !available_cities.is_empty() {
            let tour_index = self.rng.random_range(0..available_cities.len());
            let city_index = available_cities.swap_remove(tour_index);
            self.tour.push(city_index);
            if available_cities.is_empty() {break;}
        }

        // Initialize helper fields for simulated annealing
        self.temp = 1000.0;
        self.dist = self.calculate_total_dist();
    }

    fn randomize_tour(&mut self) {
        let mut available_cities = Vec::new();

        self.tour.clear();

        // Initializing the cities
        for i in 0..self.n {
            available_cities.push(i);
        }

        while !available_cities.is_empty() {
            let tour_index = self.rng.random_range(0..available_cities.len());
            let city_index = available_cities.swap_remove(tour_index);
            self.tour.push(city_index);
            if available_cities.is_empty() {break;}
        }

        self.temp = 1000.0;
        self.dist = self.calculate_total_dist();
        
    }

    // simulated annealing
    fn update_sa(&mut self) -> bool {
        if self.temp < 0.00001 { return false; }
        
        // randomly choosing 2 city indexes
        let i = self.rng.random_range(0..self.n);
        let j = self.rng.random_range(0..self.n);
        
        // if 2 same cities are chosen, dont do anything
        if i == j { return true; }
        let (smaller, bigger) = if i > j {(j, i)} else {(i, j)};
        let current_dist = self.calculate_total_dist();

        // 2-opt. reversing a small portion of the tour to remove tangles
        self.tour[smaller..=bigger].reverse();

        let next_dist = self.calculate_total_dist();

        // checking if we should accept the new tour
        let mut accept = false;
        if next_dist < current_dist {
            accept = true;
        } else {
            let delta = (next_dist - current_dist) as f64;
            let acceptance_prob = (-delta / self.temp).exp();
            if self.rng.random_range(0.0..=1.0) < acceptance_prob {
                accept = true;
            }
        }

        if accept {
            self.dist = next_dist;
        } else {
            // Undo the change and keep the old distance
            self.tour[smaller..=bigger].reverse();
            self.dist = current_dist;
        }

        // cooling down
        self.temp *= 0.9998;
        true
    }

    fn update_two_opt(&mut self) -> bool {
        let mut best_dist = self.calculate_total_dist();
        let mut flip_happened = false;
        for i in 0..(self.n-1) {
            for j in (i+1)..self.n {
                // 2-opt. reversing a small portion of the tour to remove tangles
                self.tour[i..=j].reverse();

                let next_dist = self.calculate_total_dist();

                // checking if we should accept the new tour
                if next_dist > best_dist {
                    self.tour[i..=j].reverse();
                }
                else {
                    best_dist = next_dist;
                    flip_happened = true;
                }
            }
        }
        self.dist = best_dist;
        flip_happened
    }

    fn display(&mut self) {
        // Displaying the tour as lines
        for i in 0..self.tour.len() {
            let c1 = self.cities[self.tour[i]];
            let c2 = self.cities[self.tour[(i + 1)%self.tour.len()]];
            draw_line(c1[0], c1[1], c2[0], c2[1], 1.0, BLACK);
        }

        // Displaying the cities as circles
        for i in 0..self.cities.len() {
            let city = self.cities[i];
            draw_circle(city[0], city[1], 10.0, GREEN);
        }

        let running_text = if self.running {&"State: running"} else {&"State: Done"};
        let running_color = if self.running {RED} else {DARKGREEN};
        let running_dims = measure_text(running_text, None, 30, 1.0);

        draw_text(&format!("Temp: {:.5}", self.temp), 20.0, 20.0, 30.0, BLACK);
        draw_text(&format!("Distance: {:.2}", self.dist), 20.0, 40.0, 30.0, BLACK);
        draw_text(running_text, screen_width() - 20.0 - running_dims.width, 20.0, 30.0, running_color);
    }

    fn calculate_total_dist(&mut self) -> f32 {
        let mut dist = 0.0;
        for i in 0..self.tour.len() {
            let c1 = self.cities[self.tour[i]];
            let c2 = self.cities[self.tour[(i + 1)%self.tour.len()]];
            dist += ((c1[0] - c2[0]).powi(2) + (c1[1] - c2[1]).powi(2)).sqrt();
        }
        dist
    }
}

#[macroquad::main("Travelling Salesman Problem")]
async fn main() {

    let mut state = State::new(100);
    state.randomize();

    // Main loop
    loop {
        // Logic & Input
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::H) {
            state.randomize();
            state.running = false;
        }
        if is_key_pressed(KeyCode::A) {
            state.alg = Alg::SA;
            state.running = true;
        }
        if is_key_pressed(KeyCode::T) {
            state.alg = Alg::TWO_OPT;
            state.running = true;
        }
        if is_key_pressed(KeyCode::R) {
            state.randomize_tour();
            state.running = false;
        }
        if is_key_pressed(KeyCode::S) {
            state.running = false;
        }

        // Rendering
        clear_background(LIGHTGRAY);
        
        state.display();
        if state.running == true {
            if matches!(state.alg, Alg::SA) {
                for _i in 0..100 {
                    if !state.update_sa() {
                        state.running = false;
                    }
                }
            }
            else if matches!(state.alg, Alg::TWO_OPT) {
                for _i in 0..1 {
                    if !state.update_two_opt() {
                        state.running = false;
                    }
                }
            }
        }

        // Wait for the next frame
        next_frame().await
    }
}