pub trait Project<T, U>
where
    U: Default + Clone,
{
    fn apply_all<S: Iterator<Item = T>>(&self, stream: S) -> U;
    fn find_state<S: Iterator<Item = T>>(
        &self,
        stream: S,
        matcher: &dyn Fn(&U) -> bool,
    ) -> Option<U>;
}

pub struct Projector<'a, T, U>
where
    U: Default + Clone,
{
    applier: &'a dyn Fn(&T, &U) -> U,
}

impl<'a, T, U> Projector<'a, T, U>
where
    U: Default + Clone,
{
    pub fn from_applier(applier: &'a dyn Fn(&T, &U) -> U) -> Self {
        Projector { applier }
    }
}

impl<'a, T, U: Default + Clone> Project<T, U> for Projector<'a, T, U> {
    fn apply_all<S: Iterator<Item = T>>(&self, stream: S) -> U {
        stream.fold(U::default(), |acc, cur| (self.applier)(&cur, &acc))
    }

    fn find_state<S: Iterator<Item = T>>(
        &self,
        stream: S,
        matcher: &dyn Fn(&U) -> bool,
    ) -> Option<U> {
        stream
            .scan(U::default(), |state, cur| {
                *state = (self.applier)(&cur, state);
                Some(state.clone())
            })
            .find(|e| matcher(e))
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[derive(Default, PartialEq, Eq, Debug, Clone)]
    struct TestEntity {
        id: Option<String>,
        timestamp: Option<String>,
    }

    struct TestEvent {
        id: String,
        timestamp: String,
    }

    fn test_applier(event: &TestEvent, entity: &TestEntity) -> TestEntity {
        TestEntity {
            id: if let Some(id) = &entity.id {
                Some(id.clone())
            } else {
                Some(event.id.clone())
            },
            timestamp: if let Some(ts) = &entity.timestamp {
                match ts.cmp(&event.timestamp) {
                    Ordering::Less => Some(event.timestamp.clone()),
                    Ordering::Greater => Some(ts.clone()),
                    Ordering::Equal => Some(ts.clone()),
                }
            } else {
                Some(event.timestamp.clone())
            },
        }
    }

    #[test]
    fn apply_all() {
        let events = vec![
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-1"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-2"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-8"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-3"),
            },
        ];
        let result = Projector::from_applier(&test_applier).apply_all(events.into_iter());
        assert_eq!(
            result,
            TestEntity {
                id: Some(String::from("id-1")),
                timestamp: Some(String::from("ts-8")),
            }
        );
    }

    #[test]
    fn find_state() {
        let events = vec![
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-1"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-3"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-2"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-8"),
            },
        ];
        let result =
            Projector::from_applier(&test_applier).find_state(events.into_iter(), &|entity| {
                match &entity.timestamp {
                    Some(ts) => ts.contains("8"),
                    None => false,
                }
            });
        assert_eq!(
            result,
            Some(TestEntity {
                id: Some(String::from("id-1")),
                timestamp: Some(String::from("ts-8")),
            })
        );
    }
}
