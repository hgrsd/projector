use crate::projector::{Project, Projector};

#[derive(Clone)]
pub struct EventData {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub care_recipient_id: String,
    pub caregiver_id: String,
}

#[derive(Clone)]
pub enum VisitEvent {
    CheckIn(EventData),
    CheckOut(EventData),
}

pub struct VisitReport {
    pub id: String,
    pub visit_events: Vec<VisitEvent>,
}

#[derive(Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Period {
    pub start: Option<String>,
    pub end: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Participant {
    pub id: String,
    pub start: Option<String>,
    pub end: Option<String>,
}

#[derive(Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Encounter {
    pub period: Period,
    pub participant: Vec<Participant>,
}

fn min_opt(t0: &Option<String>, t1: &String) -> String {
    if let Some(unw0) = t0 {
        String::min(unw0.clone(), t1.clone())
    } else {
        t1.clone()
    }
}

fn max_opt(t0: &Option<String>, t1: &String) -> String {
    if let Some(unw0) = t0 {
        String::max(unw0.clone(), t1.clone())
    } else {
        t1.clone()
    }
}

fn apply_participant(event: &VisitEvent, participant: &Vec<Participant>) -> Vec<Participant> {
    let id = match &event {
        VisitEvent::CheckIn(c) => &c.caregiver_id,
        VisitEvent::CheckOut(c) => &c.caregiver_id,
    };

    let new_entry = match participant.into_iter().find(|p| p.id == *id) {
        Some(p) => match &event {
            VisitEvent::CheckIn(c) => Participant {
                id: id.clone(),
                start: Some(min_opt(&p.start, &c.timestamp)),
                end: p.end.clone(),
            },
            VisitEvent::CheckOut(c) => Participant {
                id: id.clone(),
                start: p.start.clone(),
                end: Some(max_opt(&p.end, &c.timestamp)),
            },
        },
        None => match &event {
            VisitEvent::CheckIn(c) => Participant {
                id: id.clone(),
                start: Some(c.timestamp.clone()),
                end: None,
            },
            VisitEvent::CheckOut(c) => Participant {
                id: id.clone(),
                start: Some(c.timestamp.clone()),
                end: None,
            },
        },
    };

    participant
        .into_iter()
        .cloned()
        .filter(|p| p.id != *id)
        .chain(vec![new_entry].into_iter())
        .collect()
}

fn apply_period(event: &VisitEvent, existing: &Period) -> Period {
    match event {
        VisitEvent::CheckIn(c) => Period {
            start: Some(min_opt(&existing.start, &c.timestamp)),
            end: existing.end.clone(),
        },
        VisitEvent::CheckOut(c) => Period {
            start: existing.start.clone(),
            end: Some(max_opt(&existing.end, &c.timestamp)),
        },
    }
}

pub fn encounter_projector() -> Projector<'static, VisitReport, Encounter> {
    Projector::from_applier(&|report, encounter| {
        report
            .visit_events
            .iter()
            .fold(encounter.clone(), |acc, event| Encounter {
                period: apply_period(&event, &acc.period),
                participant: apply_participant(&event, &acc.participant),
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let report_0 = VisitReport {
            id: "1".to_owned(),
            visit_events: vec![
                VisitEvent::CheckIn(EventData {
                    event_type: "check_in".to_owned(),
                    id: "0".to_owned(),
                    care_recipient_id: "0".to_owned(),
                    caregiver_id: "0".to_owned(),
                    timestamp: "2021-01-01T00:00:00.000Z".to_owned(),
                }),
                VisitEvent::CheckOut(EventData {
                    event_type: "check_out".to_owned(),
                    id: "0".to_owned(),
                    care_recipient_id: "0".to_owned(),
                    caregiver_id: "0".to_owned(),
                    timestamp: "2021-01-01T10:00:00.000Z".to_owned(),
                }),
            ],
        };
        let report_1 = VisitReport {
            id: "1".to_owned(),
            visit_events: vec![VisitEvent::CheckIn(EventData {
                event_type: "check_in".to_owned(),
                id: "0".to_owned(),
                care_recipient_id: "0".to_owned(),
                caregiver_id: "1".to_owned(),
                timestamp: "2020-12-31T00:00:00.001Z".to_owned(),
            })],
        };
        let stream = vec![report_0, report_1];
        let result = encounter_projector().from_stream(stream.into_iter());
        assert_eq!(
            result.period,
            Period {
                start: Some("2020-12-31T00:00:00.001Z".to_owned()),
                end: Some("2021-01-01T10:00:00.000Z".to_owned()),
            },
        );
        assert_eq!(
            result.participant,
            vec![
                Participant {
                    id: "0".to_owned(),
                    start: Some("2021-01-01T00:00:00.000Z".to_owned()),
                    end: Some("2021-01-01T10:00:00.000Z".to_owned()),
                },
                Participant {
                    id: "1".to_owned(),
                    start: Some("2020-12-31T00:00:00.001Z".to_owned()),
                    end: None,
                }
            ],
        );
    }
}
