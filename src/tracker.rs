use crate::simulator::{CarData, SimEvent};
use std::hash::BuildHasher;

type HashBuilder = hashbrown::DefaultHashBuilder;

#[derive(Default)]
struct CarDataLookup {
    data: Vec<Vec<CarData>>,
    table: hashbrown::HashTable<usize>,
    builder: HashBuilder,
}

impl CarDataLookup {
    fn data_for(&self, n: usize) -> (bool, &Vec<CarData>) {
        Self::data_for_static(&self.data, n)
    }
    fn data_for_static(data: &[Vec<CarData>], n: usize) -> (bool, &Vec<CarData>) {
        (n & 1 != 0, &data[n])
    }
    fn hash_for(&self, n: usize) -> u64 {
        Self::hash_for_static(&self.data, &self.builder, n)
    }
    fn hash_for_static(data: &[Vec<CarData>], builder: &HashBuilder, n: usize) -> u64 {
        builder.hash_one(Self::data_for_static(data, n))
    }
    fn add(&mut self, item: Vec<CarData>) -> bool {
        self.data.push(item);
        let hash = self.hash_for(self.data.len() - 1);
        let found = self
            .table
            .find(hash, |&n| {
                self.data_for(n) == self.data_for(self.data.len() - 1)
            })
            .is_some();
        self.table.insert_unique(hash, self.data.len() - 1, |&n| {
            Self::hash_for_static(&self.data, &self.builder, n)
        });
        found
    }
    fn values(&self) -> &Vec<Vec<CarData>> {
        &self.data
    }
}

pub struct Tracker {
    round_data: CarDataLookup,
    finished: Vec<usize>,
    crashed: Vec<bool>,
    loop_detected: bool,
}

pub fn compute_not_finishing(num_cars: usize, finished: &Vec<usize>) -> Vec<bool> {
    let mut unseen = vec![true; num_cars];
    for &n in finished {
        unseen[n] = false;
    }
    unseen
}

impl Tracker {
    pub fn new(num_cars: usize) -> Self {
        let mut round_data = CarDataLookup::default();
        round_data.add(Vec::new());
        Self {
            round_data,
            finished: vec![],
            crashed: vec![false; num_cars],
            loop_detected: false,
        }
    }
    pub fn get_finishes(&self) -> &Vec<usize> {
        &self.finished
    }

    pub fn get_crashes(&self) -> &Vec<bool> {
        &self.crashed
    }

    pub fn compute_final_crashes(&mut self, num_cars: usize) {
        self.crashed = compute_not_finishing(num_cars, &self.finished);
    }

    pub fn get_cars(&self) -> &Vec<Vec<CarData>> {
        self.round_data.values()
    }
    pub fn process_event(&mut self, ev: SimEvent) {
        match ev {
            SimEvent::Round(cars) => self.add_round(cars),
            SimEvent::Finished(car) => self.finished.push(car),
            SimEvent::Crashed(car) => self.crashed[car] = true,
        }
    }

    pub fn add_round(&mut self, round: Vec<CarData>) {
        self.loop_detected |= self.round_data.add(round);
    }
    pub fn rounds_available(&self) -> usize {
        self.get_cars().len()
    }
    pub fn is_loop_detected(&self) -> bool {
        self.loop_detected
    }
}
