pub trait Project<T, U>
where
    U: Default,
{
    fn from_stream<S: Iterator<Item = T>>(&self, stream: S) -> U;
}

pub struct Projector<'a, T, U>
where
    U: Default,
{
    applier: &'a dyn Fn(&T, &U) -> U,
}

impl<'a, T, U> Projector<'a, T, U>
where
    U: Default,
{
    pub fn from_applier(applier: &'a dyn Fn(&T, &U) -> U) -> Self {
        Projector { applier }
    }
}

impl<'a, T, U: Default> Project<T, U> for Projector<'a, T, U> {
    fn from_stream<S: Iterator<Item = T>>(&self, stream: S) -> U {
        stream.fold(U::default(), |acc, cur| (self.applier)(&cur, &acc))
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
            Projector::from_applier(&|event: &TestEvent, entity: &TestEntity| TestEntity {
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
        let result = projector.from_stream(events.into_iter());
        assert_eq!(
            result,
            TestEntity {
                id: Some(String::from("id-1")),
                timestamp: Some(String::from("ts-3")),
            }
        );
    }
}
