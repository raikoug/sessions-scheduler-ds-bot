use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{DateTime, Utc};

use crate::models::{AvailabilityInstance, OverlapSlot};

#[derive(Debug, Clone)]
struct Event {
    user_id: String,
    delta: i32,
}

pub fn compute_full_overlaps(instances: &[AvailabilityInstance]) -> Vec<OverlapSlot> {
    let mut users: BTreeSet<String> = BTreeSet::new();
    let mut user_names: HashMap<String, String> = HashMap::new();
    let mut events: BTreeMap<DateTime<Utc>, Vec<Event>> = BTreeMap::new();

    for instance in instances {
        if instance.end_utc <= instance.start_utc {
            continue;
        }
        users.insert(instance.user_id.clone());
        user_names
            .entry(instance.user_id.clone())
            .or_insert_with(|| instance.username.clone());
        events.entry(instance.start_utc).or_default().push(Event {
            user_id: instance.user_id.clone(),
            delta: 1,
        });
        events.entry(instance.end_utc).or_default().push(Event {
            user_id: instance.user_id.clone(),
            delta: -1,
        });
    }

    if users.is_empty() || events.len() < 2 {
        return Vec::new();
    }

    let ordered_times: Vec<DateTime<Utc>> = events.keys().copied().collect();
    let mut active_counts: HashMap<String, i32> = HashMap::new();
    let mut raw_segments: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();

    for window in ordered_times.windows(2) {
        let current = window[0];
        let next = window[1];

        if let Some(current_events) = events.get(&current) {
            for event in current_events {
                *active_counts.entry(event.user_id.clone()).or_insert(0) += event.delta;
            }
        }

        if next > current && all_users_active(&users, &active_counts) {
            raw_segments.push((current, next));
        }
    }

    let merged = merge_adjacent(raw_segments);
    let participant_user_ids: Vec<String> = users.into_iter().collect();
    let participant_usernames: Vec<String> = participant_user_ids
        .iter()
        .map(|id| user_names.get(id).cloned().unwrap_or_else(|| id.clone()))
        .collect();

    merged
        .into_iter()
        .enumerate()
        .map(|(idx, (start_utc, end_utc))| OverlapSlot {
            display_index: idx as i64 + 1,
            start_utc,
            end_utc,
            participant_user_ids: participant_user_ids.clone(),
            participant_usernames: participant_usernames.clone(),
        })
        .collect()
}

fn all_users_active(users: &BTreeSet<String>, active_counts: &HashMap<String, i32>) -> bool {
    users
        .iter()
        .all(|user_id| active_counts.get(user_id).copied().unwrap_or(0) > 0)
}

fn merge_adjacent(
    mut segments: Vec<(DateTime<Utc>, DateTime<Utc>)>,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    if segments.is_empty() {
        return segments;
    }

    segments.sort_by_key(|(start, _)| *start);
    let mut merged: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();

    for (start, end) in segments {
        if let Some((_, last_end)) = merged.last_mut() {
            if *last_end == start {
                *last_end = end;
                continue;
            }
        }
        merged.push((start, end));
    }

    merged
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn computes_dynamic_overlap() {
        let instances = vec![
            AvailabilityInstance {
                availability_id: "a".into(),
                user_id: "u1".into(),
                username: "A".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 25, 19, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 25, 21, 0, 0).unwrap(),
            },
            AvailabilityInstance {
                availability_id: "b".into(),
                user_id: "u2".into(),
                username: "B".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 25, 20, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 25, 22, 0, 0).unwrap(),
            },
        ];

        let overlaps = compute_full_overlaps(&instances);
        assert_eq!(overlaps.len(), 1);
        assert_eq!(
            overlaps[0].start_utc,
            Utc.with_ymd_and_hms(2026, 5, 25, 20, 0, 0).unwrap()
        );
        assert_eq!(
            overlaps[0].end_utc,
            Utc.with_ymd_and_hms(2026, 5, 25, 21, 0, 0).unwrap()
        );
    }

    #[test]
    fn computes_full_overlap_for_three_users() {
        let instances = vec![
            AvailabilityInstance {
                availability_id: "a".into(),
                user_id: "u1".into(),
                username: "A".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 26, 18, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 26, 22, 0, 0).unwrap(),
            },
            AvailabilityInstance {
                availability_id: "b".into(),
                user_id: "u2".into(),
                username: "B".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 26, 19, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 26, 21, 30, 0).unwrap(),
            },
            AvailabilityInstance {
                availability_id: "c".into(),
                user_id: "u3".into(),
                username: "C".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 26, 20, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 26, 23, 0, 0).unwrap(),
            },
        ];

        let overlaps = compute_full_overlaps(&instances);
        assert_eq!(overlaps.len(), 1);
        assert_eq!(
            overlaps[0].start_utc,
            Utc.with_ymd_and_hms(2026, 5, 26, 20, 0, 0).unwrap()
        );
        assert_eq!(
            overlaps[0].end_utc,
            Utc.with_ymd_and_hms(2026, 5, 26, 21, 30, 0).unwrap()
        );
        assert_eq!(overlaps[0].participant_user_ids.len(), 3);
    }

    #[test]
    fn returns_no_overlap_when_five_users_do_not_all_match() {
        let instances = vec![
            AvailabilityInstance {
                availability_id: "a".into(),
                user_id: "u1".into(),
                username: "A".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 27, 18, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 27, 22, 0, 0).unwrap(),
            },
            AvailabilityInstance {
                availability_id: "b".into(),
                user_id: "u2".into(),
                username: "B".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 27, 19, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 27, 23, 0, 0).unwrap(),
            },
            AvailabilityInstance {
                availability_id: "c".into(),
                user_id: "u3".into(),
                username: "C".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 27, 20, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 27, 21, 0, 0).unwrap(),
            },
            AvailabilityInstance {
                availability_id: "d".into(),
                user_id: "u4".into(),
                username: "D".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 27, 20, 30, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 27, 22, 30, 0).unwrap(),
            },
            AvailabilityInstance {
                availability_id: "e".into(),
                user_id: "u5".into(),
                username: "E".into(),
                start_utc: Utc.with_ymd_and_hms(2026, 5, 27, 17, 0, 0).unwrap(),
                end_utc: Utc.with_ymd_and_hms(2026, 5, 27, 20, 15, 0).unwrap(),
            },
        ];

        let overlaps = compute_full_overlaps(&instances);
        assert!(overlaps.is_empty());
    }
}
