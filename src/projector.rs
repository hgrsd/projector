use std::marker::PhantomData;

pub struct Projector<'a, T, U, V>
where
    T: Default + Clone,
    V: Fn(&T, &U) -> T,
{
    _l0: &'a PhantomData<()>,
    _e0: PhantomData<T>,
    _e1: PhantomData<U>,
    applier: V,
}

impl<'a, T, U, V> Projector<'a, T, U, V>
where
    T: Default + Clone,
    V: Fn(&T, &U) -> T,
{
    /// Create a projector from an applier.
    ///
    /// An applier is a function that takes in a reference to an event and entity,
    /// and returns the new state for the entity.
    ///
    /// An entity must implement Clone and Default.
    ///
    /// ```
    /// use crate::projector::Projector;
    ///
    /// enum PaymentEvent {
    ///   MoneySpent(i32),
    ///   MoneyReceived(i32),
    /// }
    ///
    /// #[derive(Clone, Default)]
    /// struct Account {
    ///   balance: i32,
    /// }
    /// let projector = Projector::from_applier(|entity: &Account, event: &PaymentEvent| {
    ///     Account {
    ///         balance: match event {
    ///             PaymentEvent::MoneySpent(m) => entity.balance - m,
    ///             PaymentEvent::MoneyReceived(m) => entity.balance + m,
    ///         }
    ///     }
    /// });
    /// ```
    pub fn from_applier(applier: V) -> Self {
        Projector {
            _l0: &PhantomData,
            _e0: PhantomData,
            _e1: PhantomData,
            applier,
        }
    }

    /// Stream all versions of an entity based on a stream of events.
    ///
    /// ```
    /// use crate::projector::Projector;
    ///
    /// enum PaymentEvent {
    ///   MoneySpent(i32),
    ///   MoneyReceived(i32),
    /// }
    ///
    /// #[derive(Clone, Default, Eq, PartialEq, Debug)]
    /// struct Account {
    ///   balance: i32,
    /// }
    ///
    /// let events = vec![
    ///     PaymentEvent::MoneySpent(300),
    ///     PaymentEvent::MoneyReceived(150),
    /// ];
    ///
    /// let projector = Projector::from_applier(|entity: &Account, event: &PaymentEvent| {
    ///     Account {
    ///         balance: match event {
    ///             PaymentEvent::MoneySpent(m) => entity.balance - m,
    ///             PaymentEvent::MoneyReceived(m) => entity.balance + m,
    ///         }
    ///     }
    /// });
    /// let versions: Vec<Account> = projector.stream_entities(events.into_iter()).collect();
    /// assert_eq!(
    ///     versions,
    ///     vec![
    ///         Account {
    ///             balance: -300,
    ///         },
    ///         Account {
    ///             balance: -150,
    ///         },
    ///     ],
    /// );
    /// ```
    pub fn stream_entities<S: Iterator<Item = U> + 'a>(
        &'a self,
        stream: S,
    ) -> impl Iterator<Item = T> + 'a {
        stream.scan(T::default(), move |state, cur| {
            *state = (self.applier)(state, &cur);
            Some(state.clone())
        })
    }

    /// Project a stream of events (i.e. an Iterator<T>) into the latest state of the entity.
    ///
    /// ```
    /// use crate::projector::Projector;
    ///
    /// enum PaymentEvent {
    ///   MoneySpent(i32),
    ///   MoneyReceived(i32),
    /// }
    ///
    /// #[derive(Clone, Default, Eq, PartialEq, Debug)]
    /// struct Account {
    ///   balance: i32,
    /// }
    ///
    /// let events = vec![
    ///     PaymentEvent::MoneySpent(300),
    ///     PaymentEvent::MoneyReceived(150),
    ///     PaymentEvent::MoneyReceived(175),
    ///     PaymentEvent::MoneySpent(35),
    ///     PaymentEvent::MoneyReceived(12),
    /// ];
    ///
    /// let projector = Projector::from_applier(|entity: &Account, event: &PaymentEvent| {
    ///     Account {
    ///         balance: match event {
    ///             PaymentEvent::MoneySpent(m) => entity.balance - m,
    ///             PaymentEvent::MoneyReceived(m) => entity.balance + m,
    ///         }
    ///     }
    /// });
    /// let state = projector.project_last_state(events.into_iter());
    /// assert_eq!(
    ///     state,
    ///     Account {
    ///         balance: 2
    ///     }
    /// );
    /// ```
    pub fn project_last_state<S: Iterator<Item = U>>(&self, stream: S) -> T {
        self.stream_entities(stream).last().unwrap_or(T::default())
    }

    /// Project a stream of events (i.e. an Iterator<T>) until an entity for which the given
    /// predicate is true is found, then return the matching entity.
    ///
    /// ```
    /// use crate::projector::Projector;
    ///
    /// enum PaymentEvent {
    ///   MoneySpent(i32),
    ///   MoneyReceived(i32),
    /// }
    ///
    /// #[derive(Clone, Default, Eq, PartialEq, Debug)]
    /// struct Account {
    ///   balance: i32,
    /// }
    ///
    /// let events = vec![
    ///     PaymentEvent::MoneySpent(300),
    ///     PaymentEvent::MoneyReceived(150),
    ///     PaymentEvent::MoneyReceived(175),
    ///     PaymentEvent::MoneySpent(35),
    /// ];
    ///
    /// let projector = Projector::from_applier(|entity: &Account, event: &PaymentEvent| {
    ///     Account {
    ///         balance: match event {
    ///             PaymentEvent::MoneySpent(m) => entity.balance - m,
    ///             PaymentEvent::MoneyReceived(m) => entity.balance + m,
    ///         }
    ///     }
    /// });
    /// let matching = projector.match_from_stream(
    ///     events.into_iter(),
    ///     |entity| entity.balance > 0,
    /// );
    /// assert_eq!(
    ///     matching,
    ///     Some(Account {
    ///         balance: 25
    ///     }),
    /// );
    /// ```
    pub fn match_from_stream<S: Iterator<Item = U>>(
        &self,
        stream: S,
        matcher: impl Fn(&T) -> bool,
    ) -> Option<T> {
        self.stream_entities(stream).find(|e| matcher(e)).to_owned()
    }
}
