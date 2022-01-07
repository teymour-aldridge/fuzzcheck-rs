use crate::data_structures::{Slab, SlabKey};
use crate::fuzzer::PoolStorageIndex;
use crate::traits::{CorpusDelta, Observations, Pool, SaveToStatsFolder, Stats};
use crate::{CompatibleWithObservations, ToCSV};
use ahash::{AHashMap, AHashSet};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::path::Path;

#[derive(Clone, Default)]
pub struct UniqueValuesPoolStats {
    pub name: String,
    pub size: usize,
}
impl ToCSV for UniqueValuesPoolStats {
    #[no_coverage]
    fn csv_headers(&self) -> Vec<crate::CSVField> {
        vec![]
    }

    #[no_coverage]
    fn to_csv_record(&self) -> Vec<crate::CSVField> {
        vec![]
    }
}

impl Display for UniqueValuesPoolStats {
    #[no_coverage]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            nu_ansi_term::Color::Blue.paint(format!("{}({})", self.name, self.size))
        )
    }
}
impl Stats for UniqueValuesPoolStats {}

#[derive(Debug)]
struct Input<T>
where
    T: Hash + Eq + Clone,
{
    best_for_values: AHashSet<(usize, T)>,
    data: PoolStorageIndex,
    score: f64,
}

/// A pool that stores an input for each different value of each sensor counter
pub struct UniqueValuesPool<T>
where
    T: Hash + Eq + Clone,
{
    name: String,
    complexities: Vec<AHashMap<T, f64>>,
    inputs: Slab<Input<T>>,
    best_input_for_value: Vec<AHashMap<T, SlabKey<Input<T>>>>,
    stats: UniqueValuesPoolStats,
}
impl<T> Debug for UniqueValuesPool<T>
where
    T: Hash + Eq + Clone + Debug,
{
    #[no_coverage]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UniqueValuesPool")
            .field("complexities", &self.complexities)
            // .field("highest_counts", &self.highest_counts)
            .field("inputs", &self.inputs)
            // .field("best_input_for_counter", &self.best_input_for_counter)
            // .field("cumulative_score_inputs", &self.ranked_inputs)
            .finish()
    }
}

impl<T> UniqueValuesPool<T>
where
    T: Hash + Eq + Clone,
{
    #[no_coverage]
    pub fn new(name: &str, size: usize) -> Self {
        Self {
            name: name.to_string(),
            complexities: vec![AHashMap::new(); size],
            inputs: Slab::new(),
            best_input_for_value: vec![AHashMap::new(); size],
            stats: UniqueValuesPoolStats {
                name: name.to_string(),
                size: 0,
            },
        }
    }
}

impl<T> Pool for UniqueValuesPool<T>
where
    T: Hash + Eq + Clone,
{
    type Stats = UniqueValuesPoolStats;

    #[no_coverage]
    fn stats(&self) -> Self::Stats {
        self.stats.clone()
    }

    #[no_coverage]
    fn ranked_test_cases(&self) -> Vec<(PoolStorageIndex, f64)> {
        let inputs = &self.inputs;
        self.inputs
            .keys()
            .map(
                #[no_coverage]
                |key| {
                    let input = &inputs[key];
                    (input.data, input.score)
                },
            )
            .collect()
    }
}
impl<T> SaveToStatsFolder for UniqueValuesPool<T>
where
    T: Hash + Eq + Clone,
{
    #[no_coverage]
    fn save_to_stats_folder(&self) -> Vec<(std::path::PathBuf, Vec<u8>)> {
        vec![]
    }
}

impl<T> UniqueValuesPool<T>
where
    T: Hash + Eq + Clone,
{
    #[no_coverage]
    fn update_stats(&mut self) {
        self.stats.size = self.inputs.len();
    }
}

impl<T, O> CompatibleWithObservations<O> for UniqueValuesPool<T>
where
    O: Observations,
    for<'a> O::Concrete<'a>: IntoIterator<Item = (usize, T)>,
    T: Hash + Eq + Clone + Copy + 'static,
{
    #[no_coverage]
    fn process<'a>(
        &'a mut self,
        input_id: PoolStorageIndex,
        observations: O::Concrete<'a>,
        complexity: f64,
    ) -> Vec<CorpusDelta> {
        let mut state = vec![];
        for (index, v) in observations {
            if let Some(&previous_cplx) = self.complexities[index].get(&v) {
                if previous_cplx > complexity {
                    // already exists but this one is better
                    state.push((index, v));
                }
            } else {
                state.push((index, v));
            }
        }
        if state.is_empty() {
            return vec![];
        }

        let new_observations = state;
        let score = new_observations.len() as f64;
        let cplx = complexity;
        let input = input_id;
        let input = Input {
            best_for_values: AHashSet::new(), // fill in later! with new_observations.into_iter().collect(),
            data: input,
            score,
        };

        let input_key = self.inputs.insert(input);

        let mut removed_keys = vec![];

        for (counter, id) in &new_observations {
            self.complexities[*counter].insert(id.clone(), cplx);

            let previous_best_key = self.best_input_for_value[*counter].get_mut(id);
            if let Some(previous_best_key) = previous_best_key {
                let previous_best = &mut self.inputs[*previous_best_key];
                let was_present_in_set = previous_best.best_for_values.remove(&(*counter, id.clone()));
                assert!(was_present_in_set);
                previous_best.score = previous_best.best_for_values.len() as f64;
                if previous_best.best_for_values.is_empty() {
                    removed_keys.push(*previous_best_key);
                }
                *previous_best_key = input_key;
            } else {
                self.best_input_for_value[*counter].insert(id.clone(), input_key);
            }
        }
        for &removed_key in &removed_keys {
            self.inputs.remove(removed_key);
        }
        let removed_keys = removed_keys
            .into_iter()
            .map(
                #[no_coverage]
                |k| self.inputs[k].data,
            )
            .collect();
        self.update_stats();
        return vec![CorpusDelta {
            path: Path::new(&self.name).to_path_buf(),
            add: true,
            remove: removed_keys,
        }];
    }
}
