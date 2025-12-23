//! Motion test geometry: where the shape is after a number of updates.

/// Left edge of the shape after `updates` steps of `step`, when its left edge may range over
/// `0..=room` (room = track width − shape width). The shape moves right in equal steps and
/// wraps to 0 after the last step that keeps it fully on the track, so every jump is the same
/// size. Bad inputs (non-finite, step ≤ 0, room < 0) give 0.
pub fn motion_x(updates: u64, step: f32, room: f32) -> f32 {
    if !(step.is_finite() && room.is_finite()) || step <= 0.0 || room < 0.0 {
        return 0.0;
    }
    // Number of positions: 0, step, 2·step, … ≤ room. Capped so the cast cannot overflow.
    let positions = ((room / step).floor() + 1.0).min(1.0e9);
    let n = if positions >= 1.0 {
        positions as u64
    } else {
        1
    };
    (updates % n) as f32 * step
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn steps_and_wraps() {
        // room 100, step 30: positions 0, 30, 60, 90.
        let xs: Vec<f32> = (0..6).map(|u| motion_x(u, 30.0, 100.0)).collect();
        assert_eq!(xs, [0.0, 30.0, 60.0, 90.0, 0.0, 30.0]);
        assert_eq!(motion_x(5, 200.0, 100.0), 0.0, "step wider than the room");
        assert_eq!(motion_x(5, 0.0, 100.0), 0.0);
        assert_eq!(motion_x(5, f32::NAN, 100.0), 0.0);
        assert_eq!(motion_x(5, 10.0, -1.0), 0.0);
        assert_eq!(
            motion_x(u64::MAX, 1.0e-30, 1.0e30),
            motion_x(u64::MAX, 1.0e-30, 1.0e30)
        );
    }

    proptest! {
        #[test]
        fn always_on_the_track(updates: u64, step in 0.001f32..10_000.0, room in 0.0f32..100_000.0) {
            let x = motion_x(updates, step, room);
            prop_assert!(x.is_finite());
            prop_assert!(x >= 0.0);
            prop_assert!(x <= room + step * 1e-3, "x={x} room={room}");
        }
    }
}
