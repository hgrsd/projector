use std::marker::PhantomData;

pub struct Projector<'a, TEntity, TEvent, TApplier>
where
    TEntity: Default + Clone,
    TApplier: Fn(&TEntity, &TEvent) -> TEntity,
{
    _l0: &'a PhantomData<()>,
    _e0: PhantomData<TEntity>,
    _e1: PhantomData<TEvent>,
    applier: TApplier,
}

impl<'a, TEntity, TEvent, TApplier> Projector<'a, TEntity, TEvent, TApplier>
where
    TEntity: Default + Clone,
    TApplier: Fn(&TEntity, &TEvent) -> TEntity,
{
    pub fn from_applier(applier: TApplier) -> Self {
        Projector {
            _l0: &PhantomData,
            _e0: PhantomData,
            _e1: PhantomData,
            applier,
        }
    }

    pub fn stream_entities<SIn: Iterator<Item = TEvent> + 'a>(
        &'a self,
        stream: SIn,
    ) -> impl Iterator<Item = TEntity> + 'a {
        stream.scan(TEntity::default(), move |state, cur| {
            *state = (self.applier)(state, &cur);
            Some(state.clone())
        })
    }

    pub fn project_last_state<S: Iterator<Item = TEvent>>(&self, stream: S) -> TEntity {
        self.stream_entities(stream)
            .last()
            .unwrap_or(TEntity::default())
    }

    pub fn match_from_stream<S: Iterator<Item = TEvent>>(
        &self,
        stream: S,
        matcher: &dyn Fn(&TEntity) -> bool,
    ) -> Option<TEntity> {
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
        let result = Projector::from_applier(&test_applier).match_from_stream(
            events.into_iter(),
            &|state| {
                if let Some(ts) = &state.timestamp {
                    ts.contains("8")
                } else {
                    false
                }
            },
        );

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
