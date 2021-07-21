#![feature(arc_new_cyclic)]
use std::rc::{Rc, Weak};

/**
 A [Mutator] is an object capable of mutating a value for the purpose of
 fuzz-testing.

 For example, a mutator could change the value
 `v1 = [1, 4, 2, 1]` to `v1' = [1, 5, 2, 1]`.
 The idea is that if v1 is an “interesting” value to test, then v1' also
 has a high chance of being “interesting” to test.

 ## Complexity

 A mutator is also responsible for keeping track of the
 [complexity](crate::Mutator::complexity) of a value. The complexity is,
 roughly speaking, how large the value is.

 For example, the complexity of a vector is the complexity of its length,
 plus  the sum of the complexities of its elements. So `vec![]` would have a
 complexity of `1.0` and `vec![76]` would have a complexity of `10.0`: `2.0`
 for its short length and `8.0` for the 8-bit integer “76”. But there is no
 fixed rule for how to compute the complexity of a value, and it is up to you
 to judge how “large” something is.

 ## Cache

 In order to mutate values efficiently, the mutator is able to make use of a
 per-value *cache*. The Cache contains information associated with the value
 that will make it faster to compute its complexity or apply a mutation to
 it. For a vector, its cache is its total complexity, along with a vector of
 the cache of each of its element.

 ## MutationStep

 The same values will be passed to the mutator many times, so that it is
 mutated in many different ways. There are different strategies to choose
 what mutation to apply to a value. The first one is to create a list of
 mutation operations, and choose one to apply randomly from this list.

 However, one may want to have better control over which mutation operation
 is used. For example, if the value to be mutated is of type `Option<T>`,
 then you may want to first mutate it to `None`, and then always mutate it
 to another `Some(t)`. This is where `MutationStep` comes in. The mutation
 step is a type you define to allow you to keep track of which mutation
 operation has already been tried. This allows you to deterministically
 apply mutations to a value such that better mutations are tried first, and
 duplicate mutations are avoided.

 It is not always possible to schedule mutations in order. For that reason,
 we have two method: [random_mutate](crate::Mutator::random_mutate) executes
 a random mutation, and [ordered_mutate](crate::Mutator::ordered_mutate) uses
 the MutationStep to schedule mutations in order. The fuzzing engine only ever
 uses `ordered_mutate` directly, but the former is sometimes necessary to
 compose mutators together.

 If you don't want to bother with ordered mutations, that is fine. In that
 case, only implement `random_mutate` and call it from the `ordered_mutate`
 method.
 ```rust
 fn random_mutate(&self, value: &mut Value, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
     // ...
 }
fn ordered_mutate(&self, value: &mut Value, cache: &mut Self::Cache, step: &mut Self::MutationStep, max_cplx: f64) -> Option<(Self::UnmutateToken, f64)> {
    Some(self.random_mutate(value, cache, max_cplx))
 }
 ```

 ## Arbitrary

 A mutator must also be able to generate new values from nothing. This is what
 the [random_arbitrary](crate::Mutator::random_arbitrary) and
 [ordered_arbitrary](crate::Mutator::ordered_arbitrary) methods are for. The
 latter one is called by the fuzzer directly and uses an `ArbitraryStep` that
 can be used to smartly generate more interesting values first and avoid
 duplicates.

 ## Unmutate

 Finally, it is important to note that values and caches are mutated
 *in-place*. The fuzzer does not clone them before handing them to the
 mutator. Therefore, the mutator also needs to know how to reverse each
 mutation it performed. To do so, each mutation needs to return a token
 describing how to reverse it. The [unmutate](crate::Mutator::unmutate)
 method will later be called with that token to get the original value
 and cache back.

 For example, if the value is `[[1, 3], [5], [9, 8]]`, the mutator may
 mutate it to `[[1, 3], [5], [9, 1, 8]]` and return the token:
 `Element(2, Remove(1))`, which means that in order to reverse the
 mutation, the element at index 2 has to be unmutated by removing
 its element at index 1. In pseudocode:

 ```ignore
 value = [[1, 3], [5], [9, 8]];
 cache: c1 (ommitted from example)
 step: s1 (ommitted from example)

 let unmutate_token = self.mutate(&mut value, &mut cache, &mut step, max_cplx);

 // value = [[1, 3], [5], [9, 1, 8]]
 // token = Element(2, Remove(1))
 // cache = c2
 // step = s2

 test(&value);

 self.unmutate(&mut value, &mut cache, unmutate_token);

 // value = [[1, 3], [5], [9, 8]]
 // cache = c1 (back to original cache)
 // step = s2 (step has not been reversed)
 ```

When a mutated value is deemed interesting by the fuzzing engine, the method
[validate_value](crate::Mutator::validate_value) is called on it in order to
get a new Cache and MutationStep for it. The same method is called when the
fuzzer reads values from a corpus to verify that they conform to the
mutator’s expectations. For example, a CharWithinRangeMutator
will check whether the character is within a certain range.

Note that in most cases, it is completely fine to never mutate a value’s cache,
since it is recomputed by [validate_value](crate::Mutator::validate_value) when
needed.
**/
pub trait Mutator<Value: Clone> {
    /// Accompanies each value to help compute its complexity and mutate it efficiently.
    type Cache;
    /// Contains information about what mutations have already been tried.
    type MutationStep;
    /// Contains information about what arbitrary values have already been generated.
    type ArbitraryStep;
    /// Describes how to reverse a mutation
    type UnmutateToken;

    /// The first ArbitraryStep value to be passed to [ordered_arbitrary](crate::Mutator::ordered_arbitrary)
    fn default_arbitrary_step(&self) -> Self::ArbitraryStep;

    /// Verifies that the value conforms to the mutator’s expectations and, if it does,
    /// returns the Cache and first MutationStep associated with that value.
    fn validate_value(&self, value: &Value) -> Option<(Self::Cache, Self::MutationStep)>;

    /// The maximum complexity that a Value can possibly have.
    fn max_complexity(&self) -> f64;
    /// The minimum complexity that a Value can possibly have.
    fn min_complexity(&self) -> f64;

    /// Computes the complexity of the value.
    ///
    /// The returned value must be greater or equal than 0.
    fn complexity(&self, value: &Value, cache: &Self::Cache) -> f64;

    /// Generates an entirely new value based on the given `ArbitraryStep`.
    ///
    /// The generated value should be smaller than the given `max_cplx`.
    /// The return value is `None` if no more new value can be generated or if
    /// it is not possible to stay within the given complexity. Otherwise, it
    /// is the value itself and its complexity, which must be equal to
    /// `self.complexity(value, cache)`
    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(Value, f64)>;

    /// Generates an entirely new value.

    /// The generated value should be smaller
    /// than the given `max_cplx`. However, if that is not possible, then
    /// it should return a value of the lowest possible complexity.
    /// Returns the value itself and its complexity, which must be equal to
    /// `self.complexity(value, cache)`
    fn random_arbitrary(&self, max_cplx: f64) -> (Value, f64);

    /// Mutates a value (and optionally its cache) based on the given
    /// `MutationStep`.

    /// The mutated value should be within the given
    /// `max_cplx`. Returns `None` if it no longer possible to mutate
    /// the value to a new state, or if it is not possible to keep it under
    /// `max_cplx`. Otherwise, return the `UnmutateToken` that describes how to
    /// undo the mutation as well as the new complexity of the value.
    fn ordered_mutate(
        &self,
        value: &mut Value,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)>;

    /// Mutates a value (and optionally its cache). The mutated value should be
    /// within the given `max_cplx`. But if that is not possible, then it
    /// should mutate the value so that it has a minimal complexity. Returns
    /// the `UnmutateToken` that describes how to undo the mutation as well as
    /// the new complexity of the value.
    fn random_mutate(&self, value: &mut Value, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64);

    /// Undoes a mutation performed on the given value and cache, described by
    /// the given `UnmutateToken`.
    fn unmutate(&self, value: &mut Value, cache: &mut Self::Cache, t: Self::UnmutateToken);
}

/**
 * A Serializer is used to encode and decode values into bytes.
 *
 * One possible implementation would be to use `serde` to implement
 * both required functions. But we also want to be able to fuzz-test
 * types that are not serializable with `serde`, which is why this
 * Serializer trait exists.
*/
pub trait Serializer {
    type Value;
    fn is_utf8(&self) -> bool;
    fn extension(&self) -> &str;
    fn from_data(&self, data: &[u8]) -> Option<Self::Value>;
    fn to_data(&self, value: &Self::Value) -> Vec<u8>;
}

/// The ArbitraryStep that is used for recursive mutators
#[derive(Clone)]
pub enum RecursingArbitraryStep<AS> {
    Default,
    Initialized(AS),
}
impl<AS> Default for RecursingArbitraryStep<AS> {
    fn default() -> Self {
        Self::Default
    }
}

/**
A wrapper that allows a mutator to call itself recursively.

For example, it is used to provide mutators for types such as:
```rust
struct S {
    content: T,
    // to mutate this field, a mutator must be able to recursively call itself
    next: Option<Box<S>>
}
```
`RecursiveMutator` is only the top-level type. It must be used in conjuction
with [RecurToMutator](crate::RecurToMutator) at points of recursion.
For example:
```rust
let s_mutator = RecursiveMutator::new(|mutator| {
    SMutator {
        content_mutator: ContentMutator::default(),
        next_mutator: OptionMutator::new(BoxMutator::new(RecurToMutator::new(mutator.clone())))
    }
});
```
*/
pub struct RecursiveMutator<M> {
    pub mutator: Rc<M>,
}
impl<M> RecursiveMutator<M> {
    /// Create a new `RecursiveMutator` using a weak reference to itself.
    pub fn new(data_fn: impl FnOnce(&Weak<M>) -> M) -> Self {
        Self {
            mutator: Rc::new_cyclic(data_fn),
        }
    }
}

/// A mutator that defers to a weak reference of a [RecursiveMutator](crate::RecursiveMutator)
pub struct RecurToMutator<M> {
    reference: Weak<M>,
}
impl<M> From<&Weak<M>> for RecurToMutator<M> {
    fn from(reference: &Weak<M>) -> Self {
        Self {
            reference: reference.clone(),
        }
    }
}

impl<T, M> Mutator<T> for RecurToMutator<M>
where
    M: Mutator<T>,
    T: Clone,
{
    type Cache = <M as Mutator<T>>::Cache;
    type MutationStep = <M as Mutator<T>>::MutationStep;
    type ArbitraryStep = RecursingArbitraryStep<<M as Mutator<T>>::ArbitraryStep>;
    type UnmutateToken = <M as Mutator<T>>::UnmutateToken;

    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        RecursingArbitraryStep::Default
    }

    fn validate_value(&self, value: &T) -> Option<(Self::Cache, Self::MutationStep)> {
        self.reference.upgrade().unwrap().validate_value(value)
    }

    fn max_complexity(&self) -> f64 {
        std::f64::INFINITY
    }

    fn min_complexity(&self) -> f64 {
        0.0 // not right, but easy hack for now
    }

    fn complexity(&self, value: &T, cache: &Self::Cache) -> f64 {
        self.reference.upgrade().unwrap().complexity(value, cache)
    }

    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(T, f64)> {
        match step {
            RecursingArbitraryStep::Default => {
                let mutator = self.reference.upgrade().unwrap();
                let mut inner_step = mutator.default_arbitrary_step();
                let result = mutator.ordered_arbitrary(&mut inner_step, max_cplx);
                *step = RecursingArbitraryStep::Initialized(inner_step);
                result
            }
            RecursingArbitraryStep::Initialized(inner_step) => self
                .reference
                .upgrade()
                .unwrap()
                .ordered_arbitrary(inner_step, max_cplx),
        }
    }

    fn random_arbitrary(&self, max_cplx: f64) -> (T, f64) {
        self.reference.upgrade().unwrap().random_arbitrary(max_cplx)
    }

    fn ordered_mutate(
        &self,
        value: &mut T,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)> {
        self.reference
            .upgrade()
            .unwrap()
            .ordered_mutate(value, cache, step, max_cplx)
    }

    fn random_mutate(&self, value: &mut T, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
        self.reference.upgrade().unwrap().random_mutate(value, cache, max_cplx)
    }

    fn unmutate(&self, value: &mut T, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        self.reference.upgrade().unwrap().unmutate(value, cache, t)
    }
}

impl<T, M> Mutator<T> for RecursiveMutator<M>
where
    M: Mutator<T>,
    T: Clone,
{
    type Cache = <M as Mutator<T>>::Cache;
    type MutationStep = <M as Mutator<T>>::MutationStep;
    type ArbitraryStep = <M as Mutator<T>>::ArbitraryStep;
    type UnmutateToken = <M as Mutator<T>>::UnmutateToken;

    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        Rc::as_ref(&self.mutator).default_arbitrary_step()
    }

    fn validate_value(&self, value: &T) -> Option<(Self::Cache, Self::MutationStep)> {
        Rc::as_ref(&self.mutator).validate_value(value)
    }

    fn max_complexity(&self) -> f64 {
        std::f64::INFINITY
    }

    fn min_complexity(&self) -> f64 {
        Rc::as_ref(&self.mutator).min_complexity()
    }

    fn complexity(&self, value: &T, cache: &Self::Cache) -> f64 {
        Rc::as_ref(&self.mutator).complexity(value, cache)
    }

    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(T, f64)> {
        Rc::as_ref(&self.mutator).ordered_arbitrary(step, max_cplx)
    }

    fn random_arbitrary(&self, max_cplx: f64) -> (T, f64) {
        Rc::as_ref(&self.mutator).random_arbitrary(max_cplx)
    }

    fn ordered_mutate(
        &self,
        value: &mut T,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)> {
        Rc::as_ref(&self.mutator).ordered_mutate(value, cache, step, max_cplx)
    }

    fn random_mutate(&self, value: &mut T, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
        Rc::as_ref(&self.mutator).random_mutate(value, cache, max_cplx)
    }

    fn unmutate(&self, value: &mut T, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        Rc::as_ref(&self.mutator).unmutate(value, cache, t)
    }
}

impl<T: Clone, M> Mutator<T> for Box<M>
where
    M: Mutator<T>,
{
    type Cache = M::Cache;
    type MutationStep = M::MutationStep;
    type ArbitraryStep = M::ArbitraryStep;
    type UnmutateToken = M::UnmutateToken;

    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        self.as_ref().default_arbitrary_step()
    }

    fn validate_value(&self, value: &T) -> Option<(Self::Cache, Self::MutationStep)> {
        self.as_ref().validate_value(value)
    }

    fn max_complexity(&self) -> f64 {
        self.as_ref().max_complexity()
    }

    fn min_complexity(&self) -> f64 {
        self.as_ref().min_complexity()
    }

    fn complexity(&self, value: &T, cache: &Self::Cache) -> f64 {
        self.as_ref().complexity(value, cache)
    }

    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(T, f64)> {
        self.as_ref().ordered_arbitrary(step, max_cplx)
    }

    fn random_arbitrary(&self, max_cplx: f64) -> (T, f64) {
        self.as_ref().random_arbitrary(max_cplx)
    }

    fn ordered_mutate(
        &self,
        value: &mut T,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)> {
        self.as_ref().ordered_mutate(value, cache, step, max_cplx)
    }

    fn random_mutate(&self, value: &mut T, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
        self.as_ref().random_mutate(value, cache, max_cplx)
    }

    fn unmutate(&self, value: &mut T, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        self.as_ref().unmutate(value, cache, t)
    }
}
