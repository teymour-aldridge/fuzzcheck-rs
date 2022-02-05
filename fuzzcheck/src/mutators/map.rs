use std::marker::PhantomData;

use crate::Mutator;

pub struct MapMutator<From, To, M, Parse, Map>
where
    From: Clone + 'static,
    To: Clone + 'static,
    M: Mutator<From>,
    Parse: Fn(&To) -> Option<From>,
    Map: Fn(&From) -> To,
{
    pub mutator: M,
    pub parse: Parse,
    pub map: Map,
    _phantom: PhantomData<(To, From)>,
}
impl<From, To, M, Parse, Map> MapMutator<From, To, M, Parse, Map>
where
    From: Clone + 'static,
    To: Clone + 'static,
    M: Mutator<From>,
    Parse: Fn(&To) -> Option<From>,
    Map: Fn(&From) -> To,
{
    #[no_coverage]
    pub fn new(mutator: M, parse: Parse, map: Map) -> Self {
        Self {
            mutator,
            parse,
            map,
            _phantom: PhantomData,
        }
    }
}

pub struct Cache<From, M>
where
    From: Clone + 'static,
    M: Mutator<From>,
{
    from_value: From,
    from_cache: M::Cache,
}
impl<From, M> Clone for Cache<From, M>
where
    From: Clone + 'static,
    M: Mutator<From>,
{
    fn clone(&self) -> Self {
        Self {
            from_value: self.from_value.clone(),
            from_cache: self.from_cache.clone(),
        }
    }
}

impl<From, To, M, Parse, Map> Mutator<To> for MapMutator<From, To, M, Parse, Map>
where
    From: Clone + 'static,
    To: Clone + 'static,
    M: Mutator<From>,
    Parse: Fn(&To) -> Option<From>,
    Map: Fn(&From) -> To,
    Self: 'static,
{
    #[doc(hidden)]
    type Cache = Cache<From, M>;
    #[doc(hidden)]
    type MutationStep = M::MutationStep;
    #[doc(hidden)]
    type ArbitraryStep = M::ArbitraryStep;
    #[doc(hidden)]
    type UnmutateToken = M::UnmutateToken;
    #[doc(hidden)]
    type LensPath = M::LensPath;

    #[doc(hidden)]
    #[no_coverage]
    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        self.mutator.default_arbitrary_step()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn validate_value(&self, to_value: &To) -> Option<Self::Cache> {
        let from_value = (self.parse)(to_value)?;
        let from_cache = self.mutator.validate_value(&from_value)?;
        Some(Cache { from_value, from_cache })
    }
    #[doc(hidden)]
    #[no_coverage]
    fn default_mutation_step(&self, _value: &To, cache: &Self::Cache) -> Self::MutationStep {
        self.mutator.default_mutation_step(&cache.from_value, &cache.from_cache)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn global_search_space_complexity(&self) -> f64 {
        self.mutator.global_search_space_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn max_complexity(&self) -> f64 {
        self.mutator.max_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn min_complexity(&self) -> f64 {
        self.mutator.min_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn complexity(&self, _value: &To, cache: &Self::Cache) -> f64 {
        self.mutator.complexity(&cache.from_value, &cache.from_cache)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<(To, f64)> {
        let (from_value, cplx) = self.mutator.ordered_arbitrary(step, max_cplx)?;
        let to_value = (self.map)(&from_value);
        Some((to_value, cplx))
    }

    #[doc(hidden)]
    #[no_coverage]
    fn random_arbitrary(&self, max_cplx: f64) -> (To, f64) {
        let (from_value, cplx) = self.mutator.random_arbitrary(max_cplx);
        let to_value = (self.map)(&from_value);
        (to_value, cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn ordered_mutate(
        &self,
        value: &mut To,
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)> {
        let (token, cplx) =
            self.mutator
                .ordered_mutate(&mut cache.from_value, &mut cache.from_cache, step, max_cplx)?;
        *value = (self.map)(&cache.from_value);
        Some((token, cplx))
    }

    #[doc(hidden)]
    #[no_coverage]
    fn random_mutate(&self, value: &mut To, cache: &mut Self::Cache, max_cplx: f64) -> (Self::UnmutateToken, f64) {
        let (token, cplx) = self
            .mutator
            .random_mutate(&mut cache.from_value, &mut cache.from_cache, max_cplx);
        *value = (self.map)(&cache.from_value);
        (token, cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn unmutate(&self, value: &mut To, cache: &mut Self::Cache, t: Self::UnmutateToken) {
        self.mutator.unmutate(&mut cache.from_value, &mut cache.from_cache, t);
        *value = (self.map)(&cache.from_value);
    }

    #[doc(hidden)]
    #[no_coverage]
    fn lens<'a>(&self, _value: &'a To, cache: &'a Self::Cache, path: &Self::LensPath) -> &'a dyn std::any::Any {
        self.mutator.lens(&cache.from_value, &cache.from_cache, path)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn all_paths(
        &self,
        _value: &To,
        cache: &Self::Cache,
        register_path: &mut dyn FnMut(std::any::TypeId, Self::LensPath),
    ) {
        self.mutator
            .all_paths(&cache.from_value, &cache.from_cache, register_path)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn crossover_mutate(
        &self,
        value: &mut To,
        cache: &mut Self::Cache,
        subvalue_provider: &dyn crate::SubValueProvider,
        max_cplx: f64,
    ) -> (Self::UnmutateToken, f64) {
        let (token, cplx) = self.mutator.crossover_mutate(
            &mut cache.from_value,
            &mut cache.from_cache,
            subvalue_provider,
            max_cplx,
        );
        *value = (self.map)(&cache.from_value);
        (token, cplx)
    }
}

pub struct AndMapMutator<From, To, M, Map>
where
    From: Clone + 'static,
    To: Clone + 'static,
    M: Mutator<From>,
    Map: Fn(&From, &mut To),
{
    pub mutator: M,
    pub map: Map,
    storage_to: To,
    _phantom: PhantomData<(To, From)>,
}
impl<From, To, M, Map> AndMapMutator<From, To, M, Map>
where
    From: Clone + 'static,
    To: Clone + 'static,
    M: Mutator<From>,
    Map: Fn(&From, &mut To),
{
    #[no_coverage]
    pub fn new(mutator: M, map: Map, storage: To) -> Self {
        Self {
            mutator,
            map,
            storage_to: storage,
            _phantom: PhantomData,
        }
    }
}

impl<From, To, M, Map> Mutator<(To, From)> for AndMapMutator<From, To, M, Map>
where
    From: Clone + 'static,
    To: Clone + 'static,
    M: Mutator<From>,
    Map: Fn(&From, &mut To),
    Self: 'static,
{
    #[doc(hidden)]
    type Cache = M::Cache;
    #[doc(hidden)]
    type MutationStep = M::MutationStep;
    #[doc(hidden)]
    type ArbitraryStep = M::ArbitraryStep;
    #[doc(hidden)]
    type UnmutateToken = M::UnmutateToken;
    #[doc(hidden)]
    type LensPath = M::LensPath;

    #[doc(hidden)]
    #[no_coverage]
    fn default_arbitrary_step(&self) -> Self::ArbitraryStep {
        self.mutator.default_arbitrary_step()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn validate_value(&self, value: &(To, From)) -> Option<Self::Cache> {
        let (_, from_value) = value;
        let from_cache = self.mutator.validate_value(&from_value)?;
        Some(from_cache)
    }
    #[doc(hidden)]
    #[no_coverage]
    fn default_mutation_step(&self, value: &(To, From), cache: &Self::Cache) -> Self::MutationStep {
        let (_, from_value) = value;
        self.mutator.default_mutation_step(from_value, cache)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn global_search_space_complexity(&self) -> f64 {
        self.mutator.global_search_space_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn max_complexity(&self) -> f64 {
        self.mutator.max_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn min_complexity(&self) -> f64 {
        self.mutator.min_complexity()
    }

    #[doc(hidden)]
    #[no_coverage]
    fn complexity(&self, value: &(To, From), cache: &Self::Cache) -> f64 {
        let (_, from_value) = value;
        self.mutator.complexity(from_value, cache)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn ordered_arbitrary(&self, step: &mut Self::ArbitraryStep, max_cplx: f64) -> Option<((To, From), f64)> {
        let (from_value, cplx) = self.mutator.ordered_arbitrary(step, max_cplx)?;
        let mut to_value = self.storage_to.clone();
        (self.map)(&from_value, &mut to_value);
        Some(((to_value, from_value), cplx))
    }

    #[doc(hidden)]
    #[no_coverage]
    fn random_arbitrary(&self, max_cplx: f64) -> ((To, From), f64) {
        let (from_value, cplx) = self.mutator.random_arbitrary(max_cplx);
        let mut to_value = self.storage_to.clone();
        (self.map)(&from_value, &mut to_value);
        ((to_value, from_value), cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn ordered_mutate(
        &self,
        value: &mut (To, From),
        cache: &mut Self::Cache,
        step: &mut Self::MutationStep,
        max_cplx: f64,
    ) -> Option<(Self::UnmutateToken, f64)> {
        let (to_value, from_value) = value;
        let (token, cplx) = self.mutator.ordered_mutate(from_value, cache, step, max_cplx)?;
        (self.map)(from_value, to_value);
        Some((token, cplx))
    }

    #[doc(hidden)]
    #[no_coverage]
    fn random_mutate(
        &self,
        value: &mut (To, From),
        cache: &mut Self::Cache,
        max_cplx: f64,
    ) -> (Self::UnmutateToken, f64) {
        let (to_value, from_value) = value;
        let (token, cplx) = self.mutator.random_mutate(from_value, cache, max_cplx);
        (self.map)(from_value, to_value);
        (token, cplx)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn unmutate(&self, value: &mut (To, From), cache: &mut Self::Cache, t: Self::UnmutateToken) {
        let (_, from_value) = value;
        self.mutator.unmutate(from_value, cache, t);
    }

    #[doc(hidden)]
    #[no_coverage]
    fn lens<'a>(&self, value: &'a (To, From), cache: &'a Self::Cache, path: &Self::LensPath) -> &'a dyn std::any::Any {
        let (_, from_value) = value;
        self.mutator.lens(from_value, cache, path)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn all_paths(
        &self,
        value: &(To, From),
        cache: &Self::Cache,
        register_path: &mut dyn FnMut(std::any::TypeId, Self::LensPath),
    ) {
        let (_, from_value) = value;
        self.mutator.all_paths(from_value, cache, register_path)
    }

    #[doc(hidden)]
    #[no_coverage]
    fn crossover_mutate(
        &self,
        value: &mut (To, From),
        cache: &mut Self::Cache,
        subvalue_provider: &dyn crate::SubValueProvider,
        max_cplx: f64,
    ) -> (Self::UnmutateToken, f64) {
        let (to_value, from_value) = value;
        let (token, cplx) = self
            .mutator
            .crossover_mutate(from_value, cache, subvalue_provider, max_cplx);
        (self.map)(from_value, to_value);
        (token, cplx)
    }
}
