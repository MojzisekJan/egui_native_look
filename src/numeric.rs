//! Exact arithmetic for the builtin integer types up to 64 bits.
use egui::emath::Numeric;
use std::any::Any;

pub(crate) fn integer<T: Numeric>(value: T) -> Option<i128> {
    if !T::INTEGRAL {
        return None;
    }
    let value = &value as &dyn Any;
    macro_rules! read { ($($t:ty),*) => { $(if let Some(value) = value.downcast_ref::<$t>() { return Some(*value as i128); })* }; }
    read!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);
    macro_rules! nonzero { ($($t:ty),*) => { $(if let Some(value) = value.downcast_ref::<$t>() { return Some(value.get() as i128); })* }; }
    nonzero!(
        std::num::NonZeroU8,
        std::num::NonZeroU16,
        std::num::NonZeroU32,
        std::num::NonZeroU64,
        std::num::NonZeroUsize
    );
    None
}

pub(crate) fn from_integer<T: Numeric>(number: i128) -> Option<T> {
    let mut output = T::MIN;
    let any = &mut output as &mut dyn Any;
    macro_rules! write { ($($t:ty),*) => { $(if let Some(value) = any.downcast_mut::<$t>() { *value = <$t>::try_from(number).ok()?; return Some(output); })* }; }
    write!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);
    macro_rules! nonzero { ($(($t:ty, $base:ty)),*) => { $(if let Some(value) = any.downcast_mut::<$t>() { *value = <$t>::new(<$base>::try_from(number).ok()?)?; return Some(output); })* }; }
    nonzero!(
        (std::num::NonZeroU8, u8),
        (std::num::NonZeroU16, u16),
        (std::num::NonZeroU32, u32),
        (std::num::NonZeroU64, u64),
        (std::num::NonZeroUsize, usize)
    );
    None
}

pub(crate) fn bounds<T: Numeric>(start: T, end: T) -> Option<(i128, i128)> {
    let (a, b) = (integer(start)?, integer(end)?);
    Some((a.min(b), a.max(b)))
}

pub(crate) fn wide<T: Numeric>(value: T, bounds: Option<(i128, i128)>) -> bool {
    const EXACT: i128 = (1i128 << 53) - 1;
    integer(value).is_some_and(|v| v.abs() > EXACT)
        || bounds.is_some_and(|(a, b)| a.abs() > EXACT || b.abs() > EXACT)
}

pub(crate) fn changed<T: Numeric>(before: T, after: T) -> bool {
    before != after && !(before.to_f64().is_nan() && after.to_f64().is_nan())
}

pub(crate) fn clamp<T: Numeric>(value: T, limits: (i128, i128)) -> Option<T> {
    from_integer(integer(value)?.clamp(limits.0, limits.1))
}

pub(crate) fn stepped<T: Numeric>(
    value: T,
    step: f64,
    count: f64,
    limits: Option<(i128, i128)>,
) -> Option<T> {
    let limits = limits?;
    let delta = (step * count).round() as i128;
    from_integer(
        integer(value)?
            .saturating_add(delta)
            .clamp(limits.0, limits.1),
    )
}

pub(crate) fn parse<T: Numeric>(text: &str, limits: (i128, i128)) -> Option<T> {
    let text = text.trim();
    // Accept .0 suffixes emitted by min_decimals without rounding non-integer input.
    let integer_text = if let Some((whole, decimals)) = text.split_once('.') {
        if decimals.is_empty() || !decimals.bytes().all(|c| c == b'0') {
            return None;
        }
        whole
    } else {
        text
    };
    from_integer(integer_text.parse::<i128>().ok()?.clamp(limits.0, limits.1))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtin_integer_conversions_round_trip_extremes() {
        macro_rules! check { ($($t:ty),*) => { $(for value in [<$t>::MIN, <$t>::MAX] {
            assert_eq!(from_integer::<$t>(integer(value).unwrap()), Some(value));
        })* }; }
        check!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize);
        assert_eq!(integer(std::num::NonZeroU64::MAX), Some(u64::MAX as i128));
        assert_eq!(from_integer::<std::num::NonZeroU64>(0), None);
        assert_eq!(integer(1.0f64), None);
    }
    #[test]
    fn integer_parsing_and_steps_never_round_large_values() {
        let bounds = (0, u64::MAX as i128);
        assert_eq!(parse::<u64>("18446744073709551615", bounds), Some(u64::MAX));
        assert_eq!(
            parse::<u64>("9007199254740993.00", bounds),
            Some((1u64 << 53) + 1)
        );
        assert_eq!(parse::<u64>("9007199254740993.5", bounds), None);
        assert_eq!(parse::<u64>("NaN", bounds), None);
        assert_eq!(stepped(u64::MAX, 1.0, 1.0, Some(bounds)), Some(u64::MAX));
        assert_eq!(
            stepped(u64::MAX, 1.0, -1.0, Some(bounds)),
            Some(u64::MAX - 1)
        );
        assert_eq!(
            stepped(
                i64::MIN,
                1.0,
                -1.0,
                Some((i64::MIN as i128, i64::MAX as i128))
            ),
            Some(i64::MIN)
        );
    }
}
