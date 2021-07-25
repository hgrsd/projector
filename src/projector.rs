trait Project<T, U>
where
    U: Default,
{
    fn apply_stream<S: IntoIterator<Item = T>>(&self, stream: S) -> U;
}

pub struct Projector<T, U>
where
    U: Default,
{
    applier: fn(&T, &U) -> U,
}

impl<T, U> Projector<T, U>
where
    U: Default,
{
    pub fn from_applier(applier: fn(&T, &U) -> U) -> Self {
        Projector { applier }
    }

    pub fn process_stream<S: IntoIterator<Item = T>>(&self, stream: S) -> U {
        stream
            .into_iter()
            .fold(U::default(), |acc, cur| (self.applier)(&cur, &acc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[derive(Default, PartialEq, Eq, Debug)]
    struct TestEntity {
        id: Option<String>,
        timestamp: Option<String>,
    }

    struct TestEvent {
        id: String,
        timestamp: String,
    }

    #[test]
    fn basic() {
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
        ];
        let projector =
            Projector::from_applier(|event: &TestEvent, entity: &TestEntity| TestEntity {
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
            });
        let result = projector.process_stream(events);
        assert_eq!(
            result,
            TestEntity {
                id: Some(String::from("id-1")),
                timestamp: Some(String::from("ts-3")),
            }
        );
    }
}
