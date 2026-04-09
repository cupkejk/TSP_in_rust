use macroquad::prelude::*;
use ::rand::{rng, RngExt, rngs::ThreadRng};

const BORDER: f32 = 50.0;

enum Alg {
    SA,
    CLASSICAL,
}

struct State {
    cities: Vec<[f32; 2]>,
    tour: Vec<usize>,
    n: usize,
    temp: f64,
    dist: f32,
    rng: ThreadRng,
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

    // simulated annealing
    fn update_sa(&mut self) {
        if self.temp < 0.01 { return; }
        
        // randomly choosing 2 city indexes
        let i = self.rng.random_range(0..self.n);
        let j = self.rng.random_range(0..self.n);
        
        // if 2 same cities are chosen, dont do anything
        if i == j { return; }
        let (smaller, bigger) = if i > j {(j, i)} else {(i, j)};
        let current_dist = self.calculate_total_dist();

        // 2-opt. reversing a small portion of the tour to remove tangles
        self.tour[smaller..=bigger].reverse();

        let next_dist = self.calculate_total_dist();

        // checking if we should accept the new tour
        if next_dist > current_dist {
            let delta = (next_dist - current_dist) as f64;
            let acceptance_prob = (-delta / self.temp).exp();

            // checking if we should accept the new tour based on current temperature
            if self.rng.random_range(0.0..=1.0) > acceptance_prob {
                self.tour[smaller..=bigger].reverse();
                self.dist = current_dist;
            }
        }

        // cooling down
        self.temp *= 0.9999;
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
    }

    fn calculate_total_dist(&mut self) -> f32 {
        let mut dist = 0.0;
        for i in 0..self.tour.len() {
            let c1 = self.cities[self.tour[i]];
            let c2 = self.cities[self.tour[(i + 1)%self.tour.len()]];
            dist += ((c1[0] - c2[0]).powi(2) + (c1[1] - c2[1]).powi(2)).sqrt();
        }
        self.dist = dist;
        dist
    }
}

#[macroquad::main("Travelling Salesman Problem")]
async fn main() {

    let mut state = State::new(100);
    state.randomize();
    let mut running = false;
    let mut alg = Alg::SA;

    // Main loop
    loop {
        // Logic & Input
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::R) {
            state.randomize();
        }
        if is_key_pressed(KeyCode::A) {
            alg = Alg::SA;
            running = true;
        }

        // Rendering
        clear_background(LIGHTGRAY);

        draw_text("Hello, Macroquad!", 20.0, 20.0, 30.0, DARKGRAY);
        
        state.display();
        if running == true {
            for _i in 0..100 {
                state.update_sa();
            }
        }

        // Wait for the next frame
        next_frame().await
    }
}