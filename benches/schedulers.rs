pub mod schedulers {
    use std::fmt::Debug;
    use egg::*;

    // #[cfg(feature = "parallel")]
    use rayon::prelude::*;


    /// A very simple [`RewriteScheduler`] that runs every rewrite every
    /// time, using an iterator approach.
    ///
    /// This is not the default scheduler; choose it with the
    /// [`with_scheduler`](Runner::with_scheduler())
    /// method.
    ///
    #[derive(Debug)]
    pub struct IteratorScheduler;

    impl<L, N> RewriteScheduler<L, N> for IteratorScheduler
    where
        L: Language,
        N: Analysis<L>,
    {
        fn search_rewrites<'a>(
            &mut self,
            iteration: usize,
            egraph: &EGraph<L, N>,
            rewrites: &[&'a Rewrite<L, N>],
            limits: &RunnerLimits,
        ) -> RunnerResult<Vec<Vec<SearchMatches<'a, L>>>> {
            rewrites
                .iter()
                .map(|rw| {
                    let ms = rw.search(egraph);
                    limits.check_limits(iteration, egraph)?;
                    Ok(ms)
                })
                .collect()
        }
    }

    /// A very simple [`RewriteScheduler`] that uses `rayon` to parallelise. It
    /// must be compiled with the `parallel` feature enabled, otherwise it will
    /// use the default [`search_rewrite`] implementation.
    ///
    /// This does not implement all the features provided by the default
    /// [`BackoffScheduler`], so must be compared to the [`IteratorScheduler`] for
    /// fair comparison.
    ///
    /// This is not the default scheduler; choose it with the
    /// [`with_scheduler`](Runner::with_scheduler())
    /// method.
    ///
    #[derive(Debug)]
    pub struct ParallelIteratorScheduler;

    impl<L, N> RewriteScheduler<L, N> for ParallelIteratorScheduler
    where
        L: Language + Sync + Send,
        L::Discriminant: Sync + Send,
        N: Analysis<L> + Sync + Send,
        N::Data: Sync + Send
    {
        // #[cfg(feature = "parallel")]
        fn search_rewrites<'a>(
            &mut self,
            iteration: usize,
            egraph: &EGraph<L, N>,
            rewrites: &[&'a Rewrite<L, N>],
            limits: &RunnerLimits,
        ) -> RunnerResult<Vec<Vec<SearchMatches<'a, L>>>> {
            // This implementation just ignores the limits
            // fake `par_map` to enforce Send + Sync, in real life use rayon
            // fn par_map<T, F, T2>(slice: &[T], f: F) -> Vec<T2>
            // where
            //     T: Send + Sync,
            //     F: Fn(&T) -> T2 + Send + Sync,
            //     T2: Send + Sync,
            // {
            //     slice.iter().map(f).collect()
            // }
            // Ok(par_map(rewrites, |rw| rw.search(egraph)))

            rewrites
                .par_iter()
                .map(|rw| {
                    let ms = rw.search(egraph);
                    limits.check_limits(iteration, egraph)?;
                    Ok(ms)
                })
                .collect() // ::<RunnerResult<Vec<Vec<SearchMatches<'a, L>>>>>()

            // TODO: Note that `Sync + Send` traits were added to both language and
            //       discriminant. Could this impact correctness?
        }
    }

}
