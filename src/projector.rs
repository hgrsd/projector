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
    pub fn from_applier(applier: V) -> Self {
        Projector {
            _l0: &PhantomData,
            _e0: PhantomData,
            _e1: PhantomData,
            applier,
        }
    }

    pub fn stream_entities<S: Iterator<Item = U> + 'a>(
        &'a self,
        stream: S,
    ) -> impl Iterator<Item = T> + 'a {
        stream.scan(T::default(), move |state, cur| {
            *state = (self.applier)(state, &cur);
            Some(state.clone())
        })
    }

    pub fn project_last_state<S: Iterator<Item = U>>(&self, stream: S) -> T {
        self.stream_entities(stream).last().unwrap_or(T::default())
    }

    pub fn match_from_stream<S: Iterator<Item = U>>(
        &self,
        stream: S,
        matcher: impl Fn(&T) -> bool,
    ) -> Option<T> {
        self.stream_entities(stream).find(|e| matcher(e)).to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[derive(Default, PartialEq, Eq, Debug, Clone, Hash)]
    struct TestEntity {
        id: Option<String>,
        timestamp: Option<String>,
    }

    struct TestEvent {
        id: String,
        timestamp: String,
    }

    fn test_applier(entity: &TestEntity, event: &TestEvent) -> TestEntity {
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
    fn latest_state() {
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
        let result = Projector::from_applier(&test_applier).project_last_state(events.into_iter());
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
                timestamp: String::from("ts-3"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-8"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-1"),
            },
        ];
        let result =
            Projector::from_applier(&test_applier).match_from_stream(events.into_iter(), |state| {
                if let Some(ts) = &state.timestamp {
                    ts.contains("8")
                } else {
                    false
                }
            });

        assert_eq!(
            result,
            Some(TestEntity {
                id: Some(String::from("id-1")),
                timestamp: Some(String::from("ts-8")),
            },)
        );
    }

    #[test]
    fn stream() {
        let events = vec![
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-3"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-8"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-1"),
            },
        ];
        let result: Vec<TestEntity> = Projector::from_applier(&test_applier)
            .stream_entities(events.into_iter())
            .collect();

        assert_eq!(
            result,
            vec![
                TestEntity {
                    id: Some(String::from("id-1")),
                    timestamp: Some(String::from("ts-3")),
                },
                TestEntity {
                    id: Some(String::from("id-1")),
                    timestamp: Some(String::from("ts-8")),
                },
                TestEntity {
                    id: Some(String::from("id-1")),
                    timestamp: Some(String::from("ts-8")),
                }
            ],
        );
    }

    #[test]
    fn stream_with_map() {
        let events = vec![
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-3"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-8"),
            },
            TestEvent {
                id: String::from("id-1"),
                timestamp: String::from("ts-1"),
            },
        ];
        let result: Vec<TestEntity> = Projector::from_applier(&test_applier)
            .stream_entities(events.into_iter())
            .map(|entity| TestEntity {
                id: entity.id,
                timestamp: Some(String::from(format!(
                    "mapped-{}",
                    entity.timestamp.unwrap()
                ))),
            })
            .collect();

        assert_eq!(
            result,
            vec![
                TestEntity {
                    id: Some(String::from("id-1")),
                    timestamp: Some(String::from("mapped-ts-3")),
                },
                TestEntity {
                    id: Some(String::from("id-1")),
                    timestamp: Some(String::from("mapped-ts-8")),
                },
                TestEntity {
                    id: Some(String::from("id-1")),
                    timestamp: Some(String::from("mapped-ts-8")),
                }
            ],
        );
    }
}
