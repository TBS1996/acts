use derive_builder::Builder;
use std::collections::BTreeSet;
use std::time::Duration;
struct Day {
    date: u32,
    start: Duration,
    length: Duration,
    slots: Vec<Slot>,
}

#[derive(Default, Builder, Clone)]
struct Slot {
    start: Duration,
    title: String,
    desc: String,
    length: Duration,
    act_len: Duration,
    fixed_time: bool,
    fixed_start: bool,
}

impl Slot {}

impl Default for Day {
    fn default() -> Self {
        Self {
            date: 0,
            start: Duration::from_secs(60 * 60 * 8),
            length: Duration::from_secs(60 * 60 * 16),
            slots: Vec::new(),
        }
    }
}

impl Day {
    fn new(start: Duration) -> Self {
        Self {
            start,
            ..Default::default()
        }
    }

    fn split_by<T, F>(data: Vec<T>, predicate: F) -> Vec<Vec<T>>
    where
        T: Clone,
        F: Fn(&T) -> bool,
    {
        data.into_iter().fold(Vec::new(), |mut acc, x| {
            if predicate(&x) {
                acc.push(Vec::new());
            }
            if let Some(last) = acc.last_mut() {
                last.push(x);
            } else {
                acc.push(vec![x]);
            }
            acc
        })
    }

    fn main() {
        let numbers = vec![1, 2, 3, -1, 4, 5, -1, 6, 7, 8, 9];
        let split_predicate = |x: &i32| *x == -1;
        let result: Vec<Vec<i32>> = split_by(numbers, split_predicate);

        println!("{:?}", result);
    }

    fn calculate_actual_length(&mut self) {
        let predicate = |slot: &Slot| slot.fixed_start;

        let splitvec = split_by(self.slots.clone(), predicate);

        for vec in splitvec {
            let tot: f32 = vec.iter().map(|slot| slot.length.as_secs_f32()).sum();
            let qty = vec.len();
        }
    }

    fn sort_slots(&mut self) {
        self.slots.sort_by_key(|slot| slot.start);
    }

    fn insert_slot(&mut self, slot: Slot) {
        self.slots.push(slot);
        self.sort_slots();
    }

    pub fn print(&self) {
        println!("current date: {}", self.date);
        println!("start time: {:?}", self.start);
        println!("Slots! :D ");

        for slot in self.slots.iter() {
            println!("{:?}: {:?} ", slot.title, slot.start);
        }
    }

    pub fn debug() {
        let day = Day::default();
        let time = |h: f32| Duration::from_secs_f32(h * 3600.);

        let some_slot = SlotBuilder::default()
            .start(time(8.))
            .title("breakfast".into())
            .length(time(0.5))
            .build()
            .unwrap();

        let another_slot = SlotBuilder::default()
            .start(time(8.))
            .title("breakfast".into())
            .length(time(0.5))
            .build()
            .unwrap();
    }
}

fn split_by<T, F>(data: Vec<T>, predicate: F) -> Vec<Vec<T>>
where
    T: Clone,
    F: Fn(&T) -> bool,
{
    data.into_iter().fold(Vec::new(), |mut acc, x| {
        if predicate(&x) {
            acc.push(Vec::new());
        }
        if let Some(last) = acc.last_mut() {
            last.push(x);
        } else {
            acc.push(vec![x]);
        }
        acc
    })
}
